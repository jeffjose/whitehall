/// Code generation backends for Whitehall transpiler
///
/// Supports multiple backends:
/// - Compose: Generate Jetpack Compose code (default)
/// - View: Generate Android View code (for RecyclerView optimization)

pub mod compose;
pub mod view;

use crate::transpiler::ast::WhitehallFile;

/// Main code generator - uses Compose backend by default
pub struct CodeGenerator {
    package: String,
    component_name: String,
    component_type: Option<String>,
}

impl CodeGenerator {
    /// Create generator with Compose backend (default)
    pub fn new(package: &str, component_name: &str, component_type: Option<&str>) -> Self {
        CodeGenerator {
            package: package.to_string(),
            component_name: component_name.to_string(),
            component_type: component_type.map(String::from),
        }
    }

    /// Generate Kotlin code using Compose backend
    pub fn generate(&mut self, ast: &WhitehallFile) -> Result<String, String> {
        // Use Compose backend for now (Phase 0.5)
        // TODO: Support View backend selection based on optimizations
        let mut backend = compose::ComposeBackend::new(
            &self.package,
            &self.component_name,
            self.component_type.as_deref(),
        );

        backend.generate(ast)
    }
}
