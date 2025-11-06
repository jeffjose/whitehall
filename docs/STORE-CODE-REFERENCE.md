# Whitehall @store Implementation - Quick Code Reference

This document contains key code snippets and line references for the @store implementation.

## File Structure

| Component | File | Lines | Purpose |
|-----------|------|-------|---------|
| AST Definitions | `src/transpiler/ast.rs` | 1-150 | Class and property structures |
| Parser | `src/transpiler/parser.rs` | 62-73, 480-651 | Parse @store classes |
| Analyzer | `src/transpiler/analyzer.rs` | 28-298 | Build store registry |
| Codegen | `src/transpiler/codegen/compose.rs` | 103-378, 2545-2684 | Generate Kotlin code |
| Tests | `tests/transpiler-examples/` | 27-29 | Store test cases |
| Examples | `examples/counter-store/` | src/ | Working demo |

---

## CRITICAL CODE SECTIONS

### 1. StoreRegistry Definition

**File:** `src/transpiler/analyzer.rs` (lines 28-62)

```rust
pub struct StoreRegistry {
    stores: HashMap<String, StoreInfo>,
}

pub struct StoreInfo {
    pub class_name: String,
    pub has_hilt: bool,
    pub has_inject: bool,
    pub package: String,
}

impl StoreRegistry {
    pub fn new() -> Self {
        StoreRegistry { stores: HashMap::new() }
    }
    
    pub fn insert(&mut self, name: String, info: StoreInfo) {
        self.stores.insert(name, info);
    }
    
    pub fn get(&self, name: &str) -> Option<&StoreInfo> {
        self.stores.get(name)
    }
    
    pub fn contains(&self, name: &str) -> bool {
        self.stores.contains_key(name)
    }
    
    pub fn is_hilt_view_model(&self, name: &str) -> bool {
        self.get(name).map(|info| info.has_hilt).unwrap_or(false)
    }
}
```

### 2. Store Collection in Analyzer

**File:** `src/transpiler/analyzer.rs` (lines 266-298)

```rust
fn collect_stores(&mut self, ast: &WhitehallFile) {
    for class in &ast.classes {
        // Check if class has @store annotation
        let has_store = class.annotations.iter().any(|a| a == "store");
        if !has_store {
            continue;
        }

        // Check for @hilt annotation (case-insensitive)
        let has_hilt = class.annotations.iter().any(|a| {
            a.eq_ignore_ascii_case("hilt") || a == "HiltViewModel"
        });

        // Check for @inject constructor (case-insensitive)
        let has_inject = class.constructor.as_ref()
            .map(|c| c.annotations.iter().any(|a| {
                a.eq_ignore_ascii_case("inject")
            }))
            .unwrap_or(false);

        // Register the store
        self.store_registry.insert(
            class.name.clone(),
            StoreInfo {
                class_name: class.name.clone(),
                has_hilt,
                has_inject,
                package: String::new(),
            },
        );
    }
}
```

### 3. Annotation Parsing in Parser

**File:** `src/transpiler/parser.rs` (lines 62-81)

```rust
// Check for annotations (@store, @HiltViewModel, etc.)
if self.peek_char() == Some('@') {
    self.advance_char(); // Skip @
    let annotation = self.parse_identifier()?;
    pending_annotations.push(annotation.clone());

    // Check if next is "class" keyword
    self.skip_whitespace();
    if self.peek_word() == Some("class") {
        classes.push(self.parse_class_declaration(pending_annotations.clone())?);
        pending_annotations.clear();
        continue;
    }
    // Otherwise, continue to next iteration to collect more annotations
}
```

### 4. Class Declaration Parsing

**File:** `src/transpiler/parser.rs` (lines 480-542)

```rust
fn parse_class_declaration(&mut self, annotations: Vec<String>) -> Result<ClassDeclaration, String> {
    self.skip_whitespace();
    self.expect_word("class")?;
    self.skip_whitespace();

    let name = self.parse_identifier()?;
    self.skip_whitespace();

    // Parse optional constructor
    let constructor = if self.peek_word() == Some("constructor")
        || self.peek_char() == Some('(')
        || self.peek_char() == Some('@') {
        Some(self.parse_constructor()?)
    } else {
        None
    };

    self.skip_whitespace();
    self.expect_char('{')?;

    // Parse class body
    let mut properties = Vec::new();
    let mut functions = Vec::new();

    loop {
        self.skip_whitespace();
        if self.peek_char() == Some('}') {
            self.advance_char();
            break;
        }

        if self.peek_word() == Some("var") || self.peek_word() == Some("val") {
            properties.push(self.parse_property_declaration()?);
        }
        else if self.peek_word() == Some("fun") || self.peek_word() == Some("suspend") {
            let is_suspend = self.consume_word("suspend");
            if is_suspend {
                self.skip_whitespace();
            }
            self.expect_word("fun")?;
            functions.push(self.parse_function_declaration(is_suspend)?);
        }
    }

    Ok(ClassDeclaration {
        annotations,
        name,
        constructor,
        properties,
        functions,
    })
}
```

