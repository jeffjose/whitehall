#![allow(dead_code)]
//! View backend - generates Android View code
//!
//! Used for RecyclerView optimization where we need native Views instead of Composables
//!
//! Note: This is future/experimental code not yet fully integrated.

use crate::transpiler::ast::{Component, ForLoopBlock, IfElseBlock, Markup, PropValue, WhitehallFile, WhenBlock};
use std::collections::HashSet;

pub struct ViewBackend {
    package: String,
    component_name: String,
    indent_level: usize,
    imports: HashSet<String>,
}

impl ViewBackend {
    pub fn new(package: &str, component_name: &str) -> Self {
        ViewBackend {
            package: package.to_string(),
            component_name: component_name.to_string(),
            indent_level: 0,
            imports: HashSet::new(),
        }
    }

    /// Generate Android View code
    pub fn generate(&mut self, file: &WhitehallFile) -> Result<String, String> {
        let mut output = String::new();

        // Package declaration
        output.push_str(&format!("package {}\n\n", self.package));

        // Collect imports while generating body
        let body = self.generate_component_body(file);

        // Generate imports
        self.add_base_imports();
        for import in self.imports.iter() {
            output.push_str(&format!("import {}\n", import));
        }
        output.push_str("\n");

        // Function signature - creates a View
        output.push_str(&format!("fun create{}(context: android.content.Context", self.component_name));

        // Add props as parameters
        for prop in &file.props {
            output.push_str(&format!(", {}: {}", prop.name, prop.prop_type));
        }

        output.push_str("): android.view.View {\n");

        // State declarations (as local variables)
        for state in &file.state {
            let keyword = if state.mutable { "var" } else { "val" };
            if let Some(type_ann) = &state.type_annotation {
                output.push_str(&format!("    {} {}: {} = {}\n", keyword, state.name, type_ann, state.initial_value));
            } else {
                output.push_str(&format!("    {} {} = {}\n", keyword, state.name, state.initial_value));
            }
        }

        if !file.state.is_empty() {
            output.push_str("\n");
        }

        // Body
        output.push_str(&body);

        output.push_str("}\n");

        Ok(output)
    }

    fn add_base_imports(&mut self) {
        self.imports.insert("android.content.Context".to_string());
        self.imports.insert("android.view.View".to_string());
        self.imports.insert("android.view.ViewGroup".to_string());
        self.imports.insert("android.widget.LinearLayout".to_string());
        self.imports.insert("android.widget.TextView".to_string());
        self.imports.insert("android.widget.Button".to_string());
        self.imports.insert("android.content.res.Resources".to_string());
    }

