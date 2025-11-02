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

        // Imports
        output.push_str("import androidx.compose.material3.Text\n");
        if !file.state.is_empty() {
            output.push_str("import androidx.compose.runtime.*\n");
        } else {
            output.push_str("import androidx.compose.runtime.Composable\n");
        }
        output.push('\n');

        // Component function
        output.push_str("@Composable\n");
        output.push_str(&format!("fun {}() {{\n", self.component_name));
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

                // For Text component, build text parameter from children
                if comp.name == "Text" && !comp.children.is_empty() {
                    output.push_str("text = ");
                    output.push_str(&self.build_text_expression(&comp.children)?);
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

    fn build_text_expression(&self, children: &[Markup]) -> Result<String, String> {
        if children.is_empty() {
            return Ok("\"\"".to_string());
        }

        // If single text node, use simple string
        if children.len() == 1 {
            match &children[0] {
                Markup::Text(text) => return Ok(format!("\"{}\"", text)),
                Markup::Interpolation(expr) => return Ok(format!("${}", expr)),
                _ => {}
            }
        }

        // Multiple children: build string template
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
