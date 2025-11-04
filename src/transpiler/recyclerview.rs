#![allow(dead_code)]
//! RecyclerView + Adapter generator for static list optimization
//!
//! Wraps View backend output in RecyclerView.Adapter boilerplate
//!
//! Note: This is future/experimental code not yet fully integrated.

use crate::transpiler::ast::{ForLoopBlock, Markup};

pub struct RecyclerViewGenerator {
    collection_name: String,
    item_name: String,
    package: String,
}

impl RecyclerViewGenerator {
    pub fn new(collection: &str, item: &str, package: &str) -> Self {
        RecyclerViewGenerator {
            collection_name: collection.to_string(),
            item_name: item.to_string(),
            package: package.to_string(),
        }
    }

    /// Generate complete RecyclerView wrapper + Adapter class
    ///
    /// Takes a for loop and generates:
    /// 1. AndroidView wrapper with RecyclerView
    /// 2. Adapter class extending RecyclerView.Adapter
    /// 3. ViewHolder with view creation and binding
    pub fn generate(&self, for_loop: &ForLoopBlock) -> Result<String, String> {
        let mut output = String::new();

        // Package
        output.push_str(&format!("package {}\n\n", self.package));

        // Imports
        output.push_str(&self.generate_imports());
        output.push_str("\n");

        // Main function that creates RecyclerView
        output.push_str(&self.generate_recyclerview_function(for_loop)?);
        output.push_str("\n\n");

        // Adapter class
        output.push_str(&self.generate_adapter_class(for_loop)?);

        Ok(output)
    }

    fn generate_imports(&self) -> String {
        let mut imports = String::new();

        imports.push_str("import android.content.Context\n");
        imports.push_str("import android.view.View\n");
        imports.push_str("import android.view.ViewGroup\n");
        imports.push_str("import androidx.compose.runtime.Composable\n");
        imports.push_str("import androidx.compose.ui.viewinterop.AndroidView\n");
        imports.push_str("import androidx.recyclerview.widget.LinearLayoutManager\n");
        imports.push_str("import androidx.recyclerview.widget.RecyclerView\n");

        // Add View-specific imports based on components used
        imports.push_str("import android.widget.LinearLayout\n");
        imports.push_str("import android.widget.TextView\n");
        imports.push_str("import android.widget.Button\n");
        imports.push_str("import com.google.android.material.card.MaterialCardView\n");

        imports
    }

    fn generate_recyclerview_function(&self, _for_loop: &ForLoopBlock) -> Result<String, String> {
        let adapter_name = self.capitalize(&self.item_name) + "Adapter";

        let mut code = String::new();

        code.push_str("@Composable\n");
        code.push_str(&format!("fun {}List(", self.capitalize(&self.collection_name)));
        code.push_str(&format!("{}: List<{}>", self.collection_name, self.capitalize(&self.item_name)));
        code.push_str(") {\n");

        // AndroidView wrapper
        code.push_str("    AndroidView(\n");
        code.push_str("        factory = { context ->\n");
        code.push_str("            RecyclerView(context).apply {\n");
        code.push_str("                layoutManager = LinearLayoutManager(context)\n");
        code.push_str(&format!("                adapter = {}({})\n", adapter_name, self.collection_name));
        code.push_str("            }\n");
        code.push_str("        }\n");
        code.push_str("    )\n");
        code.push_str("}\n");

        Ok(code)
    }

    fn generate_adapter_class(&self, for_loop: &ForLoopBlock) -> Result<String, String> {
        let adapter_name = self.capitalize(&self.item_name) + "Adapter";
        let item_type = self.capitalize(&self.item_name);

        let mut code = String::new();

        // Class declaration
        code.push_str(&format!("private class {}(\n", adapter_name));
        code.push_str(&format!("    private val {}: List<{}>\n", self.collection_name, item_type));
        code.push_str(&format!(") : RecyclerView.Adapter<{}.ViewHolder>() {{\n\n", adapter_name));

        // getItemCount
        code.push_str("    override fun getItemCount(): Int = ");
        code.push_str(&format!("{}.size\n\n", self.collection_name));

        // onCreateViewHolder
        code.push_str("    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder {\n");
        code.push_str("        val context = parent.context\n");
        code.push_str("        val itemView = ");
        code.push_str(&self.generate_view_creation(for_loop)?);
        code.push_str("\n        return ViewHolder(itemView)\n");
        code.push_str("    }\n\n");

        // onBindViewHolder
        code.push_str("    override fun onBindViewHolder(holder: ViewHolder, position: Int) {\n");
        code.push_str(&format!("        val {} = {}[position]\n", self.item_name, self.collection_name));
        code.push_str("        // Bind data to views\n");
        code.push_str(&self.generate_view_binding(for_loop));
        code.push_str("    }\n\n");

        // ViewHolder class
        code.push_str("    class ViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {\n");
        code.push_str(&self.generate_view_holder_fields(for_loop));
        code.push_str("    }\n");

        code.push_str("}\n");

        Ok(code)
    }

