/// Parser for Whitehall syntax

use crate::transpiler::ast::{
    ClassDeclaration, Component, ComponentProp, ConstructorDeclaration, ElseIfBranch,
    ForLoopBlock, FunctionDeclaration, IfElseBlock, Import, LifecycleHook, Markup,
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
        format!("[Line {}:{}] {}", line, col, message)
    }

    pub fn parse(&mut self) -> Result<WhitehallFile, String> {
        let mut imports = Vec::new();
        let mut props = Vec::new();
        let mut state = Vec::new();
        let mut functions = Vec::new();
        let mut lifecycle_hooks = Vec::new();
        let mut classes = Vec::new();
        let mut pending_annotations = Vec::new();

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
            } else if self.peek_word() == Some("class") || self.peek_word() == Some("object") {
                // Standalone class/object without annotation (e.g., class with var properties)
                classes.push(self.parse_class_declaration(Vec::new())?);
            } else if self.consume_word("import") {
                imports.push(self.parse_import()?);
            } else if self.peek_word() == Some("var") || self.peek_word() == Some("val") {
                state.push(self.parse_state_declaration()?);
            } else if self.consume_word("fun") {
                functions.push(self.parse_function_declaration(false)?);
            } else if self.consume_word("onMount") {
                lifecycle_hooks.push(self.parse_lifecycle_hook("onMount")?);
            } else if self.consume_word("onDispose") {
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

        Ok(WhitehallFile {
            imports,
            props,
            state,
            functions,
            lifecycle_hooks,
            classes,
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

        Ok(FunctionDeclaration {
            name,
            params,
            return_type,
            body: body.trim().to_string(),
            is_suspend,
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

            // Check for property (var/val)
            if self.peek_word() == Some("var") || self.peek_word() == Some("val") {
                properties.push(self.parse_property_declaration()?);
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

    fn parse_property_declaration(&mut self) -> Result<PropertyDeclaration, String> {
        // Parse: var name: Type = value or val name = value or val name get() = expression
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
        if self.peek_char() == Some('"') {
            self.parse_string()
        } else if self.peek_char() == Some('[') {
            // Array literal syntax: [1, 2, 3] -> will be converted to listOf() later
            self.parse_array_literal()
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
            Ok(self.input[start..self.pos].trim().to_string())
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
        } else if remaining.starts_with("onMount") {
            Some("onMount")
        } else if remaining.starts_with("onDispose") {
            Some("onDispose")
        } else {
            None
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
