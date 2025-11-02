/// Code generator for Kotlin/Compose output

use crate::transpiler::ast::{Markup, WhitehallFile};

pub struct CodeGenerator {
    package: String,
    component_name: String,
    component_type: Option<String>,
    indent_level: usize,
}

impl CodeGenerator {
    pub fn new(package: &str, component_name: &str, component_type: Option<&str>) -> Self {
        CodeGenerator {
            package: package.to_string(),
            component_name: component_name.to_string(),
            component_type: component_type.map(|s| s.to_string()),
            indent_level: 0,
        }
    }

    pub fn generate(&mut self, file: &WhitehallFile) -> Result<String, String> {
        let mut output = String::new();

        // Package declaration
        output.push_str(&format!("package {}\n\n", self.package));

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

        // Add kotlinx.coroutines.launch if there are lifecycle hooks
        if !file.lifecycle_hooks.is_empty() {
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
            output.push_str("navController: NavController");
            if !file.props.is_empty() {
                output.push_str(", ");
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

        // Generate state declarations
        for state in &file.state {
            output.push_str(&self.indent());
            if state.mutable {
                if let Some(ref type_ann) = state.type_annotation {
                    // With type annotation: var name by remember { mutableStateOf<Type>(value) }
                    output.push_str(&format!(
                        "var {} by remember {{ mutableStateOf<{}>({}) }}\n",
                        state.name, type_ann, state.initial_value
                    ));
                } else {
                    // Without type annotation: var name by remember { mutableStateOf(value) }
                    output.push_str(&format!(
                        "var {} by remember {{ mutableStateOf({}) }}\n",
                        state.name, state.initial_value
                    ));
                }
            } else {
                output.push_str(&format!("val {} = {}\n", state.name, state.initial_value));
            }
        }

        if !file.state.is_empty() {
            output.push('\n');
        }

        // Generate coroutineScope if there are lifecycle hooks
        if !file.lifecycle_hooks.is_empty() {
            output.push_str(&self.indent());
            output.push_str("val coroutineScope = rememberCoroutineScope()\n\n");
        }

        // Generate lifecycle hooks
        for hook in &file.lifecycle_hooks {
            output.push_str(&self.indent());
            match hook.hook_type.as_str() {
                "onMount" => {
                    output.push_str("LaunchedEffect(Unit) {\n");
                    // Indent hook body and add coroutineScope prefix to launch calls
                    for line in hook.body.lines() {
                        output.push_str(&self.indent());
                        output.push_str("    ");
                        // Add coroutineScope. prefix to launch calls
                        let transformed_line = line.trim();
                        if transformed_line.starts_with("launch ") || transformed_line.starts_with("launch{") {
                            output.push_str("coroutineScope.");
                        }
                        output.push_str(line);
                        output.push('\n');
                    }
                    output.push_str(&self.indent());
                    output.push_str("}\n\n");
                }
                _ => {
                    // Other hooks can be added later
                }
            }
        }

        // Generate function declarations
        for func in &file.functions {
            output.push_str(&self.indent());
            output.push_str(&format!("fun {}() {{\n", func.name));
            // Output function body with proper indentation and transformations
            for line in func.body.lines() {
                output.push_str(&self.indent());
                output.push_str("    ");

                // Transform $routes.login → Routes.Login
                let mut transformed_line = self.transform_route_aliases(line);

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
                            if p.value.starts_with('"') && p.value.ends_with('"') {
                                p.value[1..p.value.len()-1].to_string()
                            } else {
                                p.value.clone()
                            }
                        })
                } else {
                    None
                };

                // Add component props (with transformations)
                for prop in &comp.props {
                    let transformed = self.transform_prop(&comp.name, &prop.name, &prop.value);
                    params.extend(transformed);
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
                    output.push_str(" {\n");

                    // If Button with text prop, generate Text child
                    if let Some(text) = button_text {
                        output.push_str(&format!("{}    Text(\"{}\")\n", indent_str, text));
                    }

                    // Generate regular children
                    for child in &comp.children {
                        output.push_str(&self.generate_markup_with_indent(child, indent + 1)?);
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
                .and_then(|parent| self.package.strip_suffix(&format!(".{}", self.package.rsplit('.').next().unwrap_or(""))))
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
        match markup {
            Markup::Component(comp) => {
                // Check if we need imports from props based on transformations
                for prop in &comp.props {
                    // Generic prop value checks
                    if prop.value.contains("Modifier") {
                        let import = "androidx.compose.ui.Modifier".to_string();
                        if !prop_imports.contains(&import) {
                            prop_imports.push(import);
                        }
                    }
                    if prop.value.contains("clickable") {
                        let import = "androidx.compose.foundation.clickable".to_string();
                        if !prop_imports.contains(&import) {
                            prop_imports.push(import);
                        }
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
                        ("Text", "fontSize") => {
                            // fontSize → fontSize = N.sp
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.sp");
                        }
                        ("Text", "fontWeight") => {
                            // fontWeight → FontWeight.Bold
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.text.font.FontWeight");
                        }
                        ("Text", "color") if prop.value.starts_with('"') => {
                            // color string → MaterialTheme.colorScheme
                            self.add_import_if_missing(prop_imports, "androidx.compose.material3.MaterialTheme");
                        }
                        _ => {}
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
                    "AsyncImage" => {
                        let import = "coil.compose.AsyncImage".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    _ => {}
                }

                // Recurse into children
                for child in &comp.children {
                    self.collect_imports_recursive(child, prop_imports, component_imports);
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
                // Recurse into loop body
                for item in &for_loop.body {
                    self.collect_imports_recursive(item, prop_imports, component_imports);
                }
                // Recurse into empty block
                if let Some(empty_block) = &for_loop.empty_block {
                    for item in empty_block {
                        self.collect_imports_recursive(item, prop_imports, component_imports);
                    }
                }
            }
            Markup::When(when_block) => {
                // Recurse into each when branch body
                for branch in &when_block.branches {
                    self.collect_imports_recursive(&branch.body, prop_imports, component_imports);
                }
            }
            _ => {}
        }
    }

    fn build_text_expression(&self, children: &[Markup]) -> Result<String, String> {
        if children.is_empty() {
            return Ok("\"\"".to_string());
        }

        // If single child
        if children.len() == 1 {
            match &children[0] {
                Markup::Text(text) => return Ok(format!("\"{}\"", text)),
                // Single interpolation - use bare expression (no quotes, no $)
                Markup::Interpolation(expr) => return Ok(expr.to_string()),
                _ => {}
            }
        }

        // Multiple children: build string template with interpolation
        let mut parts = Vec::new();
        for child in children {
            match child {
                Markup::Text(text) => parts.push(text.to_string()),
                Markup::Interpolation(expr) => parts.push(format!("${}", expr)),
                _ => return Err("Unexpected child in text".to_string()),
            }
        }

        Ok(format!("\"{}\"", parts.join("")))
    }

    fn add_import_if_missing(&self, imports: &mut Vec<String>, import: &str) {
        let import_str = import.to_string();
        if !imports.contains(&import_str) {
            imports.push(import_str);
        }
    }

    fn transform_prop(&self, component: &str, prop_name: &str, prop_value: &str) -> Vec<String> {
        // Handle bind:value special syntax
        if prop_name == "bind:value" {
            let var_name = prop_value.trim();
            return vec![
                format!("value = {}", var_name),
                format!("onValueChange = {{ {} = it }}", var_name),
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

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }
}
