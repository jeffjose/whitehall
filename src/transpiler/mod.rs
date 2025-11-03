/// Whitehall to Kotlin/Compose transpiler
///
/// Entry point for transpilation

mod analyzer;
mod ast;
mod codegen;
mod optimizer;
mod parser;
mod recyclerview;

use analyzer::Analyzer;
use codegen::CodeGenerator;
use optimizer::Optimizer;
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
    // 1. Parse input to AST
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    // 2. Analyze: build semantic information
    //    Phase 0-2: Collect symbols, track usage, detect optimizations
    let semantic_info = Analyzer::analyze(&ast)?;

    // 3. Optimize: plan optimizations
    //    Phase 3-4: Receive hints, apply threshold, generate plans
    let optimized_ast = Optimizer::optimize(ast, semantic_info);

    // 4. Generate Kotlin code
    //    Phase 5: Consume optimizations and route to appropriate backend
    let mut codegen = CodeGenerator::new(package, component_name, component_type);
    codegen.generate(&optimized_ast)
}
