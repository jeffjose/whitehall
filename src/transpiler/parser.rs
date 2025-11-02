/// Parser for Whitehall syntax

use crate::transpiler::ast::{
    Component, ComponentProp, ElseIfBranch, IfElseBlock, Import, Markup, PropDeclaration,
    StateDeclaration, WhitehallFile,
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

        // Parse imports, props, and state declarations (before markup)
        loop {
            self.skip_whitespace();
            if self.consume_word("import") {
                imports.push(self.parse_import()?);
            } else if self.consume_word("@prop") {
                props.push(self.parse_prop_declaration()?);
            } else if self.peek_word() == Some("var") || self.peek_word() == Some("val") {
                state.push(self.parse_state_declaration()?);
            } else {
                break;
            }
        }

        let markup = self.parse_markup()?;
        Ok(WhitehallFile {
            imports,
            props,
            state,
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
                '=' | '\n' if paren_depth == 0 && angle_depth == 0 && bracket_depth == 0 => break,
                _ => self.pos += 1,
            }
        }

        Ok(self.input[start..self.pos].trim().to_string())
    }

    fn parse_state_declaration(&mut self) -> Result<StateDeclaration, String> {
        // Parse: var name = "value" or val name = value
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
        self.expect_char('=')?;
        self.skip_whitespace();

        // Parse initial value (simple string or expression for now)
        let initial_value = self.parse_value()?;

        Ok(StateDeclaration {
            name,
            mutable,
            initial_value,
        })
    }

    fn parse_value(&mut self) -> Result<String, String> {
        self.skip_whitespace();
        if self.peek_char() == Some('"') {
            self.parse_string()
        } else {
            // Parse until newline or EOF
            let start = self.pos;
            while let Some(ch) = self.peek_char() {
                if ch == '\n' {
                    break;
                }
                self.pos += 1;
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
        // Parse: propName={expression}
        let name = self.parse_identifier()?;
        self.skip_whitespace();
        self.expect_char('=')?;
        self.skip_whitespace();
        self.expect_char('{')?;

        // Parse expression until closing brace (handle nested braces)
        let mut value = String::new();
        let mut depth = 1;

        while let Some(ch) = self.peek_char() {
            if ch == '{' {
                depth += 1;
                value.push(ch);
                self.pos += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    self.pos += 1;
                    break;
                }
                value.push(ch);
                self.pos += 1;
            } else {
                value.push(ch);
                self.pos += 1;
            }
        }

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
        while let Some(ch) = self.peek_char() {
            if ch == delimiter {
                break;
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
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.pos += 1;
            } else {
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
