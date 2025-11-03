# View Backend Architecture

**Status**: 🔄 Design Phase - Required for RecyclerView Optimization

**Last Updated**: 2025-01-03

---

## Problem Statement

To optimize static lists to RecyclerView, we need to generate **Android Views** instead of **Composables**.

**Current**: We only have a Compose backend
```whitehall
<Card padding={8}>
  <Text>{contact.name}</Text>
</Card>
```
↓
```kotlin
Card(modifier = Modifier.padding(8.dp)) {
    Text(text = contact.name)
}
```

**Needed**: View backend for RecyclerView optimization
```whitehall
<Card padding={8}>
  <Text>{contact.name}</Text>
</Card>
```
↓
```kotlin
MaterialCardView(context).apply {
    setPadding(8.dp.toPx())
    addView(TextView(context).apply {
        text = contact.name
    })
}
```

---

## Architecture Design

### Current CodeGen Structure

```
src/transpiler/
├── mod.rs           # Entry point
├── ast.rs           # AST definitions
├── parser.rs        # Whitehall → AST
├── analyzer.rs      # Semantic analysis
├── optimizer.rs     # Optimization planning
└── codegen.rs       # AST → Kotlin/Compose (ONLY)
```

### Proposed Structure

```
src/transpiler/
├── mod.rs           # Entry point
├── ast.rs           # AST definitions
├── parser.rs        # Whitehall → AST
├── analyzer.rs      # Semantic analysis
├── optimizer.rs     # Optimization planning
├── codegen/
│   ├── mod.rs       # CodeGen trait + routing
│   ├── compose.rs   # Compose backend (existing codegen.rs)
│   └── view.rs      # View backend (NEW)
└── recyclerview.rs  # RecyclerView adapter generation (NEW)
```

---

## Module Design

### 1. CodeGen Trait (codegen/mod.rs)

Define interface for backends:

```rust
// src/transpiler/codegen/mod.rs

pub mod compose;
pub mod view;

use crate::transpiler::ast::*;
use crate::transpiler::optimizer::OptimizedAST;

/// Code generation backend
pub trait Backend {
    /// Generate code for a component
    fn generate_component(&mut self, component: &Component) -> String;

    /// Generate code for markup (recursive)
    fn generate_markup(&mut self, markup: &Markup) -> String;

    /// Generate imports
    fn generate_imports(&self) -> Vec<String>;

    /// Generate component function signature
    fn generate_signature(&self, name: &str, props: &[PropDeclaration]) -> String;
}

/// Main code generator - routes to appropriate backend
pub struct CodeGenerator {
    package: String,
    component_name: String,
    component_type: Option<String>,
    backend: Box<dyn Backend>,
}

impl CodeGenerator {
    /// Create generator with Compose backend (default)
    pub fn new(package: &str, component_name: &str, component_type: Option<&str>) -> Self {
        CodeGenerator {
            package: package.to_string(),
            component_name: component_name.to_string(),
            component_type: component_type.map(String::from),
            backend: Box::new(compose::ComposeBackend::new()),
        }
    }

    /// Create generator with View backend (for optimizations)
    pub fn with_view_backend(package: &str, component_name: &str) -> Self {
        CodeGenerator {
            package: package.to_string(),
            component_name: component_name.to_string(),
            component_type: None,
            backend: Box::new(view::ViewBackend::new()),
        }
    }

    /// Generate code using selected backend
    pub fn generate(&mut self, ast: &WhitehallFile) -> Result<String, String> {
        let mut output = String::new();

        // Package
        output.push_str(&format!("package {}\n\n", self.package));

        // Imports
        for import in self.backend.generate_imports() {
            output.push_str(&format!("import {}\n", import));
        }
        output.push_str("\n");

        // Component function signature
        output.push_str(&self.backend.generate_signature(&self.component_name, &ast.props));
        output.push_str(" {\n");

        // Body
        output.push_str(&self.backend.generate_markup(&ast.markup));

        output.push_str("}\n");

        Ok(output)
    }
}
```

---

### 2. Compose Backend (codegen/compose.rs)

Move existing codegen.rs logic here:

