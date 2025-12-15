/// Compose backend - generates Jetpack Compose code

use crate::transpiler::analyzer::StoreRegistry;
use crate::transpiler::ast::{ClassDeclaration, Component, ForLoopBlock, Markup, PropValue, WhitehallFile};
use crate::transpiler::optimizer::Optimization;

pub struct ComposeBackend {
    package: String,
    component_name: String,
    component_type: Option<String>,
    indent_level: usize,
    nullable_vars: std::collections::HashSet<String>,
    var_types: std::collections::HashMap<String, (String, String)>, // Maps variable name to (type, default_value)
    optimizations: Vec<Optimization>, // Phase 6: Optimization plans for this component
    store_registry: Option<StoreRegistry>, // Phase 2: Store registry for @store detection
    uses_viewmodel: bool, // Phase 2: Track if any stores are used (for imports)
    uses_hilt_viewmodel: bool, // Phase 2: Track if any Hilt stores are used (for imports)
    uses_dispatchers: bool, // Phase 2: Track if dispatcher syntax is used (io/cpu/main)
    uses_experimental_material3: bool, // Track if experimental Material3 APIs are used (DropdownMenu, etc.)
    uses_fetch: bool, // Track if $fetch() API is used (for Ktor imports)
    uses_routes: bool, // Track if $routes or $navigate is used (for Routes import)
    // Phase 1.1: ViewModel wrapper context
    in_viewmodel_wrapper: bool, // Are we generating markup inside a ViewModel wrapper?
    mutable_vars: std::collections::HashSet<String>, // Mutable vars (need uiState prefix)
    derived_props: std::collections::HashSet<String>, // Derived properties (need viewModel prefix)
    function_names: std::collections::HashSet<String>, // Functions (need viewModel prefix)
}

/// Convert hex color string to Color(0x...) format
/// Supports: #RGB, #RRGGBB, #RRGGBBAA (web RGBA format - alpha at end)
/// Note: Converts #RRGGBBAA (RGBA) to 0xAARRGGBB (ARGB) for Android
/// Returns an error if the hex format is invalid
fn convert_hex_to_color(hex: &str) -> Result<String, String> {
    let hex_clean = hex.trim();

    // Validate that all characters are valid hex digits
    if !hex_clean.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("Invalid hex color '{}': contains non-hexadecimal characters", hex));
    }

    match hex_clean.len() {
        3 => {
            // #RGB → #RRGGBB with full alpha
            let r = hex_clean.chars().nth(0).unwrap();
            let g = hex_clean.chars().nth(1).unwrap();
            let b = hex_clean.chars().nth(2).unwrap();
            Ok(format!("Color(0xFF{r}{r}{g}{g}{b}{b})"))
        }
        6 => {
            // #RRGGBB → add full alpha
            Ok(format!("Color(0xFF{})", hex_clean.to_uppercase()))
        }
        8 => {
            // #RRGGBBAA (web/CSS RGBA format) → 0xAARRGGBB (Android ARGB format)
            // Extract components: RR GG BB AA
            let rr = &hex_clean[0..2];
            let gg = &hex_clean[2..4];
            let bb = &hex_clean[4..6];
            let aa = &hex_clean[6..8];
            // Reorder to ARGB: AA RR GG BB
            Ok(format!("Color(0x{}{}{}{})", aa.to_uppercase(), rr.to_uppercase(), gg.to_uppercase(), bb.to_uppercase()))
        }
        _ => {
            // Invalid format length
            Err(format!(
                "Invalid hex color '{}': expected 3, 6, or 8 characters (e.g., #RGB, #RRGGBB, #RRGGBBAA), got {}",
                hex, hex_clean.len()
            ))
        }
    }
}