### 5. Property Declaration with Getters

**File:** `src/transpiler/parser.rs` (lines 587-651)

```rust
fn parse_property_declaration(&mut self) -> Result<PropertyDeclaration, String> {
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
```

### 6. Store Detection at Usage Sites

**File:** `src/transpiler/codegen/compose.rs` (lines 103-135)

```rust
fn detect_store_instantiation(&self, value: &str) -> Option<StoreInfo> {
    // Pattern: "ClassName()" or "ClassName(...params...)"
    let trimmed = value.trim();
    if !trimmed.ends_with(')') {
        return None;
    }

    // Extract class name before '('
    if let Some(paren_pos) = trimmed.find('(') {
        let class_name = trimmed[..paren_pos].trim();

        // Check if it's in the store registry
        if let Some(ref registry) = self.store_registry {
            return registry.get(class_name).cloned();
        }
    }

    None
}

fn detect_store_usage(&mut self, file: &WhitehallFile) {
    for state in &file.state {
        let transformed_value = self.transform_array_literal(&state.initial_value, false);
        if let Some(store_info) = self.detect_store_instantiation(&transformed_value) {
            self.uses_viewmodel = true;
            if store_info.has_hilt {
                self.uses_hilt_viewmodel = true;
            }
        }
    }
}
```

### 7. ViewModel Generation at Usage Site

**File:** `src/transpiler/codegen/compose.rs` (lines 358-378)

```rust
// Check if this is a store instantiation
if let Some(store_info) = self.detect_store_instantiation(&transformed_value) {
    // Track store usage for imports
    self.uses_viewmodel = true;
    // Use hiltViewModel if either @hilt or @inject is present
    let needs_hilt = store_info.has_hilt || store_info.has_inject;
    if needs_hilt {
        self.uses_hilt_viewmodel = true;
    }

    // Generate viewModel or hiltViewModel based on annotations
    let view_model_call = if needs_hilt {
        format!("hiltViewModel<{}>()", store_info.class_name)
    } else {
        format!("viewModel<{}>()", store_info.class_name)
    };

    output.push_str(&format!("val {} = {}\n", state.name, view_model_call));

    // Add collectAsState for uiState
    output.push_str(&self.indent());
    output.push_str(&format!("val uiState by {}.uiState.collectAsState()\n", state.name));
}
```

### 8. ViewModel Class Generation

**File:** `src/transpiler/codegen/compose.rs` (lines 2545-2684)

