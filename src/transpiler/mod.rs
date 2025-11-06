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

/// Transpilation result that can represent one or more output files
#[derive(Debug, Clone)]
pub enum TranspileResult {
    /// Single output file (standard case)
    Single(String),
    /// Multiple output files (e.g., Component + ViewModel for inline vars)
    /// Each tuple is (filename_suffix, content)
    /// filename_suffix examples: "" for main file, "ViewModel" for ViewModel file
    Multiple(Vec<(String, String)>),
}

impl TranspileResult {
    /// Get the primary content (for backward compatibility)
    pub fn primary_content(&self) -> &str {
        match self {
            TranspileResult::Single(content) => content,
            TranspileResult::Multiple(files) => {
                // First file is primary
                files.first().map(|(_, content)| content.as_str()).unwrap_or("")
            }
        }
    }

    /// Check if this is a multi-file result
    pub fn is_multiple(&self) -> bool {
        matches!(self, TranspileResult::Multiple(_))
    }

    /// Get all files (for multi-file handling)
    pub fn files(&self) -> Vec<(String, String)> {
        match self {
            TranspileResult::Single(content) => vec![(String::new(), content.clone())],
            TranspileResult::Multiple(files) => files.clone(),
        }
    }
}

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
/// TranspileResult (Single or Multiple files) or error message
pub fn transpile(
    input: &str,
    package: &str,
    component_name: &str,
    component_type: Option<&str>,
) -> Result<TranspileResult, String> {
    transpile_with_registry(input, package, component_name, component_type, None)
}

/// Transpile with optional global store registry
pub fn transpile_with_registry(
    input: &str,
    package: &str,
    component_name: &str,
    component_type: Option<&str>,
    global_store_registry: Option<&analyzer::StoreRegistry>,
) -> Result<TranspileResult, String> {
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
    //    Returns TranspileResult (Single or Multiple files)
    let mut codegen = CodeGenerator::new(package, component_name, component_type);
    codegen.generate(&optimized_ast)
}

/// Parse source code to extract AST for store registry building
/// This is a lightweight parse that only extracts the AST structure
pub fn parse_for_stores(input: &str) -> Result<ast::WhitehallFile, String> {
    let mut parser = Parser::new(input);
    parser.parse()
}
