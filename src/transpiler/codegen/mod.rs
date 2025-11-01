/// Code generator for Kotlin/Compose output
use crate::transpiler::ast::*;
use std::collections::HashSet;

pub struct CodeGenerator {
    package_name: String,
    component_name: String,
    indent_level: usize,
    required_imports: HashSet<String>,
}

impl CodeGenerator {
    pub fn new(package_name: &str, component_name: &str) -> Self {
        CodeGenerator {
            package_name: package_name.to_string(),
            component_name: component_name.to_string(),
            indent_level: 0,
            required_imports: HashSet::new(),
        }
    }

    pub fn generate(&mut self, file: &WhitehallFile) -> Result<String, String> {
        let mut output = String::new();

        // Package declaration
        output.push_str(&format!("package {}\n\n", self.package_name));

        // Collect required imports based on AST
        self.collect_imports(file);

        // Add imports
        output.push_str(&self.generate_imports(file));
        output.push('\n');

        // Component function
        output.push_str("@Composable\n");
        output.push_str(&format!("fun {}(", self.component_name));

        // Props as function parameters
        if !file.props.is_empty() {
            output.push('\n');
            for (i, prop) in file.props.iter().enumerate() {
                output.push_str(&format!("    {}: {}", prop.name, prop.prop_type));
                if let Some(default) = &prop.default_value {
                    output.push_str(&format!(" = {}", default));
                }
                if i < file.props.len() - 1 {
                    output.push(',');
                }
                output.push('\n');
            }
        }

        output.push_str(") {\n");
        self.indent_level += 1;

        // State declarations
        for state in &file.state {
            output.push_str(&self.indent());
            if state.mutable {
                output.push_str(&format!("var {} by remember {{ mutableStateOf", state.name));
                if let Some(ref t) = state.state_type {
                    output.push_str(&format!("<{}>", t));
                }
                output.push_str(&format!("({}) }}\n", state.initial_value));
            } else {
                output.push_str(&format!("val {}", state.name));
                if let Some(ref t) = state.state_type {
                    output.push_str(&format!(": {}", t));
                }
                output.push_str(&format!(" = {}\n", state.initial_value));
            }
        }

        // Derived state
        for derived in &file.derived_state {
            output.push_str(&self.indent());
            output.push_str(&format!("val {}", derived.name));
            if let Some(ref t) = derived.state_type {
                output.push_str(&format!(": {}", t));
            }
            output.push_str(&format!(" = {}\n", derived.expression));
        }

        if !file.state.is_empty() || !file.derived_state.is_empty() {
            output.push('\n');
        }

        // Functions
        for func in &file.functions {
            output.push_str(&self.indent());
            output.push_str(&format!("fun {}(", func.name));

            for (i, param) in func.params.iter().enumerate() {
                output.push_str(&format!("{}: {}", param.name, param.param_type));
                if i < func.params.len() - 1 {
                    output.push_str(", ");
                }
            }

            output.push(')');

            if let Some(ref ret) = func.return_type {
                output.push_str(&format!(": {}", ret));
            }

            output.push_str(" {\n");
            self.indent_level += 1;
            output.push_str(&self.indent());
            output.push_str(&func.body);
            output.push('\n');
            self.indent_level -= 1;
            output.push_str(&self.indent());
            output.push_str("}\n\n");
        }

        // Lifecycle hooks
        if !file.lifecycle.is_empty() {
            // Need coroutineScope if we have onMount with launch
            let has_launch = file.lifecycle.iter().any(|hook| match hook {
                LifecycleHook::OnMount { body } => body.contains("launch"),
            });

            if has_launch {
                output.push_str(&self.indent());
                output.push_str("val coroutineScope = rememberCoroutineScope()\n\n");
            }

            for hook in &file.lifecycle {
                match hook {
                    LifecycleHook::OnMount { body } => {
                        output.push_str(&self.indent());
                        output.push_str("LaunchedEffect(Unit) {\n");
                        self.indent_level += 1;
                        output.push_str(&self.indent());
                        output.push_str("coroutineScope.");
                        output.push_str(body);
                        output.push('\n');
                        self.indent_level -= 1;
                        output.push_str(&self.indent());
                        output.push_str("}\n\n");
                    }
                }
            }
        }

        // Markup/UI
        output.push_str(&self.generate_markup(&file.markup)?);

        self.indent_level -= 1;
        output.push_str("}\n");

        Ok(output)
    }