/// Configuration for which modifiers to build in the unified modifier builder
#[derive(Default)]
struct ModifierConfig {
    /// Handle width/height props (100% → fillMax*, fixed → .width()/.height())
    handle_size: bool,
    /// Handle padding/p/px/py/pt/pb/pl/pr props
    handle_padding: bool,
    /// Handle backgroundColor prop
    handle_background: bool,
    /// Handle onClick as clickable modifier (for components without native onClick)
    handle_click_as_modifier: bool,
    /// Handle fillMaxWidth/fillMaxHeight/fillMaxSize props
    handle_fill_max: bool,
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
            optimizations: Vec::new(), // Phase 6: Start with empty optimizations
            store_registry: None, // Phase 2: Will be set by generate_with_optimizations
            uses_viewmodel: false, // Phase 2: Track store usage for imports
            uses_hilt_viewmodel: false, // Phase 2: Track Hilt store usage for imports
            uses_dispatchers: false, // Phase 2: Track dispatcher syntax usage
            uses_experimental_material3: false, // Track experimental Material3 API usage
            uses_fetch: false, // Track $fetch() API usage
            uses_routes: false, // Track $routes/$navigate usage for Routes import
            in_viewmodel_wrapper: false, // Phase 1.1: Not in ViewModel wrapper by default
            mutable_vars: std::collections::HashSet::new(), // Phase 1.1: Track mutable vars
            derived_props: std::collections::HashSet::new(), // Phase 1.1: Track derived properties
            function_names: std::collections::HashSet::new(), // Phase 1.1: Track functions
        }
    }

    /// Phase 6: Generate with optimization support
    ///
    /// This method receives optimization plans and routes for loops accordingly:
    /// - RecyclerView optimization: Uses RecyclerViewGenerator + ViewBackend
    /// - Default: Standard Compose generation
    pub fn generate_with_optimizations(
        &mut self,
        file: &WhitehallFile,
        optimizations: &[crate::transpiler::optimizer::Optimization],
        semantic_info: &crate::transpiler::analyzer::SemanticInfo,
    ) -> Result<crate::transpiler::TranspileResult, String> {
        // Phase 6: Store optimizations for use during for loop generation
        self.optimizations = optimizations.to_vec();

        // Phase 2: Store registry for @store detection
        self.store_registry = Some(semantic_info.store_registry.clone());

        // Generate code - for loop generation will check optimizations
        self.generate(file)
    }

    /// Check if a value is a store instantiation (e.g., "CounterStore()")
    /// Returns the store info if it matches a registered @store class
    fn detect_store_instantiation(&self, value: &str) -> Option<crate::transpiler::analyzer::StoreInfo> {
        // Check if value matches pattern: ClassName() or ClassName(...)
        let trimmed = value.trim();
        if !trimmed.ends_with(')') {
            return None;
        }

        // Extract class name before '('
        if let Some(paren_pos) = trimmed.find('(') {
            let class_name = trimmed[..paren_pos].trim();

            // Check if it's in the store registry
            if let Some(ref registry) = self.store_registry {
                return registry.get(class_name).cloned();
            }
        }

        None
    }

    /// Pre-pass: Detect if the file uses any stores
    /// Sets the uses_viewmodel and uses_hilt_viewmodel flags for import generation
    fn detect_store_usage(&mut self, file: &WhitehallFile) {
        for state in &file.state {
            let mut transformed_value = self.transform_array_literal(&state.initial_value, false);
            transformed_value = self.transform_range_literal(&transformed_value);
            if let Some(store_info) = self.detect_store_instantiation(&transformed_value) {
                self.uses_viewmodel = true;
                if store_info.has_hilt {
                    self.uses_hilt_viewmodel = true;
                }
            }
        }
    }

    /// Pre-pass: Detect if the file uses dispatcher syntax (io/cpu/main)
    /// Sets the uses_dispatchers flag for scope generation
    fn detect_dispatcher_usage(&mut self, file: &WhitehallFile) {
        // Check markup for dispatcher patterns
        let markup_str = format!("{:?}", file.markup);
        if markup_str.contains("io {") || markup_str.contains("cpu {") || markup_str.contains("main {") {
            self.uses_dispatchers = true;
        }
    }

    pub fn generate(&mut self, file: &WhitehallFile) -> Result<crate::transpiler::TranspileResult, String> {
        // Check if this file contains a reactive class (in store registry)
        // This includes: classes with var properties OR @store object singletons
        let store_class = file.classes.iter().find(|c| {
            // Check if class is in store registry
            if let Some(registry) = &self.store_registry {
                registry.contains(&c.name)
            } else {
                false
            }
        });

        if let Some(class) = store_class {
            // Generate ViewModel or singleton code based on registry info
            let kotlin_code = self.generate_store_class(file, class)?;
            return Ok(crate::transpiler::TranspileResult::Single(kotlin_code));
        }

        // Check if this component has inline vars (ComponentInline in registry)
        let is_component_viewmodel = if let Some(registry) = &self.store_registry {
            registry.get(&self.component_name)
                .map(|info| info.source == crate::transpiler::analyzer::StoreSource::ComponentInline)
                .unwrap_or(false)
        } else {
            false
        };

        if is_component_viewmodel {
            // Pre-pass: Detect $routes/$navigate usage (for Routes import in wrapper)
            self.detect_routes_usage(file);
            // Component has inline vars → Generate ViewModel + wrapper component
            return self.generate_component_viewmodel(file);
        }

        // Pre-pass: Detect store usage for import generation
        self.detect_store_usage(file);

        // Pre-pass: Detect dispatcher usage (io/cpu/main)
        self.detect_dispatcher_usage(file);

        // Pre-pass: Detect $fetch() API usage
        self.detect_fetch_usage(file);

        // Pre-pass: Detect $routes/$navigate usage (for Routes import)
        self.detect_routes_usage(file);

        // Otherwise, generate standard Composable component
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

        // Check if experimental Material3 APIs are used (DropdownMenu → ExposedDropdownMenuBox)
        if component_imports.iter().any(|imp| imp.contains("ExposedDropdownMenuBox")) {
            self.uses_experimental_material3 = true;
        }

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

        // Add ViewModel imports for @store usage
        if self.uses_viewmodel {
            imports.push("androidx.lifecycle.viewmodel.compose.viewModel".to_string());
        }
        if self.uses_hilt_viewmodel {
            imports.push("androidx.lifecycle.viewmodel.compose.hiltViewModel".to_string());
        }

        // Add ExperimentalMaterial3Api import if using experimental APIs (DropdownMenu, etc.)
        if self.uses_experimental_material3 {
            imports.push("androidx.compose.material3.ExperimentalMaterial3Api".to_string());
        }

        // Add Ktor imports for $fetch() API usage
        if self.uses_fetch {
            imports.push("io.ktor.client.HttpClient".to_string());
            imports.push("io.ktor.client.call.body".to_string());
            imports.push("io.ktor.client.engine.okhttp.OkHttp".to_string());
            imports.push("io.ktor.client.plugins.contentnegotiation.ContentNegotiation".to_string());
            imports.push("io.ktor.client.request.get".to_string());
            imports.push("io.ktor.serialization.kotlinx.json.json".to_string());
            imports.push("kotlinx.serialization.json.Json".to_string());
        }

        // Add Routes import if $routes or $navigate is used (for screens)
        if self.uses_routes && self.component_type.as_deref() == Some("screen") {
            // Extract base package (e.g., com.example.app.screens -> com.example.app)
            let base_package = self.get_base_package();
            imports.push(format!("{}.routes.Routes", base_package));
        }

        // Note: Dispatchers import is added later if needed (see end of generate function)

        // Deduplicate and sort imports alphabetically (standard Kotlin convention)
        imports.sort();
        imports.dedup();

        // Write all imports
        for import in imports {
            output.push_str(&format!("import {}\n", import));
        }

        output.push('\n');

        // Generate HttpClient singleton if $fetch() is used
        if self.uses_fetch {
            output.push_str(&self.generate_http_client());
        }

        // Component function
        // Add @OptIn if using experimental Material3 APIs
        if self.uses_experimental_material3 {
            output.push_str("@OptIn(ExperimentalMaterial3Api::class)\n");
        }
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
            // Track mutable vars for stability analysis
            self.mutable_vars.insert(state.name.clone());

            output.push_str(&self.indent());

            // Transform array literals: [1,2,3] -> mutableListOf(1,2,3)
            // Transform range literals: 1..10 -> (1..10).toList()
            let mut transformed_value = self.transform_array_literal(&state.initial_value, true);
            transformed_value = self.transform_range_literal(&transformed_value);

            if let Some(ref type_ann) = state.type_annotation {
                // Store type and default value for bind:value transformations
                self.var_types.insert(
                    state.name.clone(),
                    (type_ann.clone(), transformed_value.clone())
                );

                // With type annotation: var name by remember { mutableStateOf<Type>(value) }
                output.push_str(&format!(
                    "var {} by remember {{ mutableStateOf<{}>({}) }}\n",
                    state.name, type_ann, transformed_value
                ));
            } else {
                // Try to infer type from initial value for bind:value support
                let inferred_type = self.infer_type_from_value(&transformed_value);
                self.var_types.insert(
                    state.name.clone(),
                    (inferred_type, transformed_value.clone())
                );

                // Without type annotation: var name by remember { mutableStateOf(value) }
                output.push_str(&format!(
                    "var {} by remember {{ mutableStateOf({}) }}\n",
                    state.name, transformed_value
                ));
            }
        }

        if !mutable_state.is_empty() {
            output.push('\n');
        }

        // Generate computed state (val)
        for state in &computed_state {
            output.push_str(&self.indent());

            // Transform array literals: [1,2,3] -> listOf(1,2,3)
            // Transform range literals: 1..10 -> (1..10).toList()
            let mut transformed_value = self.transform_array_literal(&state.initial_value, false);
            transformed_value = self.transform_range_literal(&transformed_value);

            // Check if this is a custom scope: $scope() → rememberCoroutineScope()
            if transformed_value.trim() == "$scope()" {
                output.push_str(&format!("val {} = rememberCoroutineScope()\n", state.name));
                // Note: Import is added via output detection at the end of generate()
                continue;
            }

            // Check if this is a store instantiation
            if let Some(store_info) = self.detect_store_instantiation(&transformed_value) {
                // Track store usage for imports
                self.uses_viewmodel = true;
                // Use hiltViewModel if either @hilt or @inject is present
                let needs_hilt = store_info.has_hilt || store_info.has_inject;
                if needs_hilt {
                    self.uses_hilt_viewmodel = true;
                }

                // Generate viewModel or hiltViewModel based on annotations
                let view_model_call = if needs_hilt {
                    format!("hiltViewModel<{}>()", store_info.class_name)
                } else {
                    format!("viewModel<{}>()", store_info.class_name)
                };

                output.push_str(&format!("val {} = {}\n", state.name, view_model_call));

                // Add collectAsState for uiState
                output.push_str(&self.indent());
                output.push_str(&format!("val uiState by {}.uiState.collectAsState()\n", state.name));
            } else if state.is_derived_state {
                // derivedStateOf needs special wrapping: val name by remember { derivedStateOf { ... } }
                // Need to format with increased indent level for proper nesting
                output.push_str(&format!("val {} by remember {{\n", state.name));

                // Temporarily increase indent for the derivedStateOf content
                self.indent_level += 1;
                let formatted_value = self.format_multiline_value(&transformed_value);
                self.indent_level -= 1;

                output.push_str(&format!("{}    {}\n", self.indent(), formatted_value));
                output.push_str(&format!("{}}}\n", self.indent()));
            } else if let Some(ref type_ann) = state.type_annotation {
                // Format multi-line values with proper indentation
                let formatted_value = self.format_multiline_value(&transformed_value);
                output.push_str(&format!("val {}: {} = {}\n", state.name, type_ann, formatted_value));
            } else {
                output.push_str(&format!("val {} = {}\n", state.name, transformed_value));
            }
        }

        if !computed_state.is_empty() {
            output.push('\n');
        }

        // Generate dispatcher scope if dispatchers are used
        if self.uses_dispatchers {
            output.push_str(&self.indent());
            output.push_str("val dispatcherScope = rememberCoroutineScope()\n\n");
        }

        // Determine if functions should come before or after lifecycle hooks
        // If there are computed state values, functions come first (test 11)
        // If there are no computed state values, lifecycle comes first (test 08)
        let functions_first = !computed_state.is_empty();

        if functions_first {
            // Generate function declarations before lifecycle
            for func in &file.functions {
                // Check if this function has markup (composable helper function)
                if let Some(ref markup) = func.markup {
                    // Generate @Composable helper function with transpiled markup
                    output.push_str(&self.indent());
                    output.push_str("@Composable\n");
                    output.push_str(&self.indent());
                    let return_type_str = if let Some(ref rt) = func.return_type {
                        format!(": {}", rt)
                    } else {
                        String::new()
                    };
                    let suspend_keyword = if func.is_suspend { "suspend " } else { "" };
                    output.push_str(&format!("{}fun {}({}){} {{\n", suspend_keyword, func.name, func.params, return_type_str));

                    // Transpile the markup
                    self.indent_level += 1;
                    let markup_code = self.generate_markup(markup)?;
                    output.push_str(&markup_code);
                    self.indent_level -= 1;

                    output.push_str(&self.indent());
                    output.push_str("}\n");
                } else {
                    // Regular function with string body
                    output.push_str(&self.indent());
                    let return_type_str = if let Some(ref rt) = func.return_type {
                        format!(": {}", rt)
                    } else {
                        String::new()
                    };
                    let suspend_keyword = if func.is_suspend { "suspend " } else { "" };
                    output.push_str(&format!("{}fun {}({}){} {{\n", suspend_keyword, func.name, func.params, return_type_str));
                    // Output function body with proper indentation and transformations
                    for line in func.body.lines() {
                        output.push_str(&self.indent());
                        output.push_str("    ");

                        // Transform $routes.login → Routes.Login
                        let mut transformed_line = self.transform_route_aliases(line);

                        // Transform $screen.params.{name} → {name}
                        transformed_line = transformed_line.replace("$screen.params.", "");

                        // Transform $fetch() calls to Ktor HttpClient calls
                        if transformed_line.contains("$fetch(") {
                            transformed_line = self.transform_fetch_call(&transformed_line);
                        }

                        // For screens, transform $navigate() to navController.navigate()
                        if self.component_type.as_deref() == Some("screen") {
                            transformed_line = transformed_line.replace("$navigate(", "navController.navigate(");
                        }

                        // For non-suspend functions with launch calls, prefix with coroutineScope.
                        if !func.is_suspend && (transformed_line.trim().starts_with("launch ") || transformed_line.trim().starts_with("launch{")) {
                            output.push_str("coroutineScope.");
                        }

                        output.push_str(&transformed_line);
                        output.push('\n');
                    }
                    output.push_str(&self.indent());
                    output.push_str("}\n");
                }
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

        // Check if any non-suspend function contains launch calls
        let has_launch_in_functions = file.functions.iter().any(|f| {
            !f.is_suspend && (f.body.contains("launch ") || f.body.contains("launch{"))
        });

        // Generate coroutineScope if there are launch calls in onMount hooks or regular functions
        if has_launch_in_on_mount || has_launch_in_functions {
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

                    let mut transformed_line = line.trim_start().replace("$screen.params.", "");
                    // Transform $fetch() calls to Ktor HttpClient calls
                    if transformed_line.contains("$fetch(") {
                        transformed_line = self.transform_fetch_call(&transformed_line);
                    }
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

                        let mut transformed_line = line.trim_start().replace("$screen.params.", "");
                        // Transform $fetch() calls to Ktor HttpClient calls
                        if transformed_line.contains("$fetch(") {
                            transformed_line = self.transform_fetch_call(&transformed_line);
                        }
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
                // Check if this function has markup (composable helper function)
                if let Some(ref markup) = func.markup {
                    // Generate @Composable helper function with transpiled markup
                    output.push_str("\n");
                    output.push_str(&self.indent());
                    output.push_str("@Composable\n");
                    output.push_str(&self.indent());
                    let return_type_str = if let Some(ref rt) = func.return_type {
                        format!(": {}", rt)
                    } else {
                        String::new()
                    };
                    let suspend_keyword = if func.is_suspend { "suspend " } else { "" };
                    output.push_str(&format!("{}fun {}({}){} {{\n", suspend_keyword, func.name, func.params, return_type_str));

                    // Transpile the markup
                    self.indent_level += 1;
                    let markup_code = self.generate_markup(markup)?;
                    output.push_str(&markup_code);
                    self.indent_level -= 1;

                    output.push_str(&self.indent());
                    output.push_str("}\n");
                } else {
                    // Regular function with string body
                    output.push_str(&self.indent());
                    let return_type_str = if let Some(ref rt) = func.return_type {
                        format!(": {}", rt)
                    } else {
                        String::new()
                    };
                    let suspend_keyword = if func.is_suspend { "suspend " } else { "" };
                    output.push_str(&format!("{}fun {}({}){} {{\n", suspend_keyword, func.name, func.params, return_type_str));
                    // Output function body with proper indentation and transformations
                    for line in func.body.lines() {
                        output.push_str(&self.indent());
                        output.push_str("    ");

                        // Transform $routes.login → Routes.Login
                        let mut transformed_line = self.transform_route_aliases(line);

                        // Transform $screen.params.{name} → {name}
                        transformed_line = transformed_line.replace("$screen.params.", "");

                        // Transform $fetch() calls to Ktor HttpClient calls
                        if transformed_line.contains("$fetch(") {
                            transformed_line = self.transform_fetch_call(&transformed_line);
                        }

                        // For screens, transform $navigate() to navController.navigate()
                        if self.component_type.as_deref() == Some("screen") {
                            transformed_line = transformed_line.replace("$navigate(", "navController.navigate(");
                        }

                        // For non-suspend functions with launch calls, prefix with coroutineScope.
                        if !func.is_suspend && (transformed_line.trim().starts_with("launch ") || transformed_line.trim().starts_with("launch{")) {
                            output.push_str("coroutineScope.");
                        }

                        output.push_str(&transformed_line);
                        output.push('\n');
                    }
                    output.push_str(&self.indent());
                    output.push_str("}\n\n");
                }
            }
        }

        // Generate markup
        let markup_code = self.generate_markup(&file.markup)?;
        output.push_str(&markup_code);

        self.indent_level -= 1;
        output.push_str("}\n");

        // Append pass-through Kotlin blocks (Phase 5: Codegen Integration)
        if !file.kotlin_blocks.is_empty() {
            output.push('\n');
            for block in &file.kotlin_blocks {
                output.push_str(&block.content);
                output.push_str("\n\n");
            }
        }

        // Check if Dispatchers were used in the generated output
        if output.contains("Dispatchers.") {
            // Add import at the beginning (we need to insert it in the imports section)
            // This is a bit hacky but works for now
            let dispatcher_import = "import kotlinx.coroutines.Dispatchers\n";
            if !output.contains(dispatcher_import) {
                // Find where to insert (after package, before first import or composable)
                if let Some(package_end) = output.find('\n') {
                    let insert_pos = package_end + 1;
                    output.insert_str(insert_pos, dispatcher_import);
                }
            }
        }

        // Check if @Serializable was used in kotlin blocks
        if output.contains("@Serializable") {
            let serializable_import = "import kotlinx.serialization.Serializable\n";
            if !output.contains(serializable_import) {
                if let Some(package_end) = output.find('\n') {
                    let insert_pos = package_end + 1;
                    output.insert_str(insert_pos, serializable_import);
                }
            }
        }

        // Check if CoroutineScope.launch was used (for custom scopes)
        if output.contains(".launch {") || output.contains(".launch(") {
            let launch_import = "import kotlinx.coroutines.launch\n";
            if !output.contains(launch_import) {
                if let Some(package_end) = output.find('\n') {
                    let insert_pos = package_end + 1;
                    output.insert_str(insert_pos, launch_import);
                }
            }
        }

        // Convert FFI function calls from snake_case to camelCase (Kotlin convention)
        output = self.convert_ffi_function_names(output, file);

        // Wrap final output in TranspileResult::Single
        Ok(crate::transpiler::TranspileResult::Single(output))
    }

    fn generate_markup(&mut self, markup: &Markup) -> Result<String, String> {
        self.generate_markup_with_indent(markup, self.indent_level)
    }

    fn generate_markup_with_indent(&mut self, markup: &Markup, indent: usize) -> Result<String, String> {
        self.generate_markup_with_context(markup, indent, None)
    }

    fn generate_markup_with_context(&mut self, markup: &Markup, indent: usize, parent: Option<&str>) -> Result<String, String> {
        match markup {
            // Trim text in Button children to remove surrounding whitespace/newlines
            Markup::Text(text) if parent == Some("Button") => {
                let indent_str = "    ".repeat(indent);
                Ok(format!("{}Text(text = \"{}\")\n", indent_str, text.trim()))
            }
            Markup::IfElse(if_block) => {
                let mut output = String::new();
                let indent_str = "    ".repeat(indent);

                // Phase 1.1: Transform condition for ViewModel wrapper
                let condition = self.transform_viewmodel_expression(&if_block.condition);

                // if block
                output.push_str(&format!("{}if ({}) {{\n", indent_str, condition));
                for child in &if_block.then_branch {
                    output.push_str(&self.generate_markup_with_indent(child, indent + 1)?);
                }
                output.push_str(&format!("{}}}", indent_str));

                // else if blocks
                for else_if in &if_block.else_ifs {
                    // Phase 1.1: Transform else-if condition for ViewModel wrapper
                    let else_if_condition = self.transform_viewmodel_expression(&else_if.condition);
                    output.push_str(&format!(" else if ({}) {{\n", else_if_condition));
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
                // IMPORTANT: Check this BEFORE RecyclerView optimization
                // LazyColumn should always use Compose items(), never RecyclerView
                if parent == Some("LazyColumn") {
                    // items(collection, key = { expr }) { item ->
                    let transformed_collection = self.transform_viewmodel_expression(&for_loop.collection);
                    output.push_str(&format!("{}items({}", indent_str, transformed_collection));

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

                // Phase 6: Check if this loop should use RecyclerView optimization
                // Only apply RecyclerView optimization if NOT in LazyColumn
                let should_use_recyclerview = self.optimizations.iter().any(|opt| {
                    matches!(opt, Optimization::UseRecyclerView { collection_name, .. }
                        if collection_name == &for_loop.collection)
                });

                if should_use_recyclerview {
                    // Generate RecyclerView with AndroidView wrapper
                    return self.generate_recyclerview_inline(for_loop, indent);
                }

                // Regular forEach handling (for non-LazyColumn contexts)
                // If there's an empty block, wrap in if/else
                if let Some(empty_body) = &for_loop.empty_block {
                    // if (collection.isEmpty()) { empty block } else { forEach }
                    let transformed_collection = self.transform_viewmodel_expression(&for_loop.collection);
                    output.push_str(&format!("{}if ({}.isEmpty()) {{\n", indent_str, transformed_collection));
                    for child in empty_body {
                        output.push_str(&self.generate_markup_with_indent(child, indent + 1)?);
                    }
                    output.push_str(&format!("{}}}", indent_str));
                    output.push_str(" else {\n");

                    // forEach block
                    output.push_str(&format!("{}    {}.forEach {{ {} ->\n",
                        indent_str, transformed_collection, for_loop.item));

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
                    let transformed_collection = self.transform_viewmodel_expression(&for_loop.collection);
                    output.push_str(&format!("{}{}.forEach {{ {} ->\n",
                        indent_str, transformed_collection, for_loop.item));

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
                let base_indent_str = "    ".repeat(indent);

                // Note: key(Unit) wrapping was previously attempted here to prevent
                // recomposition of stable components, but it doesn't work as expected:
                // 1. key() is for list item identity, not recomposition prevention
                // 2. Using the same key (Unit) for multiple components causes confusion
                // 3. Compose compiler already handles skipping via @Stable inference
                //
                // Proper optimization would require:
                // - @Stable annotations on data classes
                // - Extracting components to separate @Composable functions
                // - Using derivedStateOf for computed values
                let should_wrap_stable = false; // Disabled - key(Unit) doesn't help

                let (effective_indent, indent_str) = if should_wrap_stable {
                    output.push_str(&base_indent_str);
                    output.push_str("key(Unit) {\n");
                    (indent + 1, "    ".repeat(indent + 1))
                } else {
                    (indent, base_indent_str.clone())
                };

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
                                // Transform ternary operators to if-else expressions
                                self.transform_ternary_to_if_else(prop_expr)
                            }
                        })
                } else {
                    None
                };

                // Special handling for Spacer with h/w shortcuts
                if comp.name == "Spacer" {
                    let h_prop = comp.props.iter().find(|p| p.name == "h");
                    let w_prop = comp.props.iter().find(|p| p.name == "w");

                    if let Some(h) = h_prop {
                        // <Spacer h={16} /> → Spacer(modifier = Modifier.height(16.dp))
                        let value = self.get_prop_expr(&h.value);
                        params.push(format!("modifier = Modifier.height({}.dp)", value));
                    } else if let Some(w) = w_prop {
                        // <Spacer w={16} /> → Spacer(modifier = Modifier.width(16.dp))
                        let value = self.get_prop_expr(&w.value);
                        params.push(format!("modifier = Modifier.width({}.dp)", value));
                    } else {
                        // <Spacer /> → default to 8dp height
                        params.push("modifier = Modifier.height(8.dp)".to_string());
                    }

                    // Handle any other props normally
                    for prop in &comp.props {
                        if prop.name != "h" && prop.name != "w" {
                            let prop_expr = self.get_prop_expr(&prop.value);
                            let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                            params.extend(transformed?);
                        }
                    }
                }
                // Special handling for DropdownMenu - transform to ExposedDropdownMenuBox
                else if comp.name == "DropdownMenu" {
                    // Mark that we're using experimental Material3 APIs
                    self.uses_experimental_material3 = true;

                    // Extract props
                    let value_prop = comp.props.iter().find(|p| p.name == "value");
                    let on_value_change = comp.props.iter().find(|p| p.name == "onValueChange");
                    let items_prop = comp.props.iter().find(|p| p.name == "items");

                    if let (Some(value), Some(on_change), Some(items)) = (value_prop, on_value_change, items_prop) {
                        let value_expr = self.get_prop_expr(&value.value);
                        let items_expr = self.get_prop_expr(&items.value);

                        // Transform onValueChange lambda
                        let on_change_expr = self.get_prop_expr(&on_change.value);
                        let transformed_on_change = self.transform_lambda_arrow(&on_change_expr);

                        // Generate unique variable name for expanded state
                        // Use a counter to ensure uniqueness (in case multiple dropdowns)
                        static DROPDOWN_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
                        let dropdown_id = DROPDOWN_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let expanded_var = if dropdown_id == 0 {
                            "expanded".to_string()
                        } else {
                            format!("expanded{}", dropdown_id)
                        };

                        // Generate the ExposedDropdownMenuBox structure
                        output.clear(); // Clear the component name we added earlier
                        output.push_str(&indent_str);
                        output.push_str(&format!("var {} by remember {{ mutableStateOf(false) }}\n", expanded_var));
                        output.push_str(&indent_str);
                        output.push_str("ExposedDropdownMenuBox(\n");
                        output.push_str(&format!("{}    expanded = {},\n", indent_str, expanded_var));
                        output.push_str(&format!("{}    onExpandedChange = {{ {} = !{} }}\n", indent_str, expanded_var, expanded_var));
                        output.push_str(&format!("{}) {{\n", indent_str));

                        // TextField
                        output.push_str(&format!("{}    TextField(\n", indent_str));
                        output.push_str(&format!("{}        value = {},\n", indent_str, value_expr));
                        output.push_str(&format!("{}        onValueChange = {{}},\n", indent_str));
                        output.push_str(&format!("{}        readOnly = true,\n", indent_str));
                        output.push_str(&format!("{}        modifier = Modifier.menuAnchor()\n", indent_str));
                        output.push_str(&format!("{}    )\n", indent_str));

                        // ExposedDropdownMenu
                        output.push_str(&format!("{}    ExposedDropdownMenu(\n", indent_str));
                        output.push_str(&format!("{}        expanded = {},\n", indent_str, expanded_var));
                        output.push_str(&format!("{}        onDismissRequest = {{ {} = false }}\n", indent_str, expanded_var));
                        output.push_str(&format!("{}    ) {{\n", indent_str));

                        // DropdownMenuItem for each item
                        output.push_str(&format!("{}        {}.forEach {{ item ->\n", indent_str, items_expr));
                        output.push_str(&format!("{}            DropdownMenuItem(\n", indent_str));
                        output.push_str(&format!("{}                text = {{ Text(item) }},\n", indent_str));
                        output.push_str(&format!("{}                onClick = {{\n", indent_str));

                        // Extract the lambda body from transformed_on_change
                        // transformed_on_change is like: { country -> selectedCountry = country }
                        // We need to call it with 'item': onValueChange(item)
                        // But since we have the lambda, we need to extract just the variable assignment
                        if let Some(arrow_pos) = transformed_on_change.find("->") {
                            let after_arrow = transformed_on_change[arrow_pos + 2..].trim();

                            // Extract parameter name from lambda
                            // Look for the opening brace and extract everything until ->
                            let before_arrow = if let Some(brace_pos) = transformed_on_change.find('{') {
                                transformed_on_change[brace_pos + 1..arrow_pos].trim()
                            } else {
                                ""
                            };
                            let param_name = before_arrow.trim();

                            // Remove trailing brace if present
                            let assignment = if after_arrow.ends_with('}') {
                                &after_arrow[..after_arrow.len()-1]
                            } else {
                                after_arrow
                            }.trim();

                            // Replace parameter with 'item' in the assignment
                            let item_assignment = assignment.replace(param_name, "item");

                            output.push_str(&format!("{}                    {}\n", indent_str, item_assignment));
                        }

                        output.push_str(&format!("{}                    {} = false\n", indent_str, expanded_var));
                        output.push_str(&format!("{}                }}\n", indent_str));
                        output.push_str(&format!("{}            )\n", indent_str));
                        output.push_str(&format!("{}        }}\n", indent_str));
                        output.push_str(&format!("{}    }}\n", indent_str));
                        output.push_str(&format!("{}}}\n", indent_str));

                        // Return early - we've generated the complete structure
                        return Ok(output);
                    } else {
                        return Err("DropdownMenu requires value, onValueChange, and items props".to_string());
                    }
                }
                // Special handling for Scaffold with topBar
                else if comp.name == "Scaffold" {
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
                            params.extend(transformed?);
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
                            params.extend(transformed?);
                        }
                    }
                }
                // Special handling for AlertDialog with composable content props
                else if comp.name == "AlertDialog" {
                    for prop in &comp.props {
                        // Props that accept composable content need lambda wrapping
                        if matches!(prop.name.as_str(), "title" | "text" | "confirmButton" | "dismissButton") {
                            match &prop.value {
                                PropValue::Markup(markup) => {
                                    // Component prop: wrap in lambda
                                    let content_code = self.generate_markup_with_indent(markup, indent + 2)?;
                                    let closing_indent = "    ".repeat(indent + 1);
                                    params.push(format!("{} = {{\n{}{}}}", prop.name, content_code, closing_indent));
                                }
                                PropValue::Expression(expr) => {
                                    // Expression prop: wrap in lambda with component
                                    // e.g., text = { Text(expr) }
                                    if prop.name == "title" || prop.name == "text" {
                                        params.push(format!("{} = {{ Text({}) }}", prop.name, expr));
                                    } else {
                                        // For buttons, just pass the expression (lambda expected)
                                        params.push(format!("{} = {}", prop.name, expr));
                                    }
                                }
                            }
                        } else {
                            // Other AlertDialog props - handle normally (onDismissRequest, etc.)
                            let prop_expr = self.get_prop_expr(&prop.value);
                            let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                            params.extend(transformed?);
                        }
                    }
                }
                // Special handling for Tab with composable text prop
                else if comp.name == "Tab" {
                    for prop in &comp.props {
                        // text prop accepts composable content and needs lambda wrapping
                        if prop.name == "text" {
                            match &prop.value {
                                PropValue::Markup(markup) => {
                                    // Component prop: wrap in lambda
                                    let content_code = self.generate_markup_with_indent(markup, indent + 2)?;
                                    let closing_indent = "    ".repeat(indent + 1);
                                    params.push(format!("text = {{\n{}{}}}", content_code, closing_indent));
                                }
                                PropValue::Expression(expr) => {
                                    // Expression prop: wrap in lambda with Text component
                                    params.push(format!("text = {{ Text({}) }}", expr));
                                }
                            }
                        } else {
                            // Other Tab props - handle normally (selected, onClick, etc.)
                            let prop_expr = self.get_prop_expr(&prop.value);
                            let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                            params.extend(transformed?);
                        }
                    }
                }
                // Special handling for FilterChip with composable label prop
                else if comp.name == "FilterChip" {
                    for prop in &comp.props {
                        // label prop accepts composable content and needs lambda wrapping
                        if prop.name == "label" {
                            match &prop.value {
                                PropValue::Markup(markup) => {
                                    // Component prop: wrap in lambda
                                    let content_code = self.generate_markup_with_indent(markup, indent + 2)?;
                                    let closing_indent = "    ".repeat(indent + 1);
                                    params.push(format!("label = {{\n{}{}}}", content_code, closing_indent));
                                }
                                PropValue::Expression(expr) => {
                                    // Expression prop: wrap in lambda with Text component
                                    params.push(format!("label = {{ Text({}) }}", expr));
                                }
                            }
                        } else {
                            // Other FilterChip props - handle normally (selected, onClick, etc.)
                            let prop_expr = self.get_prop_expr(&prop.value);
                            let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                            params.extend(transformed?);
                        }
                    }
                }
                // Special handling for Image component (alias for AsyncImage with web-friendly props)
                //
                // CSS-like behavior for Image sizing and fit:
                //
                // Size props (width/height):
                //   - "100%" → fillMaxWidth()/fillMaxHeight() modifier
                //   - "200" or "200dp" → fixed size modifier
                //   - No size → intrinsic size (image determines its own size)
                //
                // Fit prop (maps to ContentScale):
                //   - "contain" → Fit (show entire image, maintain aspect ratio, may letterbox)
                //   - "cover" → Crop (fill container, maintain aspect ratio, may crop)
                //   - "fill" → FillBounds (stretch to fill, may distort)
                //   - "none" → None (no scaling)
                //   - "scale-down" → Inside (like contain but never scale up)
                //
                // Smart defaults (when no fit specified):
                //   - width="100%" only → FillWidth (scale to fill width, maintain aspect ratio)
                //   - height="100%" only → FillHeight (scale to fill height, maintain aspect ratio)
                //   - both 100% → Fit (fit within bounds)
                //   - fixed size → Fit (fit within the fixed size)
                //   - no size props → Fit (default)
                //
                else if comp.name == "Image" {
                    // Collect size-related props for modifier
                    let width_prop = comp.props.iter().find(|p| p.name == "width");
                    let height_prop = comp.props.iter().find(|p| p.name == "height");
                    let fill_max_width = comp.props.iter().find(|p| p.name == "fillMaxWidth");
                    let fill_max_height = comp.props.iter().find(|p| p.name == "fillMaxHeight");
                    let fill_max_size = comp.props.iter().find(|p| p.name == "fillMaxSize");
                    let fit_prop = comp.props.iter().find(|p| p.name == "fit");
                    let content_scale_prop = comp.props.iter().find(|p| p.name == "contentScale");
                    let padding_prop = comp.props.iter().find(|p| p.name == "padding");

                    // Collect padding shortcuts for Tailwind-style cascade
                    let padding_shortcuts: Vec<_> = comp.props.iter()
                        .filter(|p| matches!(p.name.as_str(), "p" | "px" | "py" | "pt" | "pb" | "pl" | "pr"))
                        .collect();

                    // Track what size modifiers we're adding for smart defaults
                    let mut has_fill_width = false;
                    let mut has_fill_height = false;
                    let mut has_fixed_width = false;
                    let mut has_fixed_height = false;

                    // Build modifier if we have size/layout/padding props
                    if width_prop.is_some() || height_prop.is_some() || fill_max_width.is_some() || fill_max_height.is_some() || fill_max_size.is_some() || padding_prop.is_some() || !padding_shortcuts.is_empty() {
                        let mut modifiers = Vec::new();

                        // fillMaxSize first (takes precedence)
                        if let Some(fs) = fill_max_size {
                            let value = self.get_prop_expr(&fs.value);
                            if value.trim() == "true" {
                                modifiers.push(".fillMaxSize()".to_string());
                                has_fill_width = true;
                                has_fill_height = true;
                            }
                        } else {
                            // fillMaxWidth
                            if let Some(fw) = fill_max_width {
                                let value = self.get_prop_expr(&fw.value);
                                if value.trim() == "true" {
                                    modifiers.push(".fillMaxWidth()".to_string());
                                    has_fill_width = true;
                                }
                            }
                            // fillMaxHeight
                            if let Some(fh) = fill_max_height {
                                let value = self.get_prop_expr(&fh.value);
                                if value.trim() == "true" {
                                    modifiers.push(".fillMaxHeight()".to_string());
                                    has_fill_height = true;
                                }
                            }
                        }

                        // Handle width using parse_dimension
                        if let Some(w) = width_prop {
                            let value = self.get_prop_expr(&w.value);
                            let (dim_expr, is_percent) = self.parse_dimension(value, "width", "Image");
                            if is_percent {
                                // Only add if not already added via fillMaxWidth/fillMaxSize
                                if !modifiers.iter().any(|m| m.contains("fillMaxWidth") || m.contains("fillMaxSize")) {
                                    modifiers.push(".fillMaxWidth()".to_string());
                                    has_fill_width = true;
                                }
                            } else if !dim_expr.is_empty() {
                                modifiers.push(format!(".width({})", dim_expr));
                                has_fixed_width = true;
                            }
                        }
                        // Handle height using parse_dimension
                        if let Some(h) = height_prop {
                            let value = self.get_prop_expr(&h.value);
                            let (dim_expr, is_percent) = self.parse_dimension(value, "height", "Image");
                            if is_percent {
                                // Only add if not already added via fillMaxHeight/fillMaxSize
                                if !modifiers.iter().any(|m| m.contains("fillMaxHeight") || m.contains("fillMaxSize")) {
                                    modifiers.push(".fillMaxHeight()".to_string());
                                    has_fill_height = true;
                                }
                            } else if !dim_expr.is_empty() {
                                modifiers.push(format!(".height({})", dim_expr));
                                has_fixed_height = true;
                            }
                        }

                        // Build padding with Tailwind-style cascade (specific beats general)
                        let base_padding = padding_prop.map(|p| self.get_prop_expr(&p.value));
                        if let Some(padding_mod) = self.build_padding_modifier(&padding_shortcuts, base_padding.as_deref()) {
                            modifiers.push(padding_mod);
                        }

                        params.push(format!("modifier = Modifier{}", modifiers.join("")));
                    }

                    for prop in &comp.props {
                        // Skip size/layout/padding props - handled as modifier above
                        if prop.name == "width" || prop.name == "height" || prop.name == "fillMaxWidth" || prop.name == "fillMaxHeight" || prop.name == "fillMaxSize" || prop.name == "padding" {
                            continue;
                        }
                        // Skip padding shortcuts - handled as modifier above
                        if matches!(prop.name.as_str(), "p" | "px" | "py" | "pt" | "pb" | "pl" | "pr") {
                            continue;
                        }

                        let prop_expr = self.get_prop_expr(&prop.value);
                        // Apply $screen.params transformation
                        let prop_expr = prop_expr.replace("$screen.params.", "");

                        match prop.name.as_str() {
                            // Web-style aliases
                            "src" => params.push(format!("model = {}", prop_expr)),
                            "alt" => params.push(format!("contentDescription = {}", prop_expr)),
                            "fit" => {
                                // Map web-style fit values to ContentScale
                                let content_scale = match prop_expr.trim().trim_matches('"') {
                                    "cover" => "ContentScale.Crop",
                                    "contain" => "ContentScale.Fit",
                                    "fill" => "ContentScale.FillBounds",
                                    "fill-width" => "ContentScale.FillWidth",
                                    "fill-height" => "ContentScale.FillHeight",
                                    "none" => "ContentScale.None",
                                    "scale-down" => "ContentScale.Inside",
                                    // Pass through Compose-style values
                                    other => {
                                        if other.starts_with("ContentScale.") {
                                            other
                                        } else {
                                            // Capitalize first letter for Compose enum
                                            &format!("ContentScale.{}", other.chars().next().unwrap().to_uppercase().collect::<String>() + &other[1..])
                                        }
                                    }
                                };
                                params.push(format!("contentScale = {}", content_scale));
                            }
                            // Compose-style props (pass through)
                            "model" => params.push(format!("model = {}", prop_expr)),
                            "contentDescription" => params.push(format!("contentDescription = {}", prop_expr)),
                            "contentScale" => {
                                // Handle both string values and ContentScale.X
                                let content_scale = match prop_expr.trim().trim_matches('"') {
                                    "Crop" | "crop" => "ContentScale.Crop",
                                    "Fit" | "fit" => "ContentScale.Fit",
                                    "FillBounds" | "fillBounds" => "ContentScale.FillBounds",
                                    "None" | "none" => "ContentScale.None",
                                    "Inside" | "inside" => "ContentScale.Inside",
                                    other if other.starts_with("ContentScale.") => other,
                                    other => &format!("ContentScale.{}", other),
                                };
                                params.push(format!("contentScale = {}", content_scale));
                            }
                            // Other props pass through
                            _ => {
                                let transformed = self.transform_prop("AsyncImage", &prop.name, &prop_expr);
                                params.extend(transformed?);
                            }
                        }
                    }

                    // Smart default for contentScale based on size props (if not explicitly set)
                    // This gives CSS-like "width: 100%; height: auto" behavior
                    if fit_prop.is_none() && content_scale_prop.is_none() {
                        let default_content_scale = if has_fill_width && has_fill_height {
                            // Both dimensions constrained → Fit (contain)
                            "ContentScale.Fit"
                        } else if has_fill_width {
                            // Only width constrained → FillWidth (CSS-like behavior)
                            "ContentScale.FillWidth"
                        } else if has_fill_height {
                            // Only height constrained → FillHeight
                            "ContentScale.FillHeight"
                        } else if has_fixed_width || has_fixed_height {
                            // Fixed size → Fit
                            "ContentScale.Fit"
                        } else {
                            // No size constraints → Fit (safe default)
                            "ContentScale.Fit"
                        };
                        params.push(format!("contentScale = {}", default_content_scale));
                    }

                    // Add contentDescription = null if not provided (AsyncImage requires it)
                    let has_content_desc = comp.props.iter().any(|p| p.name == "alt" || p.name == "contentDescription");
                    if !has_content_desc {
                        params.push("contentDescription = null".to_string());
                    }

                    // Change component name to AsyncImage for output
                    output.clear();
                    output.push_str(&indent_str);
                    output.push_str("AsyncImage");
                }
                // Special handling for Text, Card, and Button with modifier props
                else if comp.name == "Text" || comp.name == "Card" || comp.name == "Button" {
                    // Use unified builder for padding and onClick (Text only)
                    let (mut modifiers, mut handled) = self.build_modifiers_for_component(comp, ModifierConfig {
                        handle_padding: true,
                        handle_fill_max: true,
                        handle_click_as_modifier: comp.name == "Text", // Text uses clickable modifier
                        ..Default::default()
                    })?;

                    // Special handling for fillMaxWidth with variable (not just true/false)
                    if let Some(fw_prop) = comp.props.iter().find(|p| p.name == "fillMaxWidth") {
                        let fw_value = self.get_prop_expr(&fw_prop.value);
                        let trimmed = fw_value.trim();
                        if trimmed != "true" && trimmed != "false" {
                            // It's a variable - use .let { if ... }
                            // Remove simple fillMaxWidth if already added
                            modifiers.retain(|m| !m.contains("fillMaxWidth"));
                            modifiers.push(format!(".let {{ if ({}) it.fillMaxWidth() else it }}", trimmed));
                        }
                    }

                    // Add explicit modifier last (if present) - special handling for chaining
                    if let Some(mod_prop) = comp.props.iter().find(|p| p.name == "modifier") {
                        let mod_value = self.get_prop_expr(&mod_prop.value);
                        let transformed = self.transform_ternary(&mod_value);
                        let transformed = self.convert_hex_in_modifier(&transformed)?;

                        // If explicit modifier starts with "Modifier.", strip it for chaining
                        let chainable = if transformed.starts_with("Modifier.") {
                            format!(".{}", &transformed[9..])
                        } else if transformed.starts_with("Modifier\n") {
                            transformed.replacen("Modifier\n", "", 1)
                        } else {
                            transformed
                        };

                        modifiers.push(chainable);
                        handled.insert("modifier".to_string());
                    }

                    // Combine into modifier parameter
                    if !modifiers.is_empty() {
                        let modifier_chain = modifiers.iter()
                            .map(|m| format!("            {}", m))
                            .collect::<Vec<_>>()
                            .join("\n");
                        params.push(format!("modifier = Modifier\n{}", modifier_chain));
                    }

                    // Add other props (excluding the ones we handled)
                    for prop in &comp.props {
                        if handled.contains(&prop.name) {
                            continue;
                        }
                        let prop_expr = self.get_prop_expr(&prop.value);
                        let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                        params.extend(transformed?);
                    }
                }
                // Special handling for Column/Row with unified modifier builder
                else if comp.name == "Column" || comp.name == "Row" {
                    let (modifiers, handled) = self.build_modifiers_for_component(comp, ModifierConfig {
                        handle_padding: true,
                        handle_background: true,
                        ..Default::default()
                    })?;

                    // Combine into modifier parameter
                    if !modifiers.is_empty() {
                        if modifiers.len() == 1 {
                            params.push(format!("modifier = Modifier{}", modifiers[0]));
                        } else {
                            let modifier_chain = modifiers.iter()
                                .map(|m| format!("                {}", m))
                                .collect::<Vec<_>>()
                                .join("\n");
                            params.push(format!("modifier = Modifier\n{}", modifier_chain));
                        }
                    }

                    // Process other props, skipping handled ones
                    for prop in &comp.props {
                        if handled.contains(&prop.name) {
                            continue;
                        }
                        let prop_expr = self.get_prop_expr(&prop.value);
                        let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                        params.extend(transformed?);
                    }
                }
                else if comp.name == "Box" {
                    // Box uses unified modifier builder for size/padding/background
                    let (modifiers, mut handled) = self.build_modifiers_for_component(comp, ModifierConfig {
                        handle_size: true,
                        handle_padding: true,
                        handle_background: true,
                        handle_fill_max: true,
                        ..Default::default()
                    })?;

                    // Handle alignment as contentAlignment parameter (not modifier)
                    let alignment = comp.props.iter().find(|p| p.name == "alignment")
                        .map(|p| self.get_prop_expr(&p.value));
                    if let Some(align) = &alignment {
                        let align_str = if align.starts_with('"') && align.ends_with('"') {
                            let a = &align[1..align.len()-1];
                            format!("Alignment.{}", a.chars().next().unwrap().to_uppercase().collect::<String>() + &a[1..])
                        } else {
                            align.to_string()
                        };
                        params.push(format!("contentAlignment = {}", align_str));
                        handled.insert("alignment".to_string());
                    }

                    // Output combined modifier if we have any
                    if !modifiers.is_empty() {
                        params.push(format!("modifier = Modifier{}", modifiers.join("")));
                    }

                    // Process other props
                    for prop in &comp.props {
                        if handled.contains(&prop.name) {
                            continue;
                        }
                        let prop_expr = self.get_prop_expr(&prop.value);
                        let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                        params.extend(transformed?);
                    }
                }
                else if comp.name == "AsyncImage" {
                    // Special handling for AsyncImage with width/height/etc
                    // Collect special props as expression strings
                    let width = comp.props.iter().find(|p| p.name == "width")
                        .map(|p| self.get_prop_expr(&p.value));
                    let height = comp.props.iter().find(|p| p.name == "height")
                        .map(|p| self.get_prop_expr(&p.value));
                    let url = comp.props.iter().find(|p| p.name == "url")
                        .map(|p| self.get_prop_expr(&p.value));
                    let content_desc = comp.props.iter().find(|p| p.name == "contentDescription");

                    // Handle AsyncImage special props
                    {
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

                                // Add contentDescription = null if not provided
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
                    let has_async_image_modifier = comp.props.iter().any(|p| p.name == "modifier");

                    for prop in &comp.props {
                        // Skip props we've already handled
                        if !has_async_image_modifier &&
                            (prop.name == "url" || prop.name == "width" || prop.name == "height" ||
                             prop.name == "placeholder" || prop.name == "error" || prop.name == "crossfade" ||
                             prop.name == "contentDescription") {
                            continue; // AsyncImage props handled above (only if no explicit modifier)
                        }
                        let prop_expr = self.get_prop_expr(&prop.value);
                        let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                        params.extend(transformed?);
                    }
                }
                // Special handling for Icon
                else if comp.name == "Icon" {
                    // Add all props
                    for prop in &comp.props {
                        let prop_expr = self.get_prop_expr(&prop.value);
                        let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                        params.extend(transformed?);
                    }
                } else {
                    // Generic component handling with unified modifier builder
                    let (modifiers, handled) = self.build_modifiers_for_component(comp, ModifierConfig {
                        handle_size: true,
                        handle_padding: true,
                        handle_fill_max: true,
                        ..Default::default()
                    })?;

                    // Output combined modifier if we have any
                    if !modifiers.is_empty() {
                        params.push(format!("modifier = Modifier{}", modifiers.join("")));
                    }

                    // Regular prop handling for other components (excluding handled props)
                    for prop in &comp.props {
                        if handled.contains(&prop.name) {
                            continue;
                        }

                        let prop_expr = self.get_prop_expr(&prop.value);
                        let transformed = self.transform_prop(&comp.name, &prop.name, prop_expr);
                        params.extend(transformed?);
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
                            output.push_str(&format!("{}    Text(text = \"${{{}}}\")\n", indent_str, transformed));
                        } else if text.starts_with("if (") {
                            // It's an if-else expression (from ternary transformation)
                            output.push_str(&format!("{}    Text(text = {})\n", indent_str, text));
                        } else {
                            // It's a literal string
                            output.push_str(&format!("{}    Text(\"{}\")\n", indent_str, text));
                        }
                    }

                    // Generate regular children (pass component name as parent for context-aware generation)
                    for (i, child) in comp.children.iter().enumerate() {
                        // For Scaffold with layout child, mark first child to add paddingValues to modifier
                        if scaffold_needs_padding && i == 0 {
                            output.push_str(&self.generate_scaffold_child(child, effective_indent + 1)?);
                        } else {
                            output.push_str(&self.generate_markup_with_context(child, effective_indent + 1, Some(&comp.name))?);
                        }
                    }
                    output.push_str(&format!("{}}}\n", indent_str));
                } else {
                    output.push('\n');
                }

                // Close the key(Unit) wrapper if we opened one
                if should_wrap_stable {
                    output.push_str(&base_indent_str);
                    output.push_str("}\n");
                }

                Ok(output)
            }
            Markup::Text(text) => {
                let indent_str = "    ".repeat(indent);
                Ok(format!("{}Text(text = \"{}\")\n", indent_str, text))
            }
            Markup::Interpolation(expr) => {
                let indent_str = "    ".repeat(indent);
                // Transform ternary operators in interpolated expressions
                let transformed_expr = self.transform_ternary_to_if_else(expr);
                Ok(format!("{}Text(text = ${})\n", indent_str, transformed_expr))
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
            let rest = &path[1..]; // Remove $

            // Extract the root package by removing common package type suffixes
            // e.g., "com.example.app.components" -> "com.example.app"
            // e.g., "com.example.app.screens" -> "com.example.app"
            let known_suffixes = [".components", ".screens", ".routes", ".lib", ".models", ".utils"];
            let root_package = known_suffixes.iter()
                .find_map(|&suffix| {
                    if self.package.ends_with(suffix) {
                        Some(&self.package[..self.package.len() - suffix.len()])
                    } else {
                        None
                    }
                })
                .unwrap_or(&self.package);

            // Add a dot between root package and the rest
            if rest.starts_with('.') {
                format!("{}{}", root_package, rest)
            } else {
                format!("{}.{}", root_package, rest)
            }
        } else {
            path.to_string()
        }
    }

    /// Convert FFI function calls from snake_case to camelCase (Kotlin convention)
    /// Finds all FFI imports (starting with $ffi) and converts function calls
    /// e.g., Math.is_prime() -> Math.isPrime()
    fn convert_ffi_function_names(&self, mut output: String, file: &WhitehallFile) -> String {
        // Extract FFI object names from imports
        let ffi_objects: Vec<String> = file.imports
            .iter()
            .filter(|imp| imp.path.starts_with("$ffi"))
            .filter_map(|imp| {
                // Extract the last segment as the object name
                // e.g., "$ffi.rust.Math" -> "Math"
                imp.path.split('.').last().map(|s| s.to_string())
            })
            .collect();

        // For each FFI object, convert snake_case function calls to camelCase
        for object_name in ffi_objects {
            // Use regex to find patterns like "ObjectName.function_name("
            let pattern = format!(r"({}\.)([\w]+)\(", regex::escape(&object_name));
            let re = regex::Regex::new(&pattern).unwrap();

            output = re.replace_all(&output, |caps: &regex::Captures| {
                let prefix = &caps[1]; // "ObjectName."
                let function_name = &caps[2]; // "function_name"

                // Convert snake_case to camelCase
                let camel_case = self.snake_to_camel(function_name);

                format!("{}{}(", prefix, camel_case)
            }).to_string();
        }

        output
    }

    /// Convert snake_case to camelCase
    /// e.g., "is_prime" -> "isPrime", "get_user_name" -> "getUserName"
    fn snake_to_camel(&self, name: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = false;

        for (i, ch) in name.chars().enumerate() {
            if ch == '_' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                // Keep first character lowercase (camelCase, not PascalCase)
                if i == 0 {
                    result.push(ch.to_ascii_lowercase());
                } else {
                    result.push(ch);
                }
            }
        }

        result
    }

    /// Convert Material3 color scheme names in expressions
    /// e.g., "primaryContainer" -> MaterialTheme.colorScheme.primaryContainer
    /// Handles expressions like: if (condition) "primary" else "secondary"
    fn convert_color_scheme_names(&self, value: &str) -> String {
        // List of Material3 color scheme names
        let color_schemes = [
            "primary", "onPrimary", "primaryContainer", "onPrimaryContainer",
            "secondary", "onSecondary", "secondaryContainer", "onSecondaryContainer",
            "tertiary", "onTertiary", "tertiaryContainer", "onTertiaryContainer",
            "error", "onError", "errorContainer", "onErrorContainer",
            "background", "onBackground",
            "surface", "onSurface", "surfaceVariant", "onSurfaceVariant",
            "surfaceTint", "inverseSurface", "inverseOnSurface", "inversePrimary",
            "outline", "outlineVariant", "scrim",
        ];

        // Use regex to find quoted strings
        let re = regex::Regex::new(r#""([^"]*)""#).unwrap();

        re.replace_all(value, |caps: &regex::Captures| {
            let color_name = &caps[1];

            // Check if this is a color scheme name
            if color_schemes.contains(&color_name) {
                format!("MaterialTheme.colorScheme.{}", color_name)
            } else if color_name.starts_with('#') {
                // Hex color - convert
                match convert_hex_to_color(&color_name[1..]) {
                    Ok(converted) => converted,
                    Err(_) => caps[0].to_string(), // Keep original if conversion fails
                }
            } else {
                // Not a color scheme name, keep quoted
                caps[0].to_string()
            }
        }).to_string()
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

                    // Check for Icons usage (Icons.Default.*, Icons.Filled.*, etc.)
                    if prop_expr.contains("Icons.Default.") {
                        self.add_import_if_missing(prop_imports, "androidx.compose.material.icons.Icons");
                        self.add_import_if_missing(prop_imports, "androidx.compose.material.icons.filled.*");
                    }
                    if prop_expr.contains("Icons.Filled.") {
                        self.add_import_if_missing(prop_imports, "androidx.compose.material.icons.Icons");
                        self.add_import_if_missing(prop_imports, "androidx.compose.material.icons.filled.*");
                    }
                    if prop_expr.contains("Icons.Outlined.") {
                        self.add_import_if_missing(prop_imports, "androidx.compose.material.icons.Icons");
                        self.add_import_if_missing(prop_imports, "androidx.compose.material.icons.outlined.*");
                    }

                    // Check for TextDecoration usage
                    if prop_expr.contains("TextDecoration") {
                        self.add_import_if_missing(prop_imports, "androidx.compose.ui.text.style.TextDecoration");
                    }

                    if prop_expr.contains("Modifier") {
                        let import = "androidx.compose.ui.Modifier".to_string();
                        if !prop_imports.contains(&import) {
                            prop_imports.push(import);
                        }

                        // Check for specific Modifier extension functions
                        if prop_expr.contains("fillMaxSize") {
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.fillMaxSize");
                        }
                        if prop_expr.contains("fillMaxWidth") {
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.fillMaxWidth");
                        }
                        if prop_expr.contains("fillMaxHeight") {
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.fillMaxHeight");
                        }
                        if prop_expr.contains(".padding(") {
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.padding");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        if prop_expr.contains(".size(") {
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.size");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        if prop_expr.contains(".height(") {
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.height");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        if prop_expr.contains(".width(") {
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.width");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        if prop_expr.contains(".weight(") {
                            // weight is a RowScope/ColumnScope extension, no import needed
                        }
                        if prop_expr.contains(".background(") {
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.background");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.graphics.Color");
                        }
                    }
                    if prop_expr.contains("Arrangement.") {
                        self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.Arrangement");
                    }
                    if prop_expr.contains("Alignment.") {
                        self.add_import_if_missing(prop_imports, "androidx.compose.ui.Alignment");
                    }
                    if prop_expr.contains("TextAlign.") {
                        self.add_import_if_missing(prop_imports, "androidx.compose.ui.text.style.TextAlign");
                    }
                    if prop_expr.contains("FontWeight.") {
                        self.add_import_if_missing(prop_imports, "androidx.compose.ui.text.font.FontWeight");
                    }
                    if prop_expr.contains("Color.") {
                        self.add_import_if_missing(prop_imports, "androidx.compose.ui.graphics.Color");
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

                    // Padding/margin shorthand props (work on any component)
                    match prop.name.as_str() {
                        "p" | "px" | "py" | "pt" | "pb" | "pl" | "pr" |
                        "m" | "mx" | "my" | "mt" | "mb" | "ml" | "mr" => {
                            // Shorthand padding/margin → modifier with padding/margin
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            if prop.name.starts_with('p') {
                                self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.padding");
                            }
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        _ => {}
                    }

                    // Component-specific prop transformations that need imports
                    match (comp.name.as_str(), prop.name.as_str()) {
                        ("Spacer", "h") => {
                            // h → modifier = Modifier.height(N.dp)
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.height");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        ("Spacer", "w") => {
                            // w → modifier = Modifier.width(N.dp)
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.width");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
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
                        ("Column", "backgroundColor") | ("Row", "backgroundColor") => {
                            // backgroundColor → modifier = Modifier.background(Color)
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.background");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.graphics.Color");
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
                        ("Text", "fontFamily") => {
                            // fontFamily → FontFamily.Monospace, etc.
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.text.font.FontFamily");
                        }
                        ("Text", "color") if prop_expr.starts_with('"') => {
                            let s = &prop_expr[1..prop_expr.len()-1];
                            // Check if hex color (needs Color import) or theme color (needs MaterialTheme)
                            if s.starts_with('#') {
                                self.add_import_if_missing(prop_imports, "androidx.compose.ui.graphics.Color");
                            } else {
                                self.add_import_if_missing(prop_imports, "androidx.compose.material3.MaterialTheme");
                            }
                        }
                        ("Card", "backgroundColor") => {
                            // backgroundColor → CardDefaults.cardColors()
                            self.add_import_if_missing(prop_imports, "androidx.compose.material3.CardDefaults");
                            self.add_import_if_missing(prop_imports, "androidx.compose.material3.MaterialTheme");
                            // Add Color import only for hex colors (Color(0xFFRRGGBB))
                            if prop_expr.starts_with('"') {
                                let s = &prop_expr[1..prop_expr.len()-1];
                                if s.starts_with('#') {
                                    self.add_import_if_missing(prop_imports, "androidx.compose.ui.graphics.Color");
                                }
                            }
                        }
                        ("Card", "elevation") => {
                            // elevation → CardDefaults.cardElevation()
                            self.add_import_if_missing(prop_imports, "androidx.compose.material3.CardDefaults");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
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
                        ("Text", "fillMaxWidth") | ("Card", "fillMaxWidth") | ("Button", "fillMaxWidth") => {
                            // fillMaxWidth → modifier chain with .fillMaxWidth()
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.fillMaxWidth");
                        }
                        ("Text", "modifier") | ("Card", "modifier") if prop_expr.contains("clickable") => {
                            // modifier with clickable
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.clickable");
                        }
                        ("Text", "onClick") => {
                            // onClick on Text → .clickable modifier (Text doesn't have native onClick)
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.clickable");
                        }
                        ("Card", "onClick") => {
                            // Card has native onClick parameter, no special imports needed
                            // (handled as regular prop)
                        }
                        ("TextField", "label") | ("TextField", "placeholder") |
                        ("OutlinedTextField", "label") | ("OutlinedTextField", "placeholder") => {
                            // label/placeholder generate Text() components
                            self.add_import_if_missing(component_imports, "androidx.compose.material3.Text");
                        }
                        ("TextField", "type") if prop_expr.trim_matches('"') == "password" => {
                            // type="password" → visualTransformation = PasswordVisualTransformation()
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.text.input.PasswordVisualTransformation");
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
                    "CircularProgressIndicator" => {
                        let import = "androidx.compose.material3.CircularProgressIndicator".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "AlertDialog" => {
                        let import = "androidx.compose.material3.AlertDialog".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "Tab" => {
                        let import = "androidx.compose.material3.Tab".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "TabRow" => {
                        let import = "androidx.compose.material3.TabRow".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "Divider" => {
                        let import = "androidx.compose.material3.Divider".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "HorizontalDivider" => {
                        let import = "androidx.compose.material3.HorizontalDivider".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "OutlinedTextField" => {
                        let import = "androidx.compose.material3.OutlinedTextField".to_string();
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
                    "FilterChip" => {
                        let import = "androidx.compose.material3.FilterChip".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                    }
                    "SnackbarHost" => {
                        let import = "androidx.compose.material3.SnackbarHost".to_string();
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
                    "DropdownMenu" => {
                        // DropdownMenu transforms to ExposedDropdownMenuBox pattern
                        // Note: ExposedDropdownMenu is part of ExposedDropdownMenuBoxScope, not a separate import
                        let imports = vec![
                            "androidx.compose.material3.ExposedDropdownMenuBox".to_string(),
                            "androidx.compose.material3.DropdownMenuItem".to_string(),
                            "androidx.compose.material3.TextField".to_string(),
                            "androidx.compose.material3.Text".to_string(),
                        ];
                        for import in imports {
                            if !component_imports.contains(&import) {
                                component_imports.push(import);
                            }
                        }
                    }
                    "Button" => {
                        let import = "androidx.compose.material3.Button".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                        // Button with text prop generates Text() inside, so need Text import
                        let has_text_prop = comp.props.iter().any(|p| p.name == "text");
                        if has_text_prop {
                            self.add_import_if_missing(component_imports, "androidx.compose.material3.Text");
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
                    "IconButton" => {
                        let import = "androidx.compose.material3.IconButton".to_string();
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
                        // Check if contentScale is used - need ContentScale import
                        let has_content_scale = comp.props.iter().any(|p| p.name == "contentScale");
                        if has_content_scale {
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.layout.ContentScale");
                        }
                    }
                    "Image" => {
                        // Image is our web-friendly alias for AsyncImage
                        let import = "coil.compose.AsyncImage".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                        // Always need ContentScale import - we add smart defaults even if not specified
                        self.add_import_if_missing(prop_imports, "androidx.compose.ui.layout.ContentScale");
                        // Check if any size/layout props are used - need Modifier and dp imports
                        let has_size_props = comp.props.iter().any(|p|
                            matches!(p.name.as_str(), "width" | "height" | "fillMaxWidth" | "fillMaxHeight" | "fillMaxSize")
                        );
                        if has_size_props {
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                            self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");
                        }
                        // Check size/layout props for specific imports
                        for prop in &comp.props {
                            let value = self.get_prop_expr(&prop.value);
                            let is_100_percent = value.trim().trim_matches('"') == "100%";

                            match prop.name.as_str() {
                                "fillMaxWidth" => {
                                    self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.fillMaxWidth");
                                }
                                "fillMaxHeight" => {
                                    self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.fillMaxHeight");
                                }
                                "fillMaxSize" => {
                                    self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.fillMaxSize");
                                }
                                "width" if is_100_percent => {
                                    self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.fillMaxWidth");
                                }
                                "height" if is_100_percent => {
                                    self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.fillMaxHeight");
                                }
                                "width" | "height" => {
                                    // Fixed pixel values need width/height imports
                                    self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.width");
                                    self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.height");
                                }
                                _ => {}
                            }
                        }
                    }
                    "Spacer" => {
                        let import = "androidx.compose.foundation.layout.Spacer".to_string();
                        if !component_imports.contains(&import) {
                            component_imports.push(import);
                        }
                        // Spacer always needs Modifier and dp
                        self.add_import_if_missing(prop_imports, "androidx.compose.ui.Modifier");
                        self.add_import_if_missing(prop_imports, "androidx.compose.ui.unit.dp");

                        // If no h/w props, we use default height, so import it
                        let has_h_or_w = comp.props.iter().any(|p| p.name == "h" || p.name == "w");
                        if !has_h_or_w {
                            self.add_import_if_missing(prop_imports, "androidx.compose.foundation.layout.height");
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
            Markup::Text(_) if parent_component == Some("Button") => {
                // Plain text inside Button is auto-wrapped in Text component
                let import = "androidx.compose.material3.Text".to_string();
                if !component_imports.contains(&import) {
                    component_imports.push(import);
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
                Markup::Text(text) => {
                    // Only trim if text contains newlines (multi-line markup artifact)
                    // Otherwise preserve intentional spaces (like "Hello, ")
                    let cleaned = if text.contains('\n') {
                        text.trim()
                    } else {
                        text.as_str()
                    };
                    return Ok(format!("\"{}\"", cleaned));
                }
                // Single interpolation - wrap in string template to ensure string conversion
                Markup::Interpolation(expr) => {
                    // Transform $screen.params.{name} → {name}
                    let expr = expr.replace("$screen.params.", "");
                    let transformed = self.transform_string_resource(&expr);
                    let with_ternary = self.transform_ternary_to_if_else(&transformed);
                    let with_vm = self.transform_viewmodel_expression(&with_ternary);
                    let with_assertions = self.add_null_assertions(&with_vm);
                    return Ok(format!("\"${{{}}}\"", with_assertions));
                }
                _ => {}
            }
        }

        // Multiple children: build string template with interpolation
        let mut parts = Vec::new();
        for child in &non_whitespace_children {
            match child {
                Markup::Text(text) => {
                    // Only trim if text contains newlines (multi-line markup artifact)
                    // Otherwise preserve intentional spaces
                    let cleaned = if text.contains('\n') {
                        text.trim()
                    } else {
                        text.as_str()
                    };
                    // Escape dollar signs in literal text for Kotlin string templates
                    parts.push(self.escape_dollar_signs(cleaned));
                }
                Markup::Interpolation(expr) => {
                    // Transform $screen.params.{name} → {name}
                    let expr = expr.replace("$screen.params.", "");
                    let str_res_transformed = self.transform_string_resource(&expr);
                    let with_ternary = self.transform_ternary_to_if_else(&str_res_transformed);
                    let with_vm = self.transform_viewmodel_expression(&with_ternary);
                    let transformed = self.add_null_assertions(&with_vm);
                    // Always use braces for safety - handles literals, keywords, and expressions
                    parts.push(format!("${{{}}}", transformed));
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

    /// Transform ternary to if-else expression (for values, not modifiers)
    /// Transforms: condition ? value : value
    /// To: if (condition) value else value
    /// Transform ternary operator to if-else expression
    /// Note: Skips Kotlin operators: ?. (safe navigation) and ?: (elvis)
    fn transform_ternary_to_if_else(&self, expr: &str) -> String {
        // Find ? and : at the same depth level for ternary operator
        // Must distinguish from Kotlin's ?. (safe navigation) and ?: (elvis)
        let mut depth = 0;
        let mut question_pos = None;
        let mut colon_pos = None;

        let chars: Vec<char> = expr.chars().collect();
        for (i, ch) in expr.char_indices() {
            match ch {
                '(' | '{' | '[' => depth += 1,
                ')' | '}' | ']' => depth -= 1,
                '?' if depth == 0 && question_pos.is_none() => {
                    // Check if this is safe navigation operator (?.) - skip it
                    if i + 1 < chars.len() && chars[i + 1] == '.' {
                        continue; // Skip safe navigation operator
                    }
                    // Check if this is Elvis operator (?:) - skip it
                    if i + 1 < chars.len() && chars[i + 1] == ':' {
                        continue; // Skip Elvis operator
                    }
                    // This is a standalone ?, mark as potential ternary operator
                    question_pos = Some(i);
                },
                ':' if depth == 0 && question_pos.is_some() && colon_pos.is_none() => {
                    // Make sure this : is not part of Elvis operator that we already skipped
                    if i > 0 && chars[i - 1] == '?' {
                        continue; // This : is part of ?:, skip it
                    }
                    colon_pos = Some(i);
                },
                _ => {}
            }
        }

        if let (Some(q), Some(c)) = (question_pos, colon_pos) {
            let condition = expr[..q].trim();
            let then_value = expr[q+1..c].trim();
            let else_value = expr[c+1..].trim();

            format!("if ({}) {} else {}", condition, then_value, else_value)
        } else {
            expr.to_string()
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

    /// Convert hex color strings in modifier expressions
    /// Transforms .background("#F6F6F6") to .background(Color(0xFFF6F6F6))
    fn convert_hex_in_modifier(&self, expr: &str) -> Result<String, String> {
        let mut result = expr.to_string();

        // Find patterns like .background("#...") or .background('#...")
        // and replace the entire call with .background(Color(...))
        let patterns = [
            (".background(\"#", '"'),
            (".background('#", '\''),
        ];

        for (pattern, quote_char) in &patterns {
            while let Some(start) = result.find(pattern) {
                let hex_start = start + pattern.len();

                // Find the closing quote
                if let Some(quote_end) = result[hex_start..].find(*quote_char) {
                    let hex_color = &result[hex_start..hex_start + quote_end];
                    let color_code = convert_hex_to_color(hex_color)?;

                    // Find the closing paren after the quote
                    let after_quote = hex_start + quote_end + 1;
                    if let Some(paren_end) = result[after_quote..].find(')') {
                        let after_paren = after_quote + paren_end + 1;

                        // Replace entire .background("...") with .background(Color(...))
                        let before = &result[..start];
                        let after = &result[after_paren..];
                        result = format!("{}.background({}){}", before, color_code, after);
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        Ok(result)
    }

    /// Transform Whitehall string interpolation {expr} to Kotlin ${expr}
    /// Handles strings like "Count: {uiState.count}" → "Count: ${uiState.count}"
    fn transform_string_interpolation(&self, value: &str) -> String {
        // Only process if value is a quoted string
        if !((value.starts_with('"') && value.ends_with('"')) ||
             (value.starts_with('\'') && value.ends_with('\''))) {
            return value.to_string();
        }

        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Transform {expr} to ${expr}
                result.push('$');
                result.push('{');
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn transform_prop(&mut self, component: &str, prop_name: &str, prop_value: &str) -> Result<Vec<String>, String> {
        // Transform $screen.params.{name} → {name} for screens
        let prop_value = prop_value.replace("$screen.params.", "");
        // Transform string interpolation first: {expr} → ${expr}
        let prop_value = self.transform_string_interpolation(&prop_value);

        // Handle bind:value special syntax BEFORE transform_viewmodel_expression
        // because we need the original variable name, not the transformed one
        if prop_name == "bind:value" {
            let var_name = prop_value.trim();

            // Phase 1.1: Handle ViewModel wrapper bind:value
            if self.in_viewmodel_wrapper && self.mutable_vars.contains(var_name) {
                // In ViewModel wrapper: bind to uiState.var and viewModel.var = it
                // Check if this variable has a numeric type
                if let Some((type_str, default_value)) = self.var_types.get(var_name) {
                    if self.is_numeric_type(type_str) {
                        // Numeric bind:value needs type conversions
                        let (to_method, default) = self.get_numeric_conversion(type_str, default_value);
                        return Ok(vec![
                            format!("value = uiState.{}.toString()", var_name),
                            format!("onValueChange = {{ viewModel.{} = it.{} ?: {} }}", var_name, to_method, default),
                        ]);
                    }
                }

                // Default bind:value (for String types) in ViewModel wrapper
                return Ok(vec![
                    format!("value = uiState.{}", var_name),
                    format!("onValueChange = {{ viewModel.{} = it }}", var_name),
                ]);
            }

            // Regular component (not in ViewModel wrapper)
            // Check if this variable has a numeric type
            if let Some((type_str, default_value)) = self.var_types.get(var_name) {
                if self.is_numeric_type(type_str) {
                    // Numeric bind:value needs type conversions
                    let (to_method, default) = self.get_numeric_conversion(type_str, default_value);
                    return Ok(vec![
                        format!("value = {}.toString()", var_name),
                        format!("onValueChange = {{ {} = it.{} ?: {} }}", var_name, to_method, default),
                    ]);
                }
            }

            // Default bind:value (for String types)
            return Ok(vec![
                format!("value = {}", var_name),
                format!("onValueChange = {{ {} = it }}", var_name),
            ]);
        }

        // Handle bind:checked special syntax (for Checkbox, Switch)
        // Also before transform_viewmodel_expression for same reason as bind:value
        if prop_name == "bind:checked" {
            let var_name = prop_value.trim();

            // In ViewModel wrapper, use uiState for checked value and viewModel for setter
            if self.in_viewmodel_wrapper && self.mutable_vars.contains(var_name) {
                return Ok(vec![
                    format!("checked = uiState.{}", var_name),
                    format!("onCheckedChange = {{ viewModel.{} = it }}", var_name),
                ]);
            }

            // Regular component
            return Ok(vec![
                format!("checked = {}", var_name),
                format!("onCheckedChange = {{ {} = it }}", var_name),
            ]);
        }

        // Phase 1.1: Transform ViewModel wrapper references
        // Must happen AFTER bind:value/bind:checked but BEFORE other transforms
        let prop_value = self.transform_viewmodel_expression(&prop_value);

        let prop_value = prop_value.as_str();

        // Handle modifier prop - convert hex colors in background() calls
        if prop_name == "modifier" {
            let transformed = self.convert_hex_in_modifier(prop_value)?;
            return Ok(vec![format!("modifier = {}", transformed)]);
        }

        // Transform route aliases first: $routes → Routes (before adding braces)
        let value = self.transform_route_aliases(prop_value);

        // Transform $navigate() → navController.navigate() for screens
        let value = if self.component_type.as_deref() == Some("screen") {
            value.replace("$navigate(", "navController.navigate(")
        } else {
            value
        };

        // Transform lambda arrow syntax: () => to {}
        let value = self.transform_lambda_arrow(&value);

        // Transform dispatcher syntax: io/cpu/main { } to viewModelScope.launch(Dispatchers.X) { }
        let value = self.transform_dispatchers(&value);

        // Note: Padding/margin shortcuts (p, px, py, etc.) are handled at the component level
        // in generate_markup_with_indent where they can be combined properly

        // Component-specific transformations
        match (component, prop_name) {
            // TextField label → label = { Text("...") }
            ("TextField", "label") | ("OutlinedTextField", "label") => {
                let label_text = if value.starts_with('"') && value.ends_with('"') {
                    value[1..value.len()-1].to_string()
                } else {
                    value
                };
                Ok(vec![format!("label = {{ Text(\"{}\") }}", label_text)])
            }
            // TextField placeholder → placeholder = { Text("...") }
            ("TextField", "placeholder") | ("OutlinedTextField", "placeholder") => {
                let placeholder_text = if value.starts_with('"') && value.ends_with('"') {
                    value[1..value.len()-1].to_string()
                } else {
                    value
                };
                Ok(vec![format!("placeholder = {{ Text(\"{}\") }}", placeholder_text)])
            }
            // TextField type → visualTransformation (e.g., type="password" → PasswordVisualTransformation())
            // Note: Import is added during collect_imports phase
            ("TextField", "type") => {
                let type_value = if value.starts_with('"') && value.ends_with('"') {
                    &value[1..value.len()-1]
                } else {
                    value.as_str()
                };

                match type_value {
                    "password" => {
                        Ok(vec!["visualTransformation = PasswordVisualTransformation()".to_string()])
                    }
                    _ => {
                        // Unknown type - just pass through as-is
                        Ok(vec![format!("type = {}", value)])
                    }
                }
            }
            // Button text is handled differently - it becomes a child, not a prop
            ("Button", "text") => {
                // Return empty vec - text will be handled as child in generate_markup
                Ok(vec![])
            }
            // Button onClick needs braces.
            // Note: Strong skipping mode (Kotlin 2.1+) automatically memoizes lambdas,
            // so explicit remember {} is not needed. The "flash" issue was actually
            // caused by layout shift from AsyncImage loading, not recomposition.
            ("Button", "onClick") | ("IconButton", "onClick") => {
                // Note: transform_lambda_arrow has already run at this point
                // So () => expr has been converted to { expr }
                // We need to detect if expr is already a complete function call

                let format_onclick = |expr: &str| {
                    format!("onClick = {{ {} }}", expr)
                };

                if !value.starts_with('{') {
                    // Bare function name: increment
                    // Add (), transform
                    let with_parens = format!("{}()", value);
                    let transformed = self.transform_viewmodel_expression(&with_parens);
                    Ok(vec![format_onclick(&transformed)])
                } else {
                    // Has braces - strip them and check inner content
                    let inner = value.trim_start_matches('{').trim_end_matches('}').trim();

                    // Check if it's a multi-line block or contains statements
                    // Multi-line blocks and statement blocks should be preserved as-is
                    let is_statement_block = inner.contains('\n') ||
                                            inner.contains(';') ||
                                            inner.contains("++") ||
                                            inner.contains("--") ||
                                            (inner.contains('=') && !inner.starts_with("=="));

                    if is_statement_block {
                        // Multi-line or statement block: preserve as-is
                        // Just transform viewmodel references
                        let transformed = self.transform_viewmodel_expression(inner);
                        Ok(vec![format_onclick(&transformed)])
                    } else {
                        // Single-line expression - check if it's complete
                        let is_complete_expr = inner.contains('(') || inner.ends_with(')');

                        if is_complete_expr {
                            // Already complete: { clearItems() } or { navigate(Routes.Home) }
                            // Just transform
                            let transformed = self.transform_viewmodel_expression(inner);
                            Ok(vec![format_onclick(&transformed)])
                        } else {
                            // Bare function with braces: {increment}
                            // Add (), transform
                            let with_parens = format!("{}()", inner);
                            let transformed = self.transform_viewmodel_expression(&with_parens);
                            Ok(vec![format_onclick(&transformed)])
                        }
                    }
                }
            }
            // Column spacing → verticalArrangement = Arrangement.spacedBy(N.dp)
            ("Column", "spacing") => {
                let spacing_value = if value.ends_with(".dp") { value.to_string() } else { format!("{}.dp", value) };
                Ok(vec![format!("verticalArrangement = Arrangement.spacedBy({})", spacing_value)])
            }
            // Column padding → modifier = Modifier.padding(N.dp)
            ("Column", "padding") => {
                let padding_value = if value.ends_with(".dp") { value.to_string() } else { format!("{}.dp", value) };
                Ok(vec![format!("modifier = Modifier.padding({})", padding_value)])
            }
            // Column/Row backgroundColor → modifier = Modifier.background(Color)
            ("Column", "backgroundColor") | ("Row", "backgroundColor") => {
                let color = if value.starts_with('"') && value.ends_with('"') {
                    let s = &value[1..value.len()-1];
                    if s.starts_with('#') {
                        convert_hex_to_color(&s[1..])?
                    } else {
                        format!("Color.{}", s.chars().next().unwrap().to_uppercase().collect::<String>() + &s[1..])
                    }
                } else {
                    value
                };
                Ok(vec![format!("modifier = Modifier.background({})", color)])
            }
            // LazyColumn spacing → verticalArrangement = Arrangement.spacedBy(N.dp)
            ("LazyColumn", "spacing") => {
                let spacing_value = if value.ends_with(".dp") { value.to_string() } else { format!("{}.dp", value) };
                Ok(vec![format!("verticalArrangement = Arrangement.spacedBy({})", spacing_value)])
            }
            // LazyColumn padding → contentPadding = PaddingValues(N.dp)
            ("LazyColumn", "padding") => {
                let padding_value = if value.ends_with(".dp") { value.to_string() } else { format!("{}.dp", value) };
                Ok(vec![format!("contentPadding = PaddingValues({})", padding_value)])
            }
            // Row spacing → horizontalArrangement = Arrangement.spacedBy(N.dp)
            ("Row", "spacing") => {
                let spacing_value = if value.ends_with(".dp") { value.to_string() } else { format!("{}.dp", value) };
                Ok(vec![format!("horizontalArrangement = Arrangement.spacedBy({})", spacing_value)])
            }
            // Row padding → modifier = Modifier.padding(N.dp)
            ("Row", "padding") => {
                let padding_value = if value.ends_with(".dp") { value.to_string() } else { format!("{}.dp", value) };
                Ok(vec![format!("modifier = Modifier.padding({})", padding_value)])
            }
            // Text fontSize → fontSize = N.sp
            ("Text", "fontSize") => {
                let font_size_value = if value.ends_with(".sp") { value.to_string() } else { format!("{}.sp", value) };
                Ok(vec![format!("fontSize = {}", font_size_value)])
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
                Ok(vec![format!("fontWeight = {}", weight)])
            }
            // Text fontFamily string → FontFamily enum
            ("Text", "fontFamily") => {
                let family = if value.starts_with('"') && value.ends_with('"') {
                    // String literal "monospace" → FontFamily.Monospace
                    // "serif" → FontFamily.Serif, "sansSerif" → FontFamily.SansSerif, etc.
                    let s = &value[1..value.len()-1];
                    // Capitalize first letter for Kotlin enum
                    format!("FontFamily.{}", s.chars().next().unwrap().to_uppercase().collect::<String>() + &s[1..])
                } else {
                    value
                };
                Ok(vec![format!("fontFamily = {}", family)])
            }
            // Text color string → MaterialTheme.colorScheme or Color(0x...)
            ("Text", "color") => {
                // First transform ternary operators to Kotlin if-else (for non-modifier context)
                let value_transformed = self.transform_ternary_to_if_else(&value);
                let value = value_transformed.as_str();

                // Helper to convert a single color value (handles hex and named colors)
                let convert_single_color = |color_str: &str| -> Result<String, String> {
                    let trimmed = color_str.trim();
                    if trimmed.starts_with('"') && trimmed.ends_with('"') {
                        let s = &trimmed[1..trimmed.len()-1];
                        if s.starts_with('#') {
                            convert_hex_to_color(&s[1..])
                        } else if s.chars().all(|c| c.is_alphanumeric()) {
                            Ok(format!("MaterialTheme.colorScheme.{}", s))
                        } else {
                            Ok(trimmed.to_string())
                        }
                    } else {
                        Ok(trimmed.to_string())
                    }
                };

                // Check if value contains if-else (from ternary transformation)
                let color = if value.contains("if (") && value.contains(") ") && value.contains(" else ") {
                    // Parse and transform if-else branches
                    // Pattern: if (condition) value1 else value2
                    if let Some(if_pos) = value.find("if (") {
                        let cond_start = if_pos + 4; // After "if ("
                        if let Some(cond_end_rel) = value[cond_start..].find(") ") {
                            let cond_end = cond_start + cond_end_rel;
                            let then_start = cond_end + 2; // After ") "
                            if let Some(else_pos_rel) = value[then_start..].find(" else ") {
                                let else_pos = then_start + else_pos_rel;
                                let condition = &value[cond_start..cond_end];
                                let then_value_str = &value[then_start..else_pos];
                                let else_value_str = &value[else_pos + 6..]; // After " else "

                                let then_converted = convert_single_color(then_value_str)?;
                                let else_converted = convert_single_color(else_value_str)?;

                                format!("if ({}) {} else {}", condition, then_converted, else_converted)
                            } else {
                                value.to_string()
                            }
                        } else {
                            value.to_string()
                        }
                    } else {
                        value.to_string()
                    }
                } else if value.starts_with('"') && value.ends_with('"') {
                    convert_single_color(value)?
                } else {
                    value.to_string()
                };
                Ok(vec![format!("color = {}", color)])
            }
            // Card onClick → Card has native onClick parameter (Material3)
            ("Card", "onClick") => {
                let transformed = self.transform_lambda_arrow(&value);
                let on_click_value = if !transformed.starts_with('{') {
                    // Bare function name: add (), transform for ViewModel, wrap in braces
                    let with_parens = format!("{}()", transformed);
                    let with_viewmodel = self.transform_viewmodel_expression(&with_parens);
                    format!("{{ {} }}", with_viewmodel)
                } else {
                    // Already a lambda expression, just transform for ViewModel
                    self.transform_viewmodel_expression(&transformed)
                };
                Ok(vec![format!("onClick = {}", on_click_value)])
            }
            // Card backgroundColor → CardDefaults.cardColors()
            ("Card", "backgroundColor") => {
                let color = if value.starts_with('"') && value.ends_with('"') {
                    let s = &value[1..value.len()-1];
                    if s.starts_with('#') {
                        // Hex color: convert to Color(0xFFRRGGBB)
                        convert_hex_to_color(&s[1..])?
                    } else {
                        // Theme color: use MaterialTheme.colorScheme
                        format!("MaterialTheme.colorScheme.{}", s)
                    }
                } else {
                    // Expression or variable - convert color scheme names within it
                    self.convert_color_scheme_names(&value)
                };
                Ok(vec![format!(
                    "colors = CardDefaults.cardColors(\n                    containerColor = {}\n                )",
                    color
                )])
            }
            // Card elevation → CardDefaults.cardElevation() (Material3 API)
            ("Card", "elevation") => {
                let elevation_value = if value.ends_with(".dp") { value.to_string() } else { format!("{}.dp", value) };
                Ok(vec![format!("elevation = CardDefaults.cardElevation(defaultElevation = {})", elevation_value)])
            }
            // Default: no transformation
            _ => {
                Ok(vec![format!("{} = {}", prop_name, value)])
            }
        }
    }

    fn transform_lambda_arrow(&self, value: &str) -> String {
        // Transform lambda arrow syntax to Kotlin lambda syntax:
        // () => expr        → { expr }
        // (param) => expr   → { param -> expr }
        // (a, b) => expr    → { a, b -> expr }

        // Check for () => pattern (no parameters)
        if value.contains("() =>") {
            let transformed = value.replace("() =>", "").trim().to_string();
            // Check if already wrapped in braces (multi-line lambda)
            if transformed.starts_with('{') && transformed.ends_with('}') {
                // Already has braces, return as-is
                transformed
            } else {
                // Wrap in braces
                format!("{{ {} }}", transformed)
            }
        }
        // Check for (params) => pattern (with parameters)
        else if let Some(arrow_pos) = value.find(" =>") {
            // Check if there's a parameter list before =>
            let before_arrow = &value[..arrow_pos].trim();

            // Look for opening paren from the end of before_arrow
            if let Some(paren_start) = before_arrow.rfind('(') {
                let potential_params = &before_arrow[paren_start..];

                // Check if it's a proper parameter list: starts with ( and ends with )
                if potential_params.starts_with('(') && potential_params.ends_with(')') {
                    // Extract parameters (strip the parens)
                    let params = &potential_params[1..potential_params.len()-1];

                    // Get the expression after =>
                    let after_arrow = value[arrow_pos + 3..].trim(); // " =>" is 3 chars

                    // Build Kotlin lambda: { params -> expr }
                    let kotlin_lambda = if after_arrow.starts_with('{') && after_arrow.ends_with('}') {
                        // Multi-line lambda already has braces
                        format!("{{ {} -> {} }}", params, &after_arrow[1..after_arrow.len()-1].trim())
                    } else {
                        // Single expression
                        format!("{{ {} -> {} }}", params, after_arrow)
                    };

                    // If there was anything before the parameter list, preserve it
                    let prefix = &before_arrow[..paren_start];
                    if prefix.is_empty() {
                        kotlin_lambda
                    } else {
                        format!("{}{}", prefix, kotlin_lambda)
                    }
                } else {
                    // Not a proper parameter list, return as-is
                    value.to_string()
                }
            } else {
                // No opening paren found, return as-is
                value.to_string()
            }
        } else {
            // No arrow function syntax found, return as-is
            value.to_string()
        }
    }

    /// Transform dispatcher syntax: io { }, cpu { }, main { } to dispatcherScope.launch(Dispatchers.X) { }
    /// For components, uses dispatcherScope (rememberCoroutineScope)
    /// For ViewModels, would use viewModelScope (future enhancement)
    fn transform_dispatchers(&self, value: &str) -> String {
        let mut result = value.to_string();

        // Pattern: io { ... } → dispatcherScope.launch(Dispatchers.IO) { ... }
        result = result.replace("io {", "dispatcherScope.launch(Dispatchers.IO) {");

        // Pattern: cpu { ... } → dispatcherScope.launch(Dispatchers.Default) { ... }
        result = result.replace("cpu {", "dispatcherScope.launch(Dispatchers.Default) {");

        // Pattern: main { ... } → dispatcherScope.launch(Dispatchers.Main) { ... }
        result = result.replace("main {", "dispatcherScope.launch(Dispatchers.Main) {");

        result
    }

    /// Get the base package by removing screen-related suffixes
    /// e.g., com.example.app.screens -> com.example.app
    /// e.g., com.example.app.screens.detail -> com.example.app
    fn get_base_package(&self) -> String {
        let parts: Vec<&str> = self.package.split('.').collect();
        // Find the "screens" part and take everything before it
        if let Some(screens_idx) = parts.iter().position(|&p| p == "screens") {
            parts[..screens_idx].join(".")
        } else {
            // Fallback: remove last segment
            if parts.len() > 1 {
                parts[..parts.len() - 1].join(".")
            } else {
                self.package.clone()
            }
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

        // Scan markup for $screen.params.{name} (e.g., in props like src={...})
        self.extract_params_from_single_markup(&file.markup, &mut params);

        let mut param_vec: Vec<String> = params.into_iter().collect();
        param_vec.sort();
        param_vec
    }

    fn extract_params_from_single_markup(&self, markup: &Markup, params: &mut std::collections::HashSet<String>) {
        match markup {
            Markup::Component(comp) => {
                // Scan props
                for prop in &comp.props {
                    if let PropValue::Expression(expr) = &prop.value {
                        self.extract_params_from_text(expr, params);
                    }
                }
                // Scan children recursively
                for child in &comp.children {
                    self.extract_params_from_single_markup(child, params);
                }
            }
            Markup::Text(text) => {
                self.extract_params_from_text(text, params);
            }
            Markup::Interpolation(expr) => {
                self.extract_params_from_text(expr, params);
            }
            Markup::Sequence(items) => {
                for item in items {
                    self.extract_params_from_single_markup(item, params);
                }
            }
            Markup::IfElse(block) => {
                // Scan condition
                self.extract_params_from_text(&block.condition, params);
                // Scan branches
                for item in &block.then_branch {
                    self.extract_params_from_single_markup(item, params);
                }
                for else_if in &block.else_ifs {
                    self.extract_params_from_text(&else_if.condition, params);
                    for item in &else_if.body {
                        self.extract_params_from_single_markup(item, params);
                    }
                }
                if let Some(else_branch) = &block.else_branch {
                    for item in else_branch {
                        self.extract_params_from_single_markup(item, params);
                    }
                }
            }
            Markup::ForLoop(block) => {
                // Scan collection expression and body
                self.extract_params_from_text(&block.collection, params);
                for item in &block.body {
                    self.extract_params_from_single_markup(item, params);
                }
                if let Some(empty) = &block.empty_block {
                    for item in empty {
                        self.extract_params_from_single_markup(item, params);
                    }
                }
            }
            Markup::When(block) => {
                // Scan branches
                for branch in &block.branches {
                    if let Some(cond) = &branch.condition {
                        self.extract_params_from_text(cond, params);
                    }
                    self.extract_params_from_single_markup(&branch.body, params);
                }
            }
        }
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

    /// Transform array literal [1, 2, 3] to listOf(1, 2, 3) or mutableListOf(1, 2, 3)
    /// Recursively transforms nested arrays
    fn transform_array_literal(&self, value: &str, is_mutable: bool) -> String {
        let trimmed = value.trim();

        // Check if it starts with [ and ends with ]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let content = &trimmed[1..trimmed.len()-1];

            // Recursively transform nested arrays (always immutable for nested arrays)
            let transformed_content = self.transform_nested_arrays(content);

            let func_name = if is_mutable { "mutableListOf" } else { "listOf" };
            format!("{}({})", func_name, transformed_content)
        } else {
            value.to_string()
        }
    }

    /// Helper to recursively transform nested arrays
    fn transform_nested_arrays(&self, content: &str) -> String {
        let mut result = String::new();
        let mut chars = content.chars().peekable();
        let mut depth = 0;
        let mut current_array = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                '[' => {
                    if depth == 0 {
                        // Start of a nested array
                        current_array.clear();
                        current_array.push(ch);
                    } else {
                        current_array.push(ch);
                    }
                    depth += 1;
                }
                ']' => {
                    current_array.push(ch);
                    depth -= 1;
                    if depth == 0 {
                        // End of nested array, transform it
                        let transformed = self.transform_array_literal(&current_array, false);
                        result.push_str(&transformed);
                        current_array.clear();
                    }
                }
                _ => {
                    if depth > 0 {
                        current_array.push(ch);
                    } else {
                        result.push(ch);
                    }
                }
            }
        }

        result
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

    /// Re-indent a multi-line expression to match current indent level
    /// Preserves relative indentation between lines
    fn reindent_expression(&self, expr: &str) -> String {
        let lines: Vec<&str> = expr.lines().collect();
        if lines.is_empty() {
            return expr.to_string();
        }

        // Find minimum indentation (excluding empty lines and first line)
        let min_indent = lines.iter()
            .skip(1) // Skip first line
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.len() - line.trim_start().len())
            .min()
            .unwrap_or(0);

        // Re-indent each line
        let reindented: Vec<String> = lines.iter().enumerate()
            .map(|(i, line)| {
                if line.trim().is_empty() {
                    // Preserve empty lines
                    String::new()
                } else if i == 0 {
                    // First line stays on same line as 'val x = '
                    line.trim().to_string()
                } else {
                    // Calculate original indentation relative to minimum
                    let original_indent = line.len() - line.trim_start().len();
                    let relative_levels = original_indent.saturating_sub(min_indent) / 2; // Source uses 2 spaces/level

                    // New indentation: current level + relative levels (both use 4 spaces/level in output)
                    let new_indent = (self.indent_level + relative_levels) * 4;
                    let spaces = " ".repeat(new_indent);
                    format!("{}{}", spaces, line.trim_start())
                }
            })
            .collect();

        reindented.join("\n")
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

    /// Parse a dimension value and return the Kotlin expression.
    ///
    /// Rules:
    /// - `{300}` (number) → `300.dp` (defaults to dp)
    /// - `{someVar}` (variable) → `someVar` (used as-is, assumed to have units)
    /// - `"300dp"` (string with unit) → `300.dp`
    /// - `"300sp"` (string with unit) → `300.sp`
    /// - `"100%"` (percentage) → returns None (caller handles as fillMax*)
    /// - `"300"` (string without unit) → `300.dp` with warning
    ///
    /// Returns: (kotlin_expr, is_percentage)
    fn parse_dimension(&self, value: &str, prop_name: &str, _component_name: &str) -> (String, bool) {
        let trimmed = value.trim();

        // Check if it's a quoted string
        let is_string = trimmed.starts_with('"') && trimmed.ends_with('"');
        let inner = if is_string {
            trimmed.trim_matches('"')
        } else {
            trimmed
        };

        // Handle percentage
        if inner == "100%" {
            return (String::new(), true);
        }

        // Handle string with units
        if is_string {
            if inner.ends_with("dp") {
                let num = inner.trim_end_matches("dp");
                return (format!("{}.dp", num), false);
            }
            if inner.ends_with("sp") {
                let num = inner.trim_end_matches("sp");
                return (format!("{}.sp", num), false);
            }
            // String without unit - be lenient but warn
            if inner.parse::<f64>().is_ok() {
                eprintln!(
                    "Warning: {}=\"{}\" has no unit, assuming \"{}dp\". Consider using {{{}}} or \"{}dp\" for clarity.",
                    prop_name, inner, inner, inner, inner
                );
                return (format!("{}.dp", inner), false);
            }
            // Some other string value (like a variable reference in quotes?)
            return (inner.to_string(), false);
        }

        // Handle numeric value (from {300})
        if trimmed.parse::<f64>().is_ok() {
            return (format!("{}.dp", trimmed), false);
        }

        // Handle variable/expression (from {someVar} or {size * 2})
        // Check if it already has .dp or .sp suffix
        if trimmed.ends_with(".dp") || trimmed.ends_with(".sp") {
            return (trimmed.to_string(), false);
        }

        // Variable without unit - use as-is (caller is responsible for units)
        (trimmed.to_string(), false)
    }

    /// Build modifiers for a component based on configuration.
    ///
    /// Returns: (modifiers_vec, handled_props_set)
    /// - modifiers_vec: List of modifier strings like ".fillMaxWidth()", ".padding(8.dp)"
    /// - handled_props_set: Set of prop names that were handled and should be skipped in normal prop processing
    fn build_modifiers_for_component(
        &self,
        comp: &crate::transpiler::ast::Component,
        config: ModifierConfig,
    ) -> Result<(Vec<String>, std::collections::HashSet<String>), String> {
        let mut modifiers = Vec::new();
        let mut handled = std::collections::HashSet::new();

        // Handle fillMaxSize/fillMaxWidth/fillMaxHeight
        if config.handle_fill_max || config.handle_size {
            if let Some(fs) = comp.props.iter().find(|p| p.name == "fillMaxSize") {
                let value = self.get_prop_expr(&fs.value);
                if value.trim() == "true" {
                    modifiers.push(".fillMaxSize()".to_string());
                }
                handled.insert("fillMaxSize".to_string());
            }
            if let Some(fw) = comp.props.iter().find(|p| p.name == "fillMaxWidth") {
                let value = self.get_prop_expr(&fw.value);
                if value.trim() == "true" && !modifiers.iter().any(|m| m.contains("fillMaxSize")) {
                    modifiers.push(".fillMaxWidth()".to_string());
                }
                handled.insert("fillMaxWidth".to_string());
            }
            if let Some(fh) = comp.props.iter().find(|p| p.name == "fillMaxHeight") {
                let value = self.get_prop_expr(&fh.value);
                if value.trim() == "true" && !modifiers.iter().any(|m| m.contains("fillMaxSize")) {
                    modifiers.push(".fillMaxHeight()".to_string());
                }
                handled.insert("fillMaxHeight".to_string());
            }
        }

        // Handle width/height props (100% → fillMax*, fixed → .width()/.height())
        if config.handle_size {
            if let Some(w) = comp.props.iter().find(|p| p.name == "width") {
                let value = self.get_prop_expr(&w.value);
                let (dim_expr, is_percent) = self.parse_dimension(&value, "width", &comp.name);
                if is_percent {
                    if !modifiers.iter().any(|m| m.contains("fillMaxWidth") || m.contains("fillMaxSize")) {
                        modifiers.push(".fillMaxWidth()".to_string());
                    }
                } else if !dim_expr.is_empty() {
                    modifiers.push(format!(".width({})", dim_expr));
                }
                handled.insert("width".to_string());
            }
            if let Some(h) = comp.props.iter().find(|p| p.name == "height") {
                let value = self.get_prop_expr(&h.value);
                let (dim_expr, is_percent) = self.parse_dimension(&value, "height", &comp.name);
                if is_percent {
                    if !modifiers.iter().any(|m| m.contains("fillMaxHeight") || m.contains("fillMaxSize")) {
                        modifiers.push(".fillMaxHeight()".to_string());
                    }
                } else if !dim_expr.is_empty() {
                    modifiers.push(format!(".height({})", dim_expr));
                }
                handled.insert("height".to_string());
            }
        }

        // Handle backgroundColor
        if config.handle_background {
            if let Some(bg) = comp.props.iter().find(|p| p.name == "backgroundColor") {
                let color = self.get_prop_expr(&bg.value);
                let color_str = if color.starts_with('"') && color.ends_with('"') {
                    let c = &color[1..color.len()-1];
                    if c.starts_with('#') {
                        convert_hex_to_color(&c[1..])?
                    } else {
                        format!("Color.{}", c.chars().next().unwrap().to_uppercase().collect::<String>() + &c[1..])
                    }
                } else {
                    color.to_string()
                };
                modifiers.push(format!(".background({})", color_str));
                handled.insert("backgroundColor".to_string());
            }
        }

        // Handle padding with Tailwind-style cascade
        if config.handle_padding {
            let padding_prop = comp.props.iter().find(|p| p.name == "padding");
            let padding_shortcuts: Vec<_> = comp.props.iter()
                .filter(|p| matches!(p.name.as_str(), "p" | "px" | "py" | "pt" | "pb" | "pl" | "pr"))
                .collect();

            let base_padding = padding_prop.map(|p| self.get_prop_expr(&p.value));
            if let Some(padding_mod) = self.build_padding_modifier(&padding_shortcuts, base_padding.as_deref()) {
                modifiers.push(padding_mod);
            }

            // Mark padding props as handled
            if padding_prop.is_some() {
                handled.insert("padding".to_string());
            }
            for shortcut in &["p", "px", "py", "pt", "pb", "pl", "pr"] {
                if comp.props.iter().any(|p| p.name == *shortcut) {
                    handled.insert(shortcut.to_string());
                }
            }
        }

        // Handle onClick as clickable modifier (for Text and similar components)
        if config.handle_click_as_modifier {
            if let Some(click_prop) = comp.props.iter().find(|p| p.name == "onClick") {
                let click_value = self.get_prop_expr(&click_prop.value);
                let transformed = self.transform_lambda_arrow(&click_value);

                let clickable_expr = if !transformed.starts_with('{') {
                    let with_parens = format!("{}()", transformed);
                    let with_viewmodel = self.transform_viewmodel_expression(&with_parens);
                    format!("{{ {} }}", with_viewmodel)
                } else {
                    self.transform_viewmodel_expression(&transformed)
                };

                modifiers.push(format!(".clickable {}", clickable_expr));
                handled.insert("onClick".to_string());
            }
        }

        Ok((modifiers, handled))
    }

    /// Build padding modifier string from Tailwind-style shortcuts with proper cascade.
    ///
    /// Priority (highest wins):
    /// 1. pt, pb, pl, pr (specific sides)
    /// 2. px, py (axis)
    /// 3. p / padding (all sides)
    ///
    /// Example: p={8} pl={16} → padding(top=8.dp, bottom=8.dp, start=16.dp, end=8.dp)
    fn build_padding_modifier(&self, props: &[&crate::transpiler::ast::ComponentProp], base_padding: Option<&str>) -> Option<String> {
        // Collect values with priority
        let mut top: Option<(String, u8)> = None;    // (value, priority)
        let mut bottom: Option<(String, u8)> = None;
        let mut start: Option<(String, u8)> = None;
        let mut end: Option<(String, u8)> = None;

        // Priority 1: base padding prop (lowest)
        if let Some(pad) = base_padding {
            let value = if pad.ends_with(".dp") { pad.to_string() } else { format!("{}.dp", pad) };
            top = Some((value.clone(), 1));
            bottom = Some((value.clone(), 1));
            start = Some((value.clone(), 1));
            end = Some((value, 1));
        }

        for prop in props {
            let value = self.get_prop_expr(&prop.value);
            let dp_value = format!("{}.dp", value);

            match prop.name.as_str() {
                // Priority 1: all sides (same as base padding)
                "p" => {
                    if top.as_ref().map_or(true, |(_, p)| *p <= 1) { top = Some((dp_value.clone(), 1)); }
                    if bottom.as_ref().map_or(true, |(_, p)| *p <= 1) { bottom = Some((dp_value.clone(), 1)); }
                    if start.as_ref().map_or(true, |(_, p)| *p <= 1) { start = Some((dp_value.clone(), 1)); }
                    if end.as_ref().map_or(true, |(_, p)| *p <= 1) { end = Some((dp_value, 1)); }
                }
                // Priority 2: axis
                "px" => {
                    if start.as_ref().map_or(true, |(_, p)| *p <= 2) { start = Some((dp_value.clone(), 2)); }
                    if end.as_ref().map_or(true, |(_, p)| *p <= 2) { end = Some((dp_value, 2)); }
                }
                "py" => {
                    if top.as_ref().map_or(true, |(_, p)| *p <= 2) { top = Some((dp_value.clone(), 2)); }
                    if bottom.as_ref().map_or(true, |(_, p)| *p <= 2) { bottom = Some((dp_value, 2)); }
                }
                // Priority 3: specific sides (highest)
                "pt" => { top = Some((dp_value, 3)); }
                "pb" => { bottom = Some((dp_value, 3)); }
                "pl" => { start = Some((dp_value, 3)); }
                "pr" => { end = Some((dp_value, 3)); }
                _ => {}
            }
        }

        // Generate padding call
        let top_val = top.map(|(v, _)| v);
        let bottom_val = bottom.map(|(v, _)| v);
        let start_val = start.map(|(v, _)| v);
        let end_val = end.map(|(v, _)| v);

        // Check if all values are the same (can use simple padding)
        if let (Some(t), Some(b), Some(s), Some(e)) = (&top_val, &bottom_val, &start_val, &end_val) {
            if t == b && b == s && s == e {
                return Some(format!(".padding({})", t));
            }
        }

        // Check for horizontal/vertical shorthand
        let h_same = start_val == end_val;
        let v_same = top_val == bottom_val;

        if h_same && v_same && start_val.is_some() && top_val.is_some() {
            let h = start_val.unwrap();
            let v = top_val.unwrap();
            if h == v {
                return Some(format!(".padding({})", h));
            }
            return Some(format!(".padding(horizontal = {}, vertical = {})", h, v));
        }

        // Build individual padding parts
        let mut parts = Vec::new();
        if let Some(t) = top_val { parts.push(format!("top = {}", t)); }
        if let Some(b) = bottom_val { parts.push(format!("bottom = {}", b)); }
        if let Some(s) = start_val { parts.push(format!("start = {}", s)); }
        if let Some(e) = end_val { parts.push(format!("end = {}", e)); }

        if parts.is_empty() {
            None
        } else {
            Some(format!(".padding({})", parts.join(", ")))
        }
    }

    /// Detect if a file uses the $fetch() API by scanning state and lifecycle hooks
    fn detect_fetch_usage(&mut self, file: &WhitehallFile) {
        // Check state initial values
        for state in &file.state {
            if state.initial_value.contains("$fetch(") {
                self.uses_fetch = true;
                return;
            }
        }
        // Check lifecycle hooks
        for hook in &file.lifecycle_hooks {
            if hook.body.contains("$fetch(") {
                self.uses_fetch = true;
                return;
            }
        }
        // Check functions
        for func in &file.functions {
            if func.body.contains("$fetch(") {
                self.uses_fetch = true;
                return;
            }
        }
    }

    /// Detect if a file uses $routes or $navigate (needs Routes import)
    fn detect_routes_usage(&mut self, file: &WhitehallFile) {
        // Check functions for $routes or $navigate
        for func in &file.functions {
            if func.body.contains("$routes") || func.body.contains("$navigate") {
                self.uses_routes = true;
                return;
            }
        }
        // Check lifecycle hooks
        for hook in &file.lifecycle_hooks {
            if hook.body.contains("$routes") || hook.body.contains("$navigate") {
                self.uses_routes = true;
                return;
            }
        }
        // Check markup (onClick handlers, etc.)
        let markup_str = format!("{:?}", file.markup);
        if markup_str.contains("$routes") || markup_str.contains("$navigate") {
            self.uses_routes = true;
        }
    }

    /// Transform $fetch() calls to Ktor HttpClient calls
    /// Input: photos = $fetch("https://api.example.com/data")
    /// Output: photos = httpClient.get("https://api.example.com/data").body()
    fn transform_fetch_call(&self, line: &str) -> String {
        // Simple regex-like replacement for $fetch("url") -> httpClient.get("url").body()
        // Handle: $fetch("url")
        let mut result = line.to_string();

        // Find $fetch( and replace with httpClient.get(
        if let Some(start) = result.find("$fetch(") {
            // Find the matching closing paren
            let after_fetch = &result[start + 7..]; // after "$fetch("
            let mut depth = 1;
            let mut end_pos = 0;

            for (i, ch) in after_fetch.char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            end_pos = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if depth == 0 {
                // Extract the URL argument
                let url_arg = &after_fetch[..end_pos];
                // Replace fetch(...) with httpClient.get(...).body()
                let replacement = format!("httpClient.get({}).body()", url_arg);
                result = format!("{}{}{}", &result[..start], replacement, &after_fetch[end_pos + 1..]);
            }
        }

        result
    }

    /// Generate HttpClient singleton for $fetch() API
    fn generate_http_client(&self) -> String {
        r#"private val httpClient = HttpClient(OkHttp) {
    install(ContentNegotiation) {
        json(Json { ignoreUnknownKeys = true })
    }
}

"#.to_string()
    }

    /// Check if an expression references any mutable state variables
    /// Used to determine if a component is "stable" (doesn't depend on state)
    /// Note: Currently unused - key(Unit) optimization was disabled
    #[allow(dead_code)]
    fn expr_references_mutable_var(&self, expr: &str) -> bool {
        for var_name in &self.mutable_vars {
            // Check for the variable as a word boundary (not substring of another identifier)
            // Simple heuristic: check if var appears and is preceded/followed by non-alphanumeric
            let pattern = format!(r"\b{}\b", regex::escape(var_name));
            if let Ok(re) = regex::Regex::new(&pattern) {
                if re.is_match(expr) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a component is "stable" - doesn't depend on any mutable state
    /// A stable component can be wrapped in key(Unit) to prevent recomposition
    /// Note: Currently unused - key(Unit) optimization was disabled
    #[allow(dead_code)]
    fn is_component_stable(&self, comp: &Component) -> bool {
        // Check all prop values for state references
        for prop in &comp.props {
            let expr = self.get_prop_expr(&prop.value);

            // String literals are stable UNLESS they contain interpolations ${...} or {...}
            if expr.starts_with('"') && expr.ends_with('"') {
                // Check if string contains interpolation that references mutable state
                // Handle both Kotlin ${...} and Whitehall {...} interpolation syntax
                let has_interpolation = expr.contains("${") ||
                    (expr.contains('{') && expr.contains('}') && !expr.contains("{{"));
                if has_interpolation {
                    // Extract variable references from ${...} or {...} expressions
                    let mut found_mutable_ref = false;
                    // Check for ${...} pattern (Kotlin style)
                    for cap in regex::Regex::new(r"\$\{([^}]+)\}").unwrap().captures_iter(expr) {
                        let inner_expr = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        if self.expr_references_mutable_var(inner_expr) {
                            found_mutable_ref = true;
                            break;
                        }
                    }
                    // Check for {...} pattern (Whitehall style) - simple pattern without lookbehind
                    // This will match {foo} but also {{foo}} - we handle escaped braces below
                    if !found_mutable_ref {
                        for cap in regex::Regex::new(r"\{([^{}]+)\}").unwrap().captures_iter(expr) {
                            let inner_expr = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                            // Skip if this is an escaped brace {{...}}
                            let match_start = cap.get(0).map(|m| m.start()).unwrap_or(0);
                            if match_start > 0 && &expr[match_start-1..match_start] == "{" {
                                continue; // Part of {{...}}, skip
                            }
                            if self.expr_references_mutable_var(inner_expr) {
                                found_mutable_ref = true;
                                break;
                            }
                        }
                    }
                    if found_mutable_ref {
                        return false;
                    }
                }
                continue;
            }
            if expr.parse::<f64>().is_ok() {
                continue;
            }
            if expr == "true" || expr == "false" {
                continue;
            }

            // onClick handlers make components unstable for recomposition
            // because lambdas are recreated on each recomposition.
            // Components with onClick should NOT be wrapped in key(Unit)
            // because key(Unit) doesn't prevent recomposition for unstable params.
            if prop.name == "onClick" {
                return false;
            }

            // Check if expression references any mutable state
            if self.expr_references_mutable_var(&expr) {
                return false;
            }
        }

        // Also check children recursively
        for child in &comp.children {
            if let Markup::Component(child_comp) = child {
                if !self.is_component_stable(child_comp) {
                    return false;
                }
            }
            // Interpolations in children reference state
            if let Markup::Interpolation(expr) = child {
                if self.expr_references_mutable_var(expr) {
                    return false;
                }
            }
        }

        true
    }

    /// Phase 1.1: Transform expression for ViewModel wrapper context
    /// Transforms variable references and function calls when in ViewModel wrapper
    fn transform_viewmodel_expression(&self, expr: &str) -> String {
        if !self.in_viewmodel_wrapper {
            return expr.to_string();
        }

        let mut result = expr.to_string();

        // Zeroth pass: Transform assignments to update method calls
        // varName = expr → viewModel.updateVarName(expr)
        // This must happen BEFORE variable references are transformed
        for var_name in &self.mutable_vars {
            // Match pattern: varName = value (but not ==, !=, <=, >=)
            // Use regex-like logic to find assignments
            let mut new_result = String::new();
            let mut remaining = result.as_str();

            while let Some(idx) = remaining.find(var_name) {
                // Add everything before the match
                new_result.push_str(&remaining[..idx]);

                // Check if this is an assignment
                let after_var = &remaining[idx + var_name.len()..];
                let trimmed_after = after_var.trim_start();

                // Check if followed by = (but not ==, !=, etc.)
                if trimmed_after.starts_with('=') && !trimmed_after.starts_with("==") {
                    // This is an assignment!
                    // Extract the value being assigned (up to comma, closing brace, or semicolon)
                    let mut depth = 0;
                    let mut value_end = 0;
                    let value_start = trimmed_after.find('=').unwrap() + 1;

                    // Get the part after = and track how many spaces we trim
                    let before_trim = &trimmed_after[value_start..];
                    let value_part = before_trim.trim_start();
                    let spaces_trimmed = before_trim.len() - value_part.len();

                    for (i, ch) in value_part.char_indices() {
                        match ch {
                            '(' | '{' | '[' => depth += 1,
                            ')' | '}' | ']' => {
                                if depth > 0 {
                                    depth -= 1;
                                } else {
                                    value_end = i;
                                    break;
                                }
                            }
                            ',' | ';' if depth == 0 => {
                                value_end = i;
                                break;
                            }
                            _ => {}
                        }
                    }

                    if value_end == 0 {
                        value_end = value_part.len();
                    }

                    let assigned_value = value_part[..value_end].trim();

                    // Generate update method call
                    let method_name = format!("update{}{}",
                        var_name.chars().next().unwrap().to_uppercase(),
                        &var_name[1..]
                    );
                    new_result.push_str(&format!("viewModel.{}({})", method_name, assigned_value));

                    // Skip past the assignment in remaining
                    // Calculate total amount to skip:
                    // - whitespace before = (trim_offset)
                    // - the = sign (included in value_start)
                    // - whitespace after = (spaces_trimmed)
                    // - the value itself (value_end)
                    let trim_offset = after_var.len() - trimmed_after.len();
                    let skip_amount = trim_offset + value_start + spaces_trimmed + value_end;

                    if skip_amount <= after_var.len() {
                        remaining = &after_var[skip_amount..];
                    } else {
                        // Safety: if calculation is wrong, just skip to end
                        remaining = "";
                    }
                } else {
                    // Not an assignment, just keep the variable name for later transformation
                    new_result.push_str(var_name);
                    remaining = after_var;
                }
            }

            // Add any remaining text
            new_result.push_str(remaining);
            result = new_result;
        }

        // First pass: Transform standalone variable references (not in interpolations)
        // This handles cases like: if (varName != null) or text = varName
        // We need to be careful to only match whole words
        // Note: function names are NOT transformed here - they're handled in the third pass with ()
        for var_name in &self.mutable_vars {
            result = self.replace_identifier(&result, var_name, &format!("uiState.{}", var_name));
        }
        for prop_name in &self.derived_props {
            result = self.replace_identifier(&result, prop_name, &format!("viewModel.{}", prop_name));
        }

        // Second pass: Transform string interpolations: ${varName} → ${uiState.varName} or ${viewModel.prop}
        let mut transformed = String::new();
        let mut chars = result.chars().peekable();
        let mut in_interpolation = false;
        let mut current_token = String::new();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                transformed.push(ch);
                in_interpolation = true;
                current_token.clear();
            } else if ch == '{' && in_interpolation {
                transformed.push(ch);
            } else if ch == '}' && in_interpolation {
                // Already handled in first pass - just copy
                transformed.push_str(&current_token);
                transformed.push(ch);
                in_interpolation = false;
                current_token.clear();
            } else if in_interpolation {
                current_token.push(ch);
            } else {
                transformed.push(ch);
            }
        }

        result = transformed;

        // Third pass: Transform function calls: functionName() → viewModel.functionName()
        for func_name in &self.function_names {
            let pattern = format!("{}(", func_name);
            let replacement = format!("viewModel.{}(", func_name);

            let parts: Vec<&str> = result.split(&pattern).collect();
            let mut new_result = String::new();
            for (i, part) in parts.iter().enumerate() {
                new_result.push_str(part);
                if i < parts.len() - 1 {
                    // Check if already prefixed
                    if !part.ends_with("viewModel.") {
                        new_result.push_str(&replacement);
                    } else {
                        new_result.push_str(&pattern);
                    }
                }
            }
            result = new_result;
        }

        result
    }

    /// Helper: Replace identifier in expression (whole word only)
    /// Replaces standalone references to an identifier with a new value
    /// Only replaces if it's a whole word (not part of another identifier)
    /// Skips replacements inside string literals but DOES transform inside ${} interpolations
    fn replace_identifier(&self, expr: &str, identifier: &str, replacement: &str) -> String {
        let mut result = String::new();
        let chars: Vec<char> = expr.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            // Handle string literals
            if ch == '"' || ch == '\'' {
                let quote = ch;
                result.push(ch);
                i += 1;

                // Process string contents
                while i < chars.len() {
                    let inner_ch = chars[i];

                    if inner_ch == '\\' && i + 1 < chars.len() {
                        // Handle escape sequences - copy both chars
                        result.push(inner_ch);
                        i += 1;
                        result.push(chars[i]);
                        i += 1;
                        continue;
                    } else if inner_ch == '$' && i + 1 < chars.len() && chars[i + 1] == '{' {
                        // String interpolation ${...} - need to transform inside
                        result.push('$');
                        result.push('{');
                        i += 2;

                        // Find the closing } and process the interpolation content
                        let interp_start = i;
                        let mut depth = 1;
                        while i < chars.len() && depth > 0 {
                            if chars[i] == '{' {
                                depth += 1;
                            } else if chars[i] == '}' {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            i += 1;
                        }

                        // Transform the interpolation content
                        let interp_content: String = chars[interp_start..i].iter().collect();
                        let transformed_interp = self.replace_identifier(&interp_content, identifier, replacement);
                        result.push_str(&transformed_interp);

                        if i < chars.len() && chars[i] == '}' {
                            result.push('}');
                            i += 1;
                        }
                        continue;
                    } else if inner_ch == quote {
                        // Found closing quote
                        result.push(inner_ch);
                        i += 1;
                        break;
                    } else {
                        // Regular character inside string (not interpolation)
                        result.push(inner_ch);
                        i += 1;
                    }
                }
                continue;
            }

            // Build identifier outside of strings
            if ch.is_alphanumeric() || ch == '_' {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }

                let word: String = chars[start..i].iter().collect();
                if word == identifier {
                    result.push_str(replacement);
                } else {
                    result.push_str(&word);
                }
            } else {
                result.push(ch);
                i += 1;
            }
        }

        result
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
    fn generate_scaffold_child(&mut self, markup: &Markup, indent: usize) -> Result<String, String> {
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

    /// Phase 6: Generate RecyclerView with AndroidView wrapper (inline optimization)
    ///
    /// This is called when we detect a UseRecyclerView optimization for a for loop.
    /// Generates an AndroidView that creates and binds a RecyclerView instead of LazyColumn.
    fn generate_recyclerview_inline(&self, for_loop: &ForLoopBlock, indent: usize) -> Result<String, String> {
        let mut output = String::new();
        let indent_str = "    ".repeat(indent);

        // Generate AndroidView that creates RecyclerView
        output.push_str(&format!("{}AndroidView(\n", indent_str));
        output.push_str(&format!("{}    factory = {{ context ->\n", indent_str));
        output.push_str(&format!("{}        RecyclerView(context).apply {{\n", indent_str));
        output.push_str(&format!("{}            layoutManager = LinearLayoutManager(context)\n", indent_str));
        output.push_str(&format!("{}            adapter = object : RecyclerView.Adapter<RecyclerView.ViewHolder>() {{\n", indent_str));
        output.push_str(&format!("{}                override fun getItemCount() = {}.size\n", indent_str, for_loop.collection));
        output.push_str(&format!("{}\n", indent_str));
        output.push_str(&format!("{}                override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): RecyclerView.ViewHolder {{\n", indent_str));
        output.push_str(&format!("{}                    val view = TextView(parent.context).apply {{\n", indent_str));
        output.push_str(&format!("{}                        layoutParams = ViewGroup.LayoutParams(\n", indent_str));
        output.push_str(&format!("{}                            ViewGroup.LayoutParams.MATCH_PARENT,\n", indent_str));
        output.push_str(&format!("{}                            ViewGroup.LayoutParams.WRAP_CONTENT\n", indent_str));
        output.push_str(&format!("{}                        )\n", indent_str));
        output.push_str(&format!("{}                        setPadding(16.dpToPx(), 16.dpToPx(), 16.dpToPx(), 16.dpToPx())\n", indent_str));
        output.push_str(&format!("{}                    }}\n", indent_str));
        output.push_str(&format!("{}                    return object : RecyclerView.ViewHolder(view) {{}}\n", indent_str));
        output.push_str(&format!("{}                }}\n", indent_str));
        output.push_str(&format!("{}\n", indent_str));
        output.push_str(&format!("{}                override fun onBindViewHolder(holder: RecyclerView.ViewHolder, position: Int) {{\n", indent_str));
        output.push_str(&format!("{}                    val {} = {}[position]\n", indent_str, for_loop.item, for_loop.collection));
        output.push_str(&format!("{}                    val textView = holder.itemView as TextView\n", indent_str));

        // Extract text from body (simplified - assumes Text component)
        // For now, just use the item variable directly
        output.push_str(&format!("{}                    textView.text = {}.toString()\n", indent_str, for_loop.item));

        output.push_str(&format!("{}                }}\n", indent_str));
        output.push_str(&format!("{}            }}\n", indent_str));
        output.push_str(&format!("{}        }}\n", indent_str));
        output.push_str(&format!("{}    }}\n", indent_str));
        output.push_str(&format!("{})\n", indent_str));

        // Add DP extension helper
        output.push_str(&format!("{}\n", indent_str));
        output.push_str(&format!("{}// Extension for DP to PX conversion\n", indent_str));
        output.push_str(&format!("{}private fun Int.dpToPx(): Int {{\n", indent_str));
        output.push_str(&format!("{}    val density = Resources.getSystem().displayMetrics.density\n", indent_str));
        output.push_str(&format!("{}    return (this * density).toInt()\n", indent_str));
        output.push_str(&format!("{}}}\n", indent_str));

        Ok(output)
    }

    /// Phase 1: Generate ViewModel or Singleton code for reactive class
    fn generate_store_class(&self, file: &WhitehallFile, class: &ClassDeclaration) -> Result<String, String> {
        // Check if this is a singleton (@store object) or ViewModel (class/component with var)
        let source = if let Some(registry) = &self.store_registry {
            registry.get(&class.name)
                .map(|info| info.source.clone())
                .unwrap_or(crate::transpiler::analyzer::StoreSource::Class)
        } else {
            crate::transpiler::analyzer::StoreSource::Class
        };

        if source == crate::transpiler::analyzer::StoreSource::Singleton {
            // Generate singleton StateFlow code
            self.generate_singleton_store(file, class)
        } else {
            // Generate ViewModel code (for both Class and ComponentInline sources)
            self.generate_view_model_store(file, class, source)
        }
    }

    /// Generate singleton StateFlow code for @store object
    fn generate_singleton_store(&self, file: &WhitehallFile, class: &ClassDeclaration) -> Result<String, String> {
        let mut output = String::new();

        // Package declaration
        output.push_str(&format!("package {}\n\n", self.package));

        // Collect all imports
        let mut imports = Vec::new();

        // Imports (no ViewModel, no viewModelScope)
        imports.push("kotlinx.coroutines.flow.MutableStateFlow".to_string());
        imports.push("kotlinx.coroutines.flow.StateFlow".to_string());
        imports.push("kotlinx.coroutines.flow.asStateFlow".to_string());
        imports.push("kotlinx.coroutines.flow.update".to_string());

        // Add user imports from file
        for import in &file.imports {
            let resolved = self.resolve_import(&import.path);
            imports.push(resolved);
        }

        // Sort and deduplicate
        imports.sort();
        imports.dedup();

        // Write imports
        for import in imports {
            output.push_str(&format!("import {}\n", import));
        }
        output.push('\n');

        // Object declaration (not class!)
        output.push_str(&format!("object {} {{\n", class.name));

        // Generate UiState data class (same as ViewModel)
        let var_properties: Vec<_> = class.properties.iter().filter(|p| p.mutable).collect();

        if !var_properties.is_empty() {
            output.push_str("    data class State(\n");
            for (i, prop) in var_properties.iter().enumerate() {
                let type_annotation = prop.type_annotation.as_ref()
                    .map(|t| format!(": {}", t))
                    .unwrap_or_else(|| String::from(": Any"));
                let initial_value = prop.initial_value.as_ref()
                    .map(|v| format!(" = {}", v))
                    .unwrap_or_default();

                let comma = if i < var_properties.len() - 1 { "," } else { "" };
                output.push_str(&format!("        val {}{}{}{}\n",
                    prop.name, type_annotation, initial_value, comma));
            }
            output.push_str("    )\n\n");

            // StateFlow setup (no viewModelScope - it's a singleton)
            output.push_str("    private val _state = MutableStateFlow(State())\n");
            output.push_str("    val state: StateFlow<State> = _state.asStateFlow()\n\n");

            // Property accessors
            for prop in &var_properties {
                output.push_str(&format!("    var {}: {}\n",
                    prop.name,
                    prop.type_annotation.as_ref().unwrap_or(&"Any".to_string())));
                output.push_str(&format!("        get() = _state.value.{}\n", prop.name));
                output.push_str(&format!("        set(value) {{ _state.update {{ it.copy({} = value) }} }}\n\n",
                    prop.name));
            }
        }

        // Derived properties (val with getter)
        for prop in &class.properties {
            if !prop.mutable {
                if let Some(getter) = &prop.getter {
                    let type_annotation = prop.type_annotation.as_ref()
                        .map(|t| format!(": {}", t))
                        .unwrap_or_default();
                    output.push_str(&format!("    val {}{}\n", prop.name, type_annotation));
                    output.push_str(&format!("        get() = {}\n\n", getter));
                }
            }
        }

        // Functions (no viewModelScope for singletons - they must manage their own scope if needed)
        for func in &class.functions {
            // Preserve suspend keyword for singleton functions
            let suspend_keyword = if func.is_suspend { "suspend " } else { "" };
            output.push_str(&format!("    {}fun {}({})", suspend_keyword, func.name, func.params));
            if let Some(return_type) = &func.return_type {
                output.push_str(&format!(": {}", return_type));
            }
            output.push_str(" {\n");
            output.push_str(&format!("        {}\n", func.body.trim()));
            output.push_str("    }\n\n");
        }

        output.push_str("}\n");

        // Append pass-through Kotlin blocks
        if !file.kotlin_blocks.is_empty() {
            output.push('\n');
            for block in &file.kotlin_blocks {
                output.push_str(&block.content);
                output.push_str("\n\n");
            }
        }

        Ok(output)
    }

    /// Generate ViewModel code for reactive class
    fn generate_view_model_store(&self, file: &WhitehallFile, class: &ClassDeclaration, source: crate::transpiler::analyzer::StoreSource) -> Result<String, String> {
        let mut output = String::new();

        // Package declaration
        output.push_str(&format!("package {}\n\n", self.package));

        // Collect all imports
        let mut imports = Vec::new();

        // Core ViewModel imports
        imports.push("androidx.lifecycle.ViewModel".to_string());
        imports.push("androidx.lifecycle.viewModelScope".to_string());
        imports.push("kotlinx.coroutines.flow.MutableStateFlow".to_string());
        imports.push("kotlinx.coroutines.flow.StateFlow".to_string());
        imports.push("kotlinx.coroutines.flow.asStateFlow".to_string());
        imports.push("kotlinx.coroutines.flow.update".to_string());
        imports.push("kotlinx.coroutines.launch".to_string());

        // Check if Hilt is needed (either @hilt annotation or @inject constructor)
        let has_hilt_annotation = class.annotations.iter().any(|a| {
            a.eq_ignore_ascii_case("hilt") || a == "HiltViewModel"
        });
        let has_inject_constructor = class.constructor.as_ref()
            .map(|c| c.annotations.iter().any(|a| a.eq_ignore_ascii_case("inject")))
            .unwrap_or(false);
        let needs_hilt = has_hilt_annotation || has_inject_constructor;

        // Add Hilt imports if needed
        if needs_hilt {
            imports.push("dagger.hilt.android.lifecycle.HiltViewModel".to_string());
            imports.push("javax.inject.Inject".to_string());
        }

        // Add user imports from file
        for import in &file.imports {
            let resolved = self.resolve_import(&import.path);
            imports.push(resolved);
        }

        // Sort and deduplicate
        imports.sort();
        imports.dedup();

        // Write imports
        for import in imports {
            output.push_str(&format!("import {}\n", import));
        }
        output.push('\n');

        // Class annotations
        if needs_hilt {
            output.push_str("@HiltViewModel\n");
        }

        // Class declaration
        output.push_str(&format!("class {}", class.name));

        // Constructor
        if let Some(constructor) = &class.constructor {
            output.push(' ');
            if !constructor.annotations.is_empty() {
                output.push_str("@Inject ");
            }
            output.push_str(&format!("constructor(\n    {}\n)", constructor.parameters));
        }

        output.push_str(" : ViewModel() {\n");

        // Separate public/default visibility properties from private properties
        let public_properties: Vec<_> = class.properties.iter()
            .filter(|p| p.visibility.is_none() || p.visibility.as_deref() == Some("public"))
            .collect();
        let private_properties: Vec<_> = class.properties.iter()
            .filter(|p| p.visibility.as_deref() == Some("private"))
            .collect();

        // Generate UiState data class (only for public/default properties without getters)
        let ui_state_properties: Vec<_> = public_properties.iter()
            .filter(|p| p.getter.is_none())
            .copied()
            .collect();

        if !ui_state_properties.is_empty() {
            output.push_str("    data class UiState(\n");
            for (i, prop) in ui_state_properties.iter().enumerate() {
                // Infer type from initial value if no type annotation
                let type_str = if let Some(type_ann) = &prop.type_annotation {
                    type_ann.clone()
                } else if let Some(init_val) = &prop.initial_value {
                    self.infer_type_from_value(init_val)
                } else {
                    "String".to_string()
                };
                let default_val = prop.initial_value.as_deref().unwrap_or("\"\"");
                let comma = if i < ui_state_properties.len() - 1 { "," } else { "" };
                output.push_str(&format!("        val {}: {} = {}{}\n", prop.name, type_str, default_val, comma));
            }
            output.push_str("    )\n\n");

            // Generate StateFlow
            output.push_str("    private val _uiState = MutableStateFlow(UiState())\n");
            output.push_str("    val uiState: StateFlow<UiState> = _uiState.asStateFlow()\n\n");
        }

        // Generate private properties as direct class fields
        for prop in &private_properties {
            let vis = "private";
            let mutability = if prop.mutable { "var" } else { "val" };
            let type_str = if let Some(type_ann) = &prop.type_annotation {
                format!(": {}", type_ann)
            } else {
                String::new()
            };
            let init_val = if let Some(val) = &prop.initial_value {
                format!(" = {}", val)
            } else {
                String::new()
            };
            output.push_str(&format!("    {} {} {}{}{}\n", vis, mutability, prop.name, type_str, init_val));
        }
        if !private_properties.is_empty() {
            output.push('\n');
        }

        // Generate property accessors (only for public/default properties)
        for prop in &public_properties {
            if prop.getter.is_some() {
                // Derived property with getter
                let type_str = if let Some(type_ann) = &prop.type_annotation {
                    type_ann.clone()
                } else if let Some(init_val) = &prop.initial_value {
                    self.infer_type_from_value(init_val)
                } else {
                    "String".to_string()
                };
                let getter_expr = prop.getter.as_ref().unwrap();
                output.push_str(&format!("    val {}: {}\n", prop.name, type_str));
                output.push_str(&format!("        get() = {}\n\n", getter_expr));
            } else {
                // Regular property with setter
                let type_str = if let Some(type_ann) = &prop.type_annotation {
                    type_ann.clone()
                } else if let Some(init_val) = &prop.initial_value {
                    self.infer_type_from_value(init_val)
                } else {
                    "String".to_string()
                };
                output.push_str(&format!("    var {}: {}\n", prop.name, type_str));
                output.push_str(&format!("        get() = _uiState.value.{}\n", prop.name));
                output.push_str(&format!("        set(value) {{ _uiState.update {{ it.copy({} = value) }} }}\n\n", prop.name));
            }
        }

        // Generate update methods for mutable state variables
        // These allow safe state updates from lambdas in the wrapper component
        // Only generate for ComponentInline (components with inline vars), not for regular store classes
        if source == crate::transpiler::analyzer::StoreSource::ComponentInline {
            for prop in &public_properties {
                if prop.getter.is_none() {
                    // Only generate update methods for mutable (non-derived) properties
                    let type_str = if let Some(type_ann) = &prop.type_annotation {
                        type_ann.clone()
                    } else if let Some(init_val) = &prop.initial_value {
                        self.infer_type_from_value(init_val)
                    } else {
                        "String".to_string()
                    };

                    // Generate update method: fun updateEmail(value: String) { _uiState.update { it.copy(email = value) } }
                    let method_name = format!("update{}{}",
                        prop.name.chars().next().unwrap().to_uppercase(),
                        &prop.name[1..]
                    );

                    // Check if user already defined a function with this name
                    let user_defined = class.functions.iter().any(|f| f.name == method_name);

                    // Only generate if user hasn't defined it
                    if !user_defined {
                        output.push_str(&format!("    fun {}(value: {}) {{\n", method_name, type_str));
                        output.push_str(&format!("        _uiState.update {{ it.copy({} = value) }}\n", prop.name));
                        output.push_str("    }\n\n");
                    }
                }
            }
        }

        // Generate functions (skip composable functions with markup - those go in wrapper)
        for func in &class.functions {
            // Skip functions with markup - they're helper composables for the wrapper
            if func.markup.is_some() {
                continue;
            }

            output.push_str(&format!("    fun {}({})", func.name, func.params));
            if let Some(return_type) = &func.return_type {
                output.push_str(&format!(": {}", return_type));
            }
            output.push_str(" {\n");

            // Wrap suspend functions in viewModelScope.launch
            if func.is_suspend {
                output.push_str("        viewModelScope.launch {\n");
                // Indent each line of the function body properly
                for line in func.body.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        output.push_str(&format!("            {}\n", trimmed));
                    }
                }
                output.push_str("        }\n");
            } else {
                output.push_str(&format!("        {}\n", func.body.trim()));
            }

            output.push_str("    }\n\n");
        }

        output.push_str("}\n");

        // Append pass-through Kotlin blocks
        if !file.kotlin_blocks.is_empty() {
            output.push('\n');
            for block in &file.kotlin_blocks {
                output.push_str(&block.content);
                output.push_str("\n\n");
            }
        }

        Ok(output)
    }

    /// Phase 1.1: Generate ViewModel + wrapper component for component with inline vars
    /// Returns TranspileResult::Multiple with two files:
    /// 1. {ComponentName}ViewModel.kt - The ViewModel class
    /// 2. {ComponentName}.kt - The wrapper component
    fn generate_component_viewmodel(&mut self, file: &WhitehallFile) -> Result<crate::transpiler::TranspileResult, String> {
        // Part 1: Generate ViewModel class
        let viewmodel_code = self.generate_component_viewmodel_class(file)?;

        // Part 2: Generate wrapper component
        let wrapper_code = self.generate_component_wrapper(file)?;

        // Return as Multiple with two files
        Ok(crate::transpiler::TranspileResult::Multiple(vec![
            (String::new(), wrapper_code),                    // Primary file: Component.kt
            ("ViewModel".to_string(), viewmodel_code),        // Secondary file: ComponentViewModel.kt
        ]))
    }

    /// Generate the ViewModel class for component inline vars
    fn generate_component_viewmodel_class(&self, file: &WhitehallFile) -> Result<String, String> {
        let mut output = String::new();
        let viewmodel_name = format!("{}ViewModel", self.component_name);

        // Check if this component uses route parameters
        let route_params = if let Some(registry) = &self.store_registry {
            if let Some(store_info) = registry.get(&self.component_name) {
                store_info.route_params.clone()
            } else {
                vec![]
            }
        } else {
            vec![]
        };
        let has_route_params = !route_params.is_empty();

        // Package declaration
        output.push_str(&format!("package {}\n\n", self.package));

        // Imports
        output.push_str("import androidx.lifecycle.ViewModel\n");

        // Add SavedStateHandle import if route params are used
        if has_route_params {
            output.push_str("import androidx.lifecycle.SavedStateHandle\n");
        }

        // Check if we have suspend functions or lifecycle hooks
        // Collect all imports into a vector
        let mut vm_imports = Vec::new();

        let has_suspend = file.functions.iter().any(|f| f.is_suspend);
        let has_lifecycle_hooks = !file.lifecycle_hooks.is_empty();

        if has_suspend || has_lifecycle_hooks {
            vm_imports.push("androidx.lifecycle.viewModelScope".to_string());
        }

        vm_imports.push("kotlinx.coroutines.flow.MutableStateFlow".to_string());
        vm_imports.push("kotlinx.coroutines.flow.StateFlow".to_string());
        vm_imports.push("kotlinx.coroutines.flow.asStateFlow".to_string());
        vm_imports.push("kotlinx.coroutines.flow.update".to_string());

        if has_suspend || has_lifecycle_hooks {
            vm_imports.push("kotlinx.coroutines.launch".to_string());
        }

        // Check if @Serializable is used in kotlin blocks
        let uses_serializable = file.kotlin_blocks.iter()
            .any(|block| block.content.contains("@Serializable"));
        if uses_serializable {
            vm_imports.push("kotlinx.serialization.Serializable".to_string());
        }

        // Check if $fetch() is used in lifecycle hooks or functions
        let uses_fetch_in_vm = file.lifecycle_hooks.iter().any(|h| h.body.contains("$fetch("))
            || file.functions.iter().any(|f| f.body.contains("$fetch("));
        if uses_fetch_in_vm {
            vm_imports.push("io.ktor.client.HttpClient".to_string());
            vm_imports.push("io.ktor.client.call.body".to_string());
            vm_imports.push("io.ktor.client.engine.okhttp.OkHttp".to_string());
            vm_imports.push("io.ktor.client.plugins.contentnegotiation.ContentNegotiation".to_string());
            vm_imports.push("io.ktor.client.request.get".to_string());
            vm_imports.push("io.ktor.serialization.kotlinx.json.json".to_string());
            vm_imports.push("kotlinx.serialization.json.Json".to_string());
        }

        // Add any user imports from the file
        for import in &file.imports {
            let import_path = self.resolve_import(&import.path);
            vm_imports.push(import_path);
        }

        // Sort imports alphabetically (standard Kotlin convention)
        vm_imports.sort();
        vm_imports.dedup();

        // Write imports
        for import in vm_imports {
            output.push_str(&format!("import {}\n", import));
        }

        output.push('\n');

        // Add pass-through Kotlin blocks (data classes, sealed classes, etc.)
        for kotlin_block in &file.kotlin_blocks {
            output.push_str(&kotlin_block.content);
            if !kotlin_block.content.ends_with('\n') {
                output.push('\n');
            }
        }

        if !file.kotlin_blocks.is_empty() {
            output.push('\n');
        }

        // Generate HttpClient singleton if $fetch() is used
        if uses_fetch_in_vm {
            output.push_str(&self.generate_http_client());
        }

        // Class declaration with optional SavedStateHandle constructor
        if has_route_params {
            output.push_str(&format!("class {}(\n", viewmodel_name));
            output.push_str("    private val savedStateHandle: SavedStateHandle\n");
            output.push_str(") : ViewModel() {\n");
        } else {
            output.push_str(&format!("class {} : ViewModel() {{\n", viewmodel_name));
        }

        // Generate UiState data class from mutable state vars only
        let mutable_state: Vec<_> = file.state.iter().filter(|s| s.mutable).collect();

        if !mutable_state.is_empty() {
            output.push_str("    data class UiState(\n");
            for (i, state) in mutable_state.iter().enumerate() {
                let type_str = if let Some(type_ann) = &state.type_annotation {
                    type_ann.clone()
                } else {
                    self.infer_type_from_value(&state.initial_value)
                };
                // Transform array literals in initial value
                let initial_value = self.transform_array_literal(&state.initial_value, false);
                let comma = if i < mutable_state.len() - 1 { "," } else { "" };
                output.push_str(&format!("        val {}: {} = {}{}\n",
                    state.name, type_str, initial_value, comma));
            }
            output.push_str("    )\n\n");

            // StateFlow setup
            output.push_str("    private val _uiState = MutableStateFlow(UiState())\n");
            output.push_str("    val uiState: StateFlow<UiState> = _uiState.asStateFlow()\n\n");

            // Property accessors for mutable vars
            for state in &mutable_state {
                let type_str = if let Some(type_ann) = &state.type_annotation {
                    type_ann.clone()
                } else {
                    self.infer_type_from_value(&state.initial_value)
                };
                output.push_str(&format!("    var {}: {}\n", state.name, type_str));
                output.push_str(&format!("        get() = _uiState.value.{}\n", state.name));
                output.push_str(&format!("        set(value) {{ _uiState.update {{ it.copy({} = value) }} }}\n\n", state.name));
            }
        }

        // Simple immutable state (val without $derived) - generate as getters in ViewModel
        // Note: $derived() state is handled in wrapper component using derivedStateOf
        let simple_val_state: Vec<_> = file.state.iter()
            .filter(|s| !s.mutable && !s.is_derived_state)
            .collect();
        for state in &simple_val_state {
            let type_str = if let Some(type_ann) = &state.type_annotation {
                format!(": {}", type_ann)
            } else {
                String::new()
            };
            // Transform array literals in initial value
            let initial_value = self.transform_array_literal(&state.initial_value, false);
            output.push_str(&format!("    val {}{}\n", state.name, type_str));
            output.push_str(&format!("        get() = {}\n\n", initial_value));
        }

        // Generate update methods for mutable state variables
        // These allow safe state updates from lambdas in the wrapper component
        for state in &mutable_state {
            let type_str = if let Some(type_ann) = &state.type_annotation {
                type_ann.clone()
            } else {
                self.infer_type_from_value(&state.initial_value)
            };

            // Generate update method: fun updateEmail(value: String) { _uiState.update { it.copy(email = value) } }
            let method_name = format!("update{}{}",
                state.name.chars().next().unwrap().to_uppercase(),
                &state.name[1..]
            );

            // Check if user already defined a function with this name
            let user_defined = file.functions.iter().any(|f| f.name == method_name);

            // Only generate if user hasn't defined it
            if !user_defined {
                output.push_str(&format!("    fun {}(value: {}) {{\n", method_name, type_str));
                output.push_str(&format!("        _uiState.update {{ it.copy({} = value) }}\n", state.name));
                output.push_str("    }\n\n");
            }
        }

        // Generate functions (skip composable functions with markup - those go in wrapper)
        for func in &file.functions {
            // Skip functions with markup - they're helper composables for the wrapper
            if func.markup.is_some() {
                continue;
            }

            output.push_str(&format!("    fun {}({})", func.name, func.params));
            if let Some(return_type) = &func.return_type {
                output.push_str(&format!(": {}", return_type));
            }
            output.push_str(" {\n");

            // Wrap suspend functions in viewModelScope.launch
            if func.is_suspend {
                output.push_str("        viewModelScope.launch {\n");
                // Indent each line of the function body properly
                for line in func.body.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        output.push_str(&format!("            {}\n", trimmed));
                    }
                }
                output.push_str("        }\n");
            } else {
                output.push_str(&format!("        {}\n", func.body.trim()));
            }

            output.push_str("    }\n\n");
        }

        // Generate init block for lifecycle hooks
        if !file.lifecycle_hooks.is_empty() {
            output.push_str("    init {\n");

            for hook in &file.lifecycle_hooks {
                match hook.hook_type.as_str() {
                    "onMount" => {
                        // Transform hook body to handle route parameters
                        let mut transformed_body = if has_route_params {
                            self.transform_lifecycle_hook_body(&hook.body, &route_params)
                        } else {
                            hook.body.clone()
                        };

                        // Strip outer launch{} if present (prevents double wrapping)
                        transformed_body = self.strip_outer_launch(&transformed_body);

                        // Lifecycle hooks in ViewModel use viewModelScope.launch
                        output.push_str("        viewModelScope.launch {\n");
                        for line in transformed_body.lines() {
                            if line.trim().is_empty() {
                                continue;
                            }
                            // Transform $fetch() calls to Ktor HttpClient calls
                            let mut transformed_line = line.trim().to_string();
                            if transformed_line.contains("$fetch(") {
                                transformed_line = self.transform_fetch_call(&transformed_line);
                            }
                            output.push_str(&format!("            {}\n", transformed_line));
                        }
                        output.push_str("        }\n");
                    }
                    "onDispose" => {
                        // onDispose doesn't make sense in ViewModel init
                        // ViewModels have onCleared() instead, but we'll skip for now
                        // TODO: Consider mapping onDispose to onCleared() override
                    }
                    _ => {}
                }
            }

            output.push_str("    }\n\n");
        }

        output.push_str("}\n");

        Ok(output)
    }

    /// Generate the wrapper component that uses the ViewModel
    fn generate_component_wrapper(&mut self, file: &WhitehallFile) -> Result<String, String> {
        let mut output = String::new();
        let viewmodel_name = format!("{}ViewModel", self.component_name);

        // Package declaration
        output.push_str(&format!("package {}\n\n", self.package));

        // Collect imports from markup and helper function markup
        let mut prop_imports = Vec::new();
        let mut component_imports = Vec::new();
        self.collect_imports_recursive(&file.markup, &mut prop_imports, &mut component_imports);

        // Also collect imports from helper composable functions
        for func in &file.functions {
            if let Some(ref markup) = func.markup {
                self.collect_imports_recursive(markup, &mut prop_imports, &mut component_imports);
            }
        }

        let mut imports = Vec::new();

        // Add prop imports first (layout, styling, etc.)
        imports.extend(prop_imports);

        // Add Composable and runtime imports
        imports.push("androidx.compose.runtime.Composable".to_string());
        imports.push("androidx.compose.runtime.collectAsState".to_string());
        imports.push("androidx.compose.runtime.getValue".to_string());

        // Add remember and derivedStateOf imports if any derived state exists
        let has_derived_state = file.state.iter().any(|s| !s.mutable && s.is_derived_state);
        if has_derived_state {
            imports.push("androidx.compose.runtime.remember".to_string());
            imports.push("androidx.compose.runtime.derivedStateOf".to_string());
        }

        // User imports (resolve $ aliases)
        for import in &file.imports {
            let resolved = self.resolve_import(&import.path);
            imports.push(resolved);
        }

        // Add component imports
        imports.extend(component_imports);

        // Add ViewModel import
        imports.push("androidx.lifecycle.viewmodel.compose.viewModel".to_string());

        // For screens, add NavController import
        if self.component_type.as_deref() == Some("screen") {
            imports.push("androidx.navigation.NavController".to_string());
        }

        // Add Routes import if $routes or $navigate is used (for screens)
        if self.uses_routes && self.component_type.as_deref() == Some("screen") {
            let base_package = self.get_base_package();
            imports.push(format!("{}.routes.Routes", base_package));
        }

        // Sort imports alphabetically (standard Kotlin convention)
        imports.sort();
        imports.dedup(); // Remove duplicates after sorting

        // Write imports
        for import in imports {
            output.push_str(&format!("import {}\n", import));
        }

        output.push('\n');

        // Component function signature
        let is_screen = self.component_type.as_deref() == Some("screen");
        let mut params = Vec::new();

        // For screens, add navController parameter first
        if is_screen {
            params.push("navController: NavController".to_string());
        }

        // Add props
        for p in &file.props {
            let default = p
                .default_value
                .as_ref()
                .map(|d| format!(" = {}", d))
                .unwrap_or_default();
            params.push(format!("{}: {}{}", p.name, p.prop_type, default));
        }

        let props_list = params.join(", ");

        output.push_str("@Composable\n");
        output.push_str(&format!("fun {}({}) {{\n", self.component_name, props_list));

        // Instantiate ViewModel and collect state
        self.indent_level += 1;
        output.push_str(&self.indent());
        output.push_str(&format!("val viewModel = viewModel<{}>()\n", viewmodel_name));
        output.push_str(&self.indent());
        output.push_str("val uiState by viewModel.uiState.collectAsState()\n");

        // Phase 1.1: Set up ViewModel wrapper context before generating markup
        // This enables transformation of variable/function references
        self.in_viewmodel_wrapper = true;

        // Collect mutable vars (need uiState prefix)
        for state in &file.state {
            if state.mutable {
                self.mutable_vars.insert(state.name.clone());
            } else if !state.is_derived_state {
                // Simple val properties (need viewModel prefix)
                // Note: $derived() state is local to wrapper, doesn't need prefix
                self.derived_props.insert(state.name.clone());
            }
            // $derived() state: no prefix needed, it's a local val in the wrapper
        }

        // Collect function names (need viewModel prefix)
        for func in &file.functions {
            self.function_names.insert(func.name.clone());
        }

        // Generate any immutable state (val) that aren't derived
        // These need to be after ViewModel context setup so variable references get transformed
        for state in &file.state {
            if !state.mutable && !state.is_derived_state {
                output.push_str(&self.indent());
                let type_annotation = state.type_annotation.as_ref()
                    .map(|t| format!(": {}", t))
                    .unwrap_or_default();
                // Transform array literals first, then variable references
                let array_transformed = self.transform_array_literal(&state.initial_value, false);
                let transformed_value = self.transform_viewmodel_expression(&array_transformed);

                // Check if value contains newlines (multi-line expression)
                if transformed_value.contains('\n') {
                    // Multi-line expression - re-indent to current level
                    let reindented = self.reindent_expression(&transformed_value);
                    output.push_str(&format!("val {}{} = {}\n", state.name, type_annotation, reindented));
                } else {
                    // Single line - output as-is
                    output.push_str(&format!("val {}{} = {}\n", state.name, type_annotation, transformed_value));
                }
            }
        }

        // Generate derived state (val x = $derived(...))
        // These use derivedStateOf and need remember {} wrapper
        for state in &file.state {
            if !state.mutable && state.is_derived_state {
                output.push_str(&self.indent());

                // Transform variable references in the derivedStateOf expression
                // e.g., photos.filter {...} → uiState.photos.filter {...}
                let transformed_value = self.transform_viewmodel_expression(&state.initial_value);

                // Generate: val name by remember { derivedStateOf { expr } }
                output.push_str(&format!("val {} by remember {{\n", state.name));
                self.indent_level += 1;
                output.push_str(&self.indent());
                output.push_str(&format!("{}\n", transformed_value));
                self.indent_level -= 1;
                output.push_str(&self.indent());
                output.push_str("}\n");
            }
        }

        // NOTE: Lifecycle hooks are now in ViewModel's init block, not in wrapper
        // The wrapper component should not contain LaunchedEffect/DisposableEffect
        // that reference ViewModel state - that causes undefined variable errors

        output.push('\n');

        // Generate markup with ViewModel wrapper context active
        let markup_code = self.generate_markup(&file.markup)?;
        output.push_str(&markup_code);

        // Clean up context
        self.in_viewmodel_wrapper = false;
        self.mutable_vars.clear();
        self.derived_props.clear();
        self.function_names.clear();

        self.indent_level -= 1;
        output.push_str("}\n");

        // Generate helper composable functions that have markup
        for func in &file.functions {
            if let Some(ref markup) = func.markup {
                output.push_str("\n@Composable\n");
                let return_type_str = if let Some(ref rt) = func.return_type {
                    format!(": {}", rt)
                } else {
                    String::new()
                };
                let suspend_keyword = if func.is_suspend { "suspend " } else { "" };
                output.push_str(&format!("{}fun {}({}){} {{\n", suspend_keyword, func.name, func.params, return_type_str));

                // Transpile the markup
                self.indent_level += 1;
                let markup_code = self.generate_markup(markup)?;
                output.push_str(&markup_code);
                self.indent_level -= 1;

                output.push_str("}\n");
            }
        }

        Ok(output)
    }

    /// Strip outer launch{} block if present (prevents double wrapping in ViewModel)
    fn strip_outer_launch(&self, body: &str) -> String {
        let trimmed = body.trim();

        // Check if body starts with "launch {" and ends with "}"
        if trimmed.starts_with("launch {") {
            let mut depth = 0;
            let chars: Vec<char> = trimmed.chars().collect();
            let mut start_idx = None;
            let mut end_idx = None;

            for (i, ch) in chars.iter().enumerate() {
                if *ch == '{' {
                    depth += 1;
                    if start_idx.is_none() {
                        start_idx = Some(i + 1); // After the opening brace
                    }
                } else if *ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = Some(i);
                        break;
                    }
                }
            }

            if let (Some(start), Some(end)) = (start_idx, end_idx) {
                let inner = &chars[start..end].iter().collect::<String>();
                return inner.trim().to_string();
            }
        }

        body.to_string()
    }

    /// Transform lifecycle hook body to handle route parameters
    /// Replaces $screen.params.xxx with savedStateHandle.get<T?>("xxx")
    fn transform_lifecycle_hook_body(&self, body: &str, route_params: &[String]) -> String {
        let mut transformed = body.to_string();

        for param in route_params {
            // Replace $screen.params.xxx with savedStateHandle.get<String?>("xxx")
            let old_pattern = format!("$screen.params.{}", param);
            let new_pattern = format!("savedStateHandle.get<String?>(\"{}\") ?: \"\"", param);
            transformed = transformed.replace(&old_pattern, &new_pattern);
        }

        transformed
    }

    /// Transform range literal RANGE[1..10] or RANGE[1..10:2] to (1..10).toList() or (1..10 step 2).toList()
    fn transform_range_literal(&self, value: &str) -> String {
        let trimmed = value.trim();

        // Check if it's a range literal marker
        if trimmed.starts_with("RANGE[") && trimmed.ends_with(']') {
            let content = &trimmed[6..trimmed.len()-1]; // Extract content between RANGE[ and ]

            // Parse the range: start..end or start..end:step
            if let Some(colon_pos) = content.find(':') {
                // Has step: start..end:step
                let range_part = &content[..colon_pos];
                let step = &content[colon_pos+1..];

                if let Some(dot_pos) = range_part.find("..") {
                    let start = &range_part[..dot_pos];
                    let end = &range_part[dot_pos+2..];

                    // Check if it's a negative step (downTo)
                    if step.starts_with('-') {
                        format!("({} downTo {}).toList()", start, end)
                    } else {
                        format!("({} rangeTo {} step {}).toList()", start, end, step)
                    }
                } else {
                    value.to_string()
                }
            } else {
                // No step: start..end
                if let Some(dot_pos) = content.find("..") {
                    let start = &content[..dot_pos];
                    let end = &content[dot_pos+2..];
                    format!("({}..{}).toList()", start, end)
                } else {
                    value.to_string()
                }
            }
        } else {
            value.to_string()
        }
    }
}
