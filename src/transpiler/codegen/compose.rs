/// Compose backend - generates Jetpack Compose code

use crate::transpiler::ast::{Markup, PropValue, WhitehallFile};

pub struct ComposeBackend {
    package: String,
    component_name: String,
    component_type: Option<String>,
    indent_level: usize,
    nullable_vars: std::collections::HashSet<String>,
    var_types: std::collections::HashMap<String, (String, String)>, // Maps variable name to (type, default_value)
}

impl ComposeBackend {
    pub fn new(package: &str, component_name: &str, component_type: Option<&str>) -> Self {
        ComposeBackend {
            package: package.to_string(),
            component_name: component_name.to_string(),
            component_type: component_type.map(|s| s.to_string()),
            indent_level: 0,
            nullable_vars: std::collections::HashSet::new(),
            var_types: std::collections::HashMap::new(),
        }
    }

    /// Phase 5: Generate with optimization support
    ///
    /// This method receives optimization plans and routes for loops accordingly:
    /// - RecyclerView optimization: Uses RecyclerViewGenerator + ViewBackend
    /// - Default: Standard Compose generation
    pub fn generate_with_optimizations(
        &mut self,
        file: &WhitehallFile,
        _optimizations: &[crate::transpiler::optimizer::Optimization],
    ) -> Result<String, String> {
        // Phase 5: For now, just call the existing generate method
        // RecyclerView routing will be added when we implement for loop generation
        // The optimizations will be checked during for loop generation
        self.generate(file)
    }

    pub fn generate(&mut self, file: &WhitehallFile) -> Result<String, String> {
        let mut output = String::new();

        // Package declaration
        output.push_str(&format!("package {}\n\n", self.package));

        // Extract route parameters if this is a screen
        let route_params = if self.component_type.as_deref() == Some("screen") {
            self.extract_route_params(file)
        } else {
            Vec::new()
        };

        // Collect prop and component imports from markup first to determine order
        let mut prop_imports = Vec::new();
        let mut component_imports = Vec::new();
        self.collect_imports_recursive(&file.markup, &mut prop_imports, &mut component_imports);

        // Import ordering:
        // If there's exactly one component import, no props, no state, no user imports: component first
        // Otherwise: Composable, prop imports, user imports, component imports
        let mut imports = Vec::new();

        let is_simple_case = component_imports.len() == 1
            && prop_imports.is_empty()
            && file.imports.is_empty();

        if is_simple_case {
            // Very simple case: single component import before Composable (e.g., Text before Composable)
            imports.extend(component_imports.clone());
        }

        // Add Composable or runtime imports
        if !file.state.is_empty() {
            imports.push("androidx.compose.runtime.*".to_string());
            // Remove specific runtime imports since we're using wildcard
            prop_imports.retain(|imp| !imp.starts_with("androidx.compose.runtime."));
        } else {
            imports.push("androidx.compose.runtime.Composable".to_string());
        }

        // Add prop imports (Modifier, clickable)
        imports.extend(prop_imports);

        // User imports (resolve $ aliases)
        for import in &file.imports {
            let resolved = self.resolve_import(&import.path);
            imports.push(resolved);
        }

        // Add component imports (if not already added in simple case)
        if !is_simple_case {
            imports.extend(component_imports);
        }

        // Add kotlinx.coroutines.launch if there are onMount hooks with launch calls
        let has_on_mount_with_launch = file.lifecycle_hooks.iter().any(|h| {
            h.hook_type == "onMount" && (h.body.contains("launch ") || h.body.contains("launch{"))
        });
        if has_on_mount_with_launch {
            imports.push("kotlinx.coroutines.launch".to_string());
        }

        // Add NavController import for screens
        if self.component_type.as_deref() == Some("screen") {
            imports.push("androidx.navigation.NavController".to_string());
        }

        // Sort imports alphabetically (standard Kotlin convention)
        imports.sort();

        // Write all imports
        for import in imports {
            output.push_str(&format!("import {}\n", import));
        }

        output.push('\n');

        // Component function
        output.push_str("@Composable\n");
        output.push_str(&format!("fun {}(", self.component_name));

        // For screens, add navController parameter
        let is_screen = self.component_type.as_deref() == Some("screen");
        if is_screen {
            if !route_params.is_empty() || !file.props.is_empty() {
                output.push('\n');
                output.push_str("    navController: NavController,\n");
                // Add route parameters
                for param in &route_params {
                    output.push_str(&format!("    {}: String", param));
                    if !file.props.is_empty() {
                        output.push(',');
                    }
                    output.push('\n');
                }
            } else {
                output.push_str("navController: NavController");
            }
        }

        // Props as function parameters
        if !file.props.is_empty() {
            if is_screen {
                // Already on same line with navController
            } else {
                output.push('\n');
            }
            for (i, prop) in file.props.iter().enumerate() {
                if !is_screen || i > 0 {
                    output.push_str("    ");
                }
                output.push_str(&prop.name);
                output.push_str(": ");
                output.push_str(&prop.prop_type);
                if let Some(default) = &prop.default_value {
                    output.push_str(" = ");
                    output.push_str(default);
                } else if prop.prop_type.ends_with('?') {
                    // Nullable types without explicit default get = null
                    output.push_str(" = null");
                }
                if i < file.props.len() - 1 {
                    output.push(',');
                }
                if !is_screen || i < file.props.len() - 1 {
                    output.push('\n');
                }
            }
        }

        output.push_str(") {\n");
        self.indent_level += 1;

        // Separate mutable (var) and computed (val) state
        let mut mutable_state = Vec::new();
        let mut computed_state = Vec::new();

        for state in &file.state {
            // Track nullable variables
            if let Some(ref type_ann) = state.type_annotation {
                if type_ann.ends_with('?') {
                    self.nullable_vars.insert(state.name.clone());
                }
            }

            if state.mutable {
                mutable_state.push(state);
            } else {
                computed_state.push(state);
            }
        }

        // Generate mutable state (var)
        for state in &mutable_state {
            output.push_str(&self.indent());
            if let Some(ref type_ann) = state.type_annotation {
                // Store type and default value for bind:value transformations
                self.var_types.insert(
                    state.name.clone(),
                    (type_ann.clone(), state.initial_value.clone())
                );

                // With type annotation: var name by remember { mutableStateOf<Type>(value) }
                output.push_str(&format!(
                    "var {} by remember {{ mutableStateOf<{}>({}) }}\n",
                    state.name, type_ann, state.initial_value
                ));
            } else {
                // Try to infer type from initial value for bind:value support
                let inferred_type = self.infer_type_from_value(&state.initial_value);
                self.var_types.insert(
                    state.name.clone(),
                    (inferred_type, state.initial_value.clone())
                );

                // Without type annotation: var name by remember { mutableStateOf(value) }
                output.push_str(&format!(
                    "var {} by remember {{ mutableStateOf({}) }}\n",
                    state.name, state.initial_value
                ));
            }
        }

        if !mutable_state.is_empty() {
            output.push('\n');
        }

        // Generate computed state (val)
        for state in &computed_state {
            output.push_str(&self.indent());

            if state.is_derived_state {
                // derivedStateOf needs special wrapping: val name by remember { derivedStateOf { ... } }
                // Need to format with increased indent level for proper nesting
                output.push_str(&format!("val {} by remember {{\n", state.name));

                // Temporarily increase indent for the derivedStateOf content
                self.indent_level += 1;
                let formatted_value = self.format_multiline_value(&state.initial_value);
                self.indent_level -= 1;

                output.push_str(&format!("{}    {}\n", self.indent(), formatted_value));
                output.push_str(&format!("{}}}\n", self.indent()));
            } else if let Some(ref type_ann) = state.type_annotation {
                // Format multi-line values with proper indentation
                let formatted_value = self.format_multiline_value(&state.initial_value);
                output.push_str(&format!("val {}: {} = {}\n", state.name, type_ann, formatted_value));
            } else {
                output.push_str(&format!("val {} = {}\n", state.name, state.initial_value));
            }
        }

        if !computed_state.is_empty() {
            output.push('\n');
        }

        // Determine if functions should come before or after lifecycle hooks
        // If there are computed state values, functions come first (test 11)
        // If there are no computed state values, lifecycle comes first (test 08)
        let functions_first = !computed_state.is_empty();

        if functions_first {
            // Generate function declarations before lifecycle
            for func in &file.functions {
                output.push_str(&self.indent());
                let return_type_str = if let Some(ref rt) = func.return_type {
                    format!(": {}", rt)
                } else {
                    String::new()
                };
                output.push_str(&format!("fun {}({}){} {{\n", func.name, func.params, return_type_str));
                // Output function body with proper indentation and transformations
                for line in func.body.lines() {
                    output.push_str(&self.indent());
                    output.push_str("    ");

                    // Transform $routes.login → Routes.Login
                    let mut transformed_line = self.transform_route_aliases(line);

                    // Transform $screen.params.{name} → {name}
                    transformed_line = transformed_line.replace("$screen.params.", "");

                    // For screens, transform navigate() to navController.navigate()
                    if self.component_type.as_deref() == Some("screen") {
                        let trimmed = transformed_line.trim();
                        if trimmed.starts_with("navigate(") {
                            output.push_str("navController.");
                        }
                    }

                    output.push_str(&transformed_line);
                    output.push('\n');
                }
                output.push_str(&self.indent());
                output.push_str("}\n");
            }

            if !file.functions.is_empty() {
                output.push('\n');
            }
        }

        // Check if we have both onMount and onDispose hooks
        let has_on_mount = file.lifecycle_hooks.iter().any(|h| h.hook_type == "onMount");
        let has_on_dispose = file.lifecycle_hooks.iter().any(|h| h.hook_type == "onDispose");
        let use_disposable_effect = has_on_mount && has_on_dispose;

        // Check if onMount body contains launch calls
        let has_launch_in_on_mount = file.lifecycle_hooks.iter().any(|h| {
            h.hook_type == "onMount" && (h.body.contains("launch ") || h.body.contains("launch{"))
        });

        // Generate coroutineScope if there are onMount hooks with launch calls
        if has_launch_in_on_mount {
            output.push_str(&self.indent());
            output.push_str("val coroutineScope = rememberCoroutineScope()\n\n");
        }

        // Generate lifecycle hooks
        if use_disposable_effect {
            // Use DisposableEffect when both onMount and onDispose are present
            output.push_str(&self.indent());
            output.push_str("DisposableEffect(Unit) {\n");

            // Generate onMount body
            if let Some(mount_hook) = file.lifecycle_hooks.iter().find(|h| h.hook_type == "onMount") {
                for line in mount_hook.body.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }

                    output.push_str(&self.indent());
                    output.push_str("    ");

                    let original_indent = line.len() - line.trim_start().len();
                    if original_indent > 0 {
                        output.push_str(&"  ".repeat(original_indent / 2));
                    }

                    let transformed_line = line.trim_start().replace("$screen.params.", "");
                    if transformed_line.trim().starts_with("launch ") || transformed_line.trim().starts_with("launch{") {
                        output.push_str("coroutineScope.");
                        output.push_str(transformed_line.trim());
                    } else {
                        output.push_str(&transformed_line);
                    }
                    output.push('\n');
                }
                output.push('\n');
            }

