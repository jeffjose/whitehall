/// Semantic analysis for Whitehall code
///
/// This module analyzes the AST to build semantic information:
/// - Symbol table (what variables exist?)
/// - Mutability tracking (is this mutated?)
/// - Optimization hints (can we optimize this?)

use std::collections::{HashMap, HashSet};

use crate::transpiler::ast::{
    Component, ForLoopBlock, IfElseBlock, LifecycleHook, Markup, PropValue, StateDeclaration,
    WhenBlock, WhitehallFile,
};

/// Semantic information about the AST
#[derive(Debug, Clone)]
pub struct SemanticInfo {
    pub symbol_table: SymbolTable,
    pub mutability_info: MutabilityInfo,
    pub optimization_hints: Vec<OptimizationHint>,
}

/// Symbol table: tracks all declarations
#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) {
        self.symbols.insert(name, symbol);
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    pub fn is_val(&self, name: &str) -> bool {
        self.get(name)
            .map(|sym| !sym.mutable)
            .unwrap_or(false)
    }

    pub fn is_prop(&self, name: &str) -> bool {
        self.get(name)
            .map(|sym| matches!(sym.kind, SymbolKind::Prop))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub mutable: bool,   // Can this be reassigned (var vs val)?
    pub mutated: bool,   // Is it actually reassigned somewhere?
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Prop,      // @prop val
    StateVar,  // var
    StateVal,  // val
    Function,
}

/// Mutability information
#[derive(Debug, Clone)]
pub struct MutabilityInfo {
    pub mutable_vars: HashSet<String>,
    pub immutable_vals: HashSet<String>,
}

impl MutabilityInfo {
    pub fn new() -> Self {
        MutabilityInfo {
            mutable_vars: HashSet::new(),
            immutable_vals: HashSet::new(),
        }
    }
}

/// Optimization hints discovered during analysis
#[derive(Debug, Clone)]
pub enum OptimizationHint {
    StaticCollection {
        name: String,
        confidence: u8, // 0-100
    },
}

