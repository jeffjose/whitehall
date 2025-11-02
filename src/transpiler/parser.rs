/// Parser for Whitehall syntax

use crate::transpiler::ast::{Component, Markup, StateDeclaration, WhitehallFile};

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
        let mut state = Vec::new();

        // Parse state declarations (before markup)
        while self.peek_word() == Some("var") || self.peek_word() == Some("val") {
            state.push(self.parse_state_declaration()?);
            self.skip_whitespace();
        }

        let markup = self.parse_markup()?;
        Ok(WhitehallFile { state, markup })
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
            Err("Expected component".to_string())
        }
    }

    fn parse_component(&mut self) -> Result<Markup, String> {
        // Parse opening tag: <ComponentName>
        self.expect_char('<')?;
        let name = self.parse_identifier()?;
        self.expect_char('>')?;

        // Parse children (text with potential interpolation)
        let children = self.parse_text_with_interpolation_until('<')?;

        // Parse closing tag: </ComponentName>
        self.expect_char('<')?;
        self.expect_char('/')?;
        let closing_name = self.parse_identifier()?;
        self.expect_char('>')?;

        if name != closing_name {
            return Err(format!(
                "Mismatched tags: opening <{}> vs closing </{}>",
                name, closing_name
            ));
        }

        Ok(Markup::Component(Component { name, children }))
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

    fn peek_word(&self) -> Option<&str> {
        let remaining = &self.input[self.pos..];
        if remaining.starts_with("var ") || remaining.starts_with("var\n") {
            Some("var")
        } else if remaining.starts_with("val ") || remaining.starts_with("val\n") {
            Some("val")
        } else {
            None
        }
    }

    fn consume_word(&mut self, word: &str) -> bool {
        let remaining = &self.input[self.pos..];
        if remaining.starts_with(word) {
            let next_pos = self.pos + word.len();
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
