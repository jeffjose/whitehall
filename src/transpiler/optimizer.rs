/// Optimization planner for Whitehall transpiler
///
/// Takes semantic information and decides which optimizations to apply.
/// Phase 0: Returns no-op optimizations (pass-through).

use crate::transpiler::analyzer::{OptimizationHint, SemanticInfo};
use crate::transpiler::ast::WhitehallFile;

/// AST with optimization metadata
#[derive(Debug, Clone)]
pub struct OptimizedAST {
    pub ast: WhitehallFile,
    pub optimizations: Vec<Optimization>,
}

/// Optimization decisions
#[derive(Debug, Clone)]
pub enum Optimization {
    /// Use RecyclerView instead of LazyColumn for static list
    UseRecyclerView {
        collection_name: String,
        confidence: u8,
    },

    /// Use single TextView instead of multiple Text composables
    UseSingleTextView { text_node_ids: Vec<usize> },

    /// Use direct Canvas API instead of Compose Canvas
    UseDirectCanvas { component_id: usize },
}

/// Optimizer: plans optimizations based on semantic analysis
pub struct Optimizer {
    semantic_info: SemanticInfo,
    optimizations: Vec<Optimization>,
}

impl Optimizer {
    pub fn new(semantic_info: SemanticInfo) -> Self {
        Optimizer {
            semantic_info,
            optimizations: Vec::new(),
        }
    }

    /// Main entry point: analyze semantic info and plan optimizations
    pub fn optimize(ast: WhitehallFile, semantic_info: SemanticInfo) -> OptimizedAST {
        let mut optimizer = Optimizer::new(semantic_info);

        // Phase 0: No-op - return empty optimizations
        // Future phases will implement:
        // - Plan RecyclerView optimizations
        // - Plan TextView optimizations
        // - Plan Canvas optimizations

        optimizer.plan_optimizations();

        OptimizedAST {
            ast,
            optimizations: optimizer.optimizations,
        }
    }

    fn plan_optimizations(&mut self) {
        // Phase 0: No optimizations yet
        // Future: iterate through optimization hints and plan optimizations

        for hint in &self.semantic_info.optimization_hints {
            match hint {
                OptimizationHint::StaticCollection { name, confidence } => {
                    // Future: only optimize if confidence >= 80
                    if *confidence >= 80 {
                        self.optimizations.push(Optimization::UseRecyclerView {
                            collection_name: name.clone(),
                            confidence: *confidence,
                        });
                    }
                }
            }
        }
    }

    // Future: Validate that optimization is safe to apply
    #[allow(dead_code)]
    fn is_safe_to_optimize(&self, _optimization: &Optimization) -> bool {
        // Conservative: always return true for now
        // Future: check for edge cases, user opt-outs, etc.
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transpiler::analyzer::{MutabilityInfo, SymbolTable};

    #[test]
    fn test_optimizer_returns_empty_optimizations() {
        let ast = WhitehallFile::default();
        let semantic_info = SemanticInfo {
            symbol_table: SymbolTable::new(),
            mutability_info: MutabilityInfo::new(),
            optimization_hints: Vec::new(),
        };

        let optimized_ast = Optimizer::optimize(ast, semantic_info);

        // Phase 0: no optimizations yet
        assert!(optimized_ast.optimizations.is_empty());
    }

    #[test]
    fn test_optimizer_preserves_ast() {
        let ast = WhitehallFile::default();
        let semantic_info = SemanticInfo {
            symbol_table: SymbolTable::new(),
            mutability_info: MutabilityInfo::new(),
            optimization_hints: Vec::new(),
        };

        let optimized_ast = Optimizer::optimize(ast.clone(), semantic_info);

        // AST should be preserved unchanged
        assert_eq!(optimized_ast.ast.props.len(), ast.props.len());
        assert_eq!(optimized_ast.ast.state.len(), ast.state.len());
    }
}