    fn generate_view_creation(&self, for_loop: &ForLoopBlock) -> Result<String, String> {
        // Use ViewBackend to generate the view hierarchy
        // For now, generate a placeholder based on the loop body

        if for_loop.body.is_empty() {
            return Ok("TextView(context)".to_string());
        }

        // Simple heuristic: if body has Card, use MaterialCardView
        let has_card = self.has_component(&for_loop.body, "Card");
        let has_column = self.has_component(&for_loop.body, "Column");

        if has_card {
            Ok("MaterialCardView(context).apply {\n            // View structure will be added here\n        }".to_string())
        } else if has_column {
            Ok("LinearLayout(context).apply {\n            orientation = LinearLayout.VERTICAL\n        }".to_string())
        } else {
            Ok("TextView(context)".to_string())
        }
    }

    fn generate_view_binding(&self, for_loop: &ForLoopBlock) -> String {
        // Generate binding code based on loop body
        // For now, placeholder
        let mut code = String::new();

        if self.has_component(&for_loop.body, "Text") {
            code.push_str(&format!("        // holder.bind({})\n", self.item_name));
        }

        code
    }

    fn generate_view_holder_fields(&self, for_loop: &ForLoopBlock) -> String {
        // Generate fields for ViewHolder based on what views need to be referenced
        let mut fields = String::new();

        if self.has_component(&for_loop.body, "Text") {
            fields.push_str("        // val textView: TextView = itemView.findViewById(...)\n");
        }

        if fields.is_empty() {
            fields.push_str("        // Fields for view references\n");
        }

        fields
    }

    fn has_component(&self, markup_list: &[Markup], component_name: &str) -> bool {
        markup_list.iter().any(|m| match m {
            Markup::Component(c) => c.name == component_name,
            Markup::Sequence(items) => self.has_component(items, component_name),
            _ => false,
        })
    }

    fn capitalize(&self, s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

// Extension function for DP to PX conversion
// This will be included in the generated output
pub fn generate_dp_extension() -> String {
    r#"
// Extension function for DP to PX conversion
private fun Int.dpToPx(): Int {
    return (this * android.content.res.Resources.getSystem().displayMetrics.density).toInt()
}
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transpiler::ast::Component;

    #[test]
    fn test_recyclerview_generator_basic() {
        let generator = RecyclerViewGenerator::new("contacts", "contact", "com.example.app");

        let for_loop = ForLoopBlock {
            item: "contact".to_string(),
            collection: "contacts".to_string(),
            key_expr: Some("it.id".to_string()),
            body: vec![
                Markup::Component(Component {
                    name: "Card".to_string(),
                    props: vec![],
                    children: vec![
                        Markup::Component(Component {
                            name: "Text".to_string(),
                            props: vec![],
                            children: vec![Markup::Interpolation("contact.name".to_string())],
                            self_closing: false,
                        }),
                    ],
                    self_closing: false,
                }),
            ],
            empty_block: None,
        };

        let output = generator.generate(&for_loop).unwrap();

        assert!(output.contains("package com.example.app"));
        assert!(output.contains("@Composable"));
        assert!(output.contains("fun ContactsList"));
        assert!(output.contains("RecyclerView(context)"));
        assert!(output.contains("ContactAdapter"));
        assert!(output.contains("RecyclerView.Adapter"));
        assert!(output.contains("override fun getItemCount()"));
        assert!(output.contains("override fun onCreateViewHolder"));
        assert!(output.contains("override fun onBindViewHolder"));
        assert!(output.contains("class ViewHolder"));
    }

    #[test]
    fn test_generate_recyclerview_function() {
        let generator = RecyclerViewGenerator::new("items", "item", "com.example.app");

        let for_loop = ForLoopBlock {
            item: "item".to_string(),
            collection: "items".to_string(),
            key_expr: None,
            body: vec![],
            empty_block: None,
        };

        let output = generator.generate_recyclerview_function(&for_loop).unwrap();

        assert!(output.contains("fun ItemsList(items: List<Item>)"));
        assert!(output.contains("AndroidView"));
        assert!(output.contains("RecyclerView(context)"));
        assert!(output.contains("LinearLayoutManager(context)"));
        assert!(output.contains("adapter = ItemAdapter(items)"));
    }

    #[test]
    fn test_dp_extension_function() {
        let extension = generate_dp_extension();

        assert!(extension.contains("fun Int.dpToPx()"));
        assert!(extension.contains("displayMetrics.density"));
    }
}