            // Generate onDispose block
            if let Some(dispose_hook) = file.lifecycle_hooks.iter().find(|h| h.hook_type == "onDispose") {
                output.push_str(&self.indent());
                output.push_str("    onDispose {\n");

                for line in dispose_hook.body.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }

                    output.push_str(&self.indent());
                    output.push_str("        ");
                    output.push_str(line.trim());
                    output.push('\n');
                }

                output.push_str(&self.indent());
                output.push_str("    }\n");
            }

            output.push_str(&self.indent());
            output.push_str("}\n\n");
        } else {
            // Use LaunchedEffect for onMount only (current behavior)
            for hook in &file.lifecycle_hooks {
                if hook.hook_type == "onMount" {
                    output.push_str(&self.indent());
                    output.push_str("LaunchedEffect(Unit) {\n");

                    for line in hook.body.lines() {
                        if line.trim().is_empty() {
                            continue;
                        }

                        output.push_str(&self.indent());
                        output.push_str("    ");

                        let original_indent = line.len() - line.trim_start().len();
                        if original_indent > 0 {
                            output.push_str(&"  ".repeat(original_indent / 2));
                        }

                        let transformed_line = line.trim_start().replace("$screen.params.", "");
                        if transformed_line.trim().starts_with("launch ") || transformed_line.trim().starts_with("launch{") {
                            output.push_str("coroutineScope.");
                            output.push_str(transformed_line.trim());
                        } else {
                            output.push_str(&transformed_line);
                        }
                        output.push('\n');
                    }

                    output.push_str(&self.indent());
                    output.push_str("}\n\n");
                }
            }
        }

        // Generate functions after lifecycle if not functions_first
        if !functions_first && !file.functions.is_empty() {
            for func in &file.functions {
                output.push_str(&self.indent());
                let return_type_str = if let Some(ref rt) = func.return_type {
                    format!(": {}", rt)
                } else {
                    String::new()
                };
                output.push_str(&format!("fun {}({}){} {{\n", func.name, func.params, return_type_str));
                // Output function body with proper indentation and transformations
                for line in func.body.lines() {
                    output.push_str(&self.indent());
                    output.push_str("    ");

                    // Transform $routes.login → Routes.Login
                    let mut transformed_line = self.transform_route_aliases(line);

                    // Transform $screen.params.{name} → {name}
                    transformed_line = transformed_line.replace("$screen.params.", "");

                    // For screens, transform navigate() to navController.navigate()
                    if self.component_type.as_deref() == Some("screen") {
                        let trimmed = transformed_line.trim();
                        if trimmed.starts_with("navigate(") {
                            output.push_str("navController.");
                        }
                    }

                    output.push_str(&transformed_line);
                    output.push('\n');
                }
                output.push_str(&self.indent());
                output.push_str("}\n\n");
            }
        }

        // Generate markup
        output.push_str(&self.generate_markup(&file.markup)?);

        self.indent_level -= 1;
        output.push_str("}\n");

        Ok(output)
    }

    fn generate_markup(&self, markup: &Markup) -> Result<String, String> {
        self.generate_markup_with_indent(markup, self.indent_level)
    }

    fn generate_markup_with_indent(&self, markup: &Markup, indent: usize) -> Result<String, String> {
        self.generate_markup_with_context(markup, indent, None)
    }

    fn generate_markup_with_context(&self, markup: &Markup, indent: usize, parent: Option<&str>) -> Result<String, String> {
        match markup {
            Markup::IfElse(if_block) => {
                let mut output = String::new();
                let indent_str = "    ".repeat(indent);

                // if block
                output.push_str(&format!("{}if ({}) {{\n", indent_str, if_block.condition));
                for child in &if_block.then_branch {
                    output.push_str(&self.generate_markup_with_indent(child, indent + 1)?);
                }
                output.push_str(&format!("{}}}", indent_str));

                // else if blocks
                for else_if in &if_block.else_ifs {
                    output.push_str(&format!(" else if ({}) {{\n", else_if.condition));
                    for child in &else_if.body {
                        output.push_str(&self.generate_markup_with_indent(child, indent + 1)?);
                    }
                    output.push_str(&format!("{}}}", indent_str));
                }

                // else block
                if let Some(else_body) = &if_block.else_branch {
                    output.push_str(" else {\n");
                    for child in else_body {
                        output.push_str(&self.generate_markup_with_indent(child, indent + 1)?);
                    }
                    output.push_str(&format!("{}}}", indent_str));
                }

                output.push('\n');
                Ok(output)
            }
            Markup::ForLoop(for_loop) => {
                let mut output = String::new();
                let indent_str = "    ".repeat(indent);

                // Special handling for LazyColumn: use items() instead of forEach
                if parent == Some("LazyColumn") {
                    // items(collection, key = { expr }) { item ->
                    output.push_str(&format!("{}items({}", indent_str, for_loop.collection));

                    if let Some(key_expr) = &for_loop.key_expr {
                        // Ensure key expression has braces
                        let formatted_key = if key_expr.trim().starts_with('{') {
                            key_expr.to_string()
                        } else {
                            format!("{{ {} }}", key_expr)
                        };
                        output.push_str(&format!(", key = {}", formatted_key));
                    }

                    output.push_str(&format!(") {{ {} ->\n", for_loop.item));

                    for child in &for_loop.body {
                        output.push_str(&self.generate_markup_with_context(child, indent + 1, None)?);
                    }

                    output.push_str(&format!("{}}}\n", indent_str));
                    return Ok(output);
                }

                // Regular forEach handling (for non-LazyColumn contexts)
                // If there's an empty block, wrap in if/else
                if let Some(empty_body) = &for_loop.empty_block {
                    // if (collection.isEmpty()) { empty block } else { forEach }
                    output.push_str(&format!("{}if ({}.isEmpty()) {{\n", indent_str, for_loop.collection));
                    for child in empty_body {
                        output.push_str(&self.generate_markup_with_indent(child, indent + 1)?);
                    }
                    output.push_str(&format!("{}}}", indent_str));
                    output.push_str(" else {\n");

                    // forEach block
                    output.push_str(&format!("{}    {}.forEach {{ {} ->\n",
                        indent_str, for_loop.collection, for_loop.item));

                    // If there's a key, wrap in key() block
                    if let Some(key_expr) = &for_loop.key_expr {
                        // Replace 'it' with actual loop variable name
                        let transformed_key = key_expr.replace("it", &for_loop.item);
                        output.push_str(&format!("{}        key({}) {{\n", indent_str, transformed_key));
                        for child in &for_loop.body {
                            output.push_str(&self.generate_markup_with_indent(child, indent + 3)?);
                        }
                        output.push_str(&format!("{}        }}\n", indent_str));
                    } else {
                        for child in &for_loop.body {
                            output.push_str(&self.generate_markup_with_indent(child, indent + 2)?);
                        }
                    }

                    output.push_str(&format!("{}    }}\n", indent_str));
                    output.push_str(&format!("{}}}\n", indent_str));
                } else {
                    // Just forEach without empty check
                    output.push_str(&format!("{}{}.forEach {{ {} ->\n",
                        indent_str, for_loop.collection, for_loop.item));

                    if let Some(key_expr) = &for_loop.key_expr {
                        let transformed_key = key_expr.replace("it", &for_loop.item);
                        output.push_str(&format!("{}    key({}) {{\n", indent_str, transformed_key));
                        for child in &for_loop.body {
                            output.push_str(&self.generate_markup_with_indent(child, indent + 2)?);
                        }
                        output.push_str(&format!("{}    }}\n", indent_str));
                    } else {
                        for child in &for_loop.body {
                            output.push_str(&self.generate_markup_with_indent(child, indent + 1)?);
                        }
                    }

                    output.push_str(&format!("{}}}\n", indent_str));
                }

                Ok(output)
            }
            Markup::When(when_block) => {
                let mut output = String::new();
                let indent_str = "    ".repeat(indent);

                output.push_str(&format!("{}when {{\n", indent_str));

                for branch in &when_block.branches {
                    if let Some(condition) = &branch.condition {
                        output.push_str(&format!("{}    {} -> ", indent_str, condition));
                    } else {
                        output.push_str(&format!("{}    else -> ", indent_str));
                    }

                    // Generate the body inline (single component)
                    let body_code = self.generate_markup_with_indent(&branch.body, 0)?;
                    // Remove leading indent and trailing newline for inline placement
                    let body_trimmed = body_code.trim();
                    output.push_str(body_trimmed);
                    output.push('\n');
                }

                output.push_str(&format!("{}}}\n", indent_str));
                Ok(output)
            }
            Markup::Component(comp) => {
                let mut output = String::new();
                let indent_str = "    ".repeat(indent);
                output.push_str(&indent_str);
                output.push_str(&comp.name);

                let mut params = Vec::new();

                // For Text component with children, add text parameter
                if comp.name == "Text" && !comp.children.is_empty() {
                    let text_expr = self.build_text_expression(&comp.children)?;
                    params.push(format!("text = {}", text_expr));
                }

                // Check if Button has text prop (convert to child later)
                let button_text = if comp.name == "Button" {
                    comp.props.iter()
                        .find(|p| p.name == "text")
                        .map(|p| {
                            // Extract text value (remove quotes if present)
                            let prop_expr = self.get_prop_expr(&p.value);
                            if prop_expr.starts_with('"') && prop_expr.ends_with('"') {
                                prop_expr[1..prop_expr.len()-1].to_string()
                            } else {
                                prop_expr.to_string()
                            }
                        })
                } else {
                    None
                };

                // Special handling for Scaffold with topBar
                if comp.name == "Scaffold" {
                    for prop in &comp.props {
                        if prop.name == "topBar" {
                            match &prop.value {
                                PropValue::Markup(markup) => {
                                    // topBar with component: wrap in lambda
                                    // Generate at indent + 2 (Scaffold at indent + 1, content at indent + 2)
                                    let topbar_code = self.generate_markup_with_indent(markup, indent + 2)?;
                                    let closing_indent = "    ".repeat(indent + 1);
                                    params.push(format!("topBar = {{\n{}{}}}", topbar_code, closing_indent));
                                }
                                PropValue::Expression(expr) => {
                                    // Regular expression - just pass through
                                    params.push(format!("topBar = {}", expr));
                                }
                            }
                        } else {
                            // Other Scaffold props - handle normally
                            let prop_expr = self.get_prop_expr(&prop.value);
                            let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                            params.extend(transformed);
                        }
                    }
                }
                // Special handling for TopAppBar title
                else if comp.name == "TopAppBar" {
                    for prop in &comp.props {
                        if prop.name == "title" {
                            match &prop.value {
                                PropValue::Expression(expr) => {
                                    // title with expression: wrap in { Text(...) }
                                    params.push(format!("title = {{ Text({}) }}", expr));
                                }
                                PropValue::Markup(markup) => {
                                    // title with component: wrap in lambda
                                    let title_code = self.generate_markup_with_indent(markup, indent + 1)?;
                                    params.push(format!("title = {{\n{}}}", title_code.trim_end()));
                                }
                            }
                        } else {
                            // Other TopAppBar props - handle normally
                            let prop_expr = self.get_prop_expr(&prop.value);
                            let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                            params.extend(transformed);
                        }
                    }
                }
                // Special handling for Text and Card with modifier props
                else if comp.name == "Text" || comp.name == "Card" {
                    // Collect modifier-related props
                    let padding = comp.props.iter().find(|p| p.name == "padding");
                    let fill_max_width = comp.props.iter().find(|p| p.name == "fillMaxWidth");
                    let explicit_modifier = comp.props.iter().find(|p| p.name == "modifier");

                    // Build modifier chain if we have modifier-related props
                    if padding.is_some() || fill_max_width.is_some() || explicit_modifier.is_some() {
                        let mut modifiers = Vec::new();

                        // Add padding first (if present)
                        if let Some(pad_prop) = padding {
                            let pad_value = self.get_prop_expr(&pad_prop.value);
                            modifiers.push(format!(".padding({}.dp)", pad_value));
                        }

                        // Add fillMaxWidth if present (as boolean prop or variable)
                        if let Some(fw_prop) = fill_max_width {
                            let fw_value = self.get_prop_expr(&fw_prop.value).trim();
                            if fw_value == "true" {
                                modifiers.push(".fillMaxWidth()".to_string());
                            } else if fw_value == "false" {
                                // Skip - don't add modifier
                            } else {
                                // It's a variable - use .let { if ... }
                                modifiers.push(format!(".let {{ if ({}) it.fillMaxWidth() else it }}", fw_value));
                            }
                        }

                        // Add explicit modifier last (if present)
                        if let Some(mod_prop) = explicit_modifier {
                            let mod_value = self.get_prop_expr(&mod_prop.value);
                            let transformed = self.transform_ternary(mod_value);
                            modifiers.push(transformed);
                        }

                        // Combine into modifier parameter
                        if !modifiers.is_empty() {
                            if modifiers.len() == 1 && !modifiers[0].starts_with('.') {
                                // Single non-chained modifier (from explicit modifier with ternary)
                                params.push(format!("modifier = Modifier{}", modifiers[0]));
                            } else {
                                // Chain of modifiers
                                let modifier_chain = modifiers.iter()
                                    .map(|m| format!("            {}", m))
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                params.push(format!("modifier = Modifier\n{}", modifier_chain));
                            }
                        }
                    }

                    // Add other props (excluding the ones we handled)
                    for prop in &comp.props {
                        if prop.name == "padding" || prop.name == "fillMaxWidth" || prop.name == "modifier" {
                            continue; // Skip, already handled
                        }
                        let prop_expr = self.get_prop_expr(&prop.value);
                        let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                        params.extend(transformed);
                    }
                } else if comp.name == "Box" || comp.name == "AsyncImage" {
                    // Special handling for Box and AsyncImage with width/height/etc
                    // Collect special props as expression strings
                    let width = comp.props.iter().find(|p| p.name == "width")
                        .map(|p| self.get_prop_expr(&p.value));
                    let height = comp.props.iter().find(|p| p.name == "height")
                        .map(|p| self.get_prop_expr(&p.value));
                    let bg_color = comp.props.iter().find(|p| p.name == "backgroundColor")
                        .map(|p| self.get_prop_expr(&p.value));
                    let alignment = comp.props.iter().find(|p| p.name == "alignment")
                        .map(|p| self.get_prop_expr(&p.value));
                    let url = comp.props.iter().find(|p| p.name == "url")
                        .map(|p| self.get_prop_expr(&p.value));
                    let content_desc = comp.props.iter().find(|p| p.name == "contentDescription");

                    // Build modifier chain for Box
                    if comp.name == "Box" && (width.is_some() || height.is_some() || bg_color.is_some() || alignment.is_some()) {
                        let mut modifiers = Vec::new();

                        // Add size modifier if width/height present
                        if let (Some(w), Some(_h)) = (width, height) {
                            modifiers.push(format!(".size({}.dp)", w));
                        }

                        // Add background modifier
                        if let Some(color) = bg_color {
                            let color_str = if color.starts_with('"') && color.ends_with('"') {
                                let c = &color[1..color.len()-1];
                                format!("Color.{}", c.chars().next().unwrap().to_uppercase().collect::<String>() + &c[1..])
                            } else {
                                color.to_string()
                            };
                            modifiers.push(format!(".background({})", color_str));
                        }

                        // Add alignment modifier
                        if let Some(align) = alignment {
                            let align_str = if align.starts_with('"') && align.ends_with('"') {
                                let a = &align[1..align.len()-1];
                                format!("Alignment.{}", a.chars().next().unwrap().to_uppercase().collect::<String>() + &a[1..])
                            } else {
                                align.to_string()
                            };
                            modifiers.push(format!(".align({})", align_str));
                        }

                        // Combine into modifier parameter with proper indentation
                        if !modifiers.is_empty() {
                            let modifier_chain = modifiers.iter()
                                .map(|m| format!("                    {}", m))
                                .collect::<Vec<_>>()
                                .join("\n");
                            params.push(format!("modifier = Modifier\n{}", modifier_chain));
                        }
                    }

                    // Handle AsyncImage special props
                    if comp.name == "AsyncImage" {
                        let has_modifier = comp.props.iter().any(|p| p.name == "modifier");
                        let placeholder = comp.props.iter().find(|p| p.name == "placeholder")
                            .map(|p| self.get_prop_expr(&p.value));
                        let error = comp.props.iter().find(|p| p.name == "error")
                            .map(|p| self.get_prop_expr(&p.value));
                        let crossfade = comp.props.iter().find(|p| p.name == "crossfade")
                            .map(|p| self.get_prop_expr(&p.value));

                        // Only transform if there's NO explicit modifier prop
                        if !has_modifier {
                            // Check if we need ImageRequest.Builder pattern
                            let needs_builder = placeholder.is_some() || error.is_some() || crossfade.is_some();

                            if needs_builder && url.is_some() {
                                // Generate ImageRequest.Builder pattern
                                let mut builder_lines = vec![
                                    "model = ImageRequest.Builder(LocalContext.current)".to_string(),
                                    format!("            .data({})", url.unwrap()),
                                ];

                                if let Some(cf) = crossfade {
                                    builder_lines.push(format!("            .crossfade({})", cf));
                                }
                                if let Some(ph) = placeholder {
                                    let drawable = if ph.starts_with('"') && ph.ends_with('"') {
                                        let name = &ph[1..ph.len()-1];
                                        format!("R.drawable.{}", name)
                                    } else {
                                        ph.to_string()
                                    };
                                    builder_lines.push(format!("            .placeholder({})", drawable));
                                }
                                if let Some(err) = error {
                                    let drawable = if err.starts_with('"') && err.ends_with('"') {
                                        let name = &err[1..err.len()-1];
                                        format!("R.drawable.{}", name)
                                    } else {
                                        err.to_string()
                                    };
                                    builder_lines.push(format!("            .error({})", drawable));
                                }
                                builder_lines.push("            .build()".to_string());

                                params.push(builder_lines.join("\n"));

                                // Add contentDescription param (will be filled from props or default)
                                if let Some(cd) = content_desc {
                                    let cd_expr = self.get_prop_expr(&cd.value);
                                    params.push(format!("contentDescription = {}", cd_expr));
                                } else {
                                    // Don't add here, let it come from props naturally
                                }

                                // width/height → modifier = Modifier.size() (before contentScale)
                                if let (Some(w), Some(_h)) = (width, height) {
                                    params.push(format!("modifier = Modifier.size({}.dp)", w));
                                }

                                // Add contentScale for advanced images
                                params.push("contentScale = ContentScale.Crop".to_string());
                            } else {
                                // Simple url → model
                                if let Some(url_val) = url {
                                    params.push(format!("model = {}", url_val));
                                }

                                // Add contentDescription
                                if content_desc.is_none() {
                                    params.push("contentDescription = null".to_string());
                                }

                                // width/height → modifier = Modifier.size() (simple case)
                                if let (Some(w), Some(_h)) = (width, height) {
                                    params.push(format!("modifier = Modifier.size({}.dp)", w));
                                }
                            }
                        }
                    }

                    // Add other props (excluding the ones we handled)
                    let has_async_image_modifier = comp.name == "AsyncImage" &&
                        comp.props.iter().any(|p| p.name == "modifier");

                    for prop in &comp.props {
                        // Skip props we've already handled
                        if comp.name == "Box" && (prop.name == "width" || prop.name == "height" || prop.name == "backgroundColor" || prop.name == "alignment") {
                            continue; // Box props handled above
                        }
                        if comp.name == "AsyncImage" && !has_async_image_modifier &&
                            (prop.name == "url" || prop.name == "width" || prop.name == "height" ||
                             prop.name == "placeholder" || prop.name == "error" || prop.name == "crossfade" ||
                             prop.name == "contentDescription") {
                            continue; // AsyncImage props handled above (only if no explicit modifier)
                        }
                        let prop_expr = self.get_prop_expr(&prop.value);
                        let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                        params.extend(transformed);
                    }
                } else {
                    // Regular prop handling for other components
                    for prop in &comp.props {
                        let prop_expr = self.get_prop_expr(&prop.value);
                        let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                        params.extend(transformed);
                    }
                }

                // Determine if this component has a trailing lambda (children block)
                // Button with text prop also gets a trailing lambda
                let has_children = (!comp.children.is_empty() && comp.name != "Text")
                    || button_text.is_some();

                // Only add parens if we have params OR no trailing lambda
                if !params.is_empty() || !has_children {
                    output.push('(');

                    // If multiple params or any long param, use multiline format
                    if params.len() > 1 || params.iter().any(|p| p.len() > 40) {
                        output.push('\n');
                        for (i, param) in params.iter().enumerate() {
                            output.push_str(&format!("{}    {}", indent_str, param));
                            if i < params.len() - 1 {
                                output.push(',');
                            }
                            output.push('\n');
                        }
                        output.push_str(&indent_str);
                    } else if !params.is_empty() {
                        output.push_str(&params[0]);
                    }

                    output.push(')');
                }

                // Generate children if any (but not for Text, which uses children for text parameter)
                if has_children {
                    // Scaffold children need paddingValues lambda parameter only if first child is layout component
                    let scaffold_needs_padding = comp.name == "Scaffold" &&
                        comp.children.first().map_or(false, |child| {
                            matches!(child, Markup::Component(c) if c.name == "Column" || c.name == "Row" || c.name == "Box")
                        });

                    if scaffold_needs_padding {
                        output.push_str(" { paddingValues ->\n");
                    } else {
                        output.push_str(" {\n");
                    }

                    // If Button with text prop, generate Text child
                    if let Some(text) = button_text {
                        // Check if text is an R.string reference
                        if text.starts_with("R.string.") {
                            let transformed = self.transform_string_resource(&text);
                            output.push_str(&format!("{}    Text(text = {})\n", indent_str, transformed));
                        } else {
                            output.push_str(&format!("{}    Text(\"{}\")\n", indent_str, text));
                        }
                    }

                    // Generate regular children (pass component name as parent for context-aware generation)
                    for (i, child) in comp.children.iter().enumerate() {
                        // For Scaffold with layout child, mark first child to add paddingValues to modifier
                        if scaffold_needs_padding && i == 0 {
                            output.push_str(&self.generate_scaffold_child(child, indent + 1)?);
                        } else {
                            output.push_str(&self.generate_markup_with_context(child, indent + 1, Some(&comp.name))?);
                        }
                    }
                    output.push_str(&format!("{}}}\n", indent_str));
                } else {
                    output.push('\n');
                }

                Ok(output)
            }
            Markup::Text(text) => {
                let indent_str = "    ".repeat(indent);
                Ok(format!("{}Text(text = \"{}\")\n", indent_str, text))
            }
            Markup::Interpolation(expr) => {
                let indent_str = "    ".repeat(indent);
                Ok(format!("{}Text(text = ${})\n", indent_str, expr))
            }
            Markup::Sequence(items) => {
                let mut output = String::new();
                for item in items {
                    output.push_str(&self.generate_markup_with_indent(item, indent)?);
                }
                Ok(output)
            }
        }
    }

    fn resolve_import(&self, path: &str) -> String {
        // Resolve $ aliases to actual package paths
        if path.starts_with('$') {
            // $models -> com.example.app.models
            // $lib -> com.example.app.lib
            // $components -> com.example.app.components
            let rest = &path[1..]; // Remove $
            let base_package = self.package.rsplit('.').nth(1)
                .and_then(|_parent| self.package.strip_suffix(&format!(".{}", self.package.rsplit('.').next().unwrap_or(""))))
                .unwrap_or(&self.package);

            format!("{}.{}", base_package, rest.replace('.', "."))
        } else {
            path.to_string()
        }
    }

    fn collect_imports_recursive(
        &self,
        markup: &Markup,
        prop_imports: &mut Vec<String>,
        component_imports: &mut Vec<String>,
    ) {
        self.collect_imports_with_parent(markup, prop_imports, component_imports, None);
    }

    fn collect_imports_with_parent(
        &self,
        markup: &Markup,
        prop_imports: &mut Vec<String>,
        component_imports: &mut Vec<String>,
        parent_component: Option<&str>,
    ) {
        match markup {
            Markup::Component(comp) => {
                // Check if we need imports from props based on transformations
                for prop in &comp.props {
                    let prop_expr = self.get_prop_expr(&prop.value);

                    // Generic prop value checks
                    if prop_expr.contains("Modifier") {
                        let import = "androidx.compose.ui.Modifier".to_string();
                        if !prop_imports.contains(&import) {
                            prop_imports.push(import);
                        }
                    }
                    if prop_expr.contains("clickable") {
                        let import = "androidx.compose.foundation.clickable".to_string();
                        if !prop_imports.contains(&import) {
                            prop_imports.push(import);
                        }
                    }
                    if prop_expr.contains("R.string.") {
                        self.add_import_if_missing(prop_imports, "androidx.compose.ui.res.stringResource");
                        self.add_import_if_missing(component_imports, "com.example.app.R");
                    }

                    // Component-specific prop transformations that need imports
                    match (comp.name.as_str(), prop.name.as_str()) {
                        ("Column", "spacing") => {
                            // spacing → verticalArrangement = Arrangement.spacedBy(N.dp)
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.Arrangement");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        ("Column", "padding") => {
                            // padding → modifier = Modifier.padding(N.dp)
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.padding");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        ("Row", "spacing") => {
                            // spacing → horizontalArrangement = Arrangement.spacedBy(N.dp)
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.Arrangement");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        ("Row", "padding") => {
                            // padding → modifier = Modifier.padding(N.dp)
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.padding");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        ("Text", "fontSize") => {
                            // fontSize → fontSize = N.sp
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.sp");
                        }
                        ("Text", "fontWeight") => {
                            // fontWeight → FontWeight.Bold
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.text.font.FontWeight");
                        }
                        ("Text", "color") if prop_expr.starts_with('"') => {
                            // color string → MaterialTheme.colorScheme
                            self.add_import_if_missing(prop_imports, "androidx.compose.material3.MaterialTheme");
                        }
                        ("Card", "backgroundColor") => {
                            // backgroundColor → CardDefaults.cardColors()
                            self.add_import_if_missing(prop_imports, "androidx.compose.material3.CardDefaults");
                            self.add_import_if_missing(prop_imports, "androidx.compose.material3.MaterialTheme");
                        }
                        ("Box", "width") | ("Box", "height") => {
                            // width/height → Modifier.size()
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.size");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        ("Box", "backgroundColor") => {
                            // backgroundColor → .background(Color.Name)
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.background");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.graphics.Color");
                        }
                        ("Box", "alignment") => {
                            // alignment → .align(Alignment.Name)
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Alignment");
                        }
                        ("AsyncImage", "url") | ("AsyncImage", "width") | ("AsyncImage", "height") => {
                            // Only add imports if AsyncImage has no explicit modifier
                            let has_modifier = comp.props.iter().any(|p| p.name == "modifier");
                            if !has_modifier {
                                // url → model, width/height → Modifier.size()
                                self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                                self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.size");
                                self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                            }
                        }
                        ("AsyncImage", "placeholder") | ("AsyncImage", "error") | ("AsyncImage", "crossfade") => {
                            // ImageRequest.Builder pattern needed
                            let has_modifier = comp.props.iter().any(|p| p.name == "modifier");
                            if !has_modifier {
                                self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                                self.add_import_if_missing(prop_imports, "androidx.compose.ui.layout.ContentScale");
                                self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                                self.add_import_if_missing(prop_imports, "coil.request.ImageRequest");
                                self.add_import_if_missing(prop_imports, "androidx.compose.ui.platform.LocalContext");
                                self.add_import_if_missing(prop_imports, "coil.compose.rememberAsyncImagePainter");
                            }
                        }
                        ("LazyColumn", "spacing") => {
                            // spacing → verticalArrangement = Arrangement.spacedBy(N.dp)
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.Arrangement");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        ("LazyColumn", "padding") => {
                            // padding → contentPadding = PaddingValues(N.dp)
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.PaddingValues");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        ("Text", "padding") | ("Card", "padding") => {
                            // padding → modifier = Modifier.padding(N.dp)
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.padding");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        ("Text", "fillMaxWidth") | ("Card", "fillMaxWidth") => {
                            // fillMaxWidth → modifier chain with .fillMaxWidth()
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.fillMaxWidth");
                        }
                        ("Text", "modifier") | ("Card", "modifier") if prop_expr.contains("clickable") => {
                            // modifier with clickable
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.clickable");
                        }
                        _ => {}
                    }

                    // Recurse into PropValue::Markup to find nested components
                    if let PropValue::Markup(markup) = &prop.value {
                        self.collect_imports_recursive(markup, prop_imports, component_imports);
                    }
                }

                // Add imports for known components (after prop imports)
                match comp.name.as_str() {
                    "Text" => {
                        let import = "androidx.compose.material3.Text".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "Column" => {
                        let import = "androidx.compose.foundation.layout.Column".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "Card" => {
                        let import = "androidx.compose.material3.Card".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "TextField" => {
                        let import = "androidx.compose.material3.TextField".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "Button" => {
                        let import = "androidx.compose.material3.Button".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "Checkbox" => {
                        let import = "androidx.compose.material3.Checkbox".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "Switch" => {
                        let import = "androidx.compose.material3.Switch".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "Scaffold" => {
                        let import = "androidx.compose.material3.Scaffold".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "TopAppBar" => {
                        let import = "androidx.compose.material3.TopAppBar".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "Row" => {
                        let import = "androidx.compose.foundation.layout.Row".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "Icon" => {
                        let import = "androidx.compose.material3.Icon".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "Box" => {
                        let import = "androidx.compose.foundation.layout.Box".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "LazyColumn" => {
                        let import = "androidx.compose.foundation.lazy.LazyColumn".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                        // LazyColumn with ForLoop children needs items import
                        let has_for_loop = comp.children.iter().any(|child| matches!(child, Markup::ForLoop(_)));
                        if has_for_loop {
                            let items_import = "androidx.compose.foundation.lazy.items".to_string();
                            if !prop_imports.contains(&items_import) {
                                prop_imports.push(items_import);
                            }
                        }
                    }
                    "AsyncImage" => {
                        let import = "coil.compose.AsyncImage".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    _ => {}
                }

                // Recurse into children, passing component name as parent
                for child in &comp.children {
                    self.collect_imports_with_parent(child, prop_imports, component_imports, Some(&comp.name));
                }
            }
            Markup::Sequence(items) => {
                for item in items {
                    self.collect_imports_recursive(item, prop_imports, component_imports);
                }
            }
            Markup::IfElse(if_block) => {
                // Recurse into then branch
                for item in &if_block.then_branch {
                    self.collect_imports_recursive(item, prop_imports, component_imports);
                }
                // Recurse into else if branches
                for else_if in &if_block.else_ifs {
                    for item in &else_if.body {
                        self.collect_imports_recursive(item, prop_imports, component_imports);
                    }
                }
                // Recurse into else branch
                if let Some(else_branch) = &if_block.else_branch {
                    for item in else_branch {
                        self.collect_imports_recursive(item, prop_imports, component_imports);
                    }
                }
            }
            Markup::ForLoop(for_loop) => {
                // If loop has a key expression and is NOT inside LazyColumn, we need the key import
                // LazyColumn uses items(collection, key = {...}) which doesn't need the import
                if for_loop.key_expr.is_some() && parent_component != Some("LazyColumn") {
                    self.add_import_if_missing(prop_imports, "androidx.compose.runtime.key");
                }
                // Recurse into loop body
                for item in &for_loop.body {
                    self.collect_imports_with_parent(item, prop_imports, component_imports, parent_component);
                }
                // Recurse into empty block
                if let Some(empty_block) = &for_loop.empty_block {
                    for item in empty_block {
                        self.collect_imports_with_parent(item, prop_imports, component_imports, parent_component);
                    }
                }
            }
            Markup::When(when_block) => {
                // Recurse into each when branch body
                for branch in &when_block.branches {
                    self.collect_imports_recursive(&branch.body, prop_imports, component_imports);
                }
            }
            Markup::Interpolation(expr) => {
                // Check for R.string references
                if expr.contains("R.string.") {
                    self.add_import_if_missing(prop_imports, "androidx.compose.ui.res.stringResource");
                    self.add_import_if_missing(component_imports, "com.example.app.R");
                }
            }
            _ => {}
        }
    }

    fn build_text_expression(&self, children: &[Markup]) -> Result<String, String> {
        if children.is_empty() {
            return Ok("\"\"".to_string());
        }

        // Filter out whitespace-only Text nodes
        let non_whitespace_children: Vec<&Markup> = children.iter()
            .filter(|child| {
                !matches!(child, Markup::Text(t) if t.trim().is_empty())
            })
            .collect();

        // If single non-whitespace child
        if non_whitespace_children.len() == 1 {
            match non_whitespace_children[0] {
                Markup::Text(text) => return Ok(format!("\"{}\"", text)),
                // Single interpolation - use bare expression (no quotes, no $)
                Markup::Interpolation(expr) => {
                    let transformed = self.transform_string_resource(expr);
                    return Ok(self.add_null_assertions(&transformed));
                }
                _ => {}
            }
        }

        // Multiple children: build string template with interpolation
        let mut parts = Vec::new();
        for child in &non_whitespace_children {
            match child {
                Markup::Text(text) => {
                    // Escape dollar signs in literal text for Kotlin string templates
                    parts.push(self.escape_dollar_signs(text));
                }
                Markup::Interpolation(expr) => {
                    let str_res_transformed = self.transform_string_resource(expr);
                    let transformed = self.add_null_assertions(&str_res_transformed);
                    // Simple variable without property access doesn't need braces
                    if !transformed.contains('.') && !transformed.contains("!!") {
                        parts.push(format!("${}", transformed));
                    } else {
                        parts.push(format!("${{{}}}", transformed));
                    }
                }
                _ => return Err("Unexpected child in text".to_string()),
            }
        }

        Ok(format!("\"{}\"", parts.join("")))
    }

    fn add_null_assertions(&self, expr: &str) -> String {
        // If expr is like "user.name" and "user" is nullable, transform to "user!!.name"
        if let Some(dot_pos) = expr.find('.') {
            let var_name = &expr[..dot_pos];
            if self.nullable_vars.contains(var_name) {
                return format!("{}!!{}", var_name, &expr[dot_pos..]);
            }
        }
        // If expr is just a bare nullable variable, add !! as well
        let trimmed = expr.trim();
        if self.nullable_vars.contains(trimmed) {
            return format!("{}!!", trimmed);
        }
        expr.to_string()
    }

    fn add_import_if_missing(&self, imports: &mut Vec<String>, import: &str) {
        let import_str = import.to_string();
        if !imports.contains(&import_str) {
            imports.push(import_str);
        }
    }

    fn transform_ternary(&self, expr: &str) -> String {
        // Transform ternary operator: condition ? value : value
        // To Kotlin: .let { if (condition) value else value }

        // Find ? and : at the same depth level
        let mut depth = 0;
        let mut question_pos = None;
        let mut colon_pos = None;

        for (i, ch) in expr.char_indices() {
            match ch {
                '(' | '{' | '[' => depth += 1,
                ')' | '}' | ']' => depth -= 1,
                '?' if depth == 0 && question_pos.is_none() => question_pos = Some(i),
                ':' if depth == 0 && question_pos.is_some() && colon_pos.is_none() => colon_pos = Some(i),
                _ => {}
            }
        }

        if let (Some(q), Some(c)) = (question_pos, colon_pos) {
            let condition = expr[..q].trim();
            let mut then_value = expr[q+1..c].trim().to_string();
            let mut else_value = expr[c+1..].trim().to_string();

            // Replace Modifier. with it. in the values for chaining
            if then_value.starts_with("Modifier.") {
                then_value = then_value.replace("Modifier.", "it.");
            }
            if else_value.starts_with("Modifier.") {
                else_value = else_value.replace("Modifier.", "it.");
            } else if else_value == "Modifier" {
                else_value = "it".to_string();
            }

            format!(".let {{ if ({}) {} else {} }}", condition, then_value, else_value)
        } else {
            expr.to_string()
        }
    }

    fn transform_prop(&self, component: &str, prop_name: &str, prop_value: &str) -> Vec<String> {
        // Handle bind:value special syntax
        if prop_name == "bind:value" {
            let var_name = prop_value.trim();

            // Check if this variable has a numeric type
            if let Some((type_str, default_value)) = self.var_types.get(var_name) {
                if self.is_numeric_type(type_str) {
                    // Numeric bind:value needs type conversions
                    let (to_method, default) = self.get_numeric_conversion(type_str, default_value);
                    return vec![
                        format!("value = {}.toString()", var_name),
                        format!("onValueChange = {{ {} = it.{} ?: {} }}", var_name, to_method, default),
                    ];
                }
            }

            // Default bind:value (for String types)
            return vec![
                format!("value = {}", var_name),
                format!("onValueChange = {{ {} = it }}", var_name),
            ];
        }

        // Handle bind:checked special syntax (for Checkbox, Switch)
        if prop_name == "bind:checked" {
            let var_name = prop_value.trim();
            return vec![
                format!("checked = {}", var_name),
                format!("onCheckedChange = {{ {} = it }}", var_name),
            ];
        }

        // Transform route aliases first: $routes → Routes (before adding braces)
        let value = self.transform_route_aliases(prop_value);

        // Transform lambda arrow syntax: () => to {}
        let value = self.transform_lambda_arrow(&value);

        // Component-specific transformations
        match (component, prop_name) {
            // TextField label → label = { Text("...") }
            ("TextField", "label") => {
                let label_text = if value.starts_with('"') && value.ends_with('"') {
                    value[1..value.len()-1].to_string()
                } else {
                    value
                };
                vec![format!("label = {{ Text(\"{}\") }}", label_text)]
            }
            // TextField placeholder → placeholder = { Text("...") }
            ("TextField", "placeholder") => {
                let placeholder_text = if value.starts_with('"') && value.ends_with('"') {
                    value[1..value.len()-1].to_string()
                } else {
                    value
                };
                vec![format!("placeholder = {{ Text(\"{}\") }}", placeholder_text)]
            }
            // Button text is handled differently - it becomes a child, not a prop
            ("Button", "text") => {
                // Return empty vec - text will be handled as child in generate_markup
                vec![]
            }
            // Button onClick needs braces
            ("Button", "onClick") => {
                if !value.starts_with('{') {
                    vec![format!("onClick = {{ {}() }}", value)]
                } else {
                    vec![format!("onClick = {}", value)]
                }
            }
            // Column spacing → verticalArrangement = Arrangement.spacedBy(N.dp)
            ("Column", "spacing") => {
                vec![format!("verticalArrangement = Arrangement.spacedBy({}.dp)", value)]
            }
            // Column padding → modifier = Modifier.padding(N.dp)
            ("Column", "padding") => {
                vec![format!("modifier = Modifier.padding({}.dp)", value)]
            }
            // LazyColumn spacing → verticalArrangement = Arrangement.spacedBy(N.dp)
            ("LazyColumn", "spacing") => {
                vec![format!("verticalArrangement = Arrangement.spacedBy({}.dp)", value)]
            }
            // LazyColumn padding → contentPadding = PaddingValues(N.dp)
            ("LazyColumn", "padding") => {
                vec![format!("contentPadding = PaddingValues({}.dp)", value)]
            }
            // Row spacing → horizontalArrangement = Arrangement.spacedBy(N.dp)
            ("Row", "spacing") => {
                vec![format!("horizontalArrangement = Arrangement.spacedBy({}.dp)", value)]
            }
            // Row padding → modifier = Modifier.padding(N.dp)
            ("Row", "padding") => {
                vec![format!("modifier = Modifier.padding({}.dp)", value)]
            }
            // Text fontSize → fontSize = N.sp
            ("Text", "fontSize") => {
                vec![format!("fontSize = {}.sp", value)]
            }
            // Text fontWeight string → FontWeight enum
            ("Text", "fontWeight") => {
                let weight = if value.starts_with('"') && value.ends_with('"') {
                    // String literal "bold" → FontWeight.Bold
                    let s = &value[1..value.len()-1];
                    format!("FontWeight.{}", s.chars().next().unwrap().to_uppercase().collect::<String>() + &s[1..])
                } else {
                    value
                };
                vec![format!("fontWeight = {}", weight)]
            }
            // Text color string → MaterialTheme.colorScheme
            ("Text", "color") => {
                let color = if value.starts_with('"') && value.ends_with('"') {
                    // String literal "secondary" → MaterialTheme.colorScheme.secondary
                    let s = &value[1..value.len()-1];
                    format!("MaterialTheme.colorScheme.{}", s)
                } else {
                    value
                };
                vec![format!("color = {}", color)]
            }
            // Card onClick → just transform the value
            ("Card", "onClick") => {
                vec![format!("onClick = {}", value)]
            }
            // Card backgroundColor → CardDefaults.cardColors()
            ("Card", "backgroundColor") => {
                // value is a string like "errorContainer", "primaryContainer", etc.
                let color_name = if value.starts_with('"') && value.ends_with('"') {
                    &value[1..value.len()-1]
                } else {
                    value.as_str()
                };
                vec![format!(
                    "colors = CardDefaults.cardColors(\n                    containerColor = MaterialTheme.colorScheme.{}\n                )",
                    color_name
                )]
            }
            // Default: no transformation
            _ => {
                vec![format!("{} = {}", prop_name, value)]
            }
        }
    }

    fn transform_lambda_arrow(&self, value: &str) -> String {
        // Transform () => expr to { expr }
        if value.contains("() =>") {
            let transformed = value.replace("() =>", "").trim().to_string();
            // Wrap in braces
            format!("{{ {} }}", transformed)
        } else {
            value.to_string()
        }
    }

    fn transform_route_aliases(&self, value: &str) -> String {
        // Transform $routes.foo.bar(params) to Routes.Foo.Bar(params)
        if let Some(routes_pos) = value.find("$routes.") {
            // Find the extent of the route path
            let after_routes = &value[routes_pos + 8..]; // Skip "$routes."

            // Find where route path ends (next '(' or whitespace or end)
            let mut route_end = after_routes.len();
            for (i, ch) in after_routes.chars().enumerate() {
                if ch == '(' || ch.is_whitespace() || ch == ')' || ch == ',' {
                    route_end = i;
                    break;
                }
            }

            let route_path = &after_routes[..route_end];
            let before = &value[..routes_pos];
            let after = &after_routes[route_end..];

            // Capitalize route segments: post.detail → Post.Detail
            let parts: Vec<&str> = route_path.split('.').collect();
            let capitalized: Vec<String> = parts.iter().map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => part.to_string(),
                }
            }).collect();

            format!("{}Routes.{}{}", before, capitalized.join("."), after)
        } else {
            value.to_string()
        }
    }

    fn extract_route_params(&self, file: &WhitehallFile) -> Vec<String> {
        let mut params = std::collections::HashSet::new();

        // Scan lifecycle hooks for $screen.params.{name}
        for hook in &file.lifecycle_hooks {
            self.extract_params_from_text(&hook.body, &mut params);
        }

        // Scan function bodies
        for func in &file.functions {
            self.extract_params_from_text(&func.body, &mut params);
        }

        // Scan state initial values
        for state in &file.state {
            self.extract_params_from_text(&state.initial_value, &mut params);
        }

        let mut param_vec: Vec<String> = params.into_iter().collect();
        param_vec.sort();
        param_vec
    }

    fn extract_params_from_text(&self, text: &str, params: &mut std::collections::HashSet<String>) {
        // Find all $screen.params.{name} patterns
        let pattern = "$screen.params.";
        for part in text.split(pattern).skip(1) {
            // Extract the param name (until non-alphanumeric)
            let param_name: String = part.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !param_name.is_empty() {
                params.insert(param_name);
            }
        }
    }

    fn format_multiline_value(&self, value: &str) -> String {
        // Check if value contains newlines
        if !value.contains('\n') {
            return value.to_string();
        }

        // Format multi-line values with proper indentation
        let lines: Vec<&str> = value.lines().collect();
        if lines.len() <= 1 {
            return value.to_string();
        }

        let mut result = String::new();
        result.push_str(lines[0]);
        result.push('\n');

        for (i, line) in lines.iter().enumerate().skip(1) {
            result.push_str(&self.indent());
            result.push_str("    "); // Additional indent for continuation
            result.push_str(line.trim());
            if i < lines.len() - 1 {
                result.push('\n');
            }
        }

        result
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    /// Helper to extract expression string from PropValue
    /// Returns the expression string for Expression variant
    /// For Markup variant, returns empty string (should be handled specially)
    fn get_prop_expr<'a>(&self, prop_value: &'a PropValue) -> &'a str {
        match prop_value {
            PropValue::Expression(expr) => expr,
            PropValue::Markup(_) => {
                // Component-as-prop-value should be handled specially at call site
                ""
            }
        }
    }

    /// Infer type from initial value
    /// Returns type string or "Unknown" if cannot infer
    fn infer_type_from_value(&self, value: &str) -> String {
        let trimmed = value.trim();

        // Check for integer literals
        if trimmed.parse::<i32>().is_ok() {
            return "Int".to_string();
        }

        // Check for double/float literals
        if trimmed.parse::<f64>().is_ok() {
            return "Double".to_string();
        }

        // Check for boolean literals
        if trimmed == "true" || trimmed == "false" {
            return "Boolean".to_string();
        }

        // Check for string literals
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            return "String".to_string();
        }

        // Default to unknown
        "Unknown".to_string()
    }

    /// Check if a type is numeric (Int, Double, Float, Long)
    fn is_numeric_type(&self, type_str: &str) -> bool {
        matches!(
            type_str,
            "Int" | "Double" | "Float" | "Long" | "Short" | "Byte"
        )
    }

    /// Get the appropriate conversion method for a numeric type
    /// Returns (to_method, default_value) e.g., ("toIntOrNull()", "0")
    fn get_numeric_conversion(&self, type_str: &str, default_value: &str) -> (String, String) {
        let to_method = match type_str {
            "Int" => "toIntOrNull()",
            "Double" => "toDoubleOrNull()",
            "Float" => "toFloatOrNull()",
            "Long" => "toLongOrNull()",
            _ => "toIntOrNull()", // Default to Int
        };

        (to_method.to_string(), default_value.to_string())
    }

    /// Escape dollar signs in literal text for Kotlin string templates
    /// Converts $ to \$ so it's treated as literal in Kotlin strings
    fn escape_dollar_signs(&self, text: &str) -> String {
        text.replace('$', "\\$")
    }

    /// Transform R.string references to stringResource() calls
    /// - R.string.xxx → stringResource(R.string.xxx)
    /// - R.string.xxx(args) → stringResource(R.string.xxx, args)
    fn transform_string_resource(&self, expr: &str) -> String {
        let trimmed = expr.trim();

        // Check if this is an R.string reference
        if !trimmed.starts_with("R.string.") {
            return expr.to_string();
        }

        // Find if there are function call parentheses
        if let Some(paren_pos) = trimmed.find('(') {
            // R.string.greeting(userName) → stringResource(R.string.greeting, userName)
            let resource_name = &trimmed[..paren_pos];
            let args_with_parens = &trimmed[paren_pos..];

            // Extract args from parentheses
            if args_with_parens.len() > 2 {
                let args = &args_with_parens[1..args_with_parens.len()-1]; // Remove ( and )
                if args.is_empty() {
                    // No args: R.string.xxx() → stringResource(R.string.xxx)
                    format!("stringResource({})", resource_name)
                } else {
                    // With args: R.string.xxx(a, b) → stringResource(R.string.xxx, a, b)
                    format!("stringResource({}, {})", resource_name, args)
                }
            } else {
                // Malformed, just wrap it
                format!("stringResource({})", trimmed)
            }
        } else {
            // Simple reference: R.string.xxx → stringResource(R.string.xxx)
            format!("stringResource({})", trimmed)
        }
    }

    /// Generate Scaffold's first child with .padding(paddingValues) prepended to modifier
    fn generate_scaffold_child(&self, markup: &Markup, indent: usize) -> Result<String, String> {
        // Only layout containers (Column, Row, Box) should get paddingValues
        if let Markup::Component(comp) = markup {
            if comp.name == "Column" || comp.name == "Row" || comp.name == "Box" {
                // Generate the component but with paddingValues prepended to modifier
                // This requires special handling - we'll temporarily modify component
                let mut modified_comp = comp.clone();

                // Build the modifier chain: paddingValues first, then layout padding if present
                let mut modifier_parts = vec![".padding(paddingValues)".to_string()];

                // Find padding prop and incorporate it into modifier
                let padding_idx = modified_comp.props.iter().position(|p| p.name == "padding");
                if let Some(idx) = padding_idx {
                    let pad_value = self.get_prop_expr(&modified_comp.props[idx].value);
                    modifier_parts.push(format!(".padding({}.dp)", pad_value));
                    // Remove padding prop so it doesn't get processed again
                    modified_comp.props.remove(idx);
                }

                // Check if there's already an explicit modifier prop
                if let Some(mod_prop_idx) = modified_comp.props.iter().position(|p| p.name == "modifier") {
                    // Modifier exists - prepend paddingValues to it
                    let existing = &modified_comp.props[mod_prop_idx].value;
                    let existing_expr = self.get_prop_expr(existing);
                    let new_modifier = if existing_expr.starts_with("Modifier") {
                        let rest = &existing_expr[8..]; // Skip "Modifier"
                        format!("Modifier\n                {}{}", modifier_parts.join("\n                "), rest)
                    } else {
                        format!("Modifier{}.then({})", modifier_parts.join(""), existing_expr)
                    };
                    modified_comp.props[mod_prop_idx].value = PropValue::Expression(new_modifier);
                } else {
                    // No modifier - create one and insert at beginning so it comes first
                    let modifier_str = format!("Modifier\n                {}", modifier_parts.join("\n                "));
                    modified_comp.props.insert(0, crate::transpiler::ast::ComponentProp {
                        name: "modifier".to_string(),
                        value: PropValue::Expression(modifier_str),
                    });
                }

                return self.generate_markup_with_indent(&Markup::Component(modified_comp), indent);
            }
        }

        // Not a layout component - generate normally
        self.generate_markup_with_indent(markup, indent)
    }
}