```rust
// src/transpiler/codegen/compose.rs

use super::Backend;
use crate::transpiler::ast::*;

pub struct ComposeBackend {
    // Existing CodeGenerator fields
}

impl ComposeBackend {
    pub fn new() -> Self {
        // ...
    }
}

impl Backend for ComposeBackend {
    fn generate_component(&mut self, component: &Component) -> String {
        // Existing generate_component logic
    }

    fn generate_markup(&mut self, markup: &Markup) -> String {
        // Existing generate_markup logic
    }

    fn generate_imports(&self) -> Vec<String> {
        vec![
            "androidx.compose.runtime.*".to_string(),
            "androidx.compose.material3.*".to_string(),
            "androidx.compose.foundation.layout.*".to_string(),
        ]
    }

    fn generate_signature(&self, name: &str, props: &[PropDeclaration]) -> String {
        // Generate @Composable function signature
        let mut sig = String::from("@Composable\n");
        sig.push_str(&format!("fun {}(", name));

        for (i, prop) in props.iter().enumerate() {
            if i > 0 { sig.push_str(",\n    "); }
            sig.push_str(&format!("{}: {}", prop.name, prop.prop_type));
        }

        sig.push(')');
        sig
    }
}
```

---

### 3. View Backend (codegen/view.rs) - NEW

Generate Android View code:

```rust
// src/transpiler/codegen/view.rs

use super::Backend;
use crate::transpiler::ast::*;

pub struct ViewBackend {
    indent_level: usize,
}

impl ViewBackend {
    pub fn new() -> Self {
        ViewBackend { indent_level: 0 }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }
}

impl Backend for ViewBackend {
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

    fn generate_imports(&self) -> Vec<String> {
        vec![
            "android.view.View".to_string(),
            "android.view.ViewGroup".to_string(),
            "android.widget.LinearLayout".to_string(),
            "android.widget.TextView".to_string(),
            "android.widget.Button".to_string(),
            "com.google.android.material.card.MaterialCardView".to_string(),
        ]
    }

    fn generate_signature(&self, name: &str, props: &[PropDeclaration]) -> String {
        // Generate regular Kotlin function that creates a View
        let mut sig = format!("fun create{}(context: android.content.Context", name);

        for prop in props {
            sig.push_str(&format!(", {}: {}", prop.name, prop.prop_type));
        }

        sig.push_str("): View");
        sig
    }
}

impl ViewBackend {
    fn generate_card(&mut self, component: &Component) -> String {
        let mut code = String::new();
        let indent = self.indent();

        code.push_str(&format!("{}MaterialCardView(context).apply {{\n", indent));
        self.indent_level += 1;

        // Handle padding prop
        if let Some(padding) = self.get_prop_value(component, "padding") {
            let indent = self.indent();
            code.push_str(&format!("{}val paddingPx = {}.dp.toPx()\n", indent, padding));
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

        // Handle spacing prop
        if let Some(spacing) = self.get_prop_value(component, "spacing") {
            code.push_str(&format!("{}// TODO: implement spacing = {}dp\n", indent, spacing));
        }

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

        // Handle color prop
        if let Some(color) = self.get_prop_value(component, "color") {
            if color == "\"secondary\"" {
                code.push_str(&format!("{}// TODO: set secondary color\n", self.indent()));
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

    fn get_prop_value(&self, component: &Component, prop_name: &str) -> Option<String> {
        component.props.iter()
            .find(|p| p.name == prop_name)
            .map(|p| match &p.value {
                PropValue::Expression(expr) => expr.clone(),
                _ => String::new(),
            })
    }
}
```

---

### 4. RecyclerView Generator (recyclerview.rs) - NEW

Generate RecyclerView + Adapter code:

