/// Semantic analysis for Whitehall code
///
/// This module analyzes the AST to build semantic information:
/// - Symbol table (what variables exist?)
/// - Mutability tracking (is this mutated?)
/// - Usage tracking (where are variables accessed?)
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
    pub mutable: bool,      // Can this be reassigned (var vs val)?
    pub mutated: bool,      // Is it actually reassigned somewhere?
    pub usage_info: UsageInfo, // Phase 1: Where/how is this variable used?
}

/// Usage information for a symbol (Phase 1)
#[derive(Debug, Clone)]
pub struct UsageInfo {
    pub access_count: usize,
    pub contexts: HashSet<UsageContext>,
    pub used_in_loops: bool,      // Accessed inside @for loop body?
    pub used_in_conditions: bool, // Accessed in @if/@when conditions?
    pub used_in_keys: bool,       // Accessed in @for key expressions?
}

impl UsageInfo {
    pub fn new() -> Self {
        UsageInfo {
            access_count: 0,
            contexts: HashSet::new(),
            used_in_loops: false,
            used_in_conditions: false,
            used_in_keys: false,
        }
    }

    pub fn record_access(&mut self, context: UsageContext) {
        self.access_count += 1;
        self.contexts.insert(context.clone());

        // Update usage flags based on context
        match context {
            UsageContext::InForLoopBody { .. } => self.used_in_loops = true,
            UsageContext::InCondition { .. } => self.used_in_conditions = true,
            UsageContext::InKeyExpression { .. } => self.used_in_keys = true,
            _ => {}
        }
    }
}

