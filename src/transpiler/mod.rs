/// Whitehall to Kotlin/Compose transpiler
///
/// Entry point for transpilation

mod ast;
mod codegen;
mod parser;

use codegen::CodeGenerator;
use parser::Parser;

/// Transpile Whitehall source code to Kotlin/Compose
///
/// # Arguments
/// * `input` - Whitehall source code
/// * `package` - Kotlin package name (e.g., "com.example.app.components")
/// * `component_name` - Component name (e.g., "MinimalText")
/// * `component_type` - Optional component type (e.g., "screen" for screens with NavController)
///
/// # Returns
/// Generated Kotlin code or error message
pub fn transpile(
    input: &str,
    package: &str,
    component_name: &str,
    component_type: Option<&str>,
) -> Result<String, String> {
    // Parse input to AST
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    // Generate Kotlin code
    let mut codegen = CodeGenerator::new(package, component_name, component_type);
    codegen.generate(&ast)
}
