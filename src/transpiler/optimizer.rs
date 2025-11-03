/// Optimization planner for Whitehall transpiler
///
/// Takes semantic information and decides which optimizations to apply.
/// Phase 3-4: Consumes optimization hints and plans optimizations.

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

    /// Phase 3-4: Analyze semantic info and plan optimizations
    ///
    /// Phase 3: Receives optimization hints from analyzer via SemanticInfo
    /// Phase 4: Consumes hints and generates Optimization plans
    pub fn optimize(ast: WhitehallFile, semantic_info: SemanticInfo) -> OptimizedAST {
        let mut optimizer = Optimizer::new(semantic_info);

        // Phase 4: Plan optimizations based on hints
        optimizer.plan_optimizations();

        OptimizedAST {
            ast,
            optimizations: optimizer.optimizations,
        }
    }

    /// Phase 4: Plan optimizations based on hints from analyzer
    ///
    /// Applies threshold: only optimize if confidence >= 80
    /// This ensures we only optimize high-confidence cases where:
    /// - Collection is val (40 pts) + not mutated (30 pts) + not prop (20 pts) + no handlers (10 pts) = 100
    /// - Or val (40) + not mutated (30) + not prop (20) = 90 (with handlers)
    /// - Or val (40) + not mutated (30) + is prop (0) + no handlers (10) = 80 (prop collections)
    fn plan_optimizations(&mut self) {
        // Phase 3: Hints are passed via SemanticInfo from analyzer
        // Phase 4: Consume hints and generate optimization plans

        for hint in &self.semantic_info.optimization_hints {
            match hint {
                OptimizationHint::StaticCollection { name, confidence } => {
                    // Phase 4: Apply 80+ confidence threshold
                    // This filters out:
                    // - var collections (max 60: 0+30+20+10)
                    // - var with handlers (max 50: 0+30+20+0)
                    // - props with handlers (70: 40+30+0+0)
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

    /// Phase 5: Validate that optimization is safe to apply
    ///
    /// Currently always returns true (conservative approach).
    /// Future: Check for edge cases, user opt-outs, feature flags, etc.
    #[allow(dead_code)]
    fn is_safe_to_optimize(&self, _optimization: &Optimization) -> bool {
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

        // Phase 3: No hints provided, so no optimizations
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

        // AST should be preserved unchanged (Phase 0-5)
        assert_eq!(optimized_ast.ast.props.len(), ast.props.len());
        assert_eq!(optimized_ast.ast.state.len(), ast.state.len());
    }

    // ========== Phase 3-4: Optimization Planning Tests ==========

    #[test]
    fn test_optimizer_accepts_high_confidence_hint() {
        let ast = WhitehallFile::default();
        let semantic_info = SemanticInfo {
            symbol_table: SymbolTable::new(),
            mutability_info: MutabilityInfo::new(),
            optimization_hints: vec![OptimizationHint::StaticCollection {
                name: "items".to_string(),
                confidence: 100,
            }],
        };

        let optimized_ast = Optimizer::optimize(ast, semantic_info);

        // Phase 4: Should generate optimization for confidence 100 (>= 80)
        assert_eq!(optimized_ast.optimizations.len(), 1);
        match &optimized_ast.optimizations[0] {
            Optimization::UseRecyclerView {
                collection_name,
                confidence,
            } => {
                assert_eq!(collection_name, "items");
                assert_eq!(*confidence, 100);
            }
            _ => panic!("Expected UseRecyclerView optimization"),
        }
    }

    #[test]
    fn test_optimizer_accepts_threshold_confidence() {
        let ast = WhitehallFile::default();
        let semantic_info = SemanticInfo {
            symbol_table: SymbolTable::new(),
            mutability_info: MutabilityInfo::new(),
            optimization_hints: vec![OptimizationHint::StaticCollection {
                name: "items".to_string(),
                confidence: 80,
            }],
        };

        let optimized_ast = Optimizer::optimize(ast, semantic_info);

        // Phase 4: Should generate optimization for confidence 80 (exactly at threshold)
        assert_eq!(optimized_ast.optimizations.len(), 1);
        match &optimized_ast.optimizations[0] {
            Optimization::UseRecyclerView {
                collection_name,
                confidence,
            } => {
                assert_eq!(collection_name, "items");
                assert_eq!(*confidence, 80);
            }
            _ => panic!("Expected UseRecyclerView optimization"),
        }
    }

    #[test]
    fn test_optimizer_rejects_medium_confidence() {
        let ast = WhitehallFile::default();
        let semantic_info = SemanticInfo {
            symbol_table: SymbolTable::new(),
            mutability_info: MutabilityInfo::new(),
            optimization_hints: vec![OptimizationHint::StaticCollection {
                name: "items".to_string(),
                confidence: 60, // var collection: 0+30+20+10
            }],
        };

        let optimized_ast = Optimizer::optimize(ast, semantic_info);

        // Phase 4: Should NOT generate optimization for confidence 60 (< 80)
        assert!(optimized_ast.optimizations.is_empty());
    }

    #[test]
    fn test_optimizer_rejects_low_confidence() {
        let ast = WhitehallFile::default();
        let semantic_info = SemanticInfo {
            symbol_table: SymbolTable::new(),
            mutability_info: MutabilityInfo::new(),
            optimization_hints: vec![OptimizationHint::StaticCollection {
                name: "items".to_string(),
                confidence: 50, // var with handlers: 0+30+20+0
            }],
        };

        let optimized_ast = Optimizer::optimize(ast, semantic_info);

        // Phase 4: Should NOT generate optimization for confidence 50 (< 80)
        assert!(optimized_ast.optimizations.is_empty());
    }

    #[test]
    fn test_optimizer_handles_multiple_hints() {
        let ast = WhitehallFile::default();
        let semantic_info = SemanticInfo {
            symbol_table: SymbolTable::new(),
            mutability_info: MutabilityInfo::new(),
            optimization_hints: vec![
                OptimizationHint::StaticCollection {
                    name: "items1".to_string(),
                    confidence: 100,
                },
                OptimizationHint::StaticCollection {
                    name: "items2".to_string(),
                    confidence: 60, // Below threshold
                },
                OptimizationHint::StaticCollection {
                    name: "items3".to_string(),
                    confidence: 90,
                },
            ],
        };

        let optimized_ast = Optimizer::optimize(ast, semantic_info);

        // Phase 4: Should optimize items1 and items3, but not items2
        assert_eq!(optimized_ast.optimizations.len(), 2);

        let names: Vec<String> = optimized_ast
            .optimizations
            .iter()
            .filter_map(|opt| match opt {
                Optimization::UseRecyclerView {
                    collection_name, ..
                } => Some(collection_name.clone()),
                _ => None,
            })
            .collect();

        assert!(names.contains(&"items1".to_string()));
        assert!(names.contains(&"items3".to_string()));
        assert!(!names.contains(&"items2".to_string()));
    }

    #[test]
    fn test_optimizer_edge_case_confidence_79() {
        let ast = WhitehallFile::default();
        let semantic_info = SemanticInfo {
            symbol_table: SymbolTable::new(),
            mutability_info: MutabilityInfo::new(),
            optimization_hints: vec![OptimizationHint::StaticCollection {
                name: "items".to_string(),
                confidence: 79, // Just below threshold
            }],
        };

        let optimized_ast = Optimizer::optimize(ast, semantic_info);

        // Phase 4: Should NOT optimize (79 < 80)
        assert!(optimized_ast.optimizations.is_empty());
    }
}
