/// Parser for Whitehall syntax

use crate::transpiler::ast::{Component, Markup, WhitehallFile};

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
        let markup = self.parse_markup()?;
        Ok(WhitehallFile { markup })
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

        // Parse text content (simple case for test 00)
        let text_content = self.parse_text_until('<')?;

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

        Ok(Markup::Component(Component {
            name,
            children: vec![Markup::Text(text_content)],
        }))
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
