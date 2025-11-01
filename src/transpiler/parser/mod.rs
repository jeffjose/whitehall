/// Parser for Whitehall source code
use crate::transpiler::ast::*;

pub struct Parser {
    source: String,
    pos: usize,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        Parser {
            source: source.to_string(),
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Result<WhitehallFile, String> {
        let mut file = WhitehallFile::new();

        self.skip_whitespace();

        // Parse Kotlin section (imports, props, state, functions, lifecycle)
        while self.pos < self.source.len() && !self.peek_char().map_or(false, |c| c == '<') {
            self.skip_whitespace();

            if self.pos >= self.source.len() {
                break;
            }

            if self.starts_with("import ") {
                file.imports.push(self.parse_import()?);
            } else if self.starts_with("@prop ") {
                file.props.push(self.parse_prop()?);
            } else if self.starts_with("var ") || self.starts_with("val ") {
                // Check if it's derived state (val with =) or mutable state (var with =)
                let state = self.parse_state()?;
                if state.mutable || state.initial_value.contains('=') {
                    if !state.mutable && state.initial_value.contains('=') {
                        // This is derived state (val x = ...)
                        file.derived_state.push(DerivedState {
                            name: state.name,
                            state_type: state.state_type,
                            expression: state.initial_value,
                        });
                    } else {
                        file.state.push(state);
                    }
                }
            } else if self.starts_with("fun ") {
                file.functions.push(self.parse_function()?);
            } else if self.starts_with("onMount ") || self.starts_with("onMount{") {
                file.lifecycle.push(self.parse_lifecycle()?);
            } else {
                // Skip unknown line
                self.skip_line();
            }

            self.skip_whitespace();
        }

        // Parse markup section
        file.markup = self.parse_markup()?;

        Ok(file)
    }

    fn parse_import(&mut self) -> Result<Import, String> {
        self.expect("import ")?;
        let path = self.read_until('\n').trim().to_string();
        Ok(Import { path })
    }

    fn parse_prop(&mut self) -> Result<PropDeclaration, String> {
        self.expect("@prop ")?;
        self.expect("val ")?;

        let name = self.read_identifier()?;
        self.expect(":")?;
        self.skip_whitespace();

        let prop_type = self.read_type()?;
        self.skip_whitespace();

        let default_value = if self.peek_char() == Some('=') {
            self.advance(); // consume '='
            self.skip_whitespace();
            Some(self.read_until('\n').trim().to_string())
        } else {
            // No default value - we're done (whitespace including newline already skipped)
            None
        };

        Ok(PropDeclaration {
            name,
            prop_type,
            default_value,
        })
    }

    fn parse_state(&mut self) -> Result<StateDeclaration, String> {
        let mutable = if self.starts_with("var ") {
            self.expect("var ")?;
            true
        } else {
            self.expect("val ")?;
            false
        };

        let name = self.read_identifier()?;
        self.skip_whitespace();

        let state_type = if self.peek_char() == Some(':') {
            self.advance(); // consume ':'
            self.skip_whitespace();
            Some(self.read_type()?)
        } else {
            None
        };

        self.skip_whitespace();
        self.expect("=")?;
        self.skip_whitespace();

        // Check if this is a multiline expression (lambda or block)
        let initial_value = if self.peek_char() == Some('{') {
            // Read balanced braces for lambda/block
            self.advance(); // consume '{'
            let body = self.read_balanced_braces()?;
            format!("{{ {} }}", body)
        } else {
            // Read until newline
            self.read_until('\n').trim().to_string()
        };

        Ok(StateDeclaration {
            name,
            mutable,
            state_type,
            initial_value,
        })
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect("fun ")?;
        let name = self.read_identifier()?;
        self.expect("(")?;

        let mut params = Vec::new();
        while self.peek_char() != Some(')') {
            self.skip_whitespace();
            if self.peek_char() == Some(')') {
                break;
            }

            let param_name = self.read_identifier()?;
            self.expect(":")?;
            self.skip_whitespace();
            let param_type = self.read_type()?;

            params.push(FunctionParam {
                name: param_name,
                param_type,
            });

            self.skip_whitespace();
            if self.peek_char() == Some(',') {
                self.advance();
            }
        }

        self.expect(")")?;
        self.skip_whitespace();

        let return_type = if self.peek_char() == Some(':') {
            self.advance();
            self.skip_whitespace();
            Some(self.read_type()?)
        } else {
            None
        };

        self.skip_whitespace();
        self.expect("{")?;

        // Read function body until matching }
        let body = self.read_balanced_braces()?;

        Ok(Function {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_lifecycle(&mut self) -> Result<LifecycleHook, String> {
        self.expect("onMount")?;
        self.skip_whitespace();
        self.expect("{")?;

        let body = self.read_balanced_braces()?;

        Ok(LifecycleHook::OnMount { body })
    }

    fn parse_markup(&mut self) -> Result<Markup, String> {
        self.skip_whitespace();

        if self.pos >= self.source.len() {
            return Ok(Markup::Sequence(Vec::new()));
        }

        let mut elements = Vec::new();

        while self.pos < self.source.len() {
            self.skip_whitespace();

            if self.pos >= self.source.len() {
                break;
            }

            if self.starts_with("@if ") {
                elements.push(self.parse_if()?);
            } else if self.starts_with("@for ") {
                elements.push(self.parse_for()?);
            } else if self.starts_with("@when ") {
                elements.push(self.parse_when()?);
            } else if self.peek_char() == Some('<') {
                elements.push(self.parse_component()?);
            } else if self.peek_char() == Some('{') {
                elements.push(self.parse_interpolation()?);
            } else {
                // Plain text
                let text = self.read_until_markup();
                if !text.trim().is_empty() {
                    elements.push(Markup::Text(text));
                }
            }

            self.skip_whitespace();
        }

        if elements.is_empty() {
            Ok(Markup::Sequence(Vec::new()))
        } else if elements.len() == 1 {
            Ok(elements.into_iter().next().unwrap())
        } else {
            Ok(Markup::Sequence(elements))
        }
    }

    fn parse_component(&mut self) -> Result<Markup, String> {
        self.expect("<")?;
        let name = self.read_identifier()?;

        let mut props = Vec::new();
        let mut self_closing = false;

        loop {
            self.skip_whitespace();

            if self.starts_with("/>") {
                self.expect("/>")?;
                self_closing = true;
                break;
            }

            if self.peek_char() == Some('>') {
                self.advance();
                break;
            }

            // Parse prop
            let prop = self.parse_prop_attribute()?;
            props.push(prop);
        }

        let children = if self_closing {
            Vec::new()
        } else {
            // Parse children until closing tag
            let mut children = Vec::new();
            loop {
                self.skip_whitespace();

                if self.starts_with(&format!("</{}>", name)) {
                    self.expect(&format!("</{}>", name))?;
                    break;
                }

                if self.pos >= self.source.len() {
                    return Err(format!("Unclosed tag: <{}>", name));
                }

                if self.starts_with("@if ") {
                    children.push(self.parse_if()?);
                } else if self.starts_with("@for ") {
                    children.push(self.parse_for()?);
                } else if self.starts_with("@when ") {
                    children.push(self.parse_when()?);
                } else if self.peek_char() == Some('<') {
                    children.push(self.parse_component()?);
                } else if self.peek_char() == Some('{') {
                    children.push(self.parse_interpolation()?);
                } else {
                    // Text content
                    let text = self.read_text_until_markup();
                    // Only skip completely empty/whitespace-only nodes
                    if !text.trim().is_empty() {
                        children.push(Markup::Text(text));
                    }
                }
            }
            children
        };

        Ok(Markup::Component(Component {
            name,
            props,
            children,
        }))
    }

    fn parse_prop_attribute(&mut self) -> Result<Prop, String> {
        let name = self.read_prop_name()?;
        self.skip_whitespace();

        if name.starts_with("bind:") {
            // Data binding: bind:value={expr}
            self.expect("=")?;
            self.skip_whitespace();
            self.expect("{")?;
            let expr = self.read_balanced_braces()?.trim().to_string();
            self.expect("}")?;

            return Ok(Prop {
                name,
                value: PropValue::Binding(expr),
            });
        }

        if self.peek_char() == Some('=') {
            self.advance();
            self.skip_whitespace();

            if self.peek_char() == Some('{') {
                // Expression: prop={expr}
                self.advance();
                let expr = self.read_balanced_braces()?.trim().to_string();
                self.expect("}")?;

                Ok(Prop {
                    name,
                    value: PropValue::Expression(expr),
                })
            } else if self.peek_char() == Some('"') {
                // String literal: prop="value"
                self.advance();
                let value = self.read_until('"').to_string();
                self.expect("\"")?;

                Ok(Prop {
                    name,
                    value: PropValue::String(value),
                })
            } else {
                Err("Expected { or \" after =".to_string())
            }
        } else {
            // Boolean prop (no value means true)
            Ok(Prop {
                name,
                value: PropValue::Expression("true".to_string()),
            })
        }
    }

    fn parse_interpolation(&mut self) -> Result<Markup, String> {
        self.expect("{")?;
        let expr = self.read_until('}').trim().to_string();
        self.expect("}")?;
        Ok(Markup::Interpolation(expr))
    }

    fn parse_if(&mut self) -> Result<Markup, String> {
        self.expect("@if (")?;
        let condition = self.read_until(')').trim().to_string();
        self.expect(")")?;
        self.skip_whitespace();
        self.expect("{")?;

        let then_branch = self.parse_markup_block()?;

        self.expect("}")?;
        self.skip_whitespace();

        let mut else_if_branches = Vec::new();
        let mut else_branch = None;

        while self.starts_with("else if (") {
            self.expect("else if (")?;
            let elif_condition = self.read_until(')').trim().to_string();
            self.expect(")")?;
            self.skip_whitespace();
            self.expect("{")?;

            let elif_body = self.parse_markup_block()?;

            self.expect("}")?;
            self.skip_whitespace();

            else_if_branches.push((elif_condition, elif_body));
        }

        if self.starts_with("else {") {
            self.expect("else {")?;
            else_branch = Some(self.parse_markup_block()?);
            self.expect("}")?;
        }

        Ok(Markup::ControlFlowIf(ControlFlowIf {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        }))
    }

    fn parse_for(&mut self) -> Result<Markup, String> {
        self.expect("@for (")?;
        let item = self.read_identifier()?;
        self.skip_whitespace();
        self.expect("in ")?;
        self.skip_whitespace();

        // Read collection (until ',' or ')')
        let collection = if self.peek_ahead_contains(',') {
            self.read_until(',').trim().to_string()
        } else {
            self.read_until(')').trim().to_string()
        };

        let key = if self.peek_char() == Some(',') {
            self.advance(); // consume ','
            self.skip_whitespace();
            self.expect("key =")?;
            self.skip_whitespace();
            Some(self.read_until(')').trim().to_string())
        } else {
            None
        };

        self.expect(")")?;
        self.skip_whitespace();
        self.expect("{")?;

        let body = self.parse_markup_block()?;

        self.expect("}")?;
        self.skip_whitespace();

        let empty = if self.starts_with("empty {") {
            self.expect("empty {")?;
            let empty_body = self.parse_markup_block()?;
            self.expect("}")?;
            Some(empty_body)
        } else {
            None
        };

        Ok(Markup::ControlFlowFor(ControlFlowFor {
            item,
            collection,
            key,
            body,
            empty,
        }))
    }

    fn parse_when(&mut self) -> Result<Markup, String> {
        self.expect("@when {")?;

        let mut branches = Vec::new();

        loop {
            self.skip_whitespace();

            if self.peek_char() == Some('}') {
                self.advance();
                break;
            }

            if self.starts_with("else ->") {
                self.expect("else ->")?;
                self.skip_whitespace();

                let body = if self.peek_char() == Some('<') || self.peek_char() == Some('{') {
                    vec![if self.peek_char() == Some('<') {
                        self.parse_component()?
                    } else {
                        self.parse_interpolation()?
                    }]
                } else {
                    // Block
                    self.expect("{")?;
                    let body = self.parse_markup_block()?;
                    self.expect("}")?;
                    body
                };

                branches.push(WhenBranch {
                    condition: None,
                    body,
                });
            } else {
                // Regular branch
                let condition = self.read_until_arrow().trim().to_string();
                self.expect("->")?;
                self.skip_whitespace();

                let body = if self.peek_char() == Some('<') {
                    vec![self.parse_component()?]
                } else if self.peek_char() == Some('{') && !self.is_block_start() {
                    vec![self.parse_interpolation()?]
                } else {
                    vec![self.parse_component()?]
                };

                branches.push(WhenBranch {
                    condition: Some(condition),
                    body,
                });
            }

            self.skip_whitespace();
        }

        Ok(Markup::ControlFlowWhen(ControlFlowWhen { branches }))
    }

    fn parse_markup_block(&mut self) -> Result<Vec<Markup>, String> {
        let mut elements = Vec::new();

        loop {
            self.skip_whitespace();

            if self.peek_char() == Some('}') || self.pos >= self.source.len() {
                break;
            }

            if self.starts_with("@if ") {
                elements.push(self.parse_if()?);
            } else if self.starts_with("@for ") {
                elements.push(self.parse_for()?);
            } else if self.starts_with("@when ") {
                elements.push(self.parse_when()?);
            } else if self.peek_char() == Some('<') {
                elements.push(self.parse_component()?);
            } else if self.peek_char() == Some('{') {
                elements.push(self.parse_interpolation()?);
            } else {
                let text = self.read_text_until_markup();
                if !text.trim().is_empty() {
                    elements.push(Markup::Text(text));
                }
            }
        }

        Ok(elements)
    }

    // Helper methods
    fn peek_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_ahead_contains(&self, c: char) -> bool {
        self.source[self.pos..].contains(c)
    }

    fn is_block_start(&self) -> bool {
        // Check if we're at the start of a block (whitespace then newline or multiple statements)
        let remaining = &self.source[self.pos..];
        remaining.trim_start().starts_with('\n') || remaining.contains('\n')
    }

    fn advance(&mut self) {
        if self.pos < self.source.len() {
            self.pos += self.source[self.pos..].chars().next().unwrap().len_utf8();
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        self.source[self.pos..].starts_with(s)
    }

    fn expect(&mut self, s: &str) -> Result<(), String> {
        if self.starts_with(s) {
            self.pos += s.len();
            Ok(())
        } else {
            Err(format!(
                "Expected '{}' at position {}, found '{}'",
                s,
                self.pos,
                &self.source[self.pos..self.pos.min(self.pos + 20)]
            ))
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() {
            let c = self.peek_char();
            if c == Some(' ') || c == Some('\t') || c == Some('\n') || c == Some('\r') {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line(&mut self) {
        while self.pos < self.source.len() && self.peek_char() != Some('\n') {
            self.advance();
        }
        if self.peek_char() == Some('\n') {
            self.advance();
        }
    }

    fn skip_until(&mut self, c: char) {
        while self.pos < self.source.len() && self.peek_char() != Some(c) {
            self.advance();
        }
    }

    fn read_identifier(&mut self) -> Result<String, String> {
        let start = self.pos;
        while self.pos < self.source.len() {
            let c = self.peek_char().unwrap();
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        if self.pos == start {
            Err("Expected identifier".to_string())
        } else {
            Ok(self.source[start..self.pos].to_string())
        }
    }

    fn read_prop_name(&mut self) -> Result<String, String> {
        // Property names can include ':' for bind:value, etc.
        let start = self.pos;
        while self.pos < self.source.len() {
            let c = self.peek_char().unwrap();
            if c.is_alphanumeric() || c == '_' || c == ':' {
                self.advance();
            } else {
                break;
            }
        }

        if self.pos == start {
            Err("Expected property name".to_string())
        } else {
            Ok(self.source[start..self.pos].to_string())
        }
    }

    fn read_type(&mut self) -> Result<String, String> {
        let start = self.pos;
        let mut paren_depth = 0;
        let mut angle_depth = 0;

        while self.pos < self.source.len() {
            let c = self.peek_char().unwrap();

            if c == '(' {
                paren_depth += 1;
                self.advance();
            } else if c == ')' {
                if paren_depth > 0 {
                    paren_depth -= 1;
                    self.advance();
                } else {
                    break; // End of function parameter list
                }
            } else if c == '<' {
                // Check if this is the start of markup or a generic type
                // Markup starts with <UppercaseLetter (like <Text>, <Column>)
                // Generic types are like List<Type>
                let next_pos = self.pos + 1;
                if paren_depth == 0 && next_pos < self.source.len() {
                    let next_char = self.source[next_pos..].chars().next();
                    // If next char is uppercase and we're not in a type context with angle brackets,
                    // this might be markup. But we need to be careful - it could also be a generic.
                    // The key: if we haven't seen any type text yet and angle_depth == 0, it's markup.
                    // If we have seen type text (start < pos), it's a generic.
                    if next_char.map_or(false, |ch| ch.is_uppercase()) && self.pos == start {
                        // This looks like markup at the start
                        break;
                    }
                }
                angle_depth += 1;
                self.advance();
            } else if c == '>' && angle_depth > 0 {
                self.advance();
                angle_depth -= 1;
            } else if (c == ',' || c == '=' || c == '\n') && paren_depth == 0 && angle_depth == 0 {
                break;
            } else if c.is_alphanumeric()
                || c == '_'
                || c == '.'
                || c == '?'
                || c == ' '
                || (c == '>' && angle_depth == 0)  // Only consume > if we're closing angle brackets
                || (c == '-' && paren_depth > 0)  // for '->' in function types
                || (c == ':' && paren_depth > 0) // for '::' in function references
            {
                self.advance();
            } else {
                break;
            }
        }

        if self.pos == start {
            Err("Expected type".to_string())
        } else {
            Ok(self.source[start..self.pos].trim().to_string())
        }
    }

    fn read_until(&mut self, c: char) -> String {
        let start = self.pos;
        while self.pos < self.source.len() && self.peek_char() != Some(c) {
            self.advance();
        }
        self.source[start..self.pos].to_string()
    }

    fn read_until_arrow(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.source.len() {
            if self.starts_with("->") {
                break;
            }
            self.advance();
        }
        self.source[start..self.pos].to_string()
    }

    fn read_until_markup(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.source.len() {
            let c = self.peek_char().unwrap();
            if c == '<' || c == '{' || c == '@' {
                break;
            }
            self.advance();
        }
        self.source[start..self.pos].to_string()
    }

    fn read_text_until_markup(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.source.len() {
            let c = self.peek_char().unwrap();
            if c == '<' || c == '{' || c == '@' {
                break;
            }
            self.advance();
        }
        self.source[start..self.pos].to_string()
    }

    fn read_balanced_braces(&mut self) -> Result<String, String> {
        let start = self.pos;
        let mut depth = 1;

        while self.pos < self.source.len() && depth > 0 {
            let c = self.peek_char().unwrap();
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
            }
            if depth > 0 {
                self.advance();
            }
        }

        if depth != 0 {
            Err("Unbalanced braces".to_string())
        } else {
            Ok(self.source[start..self.pos].to_string())
        }
    }
}
