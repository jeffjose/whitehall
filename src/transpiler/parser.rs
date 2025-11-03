/// Parser for Whitehall syntax

use crate::transpiler::ast::{
    Component, ComponentProp, ElseIfBranch, ForLoopBlock, FunctionDeclaration, IfElseBlock,
    Import, LifecycleHook, Markup, PropDeclaration, PropValue, StateDeclaration, WhenBlock,
    WhenBranch, WhitehallFile,
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

    pub fn parse(&mut self) -> Result<WhitehallFile, String> {
        let mut imports = Vec::new();
        let mut props = Vec::new();
        let mut state = Vec::new();
        let mut functions = Vec::new();
        let mut lifecycle_hooks = Vec::new();

        // Parse imports, props, state, functions, and lifecycle hooks (before markup)
        loop {
            self.skip_whitespace();
            if self.consume_word("import") {
                imports.push(self.parse_import()?);
            } else if self.consume_word("@prop") {
                props.push(self.parse_prop_declaration()?);
            } else if self.peek_word() == Some("var") || self.peek_word() == Some("val") {
                state.push(self.parse_state_declaration()?);
            } else if self.consume_word("fun") {
                functions.push(self.parse_function_declaration()?);
            } else if self.consume_word("onMount") {
                lifecycle_hooks.push(self.parse_lifecycle_hook("onMount")?);
            } else if self.consume_word("onDispose") {
                lifecycle_hooks.push(self.parse_lifecycle_hook("onDispose")?);
            } else {
                break;
            }
        }

        let markup = self.parse_markup()?;
        Ok(WhitehallFile {
            imports,
            props,
            state,
            functions,
            lifecycle_hooks,
            markup,
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
            self.pos += 1;
        }

        let path = self.input[start..self.pos].trim().to_string();
        Ok(Import { path })
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
                    self.pos += 1;
                }
                ')' => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                '<' => {
                    angle_depth += 1;
                    self.pos += 1;
                }
                '>' => {
                    if angle_depth > 0 {
                        angle_depth -= 1;
                        self.pos += 1;
                    } else {
                        // Could be part of -> operator, keep going if in parens
                        if paren_depth > 0 || bracket_depth > 0 {
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                }
                '[' => {
                    bracket_depth += 1;
                    self.pos += 1;
                }
                ']' => {
                    if bracket_depth > 0 {
                        bracket_depth -= 1;
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                '=' | '\n' | '{' if paren_depth == 0 && angle_depth == 0 && bracket_depth == 0 => break,
                '-' if self.peek_ahead(1) == Some('>') => {
                    // -> is part of function type, continue parsing
                    self.pos += 2; // Skip ->
                }
                _ => self.pos += 1,
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
        self.expect_char('=')?;
        self.skip_whitespace();

        // Parse initial value (simple string or expression for now)
        let initial_value = self.parse_value()?;

        // Detect if this uses derivedStateOf pattern
        let is_derived_state = initial_value.trim().starts_with("derivedStateOf");

        Ok(StateDeclaration {
            name,
            mutable,
            type_annotation,
            initial_value,
            is_derived_state,
        })
    }

    fn parse_function_declaration(&mut self) -> Result<FunctionDeclaration, String> {
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
            self.pos += 1;
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
        let mut body = String::new();
        let mut depth = 1;

        while depth > 0 {
            match self.peek_char() {
                Some('{') => {
                    body.push('{');
                    depth += 1;
                    self.pos += 1;
                }
                Some('}') => {
                    if depth > 1 {
                        body.push('}');
                    }
                    depth -= 1;
                    self.pos += 1;
                }
                Some(ch) => {
                    body.push(ch);
                    self.pos += 1;
                }
                None => return Err("Unexpected EOF in function body".to_string()),
            }
        }

        Ok(FunctionDeclaration {
            name,
            params,
            return_type,
            body: body.trim().to_string(),
        })
    }

    fn parse_lifecycle_hook(&mut self, hook_type: &str) -> Result<LifecycleHook, String> {
        // Parse: onMount { body }
        self.skip_whitespace();
        self.expect_char('{')?;

        let mut body = String::new();
        let mut depth = 1;

        while depth > 0 {
            match self.peek_char() {
                Some('{') => {
                    body.push('{');
                    depth += 1;
                    self.pos += 1;
                }
                Some('}') => {
                    if depth > 1 {
                        body.push('}');
                    }
                    depth -= 1;
                    self.pos += 1;
                }
                Some(ch) => {
                    body.push(ch);
                    self.pos += 1;
                }
                None => return Err("Unexpected EOF in lifecycle hook body".to_string()),
            }
        }

        Ok(LifecycleHook {
            hook_type: hook_type.to_string(),
            body: body.trim().to_string(),
        })
    }

    fn parse_value(&mut self) -> Result<String, String> {
        self.skip_whitespace();
        if self.peek_char() == Some('"') {
            self.parse_string()
        } else {
            // Parse value (handle braces for complex expressions like filter { ... })
            let start = self.pos;
            let mut brace_depth = 0;

            while let Some(ch) = self.peek_char() {
                if ch == '{' {
                    brace_depth += 1;
                    self.pos += 1;
                } else if ch == '}' {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                        self.pos += 1;
                    } else {
                        break; // Closing brace not part of value
                    }
                } else if ch == '\n' && brace_depth == 0 {
                    break; // End of value at newline (unless inside braces)
                } else {
                    self.pos += 1;
                }
            }
            Ok(self.input[start..self.pos].trim().to_string())
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect_char('"')?;
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                break;
            }
            self.pos += 1;
        }
        let value = self.input[start..self.pos].to_string();
        self.expect_char('"')?;
        Ok(format!("\"{}\"", value))
    }

    fn parse_markup(&mut self) -> Result<Markup, String> {
        self.skip_whitespace();

        if self.peek_char() == Some('<') {
            self.parse_component()
        } else {
            let remaining = if self.pos < self.input.len() {
                let end = (self.pos + 50).min(self.input.len());
                &self.input[self.pos..end]
            } else {
                "EOF"
            };
            Err(format!("Expected component, found: {:?}", remaining))
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
            self.pos += 1;
            let rest = self.parse_identifier()?;
            name.push_str(&rest);
        }

        self.skip_whitespace();
        self.expect_char('=')?;
        self.skip_whitespace();

        let value = if self.peek_char() == Some('"') {
            // String literal: prop="value"
            self.expect_char('"')?;
            let mut str_value = String::new();
            while let Some(ch) = self.peek_char() {
                if ch == '"' {
                    self.expect_char('"')?;
                    break;
                } else if ch == '\\' {
                    // Handle escape sequences
                    self.pos += 1;
                    if let Some(escaped) = self.peek_char() {
                        str_value.push('\\');
                        str_value.push(escaped);
                        self.pos += 1;
                    }
                } else {
                    str_value.push(ch);
                    self.pos += 1;
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
                        self.pos += 1;
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            self.pos += 1;
                            break;
                        }
                        expr_value.push(ch);
                        self.pos += 1;
                    } else {
                        expr_value.push(ch);
                        self.pos += 1;
                    }
                }
                PropValue::Expression(expr_value)
            }
        } else {
            return Err(format!("Expected prop value (either {{expr}} or \"string\")"));
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
                let closing_name = self.parse_identifier()?;
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
            if self.peek_char() == Some('@') {
                children.push(self.parse_control_flow()?);
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
        loop {
            self.skip_whitespace();
            if self.consume_word("else") {
                self.skip_whitespace();
                if self.consume_word("if") {
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
        self.skip_whitespace();
        self.expect_char('(')?;

        // Parse item name
        let item = self.parse_identifier()?;

        self.skip_whitespace();
        if !self.consume_word("in") {
            return Err("Expected 'in' after loop variable".to_string());
        }

        self.skip_whitespace();
        // Parse collection name (up to comma or closing paren)
        let mut collection = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == ',' || ch == ')' {
                break;
            }
            collection.push(ch);
            self.pos += 1;
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
                    self.pos += 1;
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

            // Check for control flow
            if self.peek_char() == Some('@') {
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
            if ch == '<' || ch == '@' || ch == '}' {
                break;
            } else if ch == '{' {
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
            } else {
                current_text.push(ch);
                self.pos += 1;
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
            } else {
                current_text.push(ch);
                self.pos += 1;
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

            self.pos += 1;
        }
        Ok(self.input[start..self.pos].to_string())
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        let start = self.pos;
        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() {
                self.pos += 1;
            } else {
                break;
            }
        }

        if start == self.pos {
            return Err("Expected identifier".to_string());
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
            self.pos += 1;
        }

        Ok(self.input[start..self.pos].to_string())
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        match self.peek_char() {
            Some(ch) if ch == expected => {
                self.pos += 1;
                Ok(())
            }
            Some(ch) => Err(format!("Expected '{}', found '{}'", expected, ch)),
            None => Err(format!("Expected '{}', found EOF", expected)),
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.input.chars().nth(self.pos + offset)
    }

    fn peek_word(&self) -> Option<&str> {
        let remaining = &self.input[self.pos..];
        if remaining.starts_with("import ") {
            Some("import")
        } else if remaining.starts_with("var ") || remaining.starts_with("var\n") {
            Some("var")
        } else if remaining.starts_with("val ") || remaining.starts_with("val\n") {
            Some("val")
        } else if remaining.starts_with("@prop") {
            Some("@prop")
        } else {
            None
        }
    }

    fn consume_word(&mut self, word: &str) -> bool {
        let remaining = &self.input[self.pos..];
        if remaining.starts_with(word) {
            let next_pos = self.pos + word.len();
            // For @prop, don't require whitespace after
            if word == "@prop" {
                self.pos = next_pos;
                return true;
            }
            if next_pos < self.input.len() {
                let next_char = self.input.chars().nth(next_pos);
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
                    self.pos += 1;
                } else {
                    break;
                }
            }

            // Skip comments
            if self.peek_char() == Some('/') {
                if self.peek_ahead(1) == Some('/') {
                    // Single-line comment: skip until newline
                    self.pos += 2; // Skip //
                    while let Some(ch) = self.peek_char() {
                        if ch == '\n' {
                            self.pos += 1; // Skip the newline too
                            break;
                        }
                        self.pos += 1;
                    }
                } else if self.peek_ahead(1) == Some('*') {
                    // Multi-line comment: skip until */
                    self.pos += 2; // Skip /*
                    while let Some(ch) = self.peek_char() {
                        if ch == '*' && self.peek_ahead(1) == Some('/') {
                            self.pos += 2; // Skip */
                            break;
                        }
                        self.pos += 1;
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