/// Context where a variable is accessed (Phase 1)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UsageContext {
    /// Accessed in a @for loop collection reference
    InForLoopCollection {
        collection: String,
    },
    /// Accessed inside @for loop body
    InForLoopBody {
        collection: String,
    },
    /// Accessed in @for key expression
    InKeyExpression {
        collection: String,
    },
    /// Accessed in @if or @when condition
    InCondition {
        condition_type: String, // "if", "else-if", "when"
    },
    /// Accessed in component prop value
    InComponentProp {
        component: String,
        prop_name: String,
    },
    /// Accessed in text interpolation
    InInterpolation,
    /// Accessed in lifecycle hook
    InLifecycleHook {
        hook_type: String,
    },
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
    // Phase 1: Track current context for usage tracking
    current_for_loop: Option<String>, // Current @for loop collection name
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer {
            symbol_table: SymbolTable::new(),
            mutable_vars: HashSet::new(),
            immutable_vals: HashSet::new(),
            current_for_loop: None, // Phase 1: Not in any loop initially
        }
    }

    /// Main entry point: analyze an AST and produce semantic info
    pub fn analyze(ast: &WhitehallFile) -> Result<SemanticInfo, String> {
        let mut analyzer = Analyzer::new();

        // Pass 1: Collect declarations (Phase 0)
        analyzer.collect_declarations(ast);

        // Pass 2: Track usage (Phase 1)
        analyzer.track_usage(ast);

        // Pass 3: Infer optimizations (Phase 2+)
        // Not implemented yet - will be added in future phases

        Ok(SemanticInfo {
            symbol_table: analyzer.symbol_table.clone(),
            mutability_info: analyzer.build_mutability_info(),
            optimization_hints: Vec::new(), // Phase 2+: No hints yet
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
                    usage_info: UsageInfo::new(), // Phase 1: Initialize usage tracking
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
                    usage_info: UsageInfo::new(), // Phase 1: Initialize usage tracking
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
                    usage_info: UsageInfo::new(), // Phase 1: Initialize usage tracking
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

    /// Phase 1: Track variable usage throughout AST
    fn track_usage(&mut self, ast: &WhitehallFile) {
        // Walk main markup
        self.walk_markup(&ast.markup);

        // Walk lifecycle hooks
        for hook in &ast.lifecycle_hooks {
            self.walk_hook(hook);
        }
    }

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
            Markup::Interpolation(expr) => {
                // Phase 1: Track variable access in interpolation {variable}
                self.record_expression_usage(expr, UsageContext::InInterpolation);
            }
            Markup::Text(_) => {
                // Plain text, no variables to track
            }
        }
    }

    fn walk_component(&mut self, component: &Component) {
        // Walk props
        for prop in &component.props {
            match &prop.value {
                PropValue::Expression(expr) => {
                    // Phase 1: Track variable access in component props
                    let context = UsageContext::InComponentProp {
                        component: component.name.clone(),
                        prop_name: prop.name.clone(),
                    };
                    self.record_expression_usage(expr, context);
                }
                PropValue::Markup(markup) => {
                    self.walk_markup(markup);
                }
            }
        }

        // Walk children
        for child in &component.children {
            self.walk_markup(child);
        }
    }

    fn walk_for_loop(&mut self, for_loop: &ForLoopBlock) {
        let collection = &for_loop.collection;

        // Phase 1: Record collection access
        self.record_variable_access(
            collection,
            UsageContext::InForLoopCollection {
                collection: collection.clone(),
            },
        );

        // Phase 1: Record key expression usage
        if let Some(key_expr) = &for_loop.key_expr {
            self.record_expression_usage(
                key_expr,
                UsageContext::InKeyExpression {
                    collection: collection.clone(),
                },
            );
        }

        // Save previous loop context and enter this loop
        let prev_loop = self.current_for_loop.clone();
        self.current_for_loop = Some(collection.clone());

        // Walk loop body (variables accessed here are "used_in_loops")
        for child in &for_loop.body {
            self.walk_markup(child);
        }

        // Restore previous loop context
        self.current_for_loop = prev_loop;

        // Walk empty block if present (not inside loop body)
        if let Some(empty_block) = &for_loop.empty_block {
            for child in empty_block {
                self.walk_markup(child);
            }
        }
    }

    fn walk_if_else(&mut self, if_else: &IfElseBlock) {
        // Phase 1: Track condition usage
        self.record_expression_usage(
            &if_else.condition,
            UsageContext::InCondition {
                condition_type: "if".to_string(),
            },
        );

        // Walk then branch
        for child in &if_else.then_branch {
            self.walk_markup(child);
        }

        // Walk else if branches
        for else_if in &if_else.else_ifs {
            self.record_expression_usage(
                &else_if.condition,
                UsageContext::InCondition {
                    condition_type: "else-if".to_string(),
                },
            );
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

    fn walk_when(&mut self, when: &WhenBlock) {
        for branch in &when.branches {
            // Phase 1: Track condition usage
            if let Some(condition) = &branch.condition {
                self.record_expression_usage(
                    condition,
                    UsageContext::InCondition {
                        condition_type: "when".to_string(),
                    },
                );
            }
            self.walk_markup(&branch.body);
        }
    }

    fn walk_hook(&mut self, hook: &LifecycleHook) {
        // Phase 1: Track usage in lifecycle hooks
        self.record_expression_usage(
            &hook.body,
            UsageContext::InLifecycleHook {
                hook_type: hook.hook_type.clone(),
            },
        );
    }

    /// Phase 1: Record usage of a variable in a specific context
    fn record_variable_access(&mut self, var_name: &str, context: UsageContext) {
        // Get mutable reference to symbol and update usage
        if let Some(symbol) = self.symbol_table.symbols.get_mut(var_name) {
            symbol.usage_info.record_access(context);
        }
    }

    /// Phase 1: Parse expression and record all variable accesses
    ///
    /// This is a simplified parser that extracts variable names from expressions.
    /// It handles common patterns like:
    /// - Simple variables: "count"
    /// - Property access: "user.name", "post.author.email"
    /// - Function calls: "formatDate(post.createdAt)"
    /// - Complex expressions: "posts.size > 0"
    ///
    /// Note: This is intentionally simple - we extract potential variable names
    /// and check against symbol table. False positives are ok (e.g., "size" in
    /// "posts.size" won't match anything in symbol table).
    fn record_expression_usage(&mut self, expr: &str, context: UsageContext) {
        // Split by common delimiters to extract potential variable names
        let delimiters = [' ', '(', ')', '.', ',', '>', '<', '=', '!', '+', '-', '*', '/', '{', '}', '[', ']'];

        let mut current_word = String::new();

        for ch in expr.chars() {
            if delimiters.contains(&ch) {
                if !current_word.is_empty() {
                    // Check if this word is a known variable
                    if self.symbol_table.contains(&current_word) {
                        // Determine actual context based on whether we're in a loop
                        let actual_context = if let Some(collection) = &self.current_for_loop {
                            // If we're inside a @for loop body, use InForLoopBody context
                            if !matches!(context, UsageContext::InForLoopCollection { .. })
                                && !matches!(context, UsageContext::InKeyExpression { .. })
                            {
                                UsageContext::InForLoopBody {
                                    collection: collection.clone(),
                                }
                            } else {
                                context.clone()
                            }
                        } else {
                            context.clone()
                        };

                        self.record_variable_access(&current_word, actual_context);
                    }
                    current_word.clear();
                }
            } else {
                current_word.push(ch);
            }
        }

        // Don't forget last word if expression doesn't end with delimiter
        if !current_word.is_empty() && self.symbol_table.contains(&current_word) {
            let actual_context = if let Some(collection) = &self.current_for_loop {
                if !matches!(context, UsageContext::InForLoopCollection { .. })
                    && !matches!(context, UsageContext::InKeyExpression { .. })
                {
                    UsageContext::InForLoopBody {
                        collection: collection.clone(),
                    }
                } else {
                    context.clone()
                }
            } else {
                context
            };

            self.record_variable_access(&current_word, actual_context);
        }
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

    // ========== Phase 1: Usage Tracking Tests ==========

    #[test]
    fn test_tracks_variable_in_interpolation() {
        use crate::transpiler::ast::Markup;

        let ast = WhitehallFile {
            state: vec![StateDeclaration {
                name: "count".to_string(),
                mutable: false,
                type_annotation: Some("Int".to_string()),
                initial_value: "0".to_string(),
                is_derived_state: false,
            }],
            markup: Markup::Interpolation("count".to_string()),
            ..Default::default()
        };

        let semantic_info = Analyzer::analyze(&ast).unwrap();
        let symbol = semantic_info.symbol_table.get("count").unwrap();

        // Should have recorded 1 access
        assert_eq!(symbol.usage_info.access_count, 1);
        assert!(symbol.usage_info.contexts.contains(&UsageContext::InInterpolation));
    }

    #[test]
    fn test_tracks_variable_in_component_prop() {
        use crate::transpiler::ast::{Component, ComponentProp, Markup, PropValue};

        let ast = WhitehallFile {
            state: vec![StateDeclaration {
                name: "title".to_string(),
                mutable: false,
                type_annotation: Some("String".to_string()),
                initial_value: "\"Hello\"".to_string(),
                is_derived_state: false,
            }],
            markup: Markup::Component(Component {
                name: "Text".to_string(),
                props: vec![ComponentProp {
                    name: "text".to_string(),
                    value: PropValue::Expression("title".to_string()),
                }],
                children: vec![],
                self_closing: false,
            }),
            ..Default::default()
        };

        let semantic_info = Analyzer::analyze(&ast).unwrap();
        let symbol = semantic_info.symbol_table.get("title").unwrap();

        assert_eq!(symbol.usage_info.access_count, 1);
        assert!(symbol.usage_info.contexts.contains(&UsageContext::InComponentProp {
            component: "Text".to_string(),
            prop_name: "text".to_string(),
        }));
    }

    #[test]
    fn test_tracks_for_loop_collection_access() {
        use crate::transpiler::ast::Markup;

        let ast = WhitehallFile {
            state: vec![StateDeclaration {
                name: "items".to_string(),
                mutable: false,
                type_annotation: Some("List<String>".to_string()),
                initial_value: "listOf()".to_string(),
                is_derived_state: false,
            }],
            markup: Markup::ForLoop(ForLoopBlock {
                item: "item".to_string(),
                collection: "items".to_string(),
                key_expr: None,
                body: vec![],
                empty_block: None,
            }),
            ..Default::default()
        };

        let semantic_info = Analyzer::analyze(&ast).unwrap();
        let symbol = semantic_info.symbol_table.get("items").unwrap();

        assert_eq!(symbol.usage_info.access_count, 1);
        assert!(symbol.usage_info.contexts.contains(&UsageContext::InForLoopCollection {
            collection: "items".to_string(),
        }));
    }

    #[test]
    fn test_tracks_variable_used_in_loop_body() {
        use crate::transpiler::ast::{Component, ComponentProp, Markup, PropValue};

        let ast = WhitehallFile {
            state: vec![
                StateDeclaration {
                    name: "posts".to_string(),
                    mutable: false,
                    type_annotation: Some("List<Post>".to_string()),
                    initial_value: "listOf()".to_string(),
                    is_derived_state: false,
                },
                StateDeclaration {
                    name: "highlight".to_string(),
                    mutable: false,
                    type_annotation: Some("String".to_string()),
                    initial_value: "\"red\"".to_string(),
                    is_derived_state: false,
                },
            ],
            markup: Markup::ForLoop(ForLoopBlock {
                item: "post".to_string(),
                collection: "posts".to_string(),
                key_expr: None,
                body: vec![Markup::Component(Component {
                    name: "Text".to_string(),
                    props: vec![ComponentProp {
                        name: "color".to_string(),
                        value: PropValue::Expression("highlight".to_string()),
                    }],
                    children: vec![],
                    self_closing: false,
                })],
                empty_block: None,
            }),
            ..Default::default()
        };

        let semantic_info = Analyzer::analyze(&ast).unwrap();

        // Check 'posts' collection
        let posts = semantic_info.symbol_table.get("posts").unwrap();
        assert_eq!(posts.usage_info.access_count, 1);
        assert!(posts.usage_info.contexts.contains(&UsageContext::InForLoopCollection {
            collection: "posts".to_string(),
        }));

        // Check 'highlight' used inside loop body
        let highlight = semantic_info.symbol_table.get("highlight").unwrap();
        assert_eq!(highlight.usage_info.access_count, 1);
        assert!(highlight.usage_info.used_in_loops);
        assert!(highlight.usage_info.contexts.contains(&UsageContext::InForLoopBody {
            collection: "posts".to_string(),
        }));
    }

    #[test]
    fn test_tracks_key_expression_usage() {
        use crate::transpiler::ast::Markup;

        let ast = WhitehallFile {
            state: vec![StateDeclaration {
                name: "contacts".to_string(),
                mutable: false,
                type_annotation: Some("List<Contact>".to_string()),
                initial_value: "listOf()".to_string(),
                is_derived_state: false,
            }],
            markup: Markup::ForLoop(ForLoopBlock {
                item: "contact".to_string(),
                collection: "contacts".to_string(),
                key_expr: Some("contact.email".to_string()), // 'contact' is loop variable, not tracked
                body: vec![],
                empty_block: None,
            }),
            ..Default::default()
        };

        let semantic_info = Analyzer::analyze(&ast).unwrap();
        let symbol = semantic_info.symbol_table.get("contacts").unwrap();

        // Should record collection access + potentially key expression
        assert!(symbol.usage_info.access_count >= 1);
        assert!(symbol.usage_info.contexts.contains(&UsageContext::InForLoopCollection {
            collection: "contacts".to_string(),
        }));
    }

    #[test]
    fn test_tracks_condition_usage() {
        use crate::transpiler::ast::Markup;

        let ast = WhitehallFile {
            state: vec![StateDeclaration {
                name: "isVisible".to_string(),
                mutable: true,
                type_annotation: Some("Boolean".to_string()),
                initial_value: "true".to_string(),
                is_derived_state: false,
            }],
            markup: Markup::IfElse(IfElseBlock {
                condition: "isVisible".to_string(),
                then_branch: vec![Markup::Text("Visible".to_string())],
                else_ifs: vec![],
                else_branch: None,
            }),
            ..Default::default()
        };

        let semantic_info = Analyzer::analyze(&ast).unwrap();
        let symbol = semantic_info.symbol_table.get("isVisible").unwrap();

        assert_eq!(symbol.usage_info.access_count, 1);
        assert!(symbol.usage_info.used_in_conditions);
        assert!(symbol.usage_info.contexts.contains(&UsageContext::InCondition {
            condition_type: "if".to_string(),
        }));
    }

    #[test]
    fn test_tracks_complex_expression() {
        use crate::transpiler::ast::{Component, ComponentProp, Markup, PropValue};

        let ast = WhitehallFile {
            state: vec![
                StateDeclaration {
                    name: "count".to_string(),
                    mutable: true,
                    type_annotation: Some("Int".to_string()),
                    initial_value: "0".to_string(),
                    is_derived_state: false,
                },
                StateDeclaration {
                    name: "max".to_string(),
                    mutable: false,
                    type_annotation: Some("Int".to_string()),
                    initial_value: "100".to_string(),
                    is_derived_state: false,
                },
            ],
            markup: Markup::Component(Component {
                name: "Text".to_string(),
                props: vec![ComponentProp {
                    name: "text".to_string(),
                    value: PropValue::Expression("count < max".to_string()),
                }],
                children: vec![],
                self_closing: false,
            }),
            ..Default::default()
        };

        let semantic_info = Analyzer::analyze(&ast).unwrap();

        // Both 'count' and 'max' should be tracked
        let count = semantic_info.symbol_table.get("count").unwrap();
        assert_eq!(count.usage_info.access_count, 1);

        let max_sym = semantic_info.symbol_table.get("max").unwrap();
        assert_eq!(max_sym.usage_info.access_count, 1);
    }

    #[test]
    fn test_tracks_multiple_accesses() {
        use crate::transpiler::ast::{Component, ComponentProp, Markup, PropValue};

        let ast = WhitehallFile {
            state: vec![StateDeclaration {
                name: "title".to_string(),
                mutable: false,
                type_annotation: Some("String".to_string()),
                initial_value: "\"Hello\"".to_string(),
                is_derived_state: false,
            }],
            markup: Markup::Sequence(vec![
                Markup::Component(Component {
                    name: "Text".to_string(),
                    props: vec![ComponentProp {
                        name: "text".to_string(),
                        value: PropValue::Expression("title".to_string()),
                    }],
                    children: vec![],
                    self_closing: false,
                }),
                Markup::Interpolation("title".to_string()),
                Markup::Component(Component {
                    name: "Text".to_string(),
                    props: vec![ComponentProp {
                        name: "text".to_string(),
                        value: PropValue::Expression("title".to_string()),
                    }],
                    children: vec![],
                    self_closing: false,
                }),
            ]),
            ..Default::default()
        };

        let semantic_info = Analyzer::analyze(&ast).unwrap();
        let symbol = semantic_info.symbol_table.get("title").unwrap();

        // Should have 3 accesses total
        assert_eq!(symbol.usage_info.access_count, 3);
    }

    #[test]
    fn test_tracks_when_condition() {
        use crate::transpiler::ast::{Markup, WhenBlock, WhenBranch};

        let ast = WhitehallFile {
            state: vec![StateDeclaration {
                name: "status".to_string(),
                mutable: true,
                type_annotation: Some("String".to_string()),
                initial_value: "\"active\"".to_string(),
                is_derived_state: false,
            }],
            markup: Markup::When(WhenBlock {
                branches: vec![
                    WhenBranch {
                        condition: Some("status == \"active\"".to_string()),
                        body: Markup::Text("Active".to_string()),
                    },
                    WhenBranch {
                        condition: None, // else
                        body: Markup::Text("Inactive".to_string()),
                    },
                ],
            }),
            ..Default::default()
        };

        let semantic_info = Analyzer::analyze(&ast).unwrap();
        let symbol = semantic_info.symbol_table.get("status").unwrap();

        assert_eq!(symbol.usage_info.access_count, 1);
        assert!(symbol.usage_info.used_in_conditions);
        assert!(symbol.usage_info.contexts.contains(&UsageContext::InCondition {
            condition_type: "when".to_string(),
        }));
    }
}
