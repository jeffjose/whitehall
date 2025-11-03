/// Code generation backends for Whitehall transpiler
///
/// Supports multiple backends:
/// - Compose: Generate Jetpack Compose code (default)
/// - View: Generate Android View code (for RecyclerView optimization)
///
/// Phase 5: Consumes optimization plans and routes to appropriate backend

pub mod compose;
pub mod view;

use crate::transpiler::optimizer::OptimizedAST;

/// Main code generator - routes to backends based on optimizations
pub struct CodeGenerator {
    package: String,
    component_name: String,
    component_type: Option<String>,
}

impl CodeGenerator {
    /// Create generator
    pub fn new(package: &str, component_name: &str, component_type: Option<&str>) -> Self {
        CodeGenerator {
            package: package.to_string(),
            component_name: component_name.to_string(),
            component_type: component_type.map(String::from),
        }
    }

    /// Phase 5: Generate Kotlin code with optimization support
    ///
    /// Routes to appropriate backend based on optimization plans:
    /// - If optimizations present: May use View backend with RecyclerView
    /// - Default: Compose backend
    pub fn generate(&mut self, optimized_ast: &OptimizedAST) -> Result<String, String> {
        // Phase 5: Pass optimizations to Compose backend
        // Compose backend will check for RecyclerView optimizations
        // and route for loops accordingly
        let mut backend = compose::ComposeBackend::new(
            &self.package,
            &self.component_name,
            self.component_type.as_deref(),
        );

        backend.generate_with_optimizations(&optimized_ast.ast, &optimized_ast.optimizations)
    }
}