/// Analyzer: performs semantic analysis on AST
pub struct Analyzer {
    symbol_table: SymbolTable,
    mutable_vars: HashSet<String>,
    immutable_vals: HashSet<String>,
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer {
            symbol_table: SymbolTable::new(),
            mutable_vars: HashSet::new(),
            immutable_vals: HashSet::new(),
        }
    }

    /// Main entry point: analyze an AST and produce semantic info
    pub fn analyze(ast: &WhitehallFile) -> Result<SemanticInfo, String> {
        let mut analyzer = Analyzer::new();

        // Phase 0: No-op - just return empty semantic info
        // Future phases will implement these:
        // - Pass 1: Collect declarations
        // - Pass 2: Track usage and mutations
        // - Pass 3: Infer optimizations

        // Placeholder: collect declarations (no-op for now)
        analyzer.collect_declarations(ast);

        Ok(SemanticInfo {
            symbol_table: analyzer.symbol_table.clone(),
            mutability_info: analyzer.build_mutability_info(),
            optimization_hints: Vec::new(), // No hints yet
        })
    }

    fn collect_declarations(&mut self, ast: &WhitehallFile) {
        // Collect props
        for prop in &ast.props {
            self.symbol_table.insert(
                prop.name.clone(),
                Symbol {
                    name: prop.name.clone(),
                    kind: SymbolKind::Prop,
                    mutable: false, // Props are always val
                    mutated: false,
                },
            );
            self.immutable_vals.insert(prop.name.clone());
        }

        // Collect state
        for state in &ast.state {
            let kind = if state.mutable {
                SymbolKind::StateVar
            } else {
                SymbolKind::StateVal
            };

            self.symbol_table.insert(
                state.name.clone(),
                Symbol {
                    name: state.name.clone(),
                    kind,
                    mutable: state.mutable,
                    mutated: false, // Will update in future passes
                },
            );

            if state.mutable {
                self.mutable_vars.insert(state.name.clone());
            } else {
                self.immutable_vals.insert(state.name.clone());
            }
        }

        // Collect functions
        for func in &ast.functions {
            self.symbol_table.insert(
                func.name.clone(),
                Symbol {
                    name: func.name.clone(),
                    kind: SymbolKind::Function,
                    mutable: false,
                    mutated: false,
                },
            );
        }
    }

    fn build_mutability_info(&self) -> MutabilityInfo {
        MutabilityInfo {
            mutable_vars: self.mutable_vars.clone(),
            immutable_vals: self.immutable_vals.clone(),
        }
    }

    // Future: Track variable usage
    #[allow(dead_code)]
    fn track_usage(&mut self, ast: &WhitehallFile) {
        self.walk_markup(&ast.markup);
        // Walk lifecycle hooks
        for hook in &ast.lifecycle_hooks {
            self.walk_hook(hook);
        }
    }

    #[allow(dead_code)]
    fn walk_markup(&mut self, markup: &Markup) {
        match markup {
            Markup::Component(component) => self.walk_component(component),
            Markup::ForLoop(for_loop) => self.walk_for_loop(for_loop),
            Markup::IfElse(if_else) => self.walk_if_else(if_else),
            Markup::When(when) => self.walk_when(when),
            Markup::Sequence(items) => {
                for item in items {
                    self.walk_markup(item);
                }
            }
            _ => {}
        }
    }

    #[allow(dead_code)]
    fn walk_component(&mut self, component: &Component) {
        // Walk props
        for prop in &component.props {
            if let PropValue::Markup(markup) = &prop.value {
                self.walk_markup(markup);
            }
        }

        // Walk children
        for child in &component.children {
            self.walk_markup(child);
        }
    }

    #[allow(dead_code)]
    fn walk_for_loop(&mut self, for_loop: &ForLoopBlock) {
        // Record that collection is accessed
        // (Future: mark as accessed in symbol table)

        // Walk loop body
        for child in &for_loop.body {
            self.walk_markup(child);
        }

        // Walk empty block if present
        if let Some(empty_block) = &for_loop.empty_block {
            for child in empty_block {
                self.walk_markup(child);
            }
        }
    }

    #[allow(dead_code)]
    fn walk_if_else(&mut self, if_else: &IfElseBlock) {
        // Walk then branch
        for child in &if_else.then_branch {
            self.walk_markup(child);
        }

        // Walk else if branches
        for else_if in &if_else.else_ifs {
            for child in &else_if.body {
                self.walk_markup(child);
            }
        }

        // Walk else branch
        if let Some(else_branch) = &if_else.else_branch {
            for child in else_branch {
                self.walk_markup(child);
            }
        }
    }

    #[allow(dead_code)]
    fn walk_when(&mut self, when: &WhenBlock) {
        for branch in &when.branches {
            self.walk_markup(&branch.body);
        }
    }

    #[allow(dead_code)]
    fn walk_hook(&mut self, _hook: &LifecycleHook) {
        // Future: parse hook body to detect mutations
    }

    // Future: Infer optimization opportunities
    #[allow(dead_code)]
    fn infer_optimizations(&self, _ast: &WhitehallFile) -> Vec<OptimizationHint> {
        Vec::new() // No optimizations yet
    }

    #[allow(dead_code)]
    fn check_static_collection(&self, for_loop: &ForLoopBlock) -> Option<OptimizationHint> {
        let collection_name = &for_loop.collection;
        let symbol = self.symbol_table.get(collection_name)?;

        let mut confidence = 0u8;

        // Is it a val (immutable)?
        if !symbol.mutable {
            confidence += 40;
        }

        // Is it ever mutated?
        if !symbol.mutated {
            confidence += 30;
        }

        // Is it declared in component (not a prop)?
        if matches!(symbol.kind, SymbolKind::StateVal) {
            confidence += 20;
        }

        // Does loop body have event handlers?
        if !self.has_event_handlers(&for_loop.body) {
            confidence += 10;
        }

        if confidence >= 50 {
            Some(OptimizationHint::StaticCollection {
                name: collection_name.clone(),
                confidence,
            })
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn has_event_handlers(&self, body: &[Markup]) -> bool {
        for markup in body {
            match markup {
                Markup::Component(component) => {
                    // Check if any prop is an event handler
                    for prop in &component.props {
                        if prop.name.starts_with("on") || prop.name.starts_with("bind:") {
                            return true;
                        }
                    }

                    // Recursively check children
                    if self.has_event_handlers(&component.children) {
                        return true;
                    }
                }
                Markup::ForLoop(for_loop) => {
                    if self.has_event_handlers(&for_loop.body) {
                        return true;
                    }
                }
                Markup::IfElse(if_else) => {
                    if self.has_event_handlers(&if_else.then_branch) {
                        return true;
                    }
                    for else_if in &if_else.else_ifs {
                        if self.has_event_handlers(&else_if.body) {
                            return true;
                        }
                    }
                    if let Some(else_branch) = &if_else.else_branch {
                        if self.has_event_handlers(else_branch) {
                            return true;
                        }
                    }
                }
                Markup::When(when) => {
                    for branch in &when.branches {
                        if let Markup::Component(c) = &branch.body {
                            if self.has_event_handlers(&[Markup::Component(c.clone())]) {
                                return true;
                            }
                        }
                    }
                }
                Markup::Sequence(items) => {
                    if self.has_event_handlers(items) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transpiler::ast::PropDeclaration;

    #[test]
    fn test_analyzer_creates_symbol_table() {
        let ast = WhitehallFile {
            props: vec![PropDeclaration {
                name: "title".to_string(),
                prop_type: "String".to_string(),
                default_value: None,
            }],
            state: vec![StateDeclaration {
                name: "count".to_string(),
                mutable: true,
                type_annotation: Some("Int".to_string()),
                initial_value: "0".to_string(),
                is_derived_state: false,
            }],
            ..Default::default()
        };

        let semantic_info = Analyzer::analyze(&ast).unwrap();

        // Should have collected both symbols
        assert!(semantic_info.symbol_table.contains("title"));
        assert!(semantic_info.symbol_table.contains("count"));

        // Should track mutability
        assert!(semantic_info.mutability_info.mutable_vars.contains("count"));
        assert!(semantic_info
            .mutability_info
            .immutable_vals
            .contains("title"));
    }

    #[test]
    fn test_analyzer_no_optimizations_yet() {
        let ast = WhitehallFile::default();
        let semantic_info = Analyzer::analyze(&ast).unwrap();

        // Phase 0: no optimizations yet
        assert!(semantic_info.optimization_hints.is_empty());
    }
}