    fn collect_imports(&mut self, file: &WhitehallFile) {
        // Always need Composable
        self.required_imports.insert("androidx.compose.runtime.Composable".to_string());

        // State management
        if !file.state.is_empty() {
            self.required_imports.insert("androidx.compose.runtime.*".to_string());
        }

        // Lifecycle
        if !file.lifecycle.is_empty() {
            self.required_imports.insert("androidx.compose.runtime.LaunchedEffect".to_string());
            self.required_imports.insert("androidx.compose.runtime.rememberCoroutineScope".to_string());
            self.required_imports.insert("kotlinx.coroutines.launch".to_string());
        }

        // Analyze markup for component imports
        self.collect_markup_imports(&file.markup);
    }

    fn collect_markup_imports(&mut self, markup: &Markup) {
        match markup {
            Markup::Component(comp) => {
                match comp.name.as_str() {
                    "Column" | "Row" => {
                        self.required_imports.insert(format!("androidx.compose.foundation.layout.{}", comp.name));
                        self.required_imports.insert("androidx.compose.foundation.layout.Arrangement".to_string());
                        self.required_imports.insert("androidx.compose.ui.unit.dp".to_string());

                        // Check for padding/spacing props
                        for prop in &comp.props {
                            if prop.name == "padding" || prop.name == "spacing" {
                                self.required_imports.insert("androidx.compose.ui.Modifier".to_string());
                                self.required_imports.insert("androidx.compose.foundation.layout.padding".to_string());
                            }
                        }
                    }
                    "Text" => {
                        self.required_imports.insert("androidx.compose.material3.Text".to_string());

                        // Check for fontSize
                        for prop in &comp.props {
                            if prop.name == "fontSize" {
                                self.required_imports.insert("androidx.compose.ui.unit.sp".to_string());
                            }
                            if prop.name == "fontWeight" {
                                self.required_imports.insert("androidx.compose.ui.text.font.FontWeight".to_string());
                            }
                            if prop.name == "color" {
                                self.required_imports.insert("androidx.compose.material3.MaterialTheme".to_string());
                            }
                        }
                    }
                    "Button" => {
                        self.required_imports.insert("androidx.compose.material3.Button".to_string());
                    }
                    "Card" => {
                        self.required_imports.insert("androidx.compose.material3.Card".to_string());
                    }
                    "TextField" => {
                        self.required_imports.insert("androidx.compose.material3.TextField".to_string());
                    }
                    "Scaffold" => {
                        self.required_imports.insert("androidx.compose.material3.Scaffold".to_string());
                    }
                    "TopAppBar" => {
                        self.required_imports.insert("androidx.compose.material3.TopAppBar".to_string());
                    }
                    _ => {}
                }

                for child in &comp.children {
                    self.collect_markup_imports(child);
                }
            }
            Markup::Sequence(elements) => {
                for elem in elements {
                    self.collect_markup_imports(elem);
                }
            }
            Markup::ControlFlowIf(ctrl) => {
                for elem in &ctrl.then_branch {
                    self.collect_markup_imports(elem);
                }
                for (_, elems) in &ctrl.else_if_branches {
                    for elem in elems {
                        self.collect_markup_imports(elem);
                    }
                }
                if let Some(else_elems) = &ctrl.else_branch {
                    for elem in else_elems {
                        self.collect_markup_imports(elem);
                    }
                }
            }
            Markup::ControlFlowFor(ctrl) => {
                for elem in &ctrl.body {
                    self.collect_markup_imports(elem);
                }
                if let Some(empty_elems) = &ctrl.empty {
                    for elem in empty_elems {
                        self.collect_markup_imports(elem);
                    }
                }
            }
            Markup::ControlFlowWhen(ctrl) => {
                for branch in &ctrl.branches {
                    for elem in &branch.body {
                        self.collect_markup_imports(elem);
                    }
                }
            }
            _ => {}
        }
    }

