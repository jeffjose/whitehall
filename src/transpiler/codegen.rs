/// Code generator for Kotlin/Compose output

use crate::transpiler::ast::{Markup, WhitehallFile};

pub struct CodeGenerator {
    package: String,
    component_name: String,
    indent_level: usize,
}

impl CodeGenerator {
    pub fn new(package: &str, component_name: &str) -> Self {
        CodeGenerator {
            package: package.to_string(),
            component_name: component_name.to_string(),
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
        // If there are NO prop imports and NO user imports, put component imports first
        // Otherwise: Composable, prop imports, user imports, component imports
        let mut imports = Vec::new();

        let has_other_imports = !prop_imports.is_empty() || !file.imports.is_empty();

        if !has_other_imports {
            // Simple case: component imports first (e.g., Text before Composable)
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

        // Add component imports (if not already added)
        if has_other_imports {
            imports.extend(component_imports);
        }

        // Write all imports
        for import in imports {
            output.push_str(&format!("import {}\n", import));
        }

        output.push('\n');

        // Component function
        output.push_str("@Composable\n");
        output.push_str(&format!("fun {}(", self.component_name));

        // Props as function parameters
        if !file.props.is_empty() {
            output.push('\n');
            for (i, prop) in file.props.iter().enumerate() {
                output.push_str("    ");
                output.push_str(&prop.name);
                output.push_str(": ");
                output.push_str(&prop.prop_type);
                if let Some(default) = &prop.default_value {
                    output.push_str(" = ");
                    output.push_str(default);
                }
                if i < file.props.len() - 1 {
                    output.push(',');
                }
                output.push('\n');
            }
        }

        output.push_str(") {\n");
        self.indent_level += 1;

        // Generate state declarations
        for state in &file.state {
            output.push_str(&self.indent());
            if state.mutable {
                output.push_str(&format!(
                    "var {} by remember {{ mutableStateOf({}) }}\n",
                    state.name, state.initial_value
                ));
            } else {
                output.push_str(&format!("val {} = {}\n", state.name, state.initial_value));
            }
        }

        if !file.state.is_empty() {
            output.push('\n');
        }

        // Generate markup
        output.push_str(&self.generate_markup(&file.markup)?);

        self.indent_level -= 1;
        output.push_str("}\n");

        Ok(output)
    }

    fn generate_markup(&self, markup: &Markup) -> Result<String, String> {
        match markup {
            Markup::Component(comp) => {
                let mut output = String::new();
                output.push_str(&self.indent());
                output.push_str(&comp.name);
                output.push('(');

                let mut params = Vec::new();

                // For Text component with children, add text parameter
                if comp.name == "Text" && !comp.children.is_empty() {
                    let text_expr = self.build_text_expression(&comp.children)?;
                    params.push(format!("text = {}", text_expr));
                }

                // Add component props
                for prop in &comp.props {
                    params.push(format!("{} = {}", prop.name, prop.value));
                }

                // If multiple params or any long param, use multiline format
                if params.len() > 1 || params.iter().any(|p| p.len() > 40) {
                    output.push('\n');
                    for (i, param) in params.iter().enumerate() {
                        output.push_str(&format!("{}    {}", self.indent(), param));
                        if i < params.len() - 1 {
                            output.push(',');
                        }
                        output.push('\n');
                    }
                    output.push_str(&self.indent());
                } else if !params.is_empty() {
                    output.push_str(&params[0]);
                }

                output.push_str(")\n");
                Ok(output)
            }
            Markup::Text(text) => Ok(format!("{}Text(text = \"{}\")\n", self.indent(), text)),
            Markup::Interpolation(expr) => {
                Ok(format!("{}Text(text = ${})\n", self.indent(), expr))
            }
            Markup::Sequence(items) => {
                let mut output = String::new();
                for item in items {
                    output.push_str(&self.generate_markup(item)?);
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
                // Check if we need Modifier and clickable imports from props
                for prop in &comp.props {
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
                }

                // Add imports for known components (after prop imports)
                match comp.name.as_str() {
                    "Text" => {
                        let import = "androidx.compose.material3.Text".to_string();
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

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }
}
