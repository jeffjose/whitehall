/// Parser for Whitehall syntax

use crate::transpiler::ast::{
    ClassDeclaration, Component, ComponentProp, ConstructorDeclaration, ElseIfBranch,
    ForLoopBlock, FunctionDeclaration, IfElseBlock, Import, KotlinBlock, LifecycleHook, Markup,
    PropDeclaration, PropertyDeclaration, PropValue, StateDeclaration, WhenBlock, WhenBranch,
    WhitehallFile,
};

pub struct Parser {
    input: String,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Parser {
            input: input.trim().to_string(),
            pos: 0,
        }
    }

    /// Convert byte position to (line, column) for error messages
    fn pos_to_line_col(&self, pos: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for (i, ch) in self.input.chars().enumerate() {
            if i >= pos {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    /// Create an error message with position information
    fn error_at_pos(&self, message: &str) -> String {
        let (line, col) = self.pos_to_line_col(self.pos);
        self.format_error(line, col, message, None)
    }

    /// Get a specific line from the source (1-indexed)
    fn get_source_line(&self, line_num: usize) -> Option<String> {
        self.input.lines().nth(line_num - 1).map(|s| s.to_string())
    }

    /// Format an error message in Cargo style
    fn format_error(&self, line: usize, col: usize, message: &str, help: Option<&str>) -> String {
        let mut error = format!("{}\n", message);
        error.push_str(&format!(" --> line {}:{}\n", line, col));
        error.push_str("  |\n");

        if let Some(source_line) = self.get_source_line(line) {
            error.push_str(&format!("{:>3} | {}\n", line, source_line));
            // Add pointer line
            let pointer = format!("{}^^^", " ".repeat(col - 1));
            error.push_str(&format!("  | {}\n", pointer));
        }

        if let Some(help_text) = help {
            error.push_str("  |\n");
            error.push_str(&format!("  = help: {}\n", help_text));
        }

        error
    }

    /// Check for common typos and return a helpful error if found
    /// Returns Some(error_message) if a typo is detected, None otherwise
    fn check_for_typos(&self) -> Option<String> {
        let remaining = &self.input[self.pos..];
        let (line, col) = self.pos_to_line_col(self.pos);

        // Common typos: missing $ prefix on magic functions
        let typo_suggestions = [
            ("onMount", "$onMount", "lifecycle hook"),
            ("onDispose", "$onDispose", "lifecycle hook"),
            ("fetch(", "$fetch(", "HTTP request function"),
            ("log(", "$log(", "logging function"),
            ("navigate(", "$navigate(", "navigation function"),
        ];

        for (typo, correct, description) in typo_suggestions {
            // Check if remaining starts with the typo followed by whitespace or {
            if remaining.starts_with(typo) {
                let after_typo = &remaining[typo.len()..];
                // Make sure it's actually the keyword (not part of a larger identifier)
                if after_typo.starts_with(' ') || after_typo.starts_with('{') ||
                   after_typo.starts_with('(') || after_typo.starts_with('\n') {
                    let msg = format!("unknown identifier '{}'", typo.trim_end_matches('('));
                    let help = format!("did you mean '{}'? ({} requires $ prefix)", correct.trim_end_matches('('), description);
                    return Some(self.format_error(line, col, &msg, Some(&help)));
                }
            }
        }

        // Check for old @if/@for/@when without @ prefix (unlikely but possible)
        let directive_typos = [
            ("if (", "@if", "conditional directive"),
            ("if(", "@if", "conditional directive"),
            ("for (", "@for", "loop directive"),
            ("for(", "@for", "loop directive"),
            ("when (", "@when", "when directive"),
            ("when(", "@when", "when directive"),
        ];

        for (typo, correct, description) in directive_typos {
            if remaining.starts_with(typo) {
                let msg = format!("unexpected '{}'", typo.trim());
                let help = format!("did you mean '{}'? ({} requires @ prefix in markup)", correct, description);
                return Some(self.format_error(line, col, &msg, Some(&help)));
            }
        }

        None
    }

    /// Check a code body (like lifecycle hook or function body) for common typos
    /// Returns Some(error_message) if a typo is detected, None otherwise
    fn check_body_for_typos(&self, body: &str, body_start_pos: usize) -> Option<String> {
        // Magic functions that require $ prefix
        let function_typos = [
            ("fetch(", "$fetch(", "HTTP request function"),
            ("log(", "$log(", "logging function"),
            ("navigate(", "$navigate(", "navigation function"),
        ];

        for (typo, correct, description) in function_typos {
            // Find all occurrences of the typo in the body
            let mut search_start = 0;
            while let Some(pos) = body[search_start..].find(typo) {
                let absolute_pos = search_start + pos;

                // Check it's not preceded by $ (which would make it correct)
                let is_preceded_by_dollar = absolute_pos > 0 &&
                    body.chars().nth(absolute_pos - 1) == Some('$');

                // Check it's not part of a larger identifier (preceded by letter/underscore)
                let is_part_of_identifier = absolute_pos > 0 && {
                    let prev_char = body.chars().nth(absolute_pos - 1).unwrap_or(' ');
                    prev_char.is_alphanumeric() || prev_char == '_'
                };

                if !is_preceded_by_dollar && !is_part_of_identifier {
                    // Calculate line/column within the body
                    let mut line = 1;
                    let mut col = 1;
                    for (i, ch) in body.chars().enumerate() {
                        if i >= absolute_pos {
                            break;
                        }
                        if ch == '\n' {
                            line += 1;
                            col = 1;
                        } else {
                            col += 1;
                        }
                    }

                    // Get the line/col from the start of the body in the original file
                    let (body_line, _) = self.pos_to_line_col(body_start_pos);
                    let actual_line = body_line + line - 1;

                    let msg = format!("unknown identifier '{}'", typo.trim_end_matches('('));
                    let help = format!("did you mean '{}'? ({} requires $ prefix)", correct.trim_end_matches('('), description);
                    return Some(self.format_error(actual_line, col, &msg, Some(&help)));
                }

                search_start = absolute_pos + 1;
            }
        }

        None
    }

    pub fn parse(&mut self) -> Result<WhitehallFile, String> {
        let mut imports = Vec::new();
        let mut props = Vec::new();
        let mut state = Vec::new();
        let mut functions = Vec::new();
        let mut lifecycle_hooks = Vec::new();
        let mut classes = Vec::new();
        let mut kotlin_blocks: Vec<KotlinBlock> = Vec::new();
        let mut pending_annotations = Vec::new();
        let mut parsed_store_class = false; // Track if we've parsed a store class

        // Parse imports, props, state, functions, lifecycle hooks, and classes (before markup)
        loop {
            self.skip_whitespace();

            // Check for annotations (@store, @HiltViewModel, etc.)
            if self.peek_char() == Some('@') {
                self.advance_char(); // Skip @
                let annotation = self.parse_identifier()?;
                pending_annotations.push(annotation.clone());

                // Check if next is "class" or "object" keyword
                self.skip_whitespace();
                let next_word = self.peek_word();
                if next_word == Some("class") || next_word == Some("object") {
                    classes.push(self.parse_class_declaration(pending_annotations.clone())?);
                    parsed_store_class = true; // Mark that we've seen a store class
                    pending_annotations.clear();
                    continue;
                } else if annotation == "prop" {
                    // Handle @prop (legacy parsing)
                    pending_annotations.clear();
                    props.push(self.parse_prop_declaration()?);
                    continue;
                }
                // Otherwise, continue to next iteration to collect more annotations
                continue;
            } else if !parsed_store_class && (self.peek_word() == Some("class") || self.peek_word() == Some("object")) {
                // Standalone class/object without annotation (e.g., class with var properties)
                // Only parse as store class if we haven't seen one yet
                // After first store class, let class/object declarations pass through
                classes.push(self.parse_class_declaration(Vec::new())?);
                parsed_store_class = true; // Mark that we've seen a store class
            } else if self.consume_word("import") {
                imports.push(self.parse_import()?);
            } else if self.is_kotlin_syntax(parsed_store_class) {
                // Pass-through: Kotlin syntax that doesn't need transformation
                // This includes data classes, sealed classes, typealias, etc.
                // After store class, also includes plain functions (extensions, helpers)
                // This check must come BEFORE var/val check to catch extension properties
                let mut block = self.capture_kotlin_block()?;

                // If there are pending annotations, prepend them to the kotlin block content
                if !pending_annotations.is_empty() {
                    let annotations_str = pending_annotations.iter()
                        .map(|a| format!("@{}", a))
                        .collect::<Vec<_>>()
                        .join("\n");
                    block.content = format!("{}\n{}", annotations_str, block.content);
                    pending_annotations.clear();
                }

                kotlin_blocks.push(block);
            } else if self.peek_word() == Some("var") || self.peek_word() == Some("val") {
                // Parse state declarations (only for non-extension properties)
                state.push(self.parse_state_declaration()?);
            } else if !parsed_store_class && self.peek_word() == Some("suspend") {
                // Parse suspend functions as component functions (before store class only)
                // After store class, is_kotlin_syntax() will catch these and pass through
                self.consume_word("suspend");
                self.skip_whitespace();
                if !self.consume_word("fun") {
                    return Err(self.error_at_pos("Expected 'fun' after 'suspend'"));
                }
                functions.push(self.parse_function_declaration(true)?);
            } else if !parsed_store_class && self.consume_word("fun") {
                // Parse plain functions as component functions (before store class only)
                // After store class, is_kotlin_syntax() will catch these and pass through
                functions.push(self.parse_function_declaration(false)?);
            } else if self.consume_word("$onMount") {
                lifecycle_hooks.push(self.parse_lifecycle_hook("onMount")?);
            } else if self.consume_word("$onDispose") {
                lifecycle_hooks.push(self.parse_lifecycle_hook("onDispose")?);
            } else if self.peek_char() == Some('<') {
                // Check for <script> tags
                let script_imports = self.try_parse_script_tag()?;
                if !script_imports.is_empty() {
                    imports.extend(script_imports);
                    continue;
                }
                // Not a script tag, break to parse markup
                break;
            } else {
                // Before breaking, check for common typos
                if let Some(error) = self.check_for_typos() {
                    return Err(error);
                }
                break;
            }
        }

        // Parse markup (optional for store-only files)
        self.skip_whitespace();
        let markup = if !classes.is_empty() && self.peek_char().is_none() {
            // Store-only file with no markup
            Markup::Text(String::new())
        } else if self.peek_char().is_some() {
            self.parse_markup()?
        } else {
            // Empty file or whitespace only
            Markup::Text(String::new())
        };

        // Phase 5: After markup, capture any remaining Kotlin blocks (e.g., data classes after component markup)
        loop {
            self.skip_whitespace();

            if self.peek_char().is_none() {
                break; // EOF
            }

            if self.peek_char() == Some('@') {
                // Collect annotations for next kotlin block
                self.advance_char(); // Skip @
                let annotation = self.parse_identifier()?;
                pending_annotations.push(annotation);
                continue;
            }

            // Check if this is a function with markup content (helper composable)
            let remaining = &self.input[self.pos..];
            if remaining.starts_with("fun ") || remaining.starts_with("suspend fun ") {
                // Peek ahead to see if function body contains markup
                let is_suspend = if remaining.starts_with("suspend fun ") {
                    self.consume_word("suspend");
                    self.skip_whitespace();
                    true
                } else {
                    false
                };

                // Save position to potentially backtrack
                let func_start_pos = self.pos;

                // Try to parse as function with markup
                if self.consume_word("fun") {
                    self.skip_whitespace();
                    let name = self.parse_identifier()?;
                    self.skip_whitespace();

                    // Parse params
                    self.expect_char('(')?;
                    let param_start = self.pos;
                    let mut paren_depth = 1;
                    while paren_depth > 0 {
                        match self.peek_char() {
                            Some('(') => paren_depth += 1,
                            Some(')') => paren_depth -= 1,
                            None => return Err("Unexpected EOF in function params".to_string()),
                            _ => {}
                        }
                        self.advance_char();
                    }
                    let params = self.input[param_start..self.pos - 1].to_string();
                    self.skip_whitespace();

                    // Parse optional return type
                    let return_type = if self.peek_char() == Some(':') {
                        self.expect_char(':')?;
                        self.skip_whitespace();
                        Some(self.parse_type()?)
                    } else {
                        None
                    };
                    self.skip_whitespace();

                    // Check if body starts with {
                    if self.peek_char() == Some('{') {
                        self.expect_char('{')?;
                        self.skip_whitespace();

                        // Check if body contains markup (starts with <)
                        if self.peek_char() == Some('<') {
                            // This is a composable function with markup - parse the markup
                            let markup = self.parse_markup()?;
                            self.skip_whitespace();
                            self.expect_char('}')?;

                            functions.push(FunctionDeclaration {
                                name,
                                params,
                                return_type,
                                body: String::new(), // Body is in markup field
                                is_suspend,
                                markup: Some(markup),
                            });
                            continue;
                        } else {
                            // Not markup - backtrack and treat as kotlin block
                            self.pos = func_start_pos;
                        }
                    } else {
                        // No body braces - backtrack
                        self.pos = func_start_pos;
                    }
                } else {
                    // Failed to parse as function - backtrack
                    self.pos = func_start_pos;
                }
            }

            // After markup, all Kotlin syntax passes through (including plain functions)
            // Context is always "after store" because we're past the component definition
            if self.is_kotlin_syntax(true) {
                let mut block = self.capture_kotlin_block()?;

                // If there are pending annotations, prepend them
                if !pending_annotations.is_empty() {
                    let annotations_str = pending_annotations.iter()
                        .map(|a| format!("@{}", a))
                        .collect::<Vec<_>>()
                        .join("\n");
                    block.content = format!("{}\n{}", annotations_str, block.content);
                    pending_annotations.clear();
                }

                kotlin_blocks.push(block);
            } else {
                // Unknown syntax after markup - stop parsing
                break;
            }
        }

        Ok(WhitehallFile {
            imports,
            props,
            state,
            functions,
            lifecycle_hooks,
            classes,
            markup,
            kotlin_blocks,
        })
    }

    fn parse_import(&mut self) -> Result<Import, String> {
        // Parse: import $models.User or import androidx.compose.ui.Modifier
        self.skip_whitespace();
        let start = self.pos;

        // Parse import path (until newline)
        while let Some(ch) = self.peek_char() {
            if ch == '\n' {
                break;
            }
            self.advance_char();
        }

        let path = self.input[start..self.pos].trim().to_string();
        Ok(Import { path })
    }

    /// Try to parse a <script> tag and extract imports
    /// Returns empty vec if not a script tag (and doesn't advance position)
    fn try_parse_script_tag(&mut self) -> Result<Vec<Import>, String> {
        // Save position in case this isn't a script tag
        let saved_pos = self.pos;

        // Check for <script>
        if self.peek_char() != Some('<') {
            return Ok(Vec::new());
        }
        self.advance_char(); // consume <

        // Check if it's "script"
        let tag_start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch == '>' || ch.is_whitespace() {
                break;
            }
            self.advance_char();
        }

        let tag_name = &self.input[tag_start..self.pos];
        if tag_name != "script" {
            // Not a script tag, restore position
            self.pos = saved_pos;
            return Ok(Vec::new());
        }

        // Skip to >
        while self.peek_char() != Some('>') {
            if self.peek_char().is_none() {
                return Err("Unexpected EOF in script tag".to_string());
            }
            self.advance_char();
        }
        self.advance_char(); // consume >

        // Extract content until </script>
        let content_start = self.pos;
        loop {
            if self.peek_char().is_none() {
                return Err("Unclosed <script> tag".to_string());
            }

            // Check for </script>
            if self.peek_char() == Some('<') &&
               self.peek_ahead(1) == Some('/') {
                let check_pos = self.pos;
                self.advance_char(); // <
                self.advance_char(); // /

                let close_tag_start = self.pos;
                while let Some(ch) = self.peek_char() {
                    if ch == '>' {
                        break;
                    }
                    self.advance_char();
                }

                let close_tag = &self.input[close_tag_start..self.pos];
                if close_tag == "script" {
                    // Found </script>, extract content (convert to String to avoid borrow issues)
                    let content = self.input[content_start..check_pos].to_string();
                    self.advance_char(); // consume >

                    // Parse imports from content
                    let mut imports = Vec::new();
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("import ") {
                            let path = trimmed["import ".len()..].trim().to_string();
                            imports.push(Import { path });
                        }
                    }

                    return Ok(imports);
                }
            }

            self.advance_char();
        }
    }

    fn parse_prop_declaration(&mut self) -> Result<PropDeclaration, String> {
        // Parse: @prop val name: Type [= default]
        self.skip_whitespace();

        // Skip 'val' (props are always val)
        if !self.consume_word("val") {
            return Err("Expected 'val' after @prop".to_string());
        }

        self.skip_whitespace();
        let name = self.parse_identifier()?;
        self.skip_whitespace();
        self.expect_char(':')?;
        self.skip_whitespace();

        // Parse type (everything until = or newline)
        let prop_type = self.parse_type()?;

        // Check for default value
        self.skip_whitespace();
        let default_value = if self.peek_char() == Some('=') {
            self.expect_char('=')?;
            self.skip_whitespace();
            Some(self.parse_value()?)
        } else {
            None
        };

        Ok(PropDeclaration {
            name,
            prop_type,
            default_value,
        })
    }

    fn parse_type(&mut self) -> Result<String, String> {
        let start = self.pos;
        let mut paren_depth = 0;
        let mut angle_depth = 0;
        let mut bracket_depth = 0;

        while let Some(ch) = self.peek_char() {
            match ch {
                '(' => {
                    paren_depth += 1;
                    self.advance_char();
                }
                ')' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                        self.advance_char();
                    } else {
                        break;
                    }
                }
                '<' => {
                    angle_depth += 1;
                    self.advance_char();
                }
                '>' => {
                    if angle_depth > 0 {
                        angle_depth -= 1;
                        self.advance_char();
                    } else {
                        // Could be part of -> operator, keep going if in parens
                        if paren_depth > 0 || bracket_depth > 0 {
                            self.advance_char();
                        } else {
                            break;
                        }
                    }
                }
                '[' => {
                    bracket_depth += 1;
                    self.advance_char();
                }
                ']' => {
                    if bracket_depth > 0 {
                        bracket_depth -= 1;
                        self.advance_char();
                    } else {
                        break;
                    }
                }
                '=' | '\n' | '{' if paren_depth == 0 && angle_depth == 0 && bracket_depth == 0 => break,
                '-' if self.peek_ahead(1) == Some('>') => {
                    // -> is part of function type, continue parsing
                    self.advance_char(); // Skip -
                    self.advance_char(); // Skip >
                }
                _ => self.advance_char(),
            }
        }

        Ok(self.input[start..self.pos].trim().to_string())
    }

    fn parse_state_declaration(&mut self) -> Result<StateDeclaration, String> {
        // Parse: var name = "value" or var name: Type = value
        let mutable = if self.consume_word("var") {
            true
        } else if self.consume_word("val") {
            false
        } else {
            return Err("Expected 'var' or 'val'".to_string());
        };

        self.skip_whitespace();
        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Check for type annotation
        let type_annotation = if self.peek_char() == Some(':') {
            self.expect_char(':')?;
            self.skip_whitespace();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.skip_whitespace();
        if self.peek_char() != Some('=') {
            let context = &self.input[self.pos..self.pos.saturating_add(50).min(self.input.len())];
            return Err(format!(
                "Expected '=' after variable '{}' with type {:?}, found: {:?} (context: {:?})",
                name,
                type_annotation,
                self.peek_char(),
                context
            ));
        }
        self.expect_char('=')?;
        self.skip_whitespace();

        // Parse initial value (simple string or expression for now)
        let mut initial_value = self.parse_value()?;

        // Detect if this uses derivedStateOf or $derived() pattern
        let mut is_derived_state = initial_value.trim().starts_with("derivedStateOf");
        let mut effective_mutable = mutable;

        // Transform $derived(expr) → derivedStateOf { expr }
        // Note: $derived() always creates immutable derived state, even if declared with var
        if initial_value.trim().starts_with("$derived(") {
            is_derived_state = true;
            effective_mutable = false; // $derived() is always treated as val
            // Extract the expression inside $derived(...)
            let trimmed = initial_value.trim();
            if let Some(inner) = trimmed.strip_prefix("$derived(").and_then(|s| s.strip_suffix(")")) {
                initial_value = format!("derivedStateOf {{ {} }}", inner.trim());
            }
        }

        Ok(StateDeclaration {
            name,
            mutable: effective_mutable,
            type_annotation,
            initial_value,
            is_derived_state,
        })
    }

    fn parse_function_declaration(&mut self, is_suspend: bool) -> Result<FunctionDeclaration, String> {
        // Parse: fun name(params): ReturnType { body } or fun name(params) { body }
        self.skip_whitespace();
        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Parse params (capture everything between parens)
        self.expect_char('(')?;
        let param_start = self.pos;
        while self.peek_char() != Some(')') {
            if self.peek_char().is_none() {
                return Err("Unexpected EOF in function params".to_string());
            }
            self.advance_char();
        }
        let params = self.input[param_start..self.pos].trim().to_string();
        self.expect_char(')')?;
        self.skip_whitespace();

        // Check for optional return type annotation
        let return_type = if self.peek_char() == Some(':') {
            self.expect_char(':')?;
            self.skip_whitespace();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.skip_whitespace();

        // Parse body (everything between { and })
        self.expect_char('{')?;

        // Save position where body starts for error reporting
        let body_start_pos = self.pos;

        let mut body = String::new();
        let mut depth = 1;

        while depth > 0 {
            match self.peek_char() {
                Some('{') => {
                    body.push('{');
                    depth += 1;
                    self.advance_char();
                }
                Some('}') => {
                    if depth > 1 {
                        body.push('}');
                    }
                    depth -= 1;
                    self.advance_char();
                }
                Some(ch) => {
                    body.push(ch);
                    self.advance_char();
                }
                None => return Err("Unexpected EOF in function body".to_string()),
            }
        }

        // Check for common typos in the body BEFORE trimming (to preserve line numbers)
        if let Some(error) = self.check_body_for_typos(&body, body_start_pos) {
            return Err(error);
        }

        Ok(FunctionDeclaration {
            name,
            params,
            return_type,
            body: body.trim().to_string(),
            is_suspend,
            markup: None,
        })
    }

    fn parse_lifecycle_hook(&mut self, hook_type: &str) -> Result<LifecycleHook, String> {
        // Parse: $onMount { body } or $onDispose { body }
        self.skip_whitespace();
        self.expect_char('{')?;

        // Save position where body starts for error reporting
        let body_start_pos = self.pos;

        let mut body = String::new();
        let mut depth = 1;

        while depth > 0 {
            match self.peek_char() {
                Some('{') => {
                    body.push('{');
                    depth += 1;
                    self.advance_char();
                }
                Some('}') => {
                    if depth > 1 {
                        body.push('}');
                    }
                    depth -= 1;
                    self.advance_char();
                }
                Some(ch) => {
                    body.push(ch);
                    self.advance_char();
                }
                None => return Err("Unexpected EOF in lifecycle hook body".to_string()),
            }
        }

        // Check for common typos in the body BEFORE trimming (to preserve line numbers)
        if let Some(error) = self.check_body_for_typos(&body, body_start_pos) {
            return Err(error);
        }

        Ok(LifecycleHook {
            hook_type: hook_type.to_string(),
            body: body.trim().to_string(),
        })
    }

    fn parse_class_declaration(&mut self, annotations: Vec<String>) -> Result<ClassDeclaration, String> {
        // Parse: class/object ClassName { ... } or class/object ClassName constructor(...) { ... }
        self.skip_whitespace();

        // Check for "class" or "object" keyword
        let is_object = if self.consume_word("object") {
            true
        } else if self.consume_word("class") {
            false
        } else {
            return Err(self.error_at_pos("Expected 'class' or 'object' keyword"));
        };

        self.skip_whitespace();

        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Parse optional constructor (can start with @inject, constructor, or direct params)
        let constructor = if self.peek_word() == Some("constructor")
            || self.peek_char() == Some('(')
            || self.peek_char() == Some('@') {
            Some(self.parse_constructor()?)
        } else {
            None
        };

        self.skip_whitespace();
        self.expect_char('{')?;

        // Parse class body: properties and functions
        let mut properties = Vec::new();
        let mut functions = Vec::new();

        loop {
            self.skip_whitespace();

            // Check for end of class
            if self.peek_char() == Some('}') {
                self.advance_char();
                break;
            }

            // Check for property (with optional visibility modifier)
            if self.peek_word() == Some("var") || self.peek_word() == Some("val")
                || self.peek_word() == Some("private") || self.peek_word() == Some("protected") || self.peek_word() == Some("public") {

                // Check for visibility modifier
                let visibility = if self.peek_word() == Some("private")
                    || self.peek_word() == Some("protected")
                    || self.peek_word() == Some("public") {
                    let vis = self.parse_identifier()?;
                    self.skip_whitespace();
                    Some(vis)
                } else {
                    None
                };

                // Now we must have var/val after optional visibility
                if self.peek_word() == Some("var") || self.peek_word() == Some("val") {
                    properties.push(self.parse_property_declaration_with_visibility(visibility)?);
                } else {
                    return Err(self.error_at_pos("Expected 'var' or 'val' after visibility modifier"));
                }
            }
            // Check for function
            else if self.peek_word() == Some("fun") || self.peek_word() == Some("suspend") {
                let is_suspend = self.consume_word("suspend"); // Optional suspend
                if is_suspend {
                    self.skip_whitespace();
                }
                self.expect_word("fun")?;
                functions.push(self.parse_function_declaration(is_suspend)?);
            }
            // Unknown content, skip
            else if self.peek_char().is_some() {
                return Err(self.error_at_pos("Unexpected content in class body"));
            } else {
                return Err("Unexpected EOF in class body".to_string());
            }
        }

        Ok(ClassDeclaration {
            annotations,
            is_object,
            name,
            constructor,
            properties,
            functions,
        })
    }

    fn parse_constructor(&mut self) -> Result<ConstructorDeclaration, String> {
        self.skip_whitespace();

        // Check for @Inject annotation
        let mut annotations = Vec::new();
        if self.peek_char() == Some('@') {
            self.advance_char();
            let annotation = self.parse_identifier()?;
            annotations.push(annotation);
            self.skip_whitespace();

            // If we have an annotation, "constructor" keyword is required
            if !self.consume_word("constructor") {
                return Err(self.error_at_pos("Expected 'constructor' keyword after @Inject"));
            }
            self.skip_whitespace();
        } else {
            // Optional "constructor" keyword for unannotated constructors
            self.consume_word("constructor");
            self.skip_whitespace();
        }

        // Parse parameters
        self.expect_char('(')?;
        let param_start = self.pos;
        let mut depth = 1;
        while depth > 0 {
            match self.peek_char() {
                Some('(') => { depth += 1; self.advance_char(); }
                Some(')') => { depth -= 1; if depth > 0 { self.advance_char(); } }
                Some(_) => { self.advance_char(); }
                None => return Err("Unexpected EOF in constructor".to_string()),
            }
        }
        let parameters = self.input[param_start..self.pos].trim().to_string();
        self.expect_char(')')?;

        Ok(ConstructorDeclaration {
            annotations,
            parameters,
        })
    }

    fn parse_property_declaration_with_visibility(&mut self, visibility: Option<String>) -> Result<PropertyDeclaration, String> {
        // Parse: [visibility] var name: Type = value or [visibility] val name = value or [visibility] val name get() = expression
        let mutable = if self.consume_word("var") {
            true
        } else if self.consume_word("val") {
            false
        } else {
            return Err("Expected 'var' or 'val'".to_string());
        };

        self.skip_whitespace();
        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Parse optional type annotation
        let type_annotation = if self.peek_char() == Some(':') {
            self.expect_char(':')?;
            self.skip_whitespace();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.skip_whitespace();

        // Check for getter (val name get() = ...)
        let (initial_value, getter) = if self.peek_word() == Some("get") {
            self.consume_word("get");
            self.skip_whitespace();
            self.expect_char('(')?;
            self.expect_char(')')?;
            self.skip_whitespace();
            self.expect_char('=')?;
            self.skip_whitespace();

            // Parse getter expression (until newline or closing brace)
            let start = self.pos;
            while let Some(ch) = self.peek_char() {
                if ch == '\n' || ch == '}' {
                    break;
                }
                self.advance_char();
            }
            (None, Some(self.input[start..self.pos].trim().to_string()))
        } else {
            // Parse initial value
            let initial_value = if self.peek_char() == Some('=') {
                self.expect_char('=')?;
                self.skip_whitespace();
                Some(self.parse_value()?)
            } else {
                None
            };

            (initial_value, None)
        };

        Ok(PropertyDeclaration {
            name,
            mutable,
            type_annotation,
            initial_value,
            getter,
            visibility,
        })
    }

    fn expect_word(&mut self, word: &str) -> Result<(), String> {
        if !self.consume_word(word) {
            Err(self.error_at_pos(&format!("Expected '{}'", word)))
        } else {
            Ok(())
        }
    }

    fn parse_value(&mut self) -> Result<String, String> {
        self.skip_whitespace();
        // Check for triple-quoted string (multi-line string) first
        if self.check_string_ahead("\"\"\"") {
            self.parse_multiline_string()
        } else if self.peek_char() == Some('"') {
            self.parse_string()
        } else if self.peek_char() == Some('[') {
            // Array literal syntax: [1, 2, 3] -> will be converted to listOf() later
            self.parse_array_literal()
        } else if self.is_range_literal() {
            // Range literal syntax: 1..10 or 1..10:2 -> will be converted to .toList() later
            self.parse_range_literal()
        } else {
            // Parse value (handle braces and parentheses for complex expressions)
            let start = self.pos;
            let mut brace_depth = 0;
            let mut paren_depth = 0;

            while let Some(ch) = self.peek_char() {
                if ch == '{' {
                    brace_depth += 1;
                    self.advance_char();
                } else if ch == '}' {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                        self.advance_char();
                    } else {
                        break; // Closing brace not part of value
                    }
                } else if ch == '(' {
                    paren_depth += 1;
                    self.advance_char();
                } else if ch == ')' {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                        self.advance_char();
                    } else {
                        break; // Closing paren not part of value
                    }
                } else if ch == '\n' && brace_depth == 0 && paren_depth == 0 {
                    // Check if line ends with continuation operator (&&, ||, +, -, etc.)
                    let trimmed = self.input[start..self.pos].trim_end();
                    if trimmed.ends_with("&&") || trimmed.ends_with("||") ||
                       trimmed.ends_with("+") || trimmed.ends_with("-") ||
                       trimmed.ends_with("*") || trimmed.ends_with("/") ||
                       trimmed.ends_with(",") {
                        // Continue parsing on next line
                        self.advance_char();
                    } else {
                        break; // End of value at newline
                    }
                } else {
                    self.advance_char();
                }
            }
            let value = self.input[start..self.pos].trim();
            // Strip trailing semicolon if present (Kotlin doesn't need them)
            let value = value.strip_suffix(';').unwrap_or(value);
            Ok(value.to_string())
        }
    }

    fn parse_array_literal(&mut self) -> Result<String, String> {
        // Parse [1, 2, 3] syntax and return as-is (will be transformed later)
        self.expect_char('[')?;
        let start = self.pos;
        let mut bracket_depth = 1;

        while let Some(ch) = self.peek_char() {
            if ch == '[' {
                bracket_depth += 1;
                self.advance_char();
            } else if ch == ']' {
                bracket_depth -= 1;
                if bracket_depth == 0 {
                    let content = self.input[start..self.pos].trim().to_string();
                    self.advance_char(); // consume closing ]
                    return Ok(format!("[{}]", content));
                }
                self.advance_char();
            } else if ch == '"' {
                // Handle strings inside array to not break on ] inside string
                self.advance_char();
                while let Some(str_ch) = self.peek_char() {
                    if str_ch == '"' {
                        self.advance_char();
                        break;
                    }
                    self.advance_char();
                }
            } else {
                self.advance_char();
            }
        }

        Err("Unterminated array literal".to_string())
    }

    /// Check if we're at the start of a range literal (e.g., 1..10, 5..20:2)
    fn is_range_literal(&self) -> bool {
        let mut temp_pos = self.pos;
        let chars: Vec<char> = self.input.chars().collect();

        // Skip optional minus sign
        if temp_pos < chars.len() && chars[temp_pos] == '-' {
            temp_pos += 1;
        }

        // Must start with a digit
        if temp_pos >= chars.len() || !chars[temp_pos].is_ascii_digit() {
            return false;
        }

        // Skip digits
        while temp_pos < chars.len() && chars[temp_pos].is_ascii_digit() {
            temp_pos += 1;
        }

        // Check for .. pattern
        if temp_pos + 1 < chars.len() && chars[temp_pos] == '.' && chars[temp_pos + 1] == '.' {
            return true;
        }

        false
    }

    /// Parse range literal: 1..10 or 1..10:2 or 10..1:-1
    /// Returns as RANGE[start..end] or RANGE[start..end:step]
    fn parse_range_literal(&mut self) -> Result<String, String> {
        let start_pos = self.pos;

        // Parse start number (may be negative)
        let mut has_minus = false;
        if self.peek_char() == Some('-') {
            has_minus = true;
            self.advance_char();
        }

        // Parse digits
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.advance_char();
            } else {
                break;
            }
        }

        let start_num = if has_minus {
            format!("-{}", &self.input[start_pos + 1..self.pos])
        } else {
            self.input[start_pos..self.pos].to_string()
        };

        // Expect ..
        if !self.consume_str("..") {
            return Err(self.error_at_pos("Expected '..' in range literal"));
        }

        // Parse end number (may be negative)
        let end_start = self.pos;
        has_minus = false;
        if self.peek_char() == Some('-') {
            has_minus = true;
            self.advance_char();
        }

        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.advance_char();
            } else {
                break;
            }
        }

        let end_num = if has_minus {
            format!("-{}", &self.input[end_start + 1..self.pos])
        } else {
            self.input[end_start..self.pos].to_string()
        };

        // Check for optional :step
        if self.peek_char() == Some(':') {
            self.advance_char(); // consume :

            let step_start = self.pos;
            has_minus = false;
            if self.peek_char() == Some('-') {
                has_minus = true;
                self.advance_char();
            }

            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_digit() {
                    self.advance_char();
                } else {
                    break;
                }
            }

            let step_num = if has_minus {
                format!("-{}", &self.input[step_start + 1..self.pos])
            } else {
                self.input[step_start..self.pos].to_string()
            };

            // Return as special marker for codegen
            Ok(format!("RANGE[{}..{}:{}]", start_num, end_num, step_num))
        } else {
            // No step, just start..end
            Ok(format!("RANGE[{}..{}]", start_num, end_num))
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect_char('"')?;
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                break;
            }
            self.advance_char();
        }
        let value = self.input[start..self.pos].to_string();
        self.expect_char('"')?;
        Ok(format!("\"{}\"", value))
    }

    /// Parse a multi-line string (triple-quoted string)
    /// Syntax: """content""" (Kotlin raw string literal)
    fn parse_multiline_string(&mut self) -> Result<String, String> {
        // Consume opening """
        for _ in 0..3 {
            self.expect_char('"')?;
        }

        let start = self.pos;

        // Find the closing """
        loop {
            if self.check_string_ahead("\"\"\"") {
                let value = self.input[start..self.pos].to_string();

                // Consume closing """
                for _ in 0..3 {
                    self.expect_char('"')?;
                }

                // Return as Kotlin raw string literal (preserves escape sequences, newlines, etc.)
                return Ok(format!("\"\"\"{}\"\"\"", value));
            }

            if self.peek_char().is_none() {
                return Err(self.error_at_pos("Unclosed multi-line string (expected closing \"\"\")"));
            }

            self.advance_char();
        }
    }

    fn parse_markup(&mut self) -> Result<Markup, String> {
        self.skip_whitespace();

        if self.peek_char() == Some('<') {
            // Try to parse first component
            let first_component = self.parse_component()?;

            // Check if there are more components at the root level
            self.skip_whitespace();

            if self.peek_char() == Some('<') {
                // Multiple root components - collect them all and wrap in Column
                let mut components = vec![first_component];

                while self.peek_char() == Some('<') {
                    components.push(self.parse_component()?);
                    self.skip_whitespace();
                }

                // Auto-wrap in Column
                Ok(Markup::Component(Component {
                    name: "Column".to_string(),
                    props: vec![],
                    children: components,
                    self_closing: false,
                }))
            } else {
                // Single root component - return as-is
                Ok(first_component)
            }
        } else {
            let remaining = if self.pos < self.input.len() {
                let end = (self.pos + 50).min(self.input.len());
                &self.input[self.pos..end]
            } else {
                "EOF"
            };
            Err(self.error_at_pos(&format!("Expected component, found: {:?}", remaining)))
        }
    }

    fn parse_component(&mut self) -> Result<Markup, String> {
        // Parse opening tag: <ComponentName ...>
        self.expect_char('<')?;
        let name = self.parse_identifier()?;
        self.skip_whitespace();

        // Parse component props
        let mut props = Vec::new();
        while self.peek_char() != Some('>') && self.peek_char() != Some('/') {
            let prop = self.parse_component_prop()?;
            props.push(prop);
            self.skip_whitespace();
        }

        // Check for self-closing tag
        let self_closing = if self.peek_char() == Some('/') {
            self.expect_char('/')?;
            self.expect_char('>')?;
            true
        } else {
            self.expect_char('>')?;
            false
        };

        let children = if self_closing {
            Vec::new()
        } else {
            // Parse children (can be text, components, or control flow)
            let children = self.parse_children(&name)?;
            children
        };

        Ok(Markup::Component(Component {
            name,
            props,
            children,
            self_closing,
        }))
    }

    fn parse_component_prop(&mut self) -> Result<ComponentProp, String> {
        // Parse: propName={expression} or propName="string" or propName={<Component />}
        // Prop names can include colons (for bind:value, on:click, etc.)
        let mut name = self.parse_identifier()?;

        // Check for colon (e.g., bind:value)
        if self.peek_char() == Some(':') {
            name.push(':');
            self.advance_char();
            let rest = self.parse_identifier()?;
            name.push_str(&rest);
        }

        self.skip_whitespace();

        // Check if this is a boolean prop (no value, like <Column fill>)
        // Boolean props are followed by: >, /, or another identifier (another prop)
        let next_char = self.peek_char();
        let is_boolean_prop = next_char == Some('>')
            || next_char == Some('/')
            || (next_char.map_or(false, |c| c.is_alphabetic()));

        let value = if is_boolean_prop && next_char != Some('=') {
            // Boolean prop without explicit value, treat as {true}
            PropValue::Expression("true".to_string())
        } else {
            // Prop with explicit value
            self.expect_char('=')?;
            self.skip_whitespace();

            if self.peek_char() == Some('"') {
            // String literal: prop="value"
            self.expect_char('"')?;
            let mut str_value = String::new();
            while let Some(ch) = self.peek_char() {
                if ch == '"' {
                    self.expect_char('"')?;
                    break;
                } else if ch == '\\' {
                    // Handle escape sequences
                    self.advance_char();
                    if let Some(escaped) = self.peek_char() {
                        str_value.push('\\');
                        str_value.push(escaped);
                        self.advance_char();
                    }
                } else {
                    str_value.push(ch);
                    self.advance_char();
                }
            }
            PropValue::Expression(format!("\"{}\"", str_value))
        } else if self.peek_char() == Some('{') {
            // Expression or component: prop={value} or prop={<Component />}
            self.expect_char('{')?;
            self.skip_whitespace();

            // Check if this is a component: {<Component />}
            if self.peek_char() == Some('<') {
                // Parse component markup
                let markup = self.parse_component()?;
                self.skip_whitespace();
                self.expect_char('}')?;
                PropValue::Markup(Box::new(markup))
            } else {
                // Parse expression until closing brace (handle nested braces)
                let mut expr_value = String::new();
                let mut depth = 1;

                while let Some(ch) = self.peek_char() {
                    if ch == '{' {
                        depth += 1;
                        expr_value.push(ch);
                        self.advance_char();
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            self.advance_char();
                            break;
                        }
                        expr_value.push(ch);
                        self.advance_char();
                    } else {
                        expr_value.push(ch);
                        self.advance_char();
                    }
                }
                PropValue::Expression(expr_value)
            }
            } else {
                return Err(format!("Expected prop value (either {{expr}} or \"string\")"));
            }
        };

        Ok(ComponentProp { name, value })
    }

    fn parse_children(&mut self, parent_name: &str) -> Result<Vec<Markup>, String> {
        let mut children = Vec::new();

        loop {
            self.skip_whitespace();

            // Check for closing tag
            if self.peek_char() == Some('<') && self.peek_ahead(1) == Some('/') {
                // Parse closing tag
                self.expect_char('<')?;
                self.expect_char('/')?;
                let closing_name = self.parse_identifier().map_err(|e| {
                    let context = &self.input[self.pos..self.pos.saturating_add(20).min(self.input.len())];
                    format!("Failed to parse closing tag name: {}. Context: {:?}", e, context)
                })?;
                self.expect_char('>')?;

                if parent_name != closing_name {
                    return Err(format!(
                        "Mismatched tags: opening <{}> vs closing </{}>",
                        parent_name, closing_name
                    ));
                }
                break;
            }

            let pos_before = self.pos;

            // Check for control flow (@if, @for, @when)
            // But not @ in text content like "@username"
            if self.peek_char() == Some('@') {
                // Look ahead to see if this is a control flow keyword
                let remaining = &self.input[self.pos..];
                if remaining.starts_with("@if ")
                    || remaining.starts_with("@for ")
                    || remaining.starts_with("@when ")
                {
                    children.push(self.parse_control_flow()?);
                } else {
                    // @ in text content, parse as text
                    let text_children = self.parse_text_with_interpolation_until_markup()?;
                    if text_children.is_empty() && self.pos == pos_before {
                        return Err(format!(
                            "Failed to parse text starting with @ at position {}",
                            self.pos
                        ));
                    }
                    children.extend(text_children);
                }
            }
            // Check for child component
            else if self.peek_char() == Some('<') {
                children.push(self.parse_component()?);
            }
            // Parse text/interpolation
            else if self.peek_char().is_some() {
                let text_children = self.parse_text_with_interpolation_until_markup()?;
                if text_children.is_empty() && self.pos == pos_before {
                    // No progress made - this shouldn't happen normally
                    return Err(format!(
                        "Infinite loop detected at position {} while parsing children of <{}>: unexpected character '{}'",
                        self.pos,
                        parent_name,
                        self.peek_char().unwrap_or('\0')
                    ));
                }
                children.extend(text_children);
            } else {
                return Err(format!("Unexpected end while parsing children of <{}>", parent_name));
            }
        }

        Ok(children)
    }

    fn parse_control_flow(&mut self) -> Result<Markup, String> {
        self.expect_char('@')?;

        if self.consume_word("if") {
            self.parse_if_else()
        } else if self.consume_word("for") {
            self.parse_for_loop()
        } else if self.consume_word("when") {
            self.parse_when()
        } else {
            Err("Unknown control flow construct".to_string())
        }
    }

    fn parse_if_else(&mut self) -> Result<Markup, String> {
        // Parse: @if (condition) { ... } [else if (...) { ... }]* [else { ... }]
        self.skip_whitespace();
        self.expect_char('(')?;
        let condition = self.parse_until_char(')')?;
        self.expect_char(')')?;
        self.skip_whitespace();
        self.expect_char('{')?;

        let then_branch = self.parse_markup_block()?;

        let mut else_ifs = Vec::new();
        let mut else_branch = None;

        // Parse else if / else
        // Support both `else` and `@else` for consistency with @if/@for syntax
        loop {
            self.skip_whitespace();
            // Accept both "else" and "@else" for better DX
            let has_else = if self.peek_char() == Some('@') {
                self.advance_char(); // consume @
                self.consume_word("else")
            } else {
                self.consume_word("else")
            };
            if has_else {
                self.skip_whitespace();
                // Accept both "if" and "@if" after else
                let has_if = if self.peek_char() == Some('@') {
                    self.advance_char(); // consume @
                    self.consume_word("if")
                } else {
                    self.consume_word("if")
                };
                if has_if {
                    // else if
                    self.skip_whitespace();
                    self.expect_char('(')?;
                    let condition = self.parse_until_char(')')?;
                    self.expect_char(')')?;
                    self.skip_whitespace();
                    self.expect_char('{')?;
                    let body = self.parse_markup_block()?;
                    else_ifs.push(ElseIfBranch { condition, body });
                } else {
                    // else
                    self.expect_char('{')?;
                    else_branch = Some(self.parse_markup_block()?);
                    break;
                }
            } else {
                break;
            }
        }

        Ok(Markup::IfElse(IfElseBlock {
            condition,
            then_branch,
            else_ifs,
            else_branch,
        }))
    }

    fn parse_for_loop(&mut self) -> Result<Markup, String> {
        // Parse: @for (item in collection, key = { expr }) { ... } [empty { ... }]
        // Or:    @for (index, item in collection, key = { expr }) { ... } [empty { ... }]
        self.skip_whitespace();
        self.expect_char('(')?;

        // Parse first identifier (could be index or item)
        let first_ident = self.parse_identifier()?;

        self.skip_whitespace();

        // Check if this is indexed form: (index, item in collection)
        let (index, item) = if self.peek_char() == Some(',') {
            // Look ahead to see if next token after comma is an identifier followed by 'in'
            let saved_pos = self.pos;
            self.advance_char(); // consume ','
            self.skip_whitespace();

            // Try to parse second identifier
            if let Ok(second_ident) = self.parse_identifier() {
                self.skip_whitespace();
                if self.consume_word("in") {
                    // This is indexed form: (index, item in ...)
                    (Some(first_ident), second_ident)
                } else {
                    // Not indexed form, restore position
                    self.pos = saved_pos;
                    if !self.consume_word("in") {
                        return Err("Expected 'in' after loop variable".to_string());
                    }
                    (None, first_ident)
                }
            } else {
                // Couldn't parse second identifier, restore and try normal form
                self.pos = saved_pos;
                if !self.consume_word("in") {
                    return Err("Expected 'in' after loop variable".to_string());
                }
                (None, first_ident)
            }
        } else if self.consume_word("in") {
            // Normal form: (item in collection)
            (None, first_ident)
        } else {
            return Err("Expected ',' or 'in' after loop variable".to_string());
        };

        self.skip_whitespace();
        // Parse collection name (up to comma or closing paren)
        let mut collection = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == ',' || ch == ')' {
                break;
            }
            collection.push(ch);
            self.advance_char();
        }
        let collection = collection.trim().to_string();

        // Check for optional key
        let mut key_expr = None;
        self.skip_whitespace();
        if self.peek_char() == Some(',') {
            self.expect_char(',')?;
            self.skip_whitespace();

            // Expect "key"
            if !self.consume_word("key") {
                return Err("Expected 'key' after comma in for loop".to_string());
            }

            self.skip_whitespace();
            self.expect_char('=')?;
            self.skip_whitespace();

            // Parse key expression (should be a lambda like { it.id })
            self.expect_char('{')?;
            let key_body = self.parse_until_char('}')?;
            self.expect_char('}')?;
            key_expr = Some(key_body.trim().to_string());
        }

        self.skip_whitespace();
        self.expect_char(')')?;
        self.skip_whitespace();
        self.expect_char('{')?;

        // Parse loop body
        let body = self.parse_markup_block()?;

        // Check for optional empty block
        let mut empty_block = None;
        self.skip_whitespace();
        if self.consume_word("empty") {
            self.skip_whitespace();
            self.expect_char('{')?;
            empty_block = Some(self.parse_markup_block()?);
        }

        Ok(Markup::ForLoop(ForLoopBlock {
            index,
            item,
            collection,
            key_expr,
            body,
            empty_block,
        }))
    }

    fn parse_when(&mut self) -> Result<Markup, String> {
        // Parse: @when { condition -> markup, ... else -> markup }
        self.skip_whitespace();
        self.expect_char('{')?;

        let mut branches = Vec::new();

        loop {
            self.skip_whitespace();

            // Check for closing brace
            if self.peek_char() == Some('}') {
                self.expect_char('}')?;
                break;
            }

            // Parse branch
            let condition = if self.consume_word("else") {
                None
            } else {
                // Parse condition until '->'
                let mut cond = String::new();
                while let Some(ch) = self.peek_char() {
                    if ch == '-' && self.peek_ahead(1) == Some('>') {
                        break;
                    }
                    cond.push(ch);
                    self.advance_char();
                }
                Some(cond.trim().to_string())
            };

            // Expect '->'
            self.skip_whitespace();
            self.expect_char('-')?;
            self.expect_char('>')?;
            self.skip_whitespace();

            // Parse body (single markup item)
            let body = if self.peek_char() == Some('<') {
                self.parse_component()?
            } else {
                return Err("Expected component after '->' in when branch".to_string());
            };

            branches.push(WhenBranch { condition, body });
        }

        Ok(Markup::When(WhenBlock { branches }))
    }

    fn parse_markup_block(&mut self) -> Result<Vec<Markup>, String> {
        // Parse markup until closing brace
        let mut items = Vec::new();

        loop {
            self.skip_whitespace();

            if self.peek_char() == Some('}') {
                self.expect_char('}')?;
                break;
            }

            let pos_before = self.pos;

            // Check for Kotlin code (val declarations, etc.)
            // Look ahead to see if this is a Kotlin declaration
            if self.is_kotlin_declaration() {
                // Parse the Kotlin line and skip it (it will be in the generated code context)
                let _kotlin_line = self.parse_kotlin_statement()?;
                // For now, we skip Kotlin statements in markup blocks
                // They will be handled by the code generator when it processes the conditional
                continue;
            }
            // Check for control flow
            else if self.peek_char() == Some('@') {
                items.push(self.parse_control_flow()?);
            }
            // Check for component
            else if self.peek_char() == Some('<') {
                items.push(self.parse_component()?);
            }
            // Text/interpolation
            else if self.peek_char().is_some() {
                let text_items = self.parse_text_with_interpolation_until_markup()?;
                if text_items.is_empty() && self.pos == pos_before {
                    // No progress made - this shouldn't happen normally
                    // Skip the current character to avoid infinite loop
                    return Err(format!(
                        "Infinite loop detected at position {}: unexpected character '{}'",
                        self.pos,
                        self.peek_char().unwrap_or('\0')
                    ));
                }
                items.extend(text_items);
            } else {
                return Err("Unexpected end in markup block".to_string());
            }
        }

        Ok(items)
    }

    fn parse_text_with_interpolation_until_markup(&mut self) -> Result<Vec<Markup>, String> {
        let mut children = Vec::new();
        let mut current_text = String::new();

        while let Some(ch) = self.peek_char() {
            // Stop at markup or control flow, but allow @ in text
            if ch == '<' {
                break;
            } else if ch == '}' {
                // Check for double closing brace: }} → literal }
                if self.peek_ahead(1) == Some('}') {
                    current_text.push('}');
                    self.advance_char(); // consume first }
                    self.advance_char(); // consume second }
                } else {
                    break;
                }
            } else if ch == '@' {
                // Check if this is a control flow keyword
                let remaining = &self.input[self.pos..];
                if remaining.starts_with("@if ")
                    || remaining.starts_with("@for ")
                    || remaining.starts_with("@when ")
                {
                    break;
                }
                // Otherwise, @ is part of text content
                current_text.push(ch);
                self.advance_char();
            } else if ch == '{' {
                // Check for double-brace escape: {{expr}} → literal {expr}
                if self.peek_ahead(1) == Some('{') {
                    current_text.push('{');
                    self.advance_char(); // consume first {
                    self.advance_char(); // consume second {
                } else {
                    // Save any accumulated text
                    if !current_text.is_empty() {
                        children.push(Markup::Text(current_text.clone()));
                        current_text.clear();
                    }
                    // Parse interpolation
                    self.expect_char('{')?;
                    let expr = self.parse_until_char('}')?;
                    self.expect_char('}')?;
                    children.push(Markup::Interpolation(expr));
                }
            } else {
                current_text.push(ch);
                self.advance_char();
            }
        }

        // Save any remaining text
        if !current_text.is_empty() {
            children.push(Markup::Text(current_text));
        }

        Ok(children)
    }

    #[allow(dead_code)]
    fn parse_text_with_interpolation_until(&mut self, delimiter: char) -> Result<Vec<Markup>, String> {
        let mut children = Vec::new();
        let mut current_text = String::new();

        while let Some(ch) = self.peek_char() {
            if ch == delimiter {
                break;
            } else if ch == '{' {
                // Check for double-brace escape: {{expr}} → literal {expr}
                if self.peek_ahead(1) == Some('{') {
                    current_text.push('{');
                    self.advance_char(); // consume first {
                    self.advance_char(); // consume second {
                } else {
                    // Save any accumulated text
                    if !current_text.is_empty() {
                        children.push(Markup::Text(current_text.clone()));
                        current_text.clear();
                    }
                    // Parse interpolation
                    self.expect_char('{')?;
                    let expr = self.parse_until_char('}')?;
                    self.expect_char('}')?;
                    children.push(Markup::Interpolation(expr));
                }
            } else if ch == '}' && self.peek_ahead(1) == Some('}') {
                // Double closing brace: }} → literal }
                current_text.push('}');
                self.advance_char(); // consume first }
                self.advance_char(); // consume second }
            } else {
                current_text.push(ch);
                self.advance_char();
            }
        }

        // Save any remaining text
        if !current_text.is_empty() {
            children.push(Markup::Text(current_text));
        }

        Ok(children)
    }

    fn parse_until_char(&mut self, delimiter: char) -> Result<String, String> {
        let start = self.pos;
        let mut paren_depth = 0;
        let mut brace_depth = 0;
        let mut bracket_depth = 0;

        while let Some(ch) = self.peek_char() {
            // Check if we've reached the delimiter at depth 0
            if ch == delimiter && paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 {
                break;
            }

            // Track nesting depth
            match ch {
                '(' => paren_depth += 1,
                ')' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                }
                '{' => brace_depth += 1,
                '}' => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                    }
                }
                '[' => bracket_depth += 1,
                ']' => {
                    if bracket_depth > 0 {
                        bracket_depth -= 1;
                    }
                }
                _ => {}
            }

            self.advance_char();
        }
        Ok(self.input[start..self.pos].to_string())
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() {
                self.advance_char();
            } else {
                break;
            }
        }

        if start == self.pos {
            return Err(self.error_at_pos("Expected identifier"));
        }

        Ok(self.input[start..self.pos].to_string())
    }

    #[allow(dead_code)]
    fn parse_text_until(&mut self, delimiter: char) -> Result<String, String> {
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch == delimiter {
                break;
            }
            self.advance_char();
        }

        Ok(self.input[start..self.pos].to_string())
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        match self.peek_char() {
            Some(ch) if ch == expected => {
                // Advance by the byte length of the character, not just 1
                self.pos += ch.len_utf8();
                Ok(())
            }
            Some(ch) => Err(self.error_at_pos(&format!("Expected '{}', found '{}'", expected, ch))),
            None => Err(self.error_at_pos(&format!("Expected '{}', found EOF", expected))),
        }
    }

    fn peek_char(&self) -> Option<char> {
        // self.pos is a byte index, not a char index
        // We need to slice at the byte position and get the first char
        self.input[self.pos..].chars().next()
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        // Same here - slice at byte position then skip offset characters
        self.input[self.pos..].chars().nth(offset)
    }

    /// Check if the input at current position starts with the given string
    fn check_string_ahead(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    /// Advance position by one character (handling multi-byte UTF-8)
    fn advance_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.pos += ch.len_utf8();
        }
    }

    fn peek_word(&self) -> Option<&str> {
        let remaining = &self.input[self.pos..];
        if remaining.starts_with("import ") {
            Some("import")
        } else if remaining.starts_with("var ") || remaining.starts_with("var\n") {
            Some("var")
        } else if remaining.starts_with("val ") || remaining.starts_with("val\n") {
            Some("val")
        } else if remaining.starts_with("private ") {
            Some("private")
        } else if remaining.starts_with("protected ") {
            Some("protected")
        } else if remaining.starts_with("public ") {
            Some("public")
        } else if remaining.starts_with("@prop") {
            Some("@prop")
        } else if remaining.starts_with("fun ") {
            Some("fun")
        } else if remaining.starts_with("class ") {
            Some("class")
        } else if remaining.starts_with("object ") {
            Some("object")
        } else if remaining.starts_with("suspend ") {
            Some("suspend")
        } else if remaining.starts_with("constructor") {
            Some("constructor")
        } else if remaining.starts_with("get") {
            Some("get")
        } else if remaining.starts_with("$onMount") {
            Some("$onMount")
        } else if remaining.starts_with("$onDispose") {
            Some("$onDispose")
        } else {
            None
        }
    }

    fn consume_str(&mut self, s: &str) -> bool {
        let remaining = &self.input[self.pos..];
        if remaining.starts_with(s) {
            self.pos += s.len();
            true
        } else {
            false
        }
    }

    fn consume_word(&mut self, word: &str) -> bool {
        let remaining = &self.input[self.pos..];
        if remaining.starts_with(word) {
            let next_pos = self.pos + word.len();
            // For @prop, get, and constructor, don't require whitespace after
            if word == "@prop" || word == "get" || word == "constructor" {
                self.pos = next_pos;
                return true;
            }
            if next_pos < self.input.len() {
                // Use byte slicing instead of chars().nth() to avoid UTF-8 byte/char index mismatch
                let next_char = self.input[next_pos..].chars().next();
                if next_char.map_or(true, |c| c.is_whitespace()) {
                    self.pos = next_pos;
                    return true;
                }
            } else {
                self.pos = next_pos;
                return true;
            }
        }
        false
    }

    fn skip_whitespace(&mut self) {
        loop {
            let start_pos = self.pos;

            // Skip whitespace characters
            while let Some(ch) = self.peek_char() {
                if ch.is_whitespace() {
                    self.advance_char();
                } else {
                    break;
                }
            }

            // Skip comments
            if self.peek_char() == Some('/') {
                if self.peek_ahead(1) == Some('/') {
                    // Single-line comment: skip until newline
                    self.advance_char(); // Skip first /
                    self.advance_char(); // Skip second /
                    while let Some(ch) = self.peek_char() {
                        if ch == '\n' {
                            self.advance_char(); // Skip the newline too
                            break;
                        }
                        self.advance_char();
                    }
                } else if self.peek_ahead(1) == Some('*') {
                    // Multi-line comment: skip until */
                    self.advance_char(); // Skip /
                    self.advance_char(); // Skip *
                    while let Some(ch) = self.peek_char() {
                        if ch == '*' && self.peek_ahead(1) == Some('/') {
                            self.advance_char(); // Skip *
                            self.advance_char(); // Skip /
                            break;
                        }
                        self.advance_char();
                    }
                } else {
                    // Not a comment, just a / character
                    break;
                }
            } else {
                break;
            }

            // If we made no progress, stop (no more whitespace or comments)
            if self.pos == start_pos {
                break;
            }
        }
    }

    // ========== Pass-Through Architecture Helpers ==========

    /// Check if the current position is at Kotlin syntax that should pass through unchanged.
    ///
    /// # Context-Aware Behavior
    ///
    /// The `after_store_class` parameter affects how plain functions are handled:
    /// - **Before store class:** `fun` declarations are parsed as component functions (transformed to ViewModel methods)
    /// - **After store class:** `fun` declarations pass through unchanged (helpers, extensions, top-level functions)
    /// - **After markup:** `fun` declarations always pass through unchanged
    ///
    /// # Always Pass-Through (regardless of context):
    /// - Data classes, sealed classes, enum classes
    /// - Type aliases, objects, interfaces
    /// - Specialized function modifiers (inline, infix, operator)
    /// - Fun interfaces (SAM)
    /// - Extension properties
    fn is_kotlin_syntax(&self, after_store_class: bool) -> bool {
        let remaining = &self.input[self.pos..];

        // Check for Kotlin keywords that we don't explicitly parse
        if remaining.starts_with("data class ") ||
           remaining.starts_with("sealed class ") ||
           remaining.starts_with("sealed interface ") ||
           remaining.starts_with("enum class ") ||
           remaining.starts_with("class ") ||  // Regular classes (must come after specific class types)
           remaining.starts_with("typealias ") ||
           remaining.starts_with("object ") ||
           // Specialized function modifiers (always pass through)
           remaining.starts_with("inline fun ") ||
           remaining.starts_with("infix fun ") ||
           remaining.starts_with("operator fun ") ||
           // Fun interfaces (SAM interfaces)
           remaining.starts_with("fun interface ") {
            return true;
        }

        // Plain functions - context-dependent behavior
        if after_store_class {
            // After store class, ALL functions pass through (including plain fun and suspend fun)
            if remaining.starts_with("fun ") {
                return true;
            }
            // Also handle suspend functions after store class
            if remaining.starts_with("suspend fun ") {
                return true;
            }
        }

        // Extension properties: val Type.property
        if remaining.starts_with("val ") {
            let after_val = &remaining[4..]; // Skip "val "
            // Look for a dot before colon or newline (indicates extension property)
            // Example: "User.fullName: String" should find '.' at position 4
            for (_i, ch) in after_val.char_indices() {
                if ch == '.' {
                    return true; // Found dot, this is an extension property
                } else if ch == ':' || ch == '\n' || ch == '=' {
                    break; // Reached end of type/property name without finding dot
                }
            }
        }

        false
        // Note: object declarations are safe to pass through here because @store objects
        // would have been handled by the annotation parsing logic earlier in the parse loop
    }

    /// Detect what type of Kotlin construct this is based on the leading keywords.
    /// This is just a hint for debugging/tooling - the content isn't parsed.
    fn detect_block_type(&self) -> crate::transpiler::ast::KotlinBlockType {
        use crate::transpiler::ast::KotlinBlockType;
        let remaining = &self.input[self.pos..];

        if remaining.starts_with("data class ") {
            KotlinBlockType::DataClass
        } else if remaining.starts_with("sealed class ") || remaining.starts_with("sealed interface ") {
            KotlinBlockType::SealedClass
        } else if remaining.starts_with("enum class ") {
            KotlinBlockType::EnumClass
        } else if remaining.starts_with("fun ") {
            KotlinBlockType::TopLevelFunction
        } else if remaining.starts_with("typealias ") {
            KotlinBlockType::TypeAlias
        } else if remaining.starts_with("object ") {
            KotlinBlockType::ObjectDeclaration
        } else {
            KotlinBlockType::Unknown
        }
    }

    /// Check if we're at the start of a new top-level Kotlin declaration.
    /// Used to detect when a pass-through block ends and a new one begins.
    fn is_top_level_keyword(&self) -> bool {
        let remaining = &self.input[self.pos..];

        // Keywords that start a new top-level declaration
        let keywords = [
            "import ",
            "class ",
            "data class ",
            "sealed class ",
            "sealed interface ",
            "enum class ",
            "object ",
            "interface ",
            "fun ",
            "val ",
            "var ",
            "typealias ",
        ];

        for keyword in &keywords {
            if remaining.starts_with(keyword) {
                return true;
            }
        }

        // Check for annotations
        if remaining.starts_with("@") {
            return true;
        }

        false
    }

    /// Capture a Kotlin code block that passes through unchanged.
    /// Phase 4: Handles string literals AND comments correctly.
    ///
    /// IMPORTANT: We must track both strings and comments because they can contain
    /// braces/parens that should not affect depth counting. For example:
    ///   data class Foo(
    ///       val x: String = "{ not a brace }",  // Also not { a brace }
    ///       val y: Int  /* Neither is { this } */
    ///   )
    ///
    /// The key insight is that strings and comments are mutually exclusive contexts:
    /// - Comment markers inside strings are just text: "// not a comment"
    /// - String delimiters inside comments are just text: /* "not a string" */
    fn capture_kotlin_block(&mut self) -> Result<KotlinBlock, String> {
        let start_pos = self.pos;
        let block_type = self.detect_block_type();

        // Track brace and paren depth to find the end of the block
        let mut brace_depth = 0;
        let mut paren_depth = 0;
        let mut found_opening_brace = false;
        let mut found_opening_paren = false;

        // String literal tracking
        let mut in_string = false;           // Regular string: "..."
        let mut in_char = false;             // Character literal: '...'
        let mut in_multiline_string = false; // Multi-line string: """..."""
        let mut escaped = false;             // Previous char was backslash

        // Comment tracking (Phase 4)
        let mut in_line_comment = false;     // Line comment: // ... until \n
        let mut in_block_comment = false;    // Block comment: /* ... */

        // Expression body tracking - once we see '=' after parens, consume until newline
        let mut in_expression_body = false;

        // Guard against infinite loops
        let mut iterations = 0;
        let max_iterations = 100000; // Safety limit

        while self.peek_char().is_some() {
            iterations += 1;
            if iterations > max_iterations {
                return Err(self.error_at_pos("Parser stuck capturing Kotlin block - possible infinite loop"));
            }

            let ch = self.peek_char().unwrap();
            let next_ch = self.peek_ahead(1);
            let next_next_ch = self.peek_ahead(2);

            // ========== ESCAPE SEQUENCES ==========
            // Handle escape sequences in regular strings and char literals
            // Multi-line strings (raw strings) don't process escape sequences the same way
            if escaped {
                escaped = false;
                self.advance_char();
                continue;
            }

            // Check for backslash escape in regular strings/chars (not multi-line)
            if ch == '\\' && (in_string || in_char) && !in_multiline_string {
                escaped = true;
                self.advance_char();
                continue;
            }

            // ========== COMMENT HANDLING ==========
            // Comments can only start when NOT in a string
            // Strings can only start when NOT in a comment
            // This ensures mutual exclusion

            let in_any_string = in_string || in_char || in_multiline_string;
            let in_any_comment = in_line_comment || in_block_comment;

            // Check for COMMENT START (only when not already in a string)
            if !in_any_string && !in_block_comment {
                // Line comment: //
                if ch == '/' && next_ch == Some('/') {
                    in_line_comment = true;
                    self.advance_char(); // First /
                    self.advance_char(); // Second /
                    continue;
                }
            }

            if !in_any_string && !in_line_comment {
                // Block comment: /*
                if ch == '/' && next_ch == Some('*') {
                    in_block_comment = true;
                    self.advance_char(); // /
                    self.advance_char(); // *
                    continue;
                }
            }

            // Check for COMMENT END
            if in_line_comment && ch == '\n' {
                // Line comment ends at newline
                in_line_comment = false;
                self.advance_char();

                // After line comment ends, check for new declaration at depth 0
                if brace_depth == 0 && paren_depth == 0 {
                    self.skip_whitespace();
                    if self.is_top_level_keyword() {
                        break;
                    }
                }
                continue;
            }

            if in_block_comment && ch == '*' && next_ch == Some('/') {
                // Block comment ends at */
                in_block_comment = false;
                self.advance_char(); // *
                self.advance_char(); // /
                continue;
            }

            // If we're in a comment, skip everything else (don't process strings or structure)
            if in_any_comment {
                self.advance_char();
                continue;
            }

            // ========== STRING HANDLING ==========
            // From here on, we're NOT in a comment, so we can process strings

            // Check for multi-line string delimiters: """
            // Must check before single " to avoid false positives
            if ch == '"' && next_ch == Some('"') && next_next_ch == Some('"') {
                // Toggle multi-line string state
                in_multiline_string = !in_multiline_string;
                self.advance_char(); // First "
                self.advance_char(); // Second "
                self.advance_char(); // Third "
                continue;
            }

            // Check for regular string/char delimiters (only when not in multi-line string)
            if !in_multiline_string {
                match ch {
                    '"' if !in_char => {
                        in_string = !in_string;
                        self.advance_char();
                        continue;
                    }
                    '\'' if !in_string => {
                        in_char = !in_char;
                        self.advance_char();
                        continue;
                    }
                    _ => {}
                }
            }

            // ========== STRUCTURAL CHARACTER TRACKING ==========
            // Only track braces/parens when NOT inside any string literal
            if !in_string && !in_char && !in_multiline_string {
                match ch {
                    '(' => {
                        found_opening_paren = true;
                        paren_depth += 1;
                        self.advance_char();
                    }
                    ')' => {
                        if paren_depth > 0 {
                            paren_depth -= 1;
                            self.advance_char();

                            // If we're back to depth 0 for parens and no braces were found
                            // Check what comes next before deciding to break
                            if paren_depth == 0 && brace_depth == 0 && found_opening_paren && !found_opening_brace {
                                // Save position to potentially backtrack
                                let saved_pos = self.pos;

                                // Skip whitespace to see what's next
                                while self.peek_char().map(|c| c.is_whitespace() && c != '\n').unwrap_or(false) {
                                    self.advance_char();
                                }

                                let next = self.peek_char();
                                let next_next = self.peek_ahead(1);  // Get next two chars for -> detection

                                // Restore position
                                self.pos = saved_pos;

                                match next {
                                    // Expression body (fun foo() = expr) or type alias (typealias A = B)
                                    Some('=') => {
                                        // Mark that we're in an expression body
                                        in_expression_body = true;
                                        // Continue capturing - there's more content
                                    }
                                    Some(':') => {
                                        // Continue capturing - there's more content (type annotation)
                                    }
                                    // Opening brace (will be handled in next iteration)
                                    Some('{') => {
                                        // Continue capturing
                                    }
                                    // Function type arrow: (A) -> B
                                    Some('-') if next_next == Some('>') => {
                                        // This is a function type arrow, continue capturing
                                    }
                                    // Newline or EOF - we're done
                                    Some('\n') | None => {
                                        break;
                                    }
                                    // Anything else - check if it's end of declaration
                                    _ => {
                                        // If we're in an expression body, don't break - keep capturing
                                        if !in_expression_body {
                                            // For data class Foo(...) pattern, break here
                                            // The closing paren is the end
                                            break;
                                        }
                                    }
                                }
                            }
                        } else {
                            // Unmatched closing paren - this is an error
                            return Err(self.error_at_pos("Unmatched closing parenthesis in Kotlin block"));
                        }
                    }
                    '{' => {
                        found_opening_brace = true;
                        brace_depth += 1;
                        self.advance_char();
                    }
                    '}' => {
                        if brace_depth > 0 {
                            brace_depth -= 1;
                            self.advance_char();

                            // If we're back to depth 0 for braces, we're done
                            if brace_depth == 0 && found_opening_brace {
                                break;
                            }
                        } else {
                            // Unmatched closing brace - this is an error
                            return Err(self.error_at_pos("Unmatched closing brace in Kotlin block"));
                        }
                    }
                    '=' => {
                        // If we see '=' at depth 0, we're in an expression body
                        if brace_depth == 0 && paren_depth == 0 && found_opening_paren {
                            in_expression_body = true;
                        }
                        self.advance_char();
                    }
                    '\n' => {
                        self.advance_char();

                        // At depth 0 (before any braces/parens or after balanced), check for new declaration
                        if brace_depth == 0 && paren_depth == 0 {
                            self.skip_whitespace();
                            if self.is_top_level_keyword() {
                                // Hit a new top-level declaration, stop capturing
                                break;
                            }
                        }
                    }
                    _ => {
                        self.advance_char();
                    }
                }
            } else {
                // Inside a string literal, just advance without tracking structure
                self.advance_char();
            }
        }

        // Sanity checks: we should not end with unclosed strings or comments
        if in_string {
            return Err(self.error_at_pos("Unclosed string literal in Kotlin block"));
        }
        if in_char {
            return Err(self.error_at_pos("Unclosed character literal in Kotlin block"));
        }
        if in_multiline_string {
            return Err(self.error_at_pos("Unclosed multi-line string literal in Kotlin block"));
        }
        if in_block_comment {
            return Err(self.error_at_pos("Unclosed block comment in Kotlin block"));
        }
        // Note: in_line_comment at EOF is OK - line comments can end at EOF

        // Extract the captured content
        let content = self.input[start_pos..self.pos].trim().to_string();

        if content.is_empty() {
            return Err(self.error_at_pos("Empty Kotlin block captured"));
        }

        Ok(KotlinBlock {
            content,
            position: start_pos,
            block_type,
        })
    }

    /// Check if the current position starts a Kotlin declaration (val, var, etc.)
    fn is_kotlin_declaration(&self) -> bool {
        let remaining = &self.input[self.pos..];

        // Check for val/var declarations
        remaining.starts_with("val ") || remaining.starts_with("var ")
    }

    /// Parse a Kotlin statement (like a val declaration) until newline or semicolon
    fn parse_kotlin_statement(&mut self) -> Result<String, String> {
        let mut statement = String::new();

        while let Some(ch) = self.peek_char() {
            if ch == '\n' || ch == ';' {
                if ch == ';' {
                    statement.push(ch);
                    self.advance_char();
                }
                break;
            }
            statement.push(ch);
            self.advance_char();
        }

        Ok(statement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text_component() {
        let mut parser = Parser::new("<Text>Hello, World!</Text>");
        let result = parser.parse();
        assert!(result.is_ok());
    }
}
