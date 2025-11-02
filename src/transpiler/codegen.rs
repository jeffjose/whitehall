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
                } else if prop.prop_type.ends_with('?') {
                    // Nullable types without explicit default get = null
                    output.push_str(" = null");
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

                // Add component props
                for prop in &comp.props {
                    params.push(format!("{} = {}", prop.name, prop.value));
                }

                // Determine if this component has a trailing lambda (children block)
                let has_children = !comp.children.is_empty() && comp.name != "Text";

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
