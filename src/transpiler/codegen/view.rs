/// View backend - generates Android View code
///
/// Used for RecyclerView optimization where we need native Views instead of Composables

use crate::transpiler::ast::{Component, Markup, PropValue, WhitehallFile};

pub struct ViewBackend {
    package: String,
    component_name: String,
    indent_level: usize,
}

impl ViewBackend {
    pub fn new(package: &str, component_name: &str) -> Self {
        ViewBackend {
            package: package.to_string(),
            component_name: component_name.to_string(),
            indent_level: 0,
        }
    }

    /// Generate Android View code (Phase 0.5: stub for now)
    pub fn generate(&mut self, _file: &WhitehallFile) -> Result<String, String> {
        // TODO: Implement View generation
        Err("View backend not yet implemented".to_string())
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    /// Generate View code for a component
    #[allow(dead_code)]
    fn generate_component(&mut self, component: &Component) -> String {
        match component.name.as_str() {
            "Card" => self.generate_card(component),
            "Column" => self.generate_column(component),
            "Row" => self.generate_row(component),
            "Text" => self.generate_text(component),
            "Button" => self.generate_button(component),
            _ => format!("// Unsupported View component: {}", component.name),
        }
    }

    fn generate_card(&mut self, component: &Component) -> String {
        let mut code = String::new();
        let indent = self.indent();

        code.push_str(&format!("{}MaterialCardView(context).apply {{\n", indent));
        self.indent_level += 1;

        // Handle padding prop
        if let Some(padding) = self.get_prop_value(component, "padding") {
            let indent = self.indent();
            code.push_str(&format!("{}val paddingPx = ({} * resources.displayMetrics.density).toInt()\n", indent, padding));
            code.push_str(&format!("{}setPadding(paddingPx, paddingPx, paddingPx, paddingPx)\n", indent));
        }

        // Generate children
        for child in &component.children {
            let child_code = self.generate_markup(child);
            code.push_str(&format!("{}addView({})\n", self.indent(), child_code));
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", indent));

        code
    }

    fn generate_column(&mut self, component: &Component) -> String {
        let mut code = String::new();
        let indent = self.indent();

        code.push_str(&format!("{}LinearLayout(context).apply {{\n", indent));
        self.indent_level += 1;

        let indent = self.indent();
        code.push_str(&format!("{}orientation = LinearLayout.VERTICAL\n", indent));

        // Generate children
        for child in &component.children {
            let child_code = self.generate_markup(child);
            code.push_str(&format!("{}addView({})\n", indent, child_code));
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn generate_row(&mut self, component: &Component) -> String {
        let mut code = String::new();
        let indent = self.indent();

        code.push_str(&format!("{}LinearLayout(context).apply {{\n", indent));
        self.indent_level += 1;

        let indent = self.indent();
        code.push_str(&format!("{}orientation = LinearLayout.HORIZONTAL\n", indent));

        // Generate children
        for child in &component.children {
            let child_code = self.generate_markup(child);
            code.push_str(&format!("{}addView({})\n", indent, child_code));
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", self.indent()));

        code
    }

    fn generate_text(&mut self, component: &Component) -> String {
        let mut code = String::new();
        let indent = self.indent();

        code.push_str(&format!("{}TextView(context).apply {{\n", indent));
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
            if weight == "\"bold\"" {
                code.push_str(&format!("{}setTypeface(null, android.graphics.Typeface.BOLD)\n", self.indent()));
            }
        }

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", indent));

        code
    }

    fn generate_button(&mut self, component: &Component) -> String {
        let mut code = String::new();
        let indent = self.indent();

        code.push_str(&format!("{}Button(context).apply {{\n", indent));
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

        self.indent_level -= 1;
        code.push_str(&format!("{}}}", indent));

        code
    }

    fn generate_markup(&mut self, markup: &Markup) -> String {
        match markup {
            Markup::Component(component) => self.generate_component(component),
            Markup::Text(text) => format!("\"{}\"", text),
            Markup::Interpolation(expr) => expr.clone(),
            Markup::Sequence(items) => {
                items.iter()
                    .map(|item| self.generate_markup(item))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            _ => String::new(),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transpiler::ast::Component;

    #[test]
    fn test_view_backend_stub() {
        let mut backend = ViewBackend::new("com.example.app", "TestComponent");
        let file = WhitehallFile::default();

        // Should error for now (not implemented)
        assert!(backend.generate(&file).is_err());
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
}