    fn generate_imports(&self, file: &WhitehallFile) -> String {
        let mut all_imports = HashSet::new();

        // User imports
        for import in &file.imports {
            if import.path.starts_with('$') {
                // Resolve $ aliases (would need config, for now keep as-is)
                all_imports.insert(import.path.replace('$', "com.example.app."));
            } else {
                all_imports.insert(import.path.clone());
            }
        }

        // Required imports
        for import in &self.required_imports {
            all_imports.insert(import.clone());
        }

        // Sort all imports
        let mut sorted_imports: Vec<String> = all_imports.into_iter().collect();
        sorted_imports.sort();

        sorted_imports.iter()
            .map(|import| format!("import {}", import))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_markup(&mut self, markup: &Markup) -> Result<String, String> {
        match markup {
            Markup::Component(comp) => self.generate_component(comp),
            Markup::Text(text) => Ok(format!("{}Text(text = \"{}\")\n", self.indent(), text)),
            Markup::Interpolation(expr) => {
                Ok(format!("{}Text(text = ${})\n", self.indent(), expr))
            }
            Markup::Sequence(elements) => {
                let mut output = String::new();
                for elem in elements {
                    output.push_str(&self.generate_markup(elem)?);
                }
                Ok(output)
            }
            Markup::ControlFlowIf(ctrl) => self.generate_if(ctrl),
            Markup::ControlFlowFor(ctrl) => self.generate_for(ctrl),
            Markup::ControlFlowWhen(ctrl) => self.generate_when(ctrl),
        }
    }

    fn generate_component(&mut self, comp: &Component) -> Result<String, String> {
        let mut output = String::new();
        output.push_str(&self.indent());

        // Special handling for Column/Row with spacing
        if comp.name == "Column" || comp.name == "Row" {
            output.push_str(&comp.name);
            output.push('(');

            let mut has_props = false;

            // Handle modifier and arrangement
            let spacing_prop = comp.props.iter().find(|p| p.name == "spacing");
            let padding_prop = comp.props.iter().find(|p| p.name == "padding");

            if let Some(spacing) = spacing_prop {
                output.push('\n');
                self.indent_level += 1;
                output.push_str(&self.indent());

                if comp.name == "Column" {
                    output.push_str("verticalArrangement = Arrangement.spacedBy(");
                } else {
                    output.push_str("horizontalArrangement = Arrangement.spacedBy(");
                }

                match &spacing.value {
                    PropValue::Expression(expr) => output.push_str(&format!("{}.dp)", expr)),
                    PropValue::String(s) => output.push_str(&format!("{}.dp)", s)),
                    _ => output.push_str("0.dp)"),
                }

                has_props = true;
            }

            if padding_prop.is_some() {
                if has_props {
                    output.push_str(",\n");
                    output.push_str(&self.indent());
                } else {
                    output.push('\n');
                    self.indent_level += 1;
                    output.push_str(&self.indent());
                }
                // Padding will be handled in modifier below
                has_props = true;
            }

            // Other props
            for prop in &comp.props {
                if prop.name != "spacing" && prop.name != "padding" {
                    if has_props {
                        output.push_str(",\n");
                        output.push_str(&self.indent());
                    } else {
                        output.push('\n');
                        self.indent_level += 1;
                        output.push_str(&self.indent());
                        has_props = true;
                    }

                    output.push_str(&self.generate_prop(prop)?);
                }
            }

            if has_props {
                output.push('\n');
                self.indent_level -= 1;
                output.push_str(&self.indent());
            }

            output.push_str(") {\n");
            self.indent_level += 1;

            for child in &comp.children {
                output.push_str(&self.generate_markup(child)?);
            }

            self.indent_level -= 1;
            output.push_str(&self.indent());
            output.push_str("}\n");

            return Ok(output);
        }

        // Regular component
        output.push_str(&comp.name);
        output.push('(');

        let mut has_props = false;

        // Special handling for Text component with text/interpolation content
        if comp.name == "Text" && !comp.children.is_empty() {
            let text_value = self.build_text_from_children(&comp.children);
            if !text_value.is_empty() {
                output.push_str(&format!("text = {}", text_value));
                has_props = true;
            }
        }

        // Add other props
        for prop in &comp.props {
            if has_props {
                output.push_str(", ");
            }
            output.push_str(&self.generate_prop(prop)?);
            has_props = true;
        }

        output.push(')');

        // Children as trailing lambda if component supports it
        if !comp.children.is_empty()
            && self.supports_trailing_lambda(&comp.name)
            && !(comp.name == "Text" && comp.children.len() == 1 && matches!(&comp.children[0], Markup::Text(_) | Markup::Interpolation(_)))
        {
            output.push_str(" {\n");
            self.indent_level += 1;

            for child in &comp.children {
                output.push_str(&self.generate_markup(child)?);
            }

            self.indent_level -= 1;
            output.push_str(&self.indent());
            output.push('}');
        }

        output.push('\n');

        Ok(output)
    }

    fn generate_prop(&self, prop: &Prop) -> Result<String, String> {
        let prop_name = self.map_prop_name(&prop.name);

        match &prop.value {
            PropValue::String(s) => Ok(format!("{} = \"{}\"", prop_name, s)),
            PropValue::Expression(expr) => {
                // Special handling for certain props
                if prop.name == "fontSize" {
                    Ok(format!("{} = {}.sp", prop_name, expr))
                } else if prop.name == "fontWeight" {
                    Ok(format!("{} = FontWeight.{}", prop_name, expr))
                } else if prop.name == "color" && expr == "\"secondary\"" {
                    Ok(format!("{} = MaterialTheme.colorScheme.secondary", prop_name))
                } else if prop.name == "onClick" {
                    Ok(format!("{} = {}", prop_name, self.convert_arrow_function(expr)))
                } else {
                    Ok(format!("{} = {}", prop_name, expr))
                }
            }
            PropValue::Binding(expr) => {
                // bind:value={username} becomes:
                // value = username, onValueChange = { username = it }
                Ok(format!(
                    "value = {}, onValueChange = {{ {} = it }}",
                    expr, expr
                ))
            }
        }
    }

    fn map_prop_name(&self, name: &str) -> String {
        match name {
            "bind:value" => "value".to_string(),
            _ => name.to_string(),
        }
    }

    fn convert_arrow_function(&self, expr: &str) -> String {
        // Convert () => foo() to { foo() }
        if expr.starts_with("() =>") {
            format!("{{ {} }}", expr[5..].trim())
        } else {
            expr.to_string()
        }
    }

    fn supports_trailing_lambda(&self, component: &str) -> bool {
        matches!(component, "Button" | "Card" | "Scaffold")
    }

    fn generate_if(&mut self, ctrl: &ControlFlowIf) -> Result<String, String> {
        let mut output = String::new();

        output.push_str(&self.indent());
        output.push_str(&format!("if ({}) {{\n", ctrl.condition));
        self.indent_level += 1;

        for elem in &ctrl.then_branch {
            output.push_str(&self.generate_markup(elem)?);
        }

        self.indent_level -= 1;
        output.push_str(&self.indent());
        output.push('}');

        for (condition, body) in &ctrl.else_if_branches {
            output.push_str(&format!(" else if ({}) {{\n", condition));
            self.indent_level += 1;

            for elem in body {
                output.push_str(&self.generate_markup(elem)?);
            }

            self.indent_level -= 1;
            output.push_str(&self.indent());
            output.push('}');
        }

        if let Some(else_body) = &ctrl.else_branch {
            output.push_str(" else {\n");
            self.indent_level += 1;

            for elem in else_body {
                output.push_str(&self.generate_markup(elem)?);
            }

            self.indent_level -= 1;
            output.push_str(&self.indent());
            output.push('}');
        }

        output.push('\n');

        Ok(output)
    }

    fn generate_for(&mut self, ctrl: &ControlFlowFor) -> Result<String, String> {
        let mut output = String::new();

        // Check if we have an empty block
        if ctrl.empty.is_some() {
            output.push_str(&self.indent());
            output.push_str(&format!("if ({}.isEmpty()) {{\n", ctrl.collection));
            self.indent_level += 1;

            for elem in ctrl.empty.as_ref().unwrap() {
                output.push_str(&self.generate_markup(elem)?);
            }

            self.indent_level -= 1;
            output.push_str(&self.indent());
            output.push_str("} else {\n");
            self.indent_level += 1;
        }

        output.push_str(&self.indent());
        output.push_str(&format!("{}.forEach {{ {} ->\n", ctrl.collection, ctrl.item));
        self.indent_level += 1;

        // Add key if specified
        if let Some(key_lambda) = &ctrl.key {
            output.push_str(&self.indent());
            // Extract key expression from lambda
            let key_expr = key_lambda.trim_start_matches('{').trim_end_matches('}').trim();
            output.push_str(&format!("key({}) {{\n", key_expr));
            self.indent_level += 1;
        }

        for elem in &ctrl.body {
            output.push_str(&self.generate_markup(elem)?);
        }

        if ctrl.key.is_some() {
            self.indent_level -= 1;
            output.push_str(&self.indent());
            output.push_str("}\n");
        }

        self.indent_level -= 1;
        output.push_str(&self.indent());
        output.push_str("}\n");

        if ctrl.empty.is_some() {
            self.indent_level -= 1;
            output.push_str(&self.indent());
            output.push_str("}\n");
        }

        Ok(output)
    }

    fn generate_when(&mut self, ctrl: &ControlFlowWhen) -> Result<String, String> {
        let mut output = String::new();

        output.push_str(&self.indent());
        output.push_str("when {\n");
        self.indent_level += 1;

        for branch in &ctrl.branches {
            output.push_str(&self.indent());

            if let Some(condition) = &branch.condition {
                output.push_str(&format!("{} -> ", condition));
            } else {
                output.push_str("else -> ");
            }

            if branch.body.len() == 1 {
                // Single expression
                match &branch.body[0] {
                    Markup::Component(comp) => {
                        output.push_str(&format!("{}()\n", comp.name));
                    }
                    _ => {
                        output.push_str("{\n");
                        self.indent_level += 1;
                        for elem in &branch.body {
                            output.push_str(&self.generate_markup(elem)?);
                        }
                        self.indent_level -= 1;
                        output.push_str(&self.indent());
                        output.push_str("}\n");
                    }
                }
            } else {
                output.push_str("{\n");
                self.indent_level += 1;

                for elem in &branch.body {
                    output.push_str(&self.generate_markup(elem)?);
                }

                self.indent_level -= 1;
                output.push_str(&self.indent());
                output.push_str("}\n");
            }
        }

        self.indent_level -= 1;
        output.push_str(&self.indent());
        output.push_str("}\n");

        Ok(output)
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    /// Build a text string expression from mixed text and interpolation children
    fn build_text_from_children(&self, children: &[Markup]) -> String {
        if children.is_empty() {
            return String::new();
        }

        // If single child, handle directly
        if children.len() == 1 {
            return match &children[0] {
                Markup::Text(text) => format!("\"{}\"", text),
                Markup::Interpolation(expr) => format!("${}", expr),
                _ => String::new(),
            };
        }

        // Multiple children - build string template
        let mut parts = Vec::new();
        for child in children {
            match child {
                Markup::Text(text) => parts.push(text.to_string()),
                Markup::Interpolation(expr) => parts.push(format!("${{{}}}", expr)),
                _ => {}
            }
        }

        format!("\"{}\"", parts.join(""))
    }
}