```rust
// src/transpiler/recyclerview.rs

use crate::transpiler::ast::*;
use crate::transpiler::codegen::view::ViewBackend;

pub struct RecyclerViewGenerator {
    collection_name: String,
    item_name: String,
    item_type: String,
    view_backend: ViewBackend,
}

impl RecyclerViewGenerator {
    pub fn new(collection: &str, item: &str, item_type: &str) -> Self {
        RecyclerViewGenerator {
            collection_name: collection.to_string(),
            item_name: item.to_string(),
            item_type: item_type.to_string(),
            view_backend: ViewBackend::new(),
        }
    }

    /// Generate RecyclerView wrapper + Adapter class
    pub fn generate(&mut self, loop_body: &[Markup]) -> String {
        let mut code = String::new();

        // AndroidView wrapper
        code.push_str(&self.generate_android_view());
        code.push_str("\n\n");

        // Adapter class
        code.push_str(&self.generate_adapter(loop_body));

        code
    }

    fn generate_android_view(&self) -> String {
        format!(
            r#"    AndroidView(
        factory = {{ context ->
            RecyclerView(context).apply {{
                layoutManager = LinearLayoutManager(context)
                adapter = {}Adapter({})
            }}
        }}
    )"#,
            self.capitalize(&self.item_name),
            self.collection_name
        )
    }

    fn generate_adapter(&mut self, loop_body: &[Markup]) -> String {
        let adapter_name = format!("{}Adapter", self.capitalize(&self.item_name));

        let mut code = String::new();

        // Adapter class declaration
        code.push_str(&format!(
            "private class {}(\n    private val {}: List<{}>\n) : RecyclerView.Adapter<{}.ViewHolder>() {{\n\n",
            adapter_name, self.collection_name, self.item_type, adapter_name
        ));

        // getItemCount
        code.push_str(&format!(
            "    override fun getItemCount(): Int = {}.size\n\n",
            self.collection_name
        ));

        // onCreateViewHolder
        code.push_str("    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder {\n");
        code.push_str("        val context = parent.context\n");
        code.push_str("        val itemView = ");

        // Generate view hierarchy from loop body
        code.push_str(&self.generate_view_hierarchy(loop_body));
        code.push_str("\n");

        code.push_str("        return ViewHolder(itemView)\n");
        code.push_str("    }\n\n");

        // onBindViewHolder
        code.push_str("    override fun onBindViewHolder(holder: ViewHolder, position: Int) {\n");
        code.push_str(&format!("        val {} = {}[position]\n", self.item_name, self.collection_name));
        code.push_str("        // TODO: Bind data to holder views\n");
        code.push_str("    }\n\n");

        // ViewHolder class
        code.push_str("    class ViewHolder(itemView: View) : RecyclerView.ViewHolder(itemView) {\n");
        code.push_str("        // TODO: Find views by ID\n");
        code.push_str("    }\n");

        code.push_str("}\n");

        code
    }

    fn generate_view_hierarchy(&mut self, markup: &[Markup]) -> String {
        if markup.is_empty() {
            return "TextView(context)".to_string();
        }

        // Use View backend to generate view code
        self.view_backend.generate_markup(&markup[0])
    }

    fn capitalize(&self, s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}
```

---

## Integration with Optimizer

Update optimizer to specify which backend to use:

```rust
// src/transpiler/optimizer.rs

#[derive(Debug, Clone)]
pub enum Optimization {
    UseRecyclerView {
        collection_name: String,
        confidence: u8,
        backend: CodeBackend,  // NEW
    },
    // ...
}

#[derive(Debug, Clone)]
pub enum CodeBackend {
    Compose,    // Default
    View,       // For RecyclerView optimization
}
```

---

## Implementation Phases

### Phase 0.5: View Backend Foundation (Week 1-2)

**Goal**: Build View backend infrastructure (no optimization yet)

**Tasks**:
1. Create `src/transpiler/codegen/` directory
2. Move existing `codegen.rs` → `codegen/compose.rs`
3. Create `codegen/mod.rs` with `Backend` trait
4. Create `codegen/view.rs` skeleton
5. Implement basic View generation (Card, Column, Row, Text)
6. Create `recyclerview.rs` with adapter generation
7. Add unit tests for View backend
8. **Still zero behavior changes** (not used yet)

**Deliverables**:
- View backend can generate Views from markup
- RecyclerView generator can create adapter boilerplate
- All tests still pass

---

### Phase 1-4: Analyzer + Optimizer (Weeks 3-5)

(As previously planned - usage tracking, static detection, planning)

---

### Phase 5: Wire View Backend (Week 6-7)

**Goal**: Use View backend for RecyclerView optimization

**Tasks**:
1. Update `CodeGenerator` to accept `OptimizedAST`
2. Check if optimization specifies `CodeBackend::View`
3. Route to View backend for RecyclerView loops
4. Generate RecyclerView + Adapter wrapper
5. Test against `tests/optimization-examples/01-static-list-optimization.md`
6. Validate Optimized Output matches

