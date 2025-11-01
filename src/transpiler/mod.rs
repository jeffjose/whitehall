/// Whitehall to Kotlin/Compose transpiler
pub mod ast;
pub mod codegen;
pub mod parser;

use crate::transpiler::parser::Parser;
use crate::transpiler::codegen::CodeGenerator;

/// Transpile Whitehall source code to Kotlin/Compose
pub fn transpile(source: &str, package_name: &str, component_name: &str) -> Result<String, String> {
    // Parse the source into AST
    let mut parser = Parser::new(source);
    let file = parser.parse()?;

    // Generate Kotlin code from AST
    let mut generator = CodeGenerator::new(package_name, component_name);
    let kotlin_code = generator.generate(&file)?;

    Ok(kotlin_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_transpile() {
        let source = r#"<Text>Hello, World!</Text>"#;
        let result = transpile(source, "com.example.app.components", "MinimalText");
        assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

        let kotlin = result.unwrap();
        println!("Generated Kotlin:\n{}", kotlin);
        assert!(kotlin.contains("@Composable"));
        assert!(kotlin.contains("fun MinimalText()"));
        assert!(kotlin.contains(r#"Text(text = "Hello, World!")"#));
    }
}