```rust
fn generate_store_class(&self, class: &ClassDeclaration) -> Result<String, String> {
    let mut output = String::new();

    // Package declaration
    output.push_str(&format!("package {}\n\n", self.package));

    // Imports
    output.push_str("import androidx.lifecycle.ViewModel\n");
    output.push_str("import androidx.lifecycle.viewModelScope\n");
    output.push_str("import kotlinx.coroutines.flow.MutableStateFlow\n");
    output.push_str("import kotlinx.coroutines.flow.StateFlow\n");
    output.push_str("import kotlinx.coroutines.flow.asStateFlow\n");
    output.push_str("import kotlinx.coroutines.flow.update\n");
    output.push_str("import kotlinx.coroutines.launch\n");

    // Check if Hilt is needed
    let has_hilt_annotation = class.annotations.iter().any(|a| {
        a.eq_ignore_ascii_case("hilt") || a == "HiltViewModel"
    });
    let has_inject_constructor = class.constructor.as_ref()
        .map(|c| c.annotations.iter().any(|a| a.eq_ignore_ascii_case("inject")))
        .unwrap_or(false);
    let needs_hilt = has_hilt_annotation || has_inject_constructor;

    // Add Hilt imports if needed
    if needs_hilt {
        output.push_str("import dagger.hilt.android.lifecycle.HiltViewModel\n");
        output.push_str("import javax.inject.Inject\n");
    }

    output.push('\n');

    // Class annotations
    if needs_hilt {
        output.push_str("@HiltViewModel\n");
    }

    // Class declaration
    output.push_str(&format!("class {}", class.name));

    // Constructor
    if let Some(constructor) = &class.constructor {
        output.push(' ');
        if !constructor.annotations.is_empty() {
            output.push_str("@Inject ");
        }
        output.push_str(&format!("constructor(\n    {}\n)", constructor.parameters));
    }

    output.push_str(" : ViewModel() {\n");

    // Generate UiState data class
    output.push_str("    data class UiState(\n");
    for (i, prop) in class.properties.iter().enumerate() {
        // Skip derived properties (with getters)
        if prop.getter.is_some() {
            continue;
        }

        // Infer type from initial value if no type annotation
        let type_str = if let Some(type_ann) = &prop.type_annotation {
            type_ann.clone()
        } else if let Some(init_val) = &prop.initial_value {
            self.infer_type_from_value(init_val)
        } else {
            "String".to_string()
        };
        let default_val = prop.initial_value.as_deref().unwrap_or("\"\"");
        output.push_str(&format!("        val {}: {} = {}", prop.name, type_str, default_val));
        if i < class.properties.len() - 1 {
            output.push(',');
        }
        output.push('\n');
    }
    output.push_str("    )\n\n");

    // Generate StateFlow
    output.push_str("    private val _uiState = MutableStateFlow(UiState())\n");
    output.push_str("    val uiState: StateFlow<UiState> = _uiState.asStateFlow()\n\n");

    // Generate property accessors
    for prop in &class.properties {
        if prop.getter.is_some() {
            // Derived property with getter
            let type_str = if let Some(type_ann) = &prop.type_annotation {
                type_ann.clone()
            } else if let Some(init_val) = &prop.initial_value {
                self.infer_type_from_value(init_val)
            } else {
                "String".to_string()
            };
            let getter_expr = prop.getter.as_ref().unwrap();
            output.push_str(&format!("    val {}: {}\n", prop.name, type_str));
            output.push_str(&format!("        get() = {}\n\n", getter_expr));
        } else {
            // Regular property with setter
            let type_str = if let Some(type_ann) = &prop.type_annotation {
                type_ann.clone()
            } else if let Some(init_val) = &prop.initial_value {
                self.infer_type_from_value(init_val)
            } else {
                "String".to_string()
            };
            output.push_str(&format!("    var {}: {}\n", prop.name, type_str));
            output.push_str(&format!("        get() = _uiState.value.{}\n", prop.name));
            output.push_str(&format!("        set(value) {{ _uiState.update {{ it.copy({} = value) }} }}\n\n", prop.name));
        }
    }

    // Generate functions
    for func in &class.functions {
        output.push_str(&format!("    fun {}({})", func.name, func.params));
        if let Some(return_type) = &func.return_type {
            output.push_str(&format!(": {}", return_type));
        }
        output.push_str(" {\n");

        // Wrap suspend functions in viewModelScope.launch
        if func.is_suspend {
            output.push_str("        viewModelScope.launch {\n");
            // Indent each line of the function body properly
            for line in func.body.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    output.push_str(&format!("            {}\n", trimmed));
                }
            }
            output.push_str("        }\n");
        } else {
            output.push_str(&format!("        {}\n", func.body.trim()));
        }

        output.push_str("    }\n\n");
    }

    output.push_str("}\n");

    Ok(output)
}
```

---

## KEY DECISION POINTS

### Decision 1: Hilt Detection (Analyzer)
**Location:** `analyzer.rs:276-285`
- Checks BOTH `@hilt` annotation AND `@Inject` constructor
- Uses `||` (OR) logic: Either one enables Hilt
- Both flags tracked separately for future flexibility

### Decision 2: Registry Lookup (Codegen)
**Location:** `compose.rs:115-116`
- Looks up class name in registry
- Returns `StoreInfo` with cached hilt/inject flags
- Enables cost-effective usage-site decisions

### Decision 3: Suspend Wrapping (Codegen)
**Location:** `compose.rs:2664-2673`
- Wrapping happens in codegen, not parser
- Simple indentation-based transformation
- Preserves original function body structure

### Decision 4: UiState Exclusion (Codegen)
**Location:** `compose.rs:2600-2603`
- Derived properties EXCLUDED from UiState
- Allows them to access other properties directly
- Simplifies generated code

---

## TESTING COMMAND

Run all transpiler examples:
```bash
cargo test transpiler_examples_test
```

Run specific store tests:
```bash
cargo test transpiler_examples_test -- 27\|28\|29
```

Run working example:
```bash
whitehall run examples/counter-store/
```

---

## TROUBLESHOOTING

### Store not detected at usage site?
- Check `detect_store_instantiation()` pattern matching
- Verify class name matches exactly (case-sensitive)
- Ensure registry was built (requires @store annotation on class definition)

### Hilt not being added?
- Check both `@hilt` annotation and `@Inject` constructor
- Verify capitalization (@Inject, not @inject - though parser is case-insensitive)
- Check `needs_hilt` logic uses `||` not `&&`

### Properties not in UiState?
- Derived properties (with getters) are intentionally excluded
- Only `var` properties should be in UiState
- `val` properties without getters also excluded

### ViewModelScope not wrapping function?
- Check function has `suspend` keyword
- Verify `FunctionDeclaration.is_suspend` is true
- Check wrapping logic at `compose.rs:2664`