---

## Component Mapping

| Whitehall | Compose | Android View |
|-----------|---------|--------------|
| `<Card>` | `Card(...)` | `MaterialCardView(context)` |
| `<Column>` | `Column(...)` | `LinearLayout(VERTICAL)` |
| `<Row>` | `Row(...)` | `LinearLayout(HORIZONTAL)` |
| `<Text>` | `Text(...)` | `TextView(context)` |
| `<Button>` | `Button(...)` | `Button(context)` |
| `<TextField>` | `TextField(...)` | `EditText(context)` |
| `<Checkbox>` | `Checkbox(...)` | `CheckBox(context)` |
| `<Image>` | `Image(...)` | `ImageView(context)` |

---

## Challenges

### Challenge 1: Compose Props → View Attributes

Compose uses modifiers, Views use direct methods:

```whitehall
<Text fontSize={16} fontWeight="bold" color="primary">
```

**Compose**:
```kotlin
Text(
    text = ...,
    fontSize = 16.sp,
    fontWeight = FontWeight.Bold,
    color = MaterialTheme.colorScheme.primary
)
```

**View**:
```kotlin
TextView(context).apply {
    textSize = 16f
    setTypeface(null, Typeface.BOLD)
    setTextColor(/* resolve primary color */)
}
```

**Solution**: Build prop → attribute mapping table in View backend

---

### Challenge 2: Layout Params

Views need `LayoutParams`, Compose uses Modifiers:

```whitehall
<Column padding={16} fill={true}>
```

**Compose**:
```kotlin
Column(modifier = Modifier.fillMaxWidth().padding(16.dp))
```

**View**:
```kotlin
LinearLayout(context).apply {
    layoutParams = ViewGroup.LayoutParams(
        ViewGroup.LayoutParams.MATCH_PARENT,
        ViewGroup.LayoutParams.WRAP_CONTENT
    )
    setPadding(16.dp.toPx(), ...)
}
```

**Solution**: Track parent context to generate correct LayoutParams

---

### Challenge 3: DP Conversion

Views need pixel values, Compose handles DP:

```whitehall
padding={16}
```

**Compose**: `16.dp` (automatic)

**View**: Need conversion helper:
```kotlin
val paddingPx = (16 * context.resources.displayMetrics.density).toInt()
```

**Solution**: Generate helper extension:
```kotlin
fun Int.dp.toPx(): Int = (this * Resources.getSystem().displayMetrics.density).toInt()
```

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_view_backend_generates_card() {
    let component = Component {
        name: "Card".to_string(),
        props: vec![],
        children: vec![],
        self_closing: false,
    };

    let mut backend = ViewBackend::new();
    let output = backend.generate_component(&component);

    assert!(output.contains("MaterialCardView(context)"));
}
```

### Integration Tests

```rust
#[test]
fn test_recyclerview_generation() {
    let input = r#"
        val contacts = listOf(...)

        @for (contact in contacts) {
          <Card><Text>{contact.name}</Text></Card>
        }
    "#;

    // Should generate RecyclerView + Adapter
    let output = transpile_with_optimization(input);

    assert!(output.contains("RecyclerView(context)"));
    assert!(output.contains("class ContactAdapter"));
    assert!(output.contains("RecyclerView.Adapter"));
}
```

---

## Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 0.5: View Backend | 2 weeks | View codegen working |
| 1-4: Analyzer/Optimizer | 3 weeks | Static detection working |
| 5: Integration | 2 weeks | RecyclerView optimization live |
| **Total** | **7 weeks** | Fully working optimization |

---

## Success Criteria

✅ **Phase 0.5 Complete When**:
- View backend can generate Android Views from markup
- RecyclerView generator creates valid adapter code
- All existing 23 tests still pass
- No behavior changes yet

✅ **Full System Complete When**:
- `tests/optimization-examples/01-static-list-optimization.md` generates Optimized Output
- `tests/optimization-examples/02-dynamic-list-no-optimization.md` stays unoptimized
- 30-40% performance improvement measurable
- Zero false positives

---

**This is the proper way. No hacks. No shortcuts. Pure engineering.** 🚀
