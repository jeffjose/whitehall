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

        // Imports (hardcoded for test 00)
        output.push_str("import androidx.compose.material3.Text\n");
        output.push_str("import androidx.compose.runtime.Composable\n\n");

        // Component function
        output.push_str("@Composable\n");
        output.push_str(&format!("fun {}() {{\n", self.component_name));
        self.indent_level += 1;

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

                // For Text component with text content
                if comp.name == "Text" && !comp.children.is_empty() {
                    if let Markup::Text(text) = &comp.children[0] {
                        output.push_str(&format!("text = \"{}\"", text));
                    }
                }

                output.push_str(")\n");
                Ok(output)
            }
            Markup::Text(text) => Ok(format!("{}Text(text = \"{}\")\n", self.indent(), text)),
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }
}
