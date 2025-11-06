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

// Re-export types needed by build_pipeline
pub use analyzer::{StoreRegistry, StoreInfo, StoreSource};
pub use ast::WhitehallFile as AST;

/// Transpile Whitehall source code to Kotlin/Compose
///
/// # Arguments
/// * `input` - Whitehall source code
/// * `package` - Kotlin package name (e.g., "com.example.app.components")
/// * `component_name` - Component name (e.g., "MinimalText")
/// * `component_type` - Optional component type (e.g., "screen" for screens with NavController)
/// * `global_store_registry` - Optional project-wide store registry for cross-file store detection
///
/// # Returns
/// Generated Kotlin code or error message
pub fn transpile(
    input: &str,
    package: &str,
    component_name: &str,
    component_type: Option<&str>,
) -> Result<String, String> {
    transpile_with_registry(input, package, component_name, component_type, None)
}

/// Transpile with optional global store registry
pub fn transpile_with_registry(
    input: &str,
    package: &str,
    component_name: &str,
    component_type: Option<&str>,
    global_store_registry: Option<&analyzer::StoreRegistry>,
) -> Result<String, String> {
    // 1. Parse input to AST
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;

    // 2. Analyze: build semantic information
    //    Phase 0-2: Collect symbols, track usage, detect optimizations
    let mut semantic_info = Analyzer::analyze(&ast)?;

    // Merge global store registry if provided
    if let Some(global_registry) = global_store_registry {
        for (name, info) in global_registry.iter() {
            // Only add if not already in local registry (local takes precedence)
            if !semantic_info.store_registry.contains(name) {
                semantic_info.store_registry.insert(name.clone(), info.clone());
            }
        }
    }

    // 3. Optimize: plan optimizations
    //    Phase 3-4: Receive hints, apply threshold, generate plans
    let optimized_ast = Optimizer::optimize(ast, semantic_info);

    // 4. Generate Kotlin code
    //    Phase 5: Consume optimizations and route to appropriate backend
    let mut codegen = CodeGenerator::new(package, component_name, component_type);
    codegen.generate(&optimized_ast)
}

/// Parse source code to extract AST for store registry building
/// This is a lightweight parse that only extracts the AST structure
pub fn parse_for_stores(input: &str) -> Result<ast::WhitehallFile, String> {
    let mut parser = Parser::new(input);
    parser.parse()
}