    fn generate_component_body(&mut self, file: &WhitehallFile) -> String {
        let mut code = String::new();

        code.push_str("    return ");
        self.indent_level = 1;
        code.push_str(&self.generate_markup(&file.markup));
        code.push_str("\n");

        code
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    /// Generate View code for markup
    fn generate_markup(&mut self, markup: &Markup) -> String {
        match markup {
            Markup::Component(component) => self.generate_component(component),
            Markup::Text(text) => format!("\"{}\"", text),
            Markup::Interpolation(expr) => expr.clone(),
            Markup::Sequence(items) => {
                // For sequences, wrap in a container
                if items.is_empty() {
                    return "TextView(context)".to_string();
                }

                // If single item, just generate it
                if items.len() == 1 {
                    return self.generate_markup(&items[0]);
                }

                // Multiple items: wrap in LinearLayout
                let mut code = String::new();
                code.push_str("LinearLayout(context).apply {\n");
                self.indent_level += 1;
                code.push_str(&format!("{}orientation = LinearLayout.VERTICAL\n", self.indent()));

                for item in items {
                    let child = self.generate_markup(item);
                    code.push_str(&format!("{}addView({})\n", self.indent(), child));
                }

                self.indent_level -= 1;
                code.push_str(&format!("{}}}", self.indent()));
                code
            }
            Markup::ForLoop(for_loop) => self.generate_for_loop(for_loop),
            Markup::IfElse(if_else) => self.generate_if_else(if_else),
            Markup::When(when) => self.generate_when(when),
        }
    }

    /// Generate View code for a component
    fn generate_component(&mut self, component: &Component) -> String {
        match component.name.as_str() {
            "Card" => self.generate_card(component),
            "Column" => self.generate_column(component),
            "Row" => self.generate_row(component),
            "Text" => self.generate_text(component),
            "Button" => self.generate_button(component),
            "TextField" => self.generate_textfield(component),
            "Checkbox" => self.generate_checkbox(component),
            "Image" => self.generate_image(component),
            _ => {
                // Unknown component - create a TextView with error message
                format!("TextView(context).apply {{ text = \"Unsupported: {}\" }}", component.name)
            }
        }
    }

    fn generate_card(&mut self, component: &Component) -> String {
        self.imports.insert("com.google.android.material.card.MaterialCardView".to_string());

        let mut code = String::new();
        code.push_str("MaterialCardView(context).apply {\n");
        self.indent_level += 1;

        // Handle padding prop
        if let Some(padding) = self.get_prop_value(component, "padding") {
            code.push_str(&format!("{}val paddingPx = ({}.dpToPx())\n", self.indent(), padding));
            code.push_str(&format!("{}setPadding(paddingPx, paddingPx, paddingPx, paddingPx)\n", self.indent()));
        }

        // Handle onClick
        if let Some(handler) = self.get_prop_value(component, "onClick") {
            code.push_str(&format!("{}setOnClickListener {{ {} }}\n", self.indent(), handler));
        }

        // Generate children
        if !component.children.is_empty() {
            if component.children.len() == 1 {
                let child_code = self.generate_markup(&component.children[0]);
                code.push_str(&format!("{}addView({})\n", self.indent(), child_code));
            } else {
                // Multiple children: wrap in LinearLayout
                code.push_str(&format!("{}addView(LinearLayout(context).apply {{\n", self.indent()));
                self.indent_level += 1;
                code.push_str(&format!("{}orientation = LinearLayout.VERTICAL\n", self.indent()));
                for child in &component.children {
                    let child_code = self.generate_markup(child);
                    code.push_str(&format!("{}addView({})\n", self.indent(), child_code));
                }
                self.indent_level -= 1;
                code.push_str(&format!("{}}})\n", self.indent()));
            }
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn generate_column(&mut self, component: &Component) -> String {
        let mut code = String::new();
        code.push_str("LinearLayout(context).apply {\n");
        self.indent_level += 1;

        code.push_str(&format!("{}orientation = LinearLayout.VERTICAL\n", self.indent()));

        // Handle spacing prop
        if let Some(spacing) = self.get_prop_value(component, "spacing") {
            code.push_str(&format!("{}// spacing: {}dp (using divider or margin)\n", self.indent(), spacing));
            // TODO: Implement spacing via divider or child margins
        }

        // Handle padding prop
        if let Some(padding) = self.get_prop_value(component, "padding") {
            code.push_str(&format!("{}val paddingPx = ({}.dpToPx())\n", self.indent(), padding));
            code.push_str(&format!("{}setPadding(paddingPx, paddingPx, paddingPx, paddingPx)\n", self.indent()));
        }

        // Generate children
        for child in &component.children {
            let child_code = self.generate_markup(child);
            code.push_str(&format!("{}addView({})\n", self.indent(), child_code));
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn generate_row(&mut self, component: &Component) -> String {
        let mut code = String::new();
        code.push_str("LinearLayout(context).apply {\n");
        self.indent_level += 1;

        code.push_str(&format!("{}orientation = LinearLayout.HORIZONTAL\n", self.indent()));

        // Handle spacing prop
        if let Some(spacing) = self.get_prop_value(component, "spacing") {
            code.push_str(&format!("{}// spacing: {}dp\n", self.indent(), spacing));
        }

        // Handle padding prop
        if let Some(padding) = self.get_prop_value(component, "padding") {
            code.push_str(&format!("{}val paddingPx = ({}.dpToPx())\n", self.indent(), padding));
            code.push_str(&format!("{}setPadding(paddingPx, paddingPx, paddingPx, paddingPx)\n", self.indent()));
        }

        // Generate children
        for child in &component.children {
            let child_code = self.generate_markup(child);
            code.push_str(&format!("{}addView({})\n", self.indent(), child_code));
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn generate_text(&mut self, component: &Component) -> String {
        let mut code = String::new();
        code.push_str("TextView(context).apply {\n");
        self.indent_level += 1;

        // Handle text content from children
        if !component.children.is_empty() {
            let text_content = self.generate_markup(&component.children[0]);
            code.push_str(&format!("{}text = {}\n", self.indent(), text_content));
        }

        // Handle fontSize prop
        if let Some(size) = self.get_prop_value(component, "fontSize") {
            code.push_str(&format!("{}textSize = {}f\n", self.indent(), size));
        }

        // Handle fontWeight prop
        if let Some(weight) = self.get_prop_value(component, "fontWeight") {
            if weight == "\"bold\"" || weight == "bold" {
                code.push_str(&format!("{}setTypeface(null, android.graphics.Typeface.BOLD)\n", self.indent()));
            }
        }

        // Handle color prop
        if let Some(color) = self.get_prop_value(component, "color") {
            if color == "\"secondary\"" || color == "secondary" {
                code.push_str(&format!("{}// TODO: setTextColor(secondary color)\n", self.indent()));
            } else if color == "\"primary\"" || color == "primary" {
                code.push_str(&format!("{}// TODO: setTextColor(primary color)\n", self.indent()));
            }
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn generate_button(&mut self, component: &Component) -> String {
        let mut code = String::new();
        code.push_str("Button(context).apply {\n");
        self.indent_level += 1;

        // Handle text from children
        if !component.children.is_empty() {
            let text_content = self.generate_markup(&component.children[0]);
            code.push_str(&format!("{}text = {}\n", self.indent(), text_content));
        }

        // Handle onClick prop
        if let Some(handler) = self.get_prop_value(component, "onClick") {
            code.push_str(&format!("{}setOnClickListener {{ {} }}\n", self.indent(), handler));
        }

        // Handle enabled/disabled prop
        if let Some(enabled) = self.get_prop_value(component, "enabled") {
            code.push_str(&format!("{}isEnabled = {}\n", self.indent(), enabled));
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn generate_textfield(&mut self, component: &Component) -> String {
        self.imports.insert("android.widget.EditText".to_string());

        let mut code = String::new();
        code.push_str("EditText(context).apply {\n");
        self.indent_level += 1;

        // Handle value binding
        if let Some(value) = self.get_prop_value(component, "bind:value") {
            code.push_str(&format!("{}setText({})\n", self.indent(), value));
        }

        // Handle placeholder (hint)
        if let Some(placeholder) = self.get_prop_value(component, "placeholder") {
            code.push_str(&format!("{}hint = {}\n", self.indent(), placeholder));
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn generate_checkbox(&mut self, component: &Component) -> String {
        self.imports.insert("android.widget.CheckBox".to_string());

        let mut code = String::new();
        code.push_str("CheckBox(context).apply {\n");
        self.indent_level += 1;

        // Handle checked binding
        if let Some(checked) = self.get_prop_value(component, "bind:checked") {
            code.push_str(&format!("{}isChecked = {}\n", self.indent(), checked));
        }

        // Handle onChange
        if let Some(handler) = self.get_prop_value(component, "onChange") {
            code.push_str(&format!("{}setOnCheckedChangeListener {{ _, isChecked -> {} }}\n", self.indent(), handler));
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn generate_image(&mut self, component: &Component) -> String {
        self.imports.insert("android.widget.ImageView".to_string());

        let mut code = String::new();
        code.push_str("ImageView(context).apply {\n");
        self.indent_level += 1;

        // Handle src prop
        if let Some(src) = self.get_prop_value(component, "src") {
            code.push_str(&format!("{}// TODO: load image from {}\n", self.indent(), src));
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn generate_for_loop(&mut self, for_loop: &ForLoopBlock) -> String {
        // For loops in View backend: create children dynamically
        let mut code = String::new();

        code.push_str("LinearLayout(context).apply {\n");
        self.indent_level += 1;
        code.push_str(&format!("{}orientation = LinearLayout.VERTICAL\n", self.indent()));

        code.push_str(&format!("{}{}.forEach {{ {} ->\n", self.indent(), for_loop.collection, for_loop.item));
        self.indent_level += 1;

        // Generate loop body
        for child in &for_loop.body {
            let child_code = self.generate_markup(child);
            code.push_str(&format!("{}addView({})\n", self.indent(), child_code));
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}\n", self.indent()));

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn generate_if_else(&mut self, if_else: &IfElseBlock) -> String {
        // For if/else in View backend: use conditional view creation
        let mut code = String::new();

        code.push_str(&format!("if ({}) {{\n", if_else.condition));
        self.indent_level += 1;

        if if_else.then_branch.len() == 1 {
            code.push_str(&format!("{}{}\n", self.indent(), self.generate_markup(&if_else.then_branch[0])));
        } else {
            code.push_str(&format!("{}LinearLayout(context).apply {{\n", self.indent()));
            self.indent_level += 1;
            for child in &if_else.then_branch {
                let child_code = self.generate_markup(child);
                code.push_str(&format!("{}addView({})\n", self.indent(), child_code));
            }
            self.indent_level -= 1;
            code.push_str(&format!("{}}}\n", self.indent()));
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}} else {{\n", self.indent()));
        self.indent_level += 1;

        if let Some(else_branch) = &if_else.else_branch {
            if else_branch.len() == 1 {
                code.push_str(&format!("{}{}\n", self.indent(), self.generate_markup(&else_branch[0])));
            } else {
                code.push_str(&format!("{}LinearLayout(context).apply {{\n", self.indent()));
                self.indent_level += 1;
                for child in else_branch {
                    let child_code = self.generate_markup(child);
                    code.push_str(&format!("{}addView({})\n", self.indent(), child_code));
                }
                self.indent_level -= 1;
                code.push_str(&format!("{}}}\n", self.indent()));
            }
        } else {
            code.push_str(&format!("{}TextView(context) // empty else\n", self.indent()));
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn generate_when(&mut self, when: &WhenBlock) -> String {
        // For when expressions: generate when block
        let mut code = String::new();

        code.push_str("when {\n");
        self.indent_level += 1;

        for branch in &when.branches {
            if let Some(condition) = &branch.condition {
                code.push_str(&format!("{}{} -> {}\n", self.indent(), condition, self.generate_markup(&branch.body)));
            } else {
                code.push_str(&format!("{}else -> {}\n", self.indent(), self.generate_markup(&branch.body)));
            }
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn get_prop_value(&self, component: &Component, prop_name: &str) -> Option<String> {
        component.props.iter()
            .find(|p| p.name == prop_name)
            .map(|p| match &p.value {
                PropValue::Expression(expr) => expr.clone(),
                _ => String::new(),
            })
    }
}

// Extension trait for DP to PX conversion (will be generated in output)
// This is a helper that will be included in the generated code

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transpiler::ast::{Component, PropDeclaration};

    #[test]
    fn test_view_backend_generates_function() {
        let mut backend = ViewBackend::new("com.example.app", "TestComponent");
        let file = WhitehallFile {
            props: vec![],
            state: vec![],
            functions: vec![],
            lifecycle_hooks: vec![],
            imports: vec![],
            classes: vec![],
            markup: Markup::Component(Component {
                name: "Text".to_string(),
                props: vec![],
                children: vec![Markup::Text("Hello".to_string())],
                self_closing: false,
            }),
        };

        let output = backend.generate(&file).unwrap();

        assert!(output.contains("package com.example.app"));
        assert!(output.contains("fun createTestComponent(context: android.content.Context): android.view.View"));
        assert!(output.contains("TextView(context)"));
        assert!(output.contains("text = \"Hello\""));
    }

    #[test]
    fn test_generate_text_component() {
        let mut backend = ViewBackend::new("com.example.app", "TestComponent");

        let component = Component {
            name: "Text".to_string(),
            props: vec![],
            children: vec![Markup::Text("Hello".to_string())],
            self_closing: false,
        };

        let output = backend.generate_component(&component);
        assert!(output.contains("TextView(context)"));
        assert!(output.contains("text = \"Hello\""));
    }

    #[test]
    fn test_generate_column_with_children() {
        let mut backend = ViewBackend::new("com.example.app", "TestComponent");

        let component = Component {
            name: "Column".to_string(),
            props: vec![],
            children: vec![
                Markup::Component(Component {
                    name: "Text".to_string(),
                    props: vec![],
                    children: vec![Markup::Text("First".to_string())],
                    self_closing: false,
                }),
                Markup::Component(Component {
                    name: "Text".to_string(),
                    props: vec![],
                    children: vec![Markup::Text("Second".to_string())],
                    self_closing: false,
                }),
            ],
            self_closing: false,
        };

        let output = backend.generate_component(&component);
        assert!(output.contains("LinearLayout(context)"));
        assert!(output.contains("orientation = LinearLayout.VERTICAL"));
        assert!(output.contains("addView"));
    }

    #[test]
    fn test_generate_with_props() {
        let mut backend = ViewBackend::new("com.example.app", "TestComponent");

        let file = WhitehallFile {
            props: vec![
                PropDeclaration {
                    name: "title".to_string(),
                    prop_type: "String".to_string(),
                    default_value: None,
                },
            ],
            state: vec![],
            functions: vec![],
            lifecycle_hooks: vec![],
            imports: vec![],
            classes: vec![],
            markup: Markup::Component(Component {
                name: "Text".to_string(),
                props: vec![],
                children: vec![Markup::Interpolation("title".to_string())],
                self_closing: false,
            }),
        };

        let output = backend.generate(&file).unwrap();

        assert!(output.contains("fun createTestComponent(context: android.content.Context, title: String)"));
        assert!(output.contains("text = title"));
    }
}
