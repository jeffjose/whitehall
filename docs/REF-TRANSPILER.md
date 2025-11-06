# Whitehall Transpiler Reference

**Comprehensive guide to the transpiler architecture and implementation**

---

## Status

✅ **Nearly Complete** - 37/38 tests passing (97.4% coverage)

All core features implemented and production-ready. One minor import detection issue remains in test 05 (data-binding).

---

## Quick Summary

The Whitehall transpiler converts `.wh` files into idiomatic Kotlin with Jetpack Compose code.

**Architecture:** Lexer-free recursive descent parser → AST → Semantic analysis → Code generation

**Location:** `src/transpiler/`

---

## Module Structure

```
src/transpiler/
├── mod.rs       # Public API: transpile(input, package, component_name) -> Result<String, String>
├── ast.rs       # Abstract Syntax Tree definitions
├── parser.rs    # Lexer-free recursive descent parser
└── codegen/
    ├── mod.rs
    └── compose.rs   # Kotlin/Compose code generation with transformations
```

---

## Compilation Pipeline

```
.wh file (string)
    ↓
Parser (parser.rs)
    ├─ Parse imports: import $models.User
    ├─ Parse props: @prop val name: Type
    ├─ Parse state: var/val declarations
    ├─ Parse classes: @store class MyStore { ... }
    ├─ Parse functions: fun name() { body }
    ├─ Parse lifecycle: onMount { }, onDispose { }
    └─ Parse markup: <Component>...</Component>
    ↓
AST (ast.rs)
    ├─ WhitehallFile { imports, props, state, classes, markup }
    ├─ Markup enum (Component, Text, Interpolation, IfElse, ForLoop, etc.)
    └─ ClassDeclaration (for @store classes)
    ↓
Semantic Analyzer (analyzer.rs)
    ├─ Build store registry (cross-file detection)
    ├─ Detect Hilt annotations
    └─ Pass SemanticInfo to codegen
    ↓
Code Generator (codegen/compose.rs)
    ├─ Check if file contains @store class
    ├─ If yes: generate_store_class() → ViewModel Kotlin
    ├─ If no: generate_component() → Composable Kotlin
    ├─ Transform props (spacing → Arrangement.spacedBy)
    ├─ Transform expressions (route aliases, lambda arrows)
    ├─ Collect imports recursively
    └─ Format output with proper indentation
    ↓
Generated Kotlin Code
```

---

## Parser Architecture

### Approach: Lexer-Free Recursive Descent

**Why lexer-free?**
- Whitehall mixes Kotlin and XML syntax (hard to tokenize cleanly)
- Recursive descent handles nested structures naturally
- Lookahead of 1-2 characters sufficient for decision-making
- Simpler codebase (~600 LOC parser vs. lexer + parser)

**Key Features:**
- Position tracking: Single `pos: usize` cursor through input
- Recursive descent: Each grammar rule is a method
- Lookahead: `peek_char()` and `peek_ahead(n)` for decisions
- Depth tracking: For nested braces/parens/brackets
- Infinite loop guards: Track position, error if no progress

### Key Parser Methods

```rust
pub fn parse(&mut self) -> Result<WhitehallFile, String>
  ├─ parse_imports()          // import $models.User
  ├─ parse_props()            // @prop val name: Type
  ├─ parse_state()            // var/val declarations
  ├─ parse_classes()          // @store class MyStore { ... }
  └─ parse_component()        // <Component>...</Component>
       ├─ parse_component_prop()    // prop={value} or prop="string"
       ├─ parse_children()          // Recursive markup parsing
       │    ├─ parse_control_flow() // @if, @for, @when
       │    ├─ parse_component()    // Nested components
       │    └─ parse_text_with_interpolation_until_markup()
       └─ parse_markup_block()      // For control flow bodies
```

---

## AST Design

### Philosophy

Simple, focused on transpilation needs (not full semantic analysis)

### Key Structures

```rust
pub struct WhitehallFile {
    pub imports: Vec<Import>,
    pub props: Vec<PropDeclaration>,
    pub state: Vec<StateDeclaration>,
    pub classes: Vec<ClassDeclaration>,    // @store classes
    pub functions: Vec<FunctionDeclaration>,
    pub lifecycle: Vec<LifecycleHook>,
    pub markup: Markup,
}

pub enum Markup {
    Component(Component),
    Text(String),
    Interpolation(String),
    Sequence(Vec<Markup>),
    IfElse(IfElseBlock),
    ForLoop(ForLoopBlock),
    When(WhenBlock),
}

pub struct Component {
    pub name: String,
    pub props: Vec<(String, PropValue)>,
    pub children: Vec<Markup>,
}

pub enum PropValue {
    Expression(String),      // prop={value}
    Markup(Box<Markup>),     // prop={<Component />}
}
```

**Key Design Choices:**
- **Flat prop/state collections**: No scoping needed (single-component files)
- **String expressions**: No expression AST, just capture strings for codegen
- **Markup enum**: Unified representation of all renderable content
- **Recursive children**: Components and control flow contain `Vec<Markup>`
- **PropValue enum**: Type-safe handling of components in prop positions

---

## Code Generator Architecture

### Structure

```rust
pub struct CodeGenerator {
    package: String,
    component_name: String,
    indent_level: usize,
    store_registry: Option<StoreRegistry>,
    // Flags for import detection
    uses_viewmodel: bool,
    uses_hilt_viewmodel: bool,
    uses_dispatcher_scope: bool,
    // Context tracking for ViewModel wrappers
    in_viewmodel_wrapper: bool,
    mutable_vars: HashSet<String>,
    derived_props: HashSet<String>,
    function_names: HashSet<String>,
}

impl CodeGenerator {
    pub fn generate(&mut self, file: &WhitehallFile) -> Result<String, String>
      ├─ Check if file contains @store class
      ├─ If yes: generate_store_class() → ViewModel
      ├─ If no: generate_component() → Composable
      │    ├─ Detect store usage
      │    ├─ Generate imports
      │    ├─ Generate function signature
      │    ├─ Generate computed state
      │    ├─ Generate mutable state (with remember)
      │    ├─ Generate functions
      │    ├─ Generate lifecycle hooks
      │    └─ Generate markup
      └─ Format and return
}
```

### Prop Transformation System

Component-specific transformations for Compose idioms:

```rust
match (component, prop_name) {
    ("Column", "spacing") => "verticalArrangement = Arrangement.spacedBy({value}.dp)",
    ("Column", "padding") => "modifier = Modifier.padding({value}.dp)",
    ("Row", "spacing") => "horizontalArrangement = Arrangement.spacedBy({value}.dp)",
    ("Text", "fontSize") => "fontSize = {value}.sp",
    ("Text", "fontWeight") => "fontWeight = FontWeight.{Capitalized}",
    ("Text", "color") => "color = MaterialTheme.colorScheme.{value}",
    ("Button", "text") => "(child) Text(\"{value}\")",
    ("TextField", "label") => "label = { Text(\"{value}\") }",
    ("Card", "backgroundColor") => "colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.{value})",
    ...
}
```

### Value Transformations

- **Lambda arrows:** `() => expr` → `{ expr }`
- **Route aliases:** `$routes.post.detail` → `Routes.Post.Detail`
- **String literals:** `"bold"` → `FontWeight.Bold`
- **Numbers:** `16` → `16.dp` or `16.sp` (context-dependent)
- **Ternary operators:** `condition ? a : b` → `.let { if (condition) a else b }`
- **$screen.params:** `$screen.params.id` → `id` (extracted as function parameter)

### Import Management

**Process:**
1. **Detection**: Scan AST for components, prop values, transformations
2. **Transformation imports**: Add imports for Arrangement, Modifier, dp, sp, FontWeight, etc.
3. **Component imports**: Map known components (Text, Column, Card) to androidx/material3
4. **Store imports**: Add androidx.lifecycle.viewmodel.compose.viewModel if store detected
5. **Alphabetical sorting**: Standard Kotlin convention
6. **Deduplication**: Track imports in HashSet, check before adding

---

## Store Registry & ViewModel Generation

### Store Registry

**Purpose:** Enable cross-file detection of @store classes

**Location:** `src/transpiler/analyzer.rs`

```rust
pub struct StoreRegistry {
    stores: HashMap<String, StoreInfo>,
}

pub struct StoreInfo {
    pub class_name: String,      // e.g., "UserProfile"
    pub has_hilt: bool,          // Detected @hilt annotation?
    pub has_inject: bool,        // Detected @Inject constructor?
    pub package: String,         // Fully qualified package name
}
```

**How it works:**
1. Semantic analyzer scans all `@store` classes during analysis
2. Builds registry with class name → StoreInfo mapping
3. Registry passed to code generator via SemanticInfo
4. Code generator checks registry when instantiating stores

### ViewModel Generation

**When:** File contains `@store` annotation on a class

**What it generates:**

```kotlin
@HiltViewModel  // If @hilt or @Inject detected
class UserProfile @Inject constructor(
    private val repository: ProfileRepository
) : ViewModel() {
    // UiState data class
    data class UiState(
        val name: String = "",
        val email: String = "",
    )

    // StateFlow
    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    // Property accessors
    var name: String
        get() = _uiState.value.name
        set(value) { _uiState.update { it.copy(name = value) } }

    // Derived properties (not in UiState)
    val fullName: String
        get() = "$firstName $lastName"

    // Regular functions
    fun save() {
        name = "Updated"
    }

    // Suspend functions (auto-wrapped)
    fun saveAsync() {
        viewModelScope.launch {
            repository.save(name, email)
        }
    }
}
```

**Key features:**
- **UiState data class**: Contains all `var` properties (mutable state)
- **Derived properties excluded**: Properties with getters are NOT in UiState
- **Property accessors**: Custom getters/setters for reactive updates
- **Suspend auto-wrapping**: `suspend fun` → wrapped in `viewModelScope.launch`
- **Hilt support**: Auto-detects `@Inject` constructor or `@hilt` annotation

### Usage Site Detection

**When:** Component instantiates a store: `val profile = UserProfile()`

**What it generates:**

```kotlin
@Composable
fun ProfileScreen() {
    // Auto-detects store and generates viewModel<T>()
    val profile = viewModel<UserProfile>(key = "profile")
    val uiState by profile.uiState.collectAsState()

    // Component can now use:
    // - uiState.name (for var properties)
    // - profile.fullName (for derived properties)
    // - profile.save() (for functions)
}
```

**With Hilt:**

```kotlin
val profile = hiltViewModel<UserProfile>()  // Uses hiltViewModel instead
```

---

## Context-Aware Code Generation

### Parent Component Awareness

The code generator tracks parent component context to make smart decisions:

**Example: LazyColumn**

```whitehall
<LazyColumn>
  @for (item in items, key = { it.id }) {
    <ItemCard item={item} />
  }
</LazyColumn>
```

**Generates:**

```kotlin
LazyColumn {
    items(items, key = { it.id }) { item ->
        ItemCard(item = item)
    }
}
```

**Instead of:**

```kotlin
LazyColumn {
    items.forEach { item ->  // Wrong! Don't use forEach in LazyColumn
        key(item.id) {
            ItemCard(item = item)
        }
    }
}
```

### Scaffold Children

```whitehall
<Scaffold topBar={<TopAppBar title="Hello" />}>
  <Text>Content</Text>
</Scaffold>
```

**Generates:**

```kotlin
Scaffold(
    topBar = {
        TopAppBar(title = { Text("Hello") })
    }
) { paddingValues ->  // Auto-inject paddingValues lambda
    Text("Content", modifier = Modifier.padding(paddingValues))
}
```

---

## Testing Strategy

### Markdown-Based Tests

**Location:** `tests/transpiler-examples/`

**Format:** Each test is a markdown file with Input and Output sections

**Benefits:**
- Executable tests + living documentation
- Easy to review (side-by-side comparison)
- Never get stale
- Quick reference for developers

**Test Categories:**

**Foundation (00-00b):** 3 tests
- Minimal text
- Text with interpolation
- Single prop

**Core Features (01-06):** 6 tests
- Basic component with props
- If/else conditionals
- For loops with keys
- When expressions
- Data binding (bind:value)
- Lifecycle hooks (onMount)

**Routing (07-08):** 2 tests
- Simple navigation
- Route parameters

**Composition (09-11):** 3 tests
- Import aliases
- Nested components
- Complex state management

**Extended Patterns (12-17):** 6 tests
- LazyColumn
- Box layout
- AsyncImage
- Modifier chains
- Lifecycle cleanup (onDispose)
- Error handling

**Advanced Features (18-26):** 9 tests
- String resources
- Checkbox/Switch
- derivedStateOf
- Colors
- Padding shortcuts
- Escape braces
- Inline lambdas
- Spacer shortcuts
- Function return types

**Stores (27-29):** 3 tests
- Hilt stores
- Hilt explicit
- Store without Hilt

**Component Inline Vars / Phase 1.1 (30-32):** 3 tests
- Basic inline vars → ViewModel generation
- Suspend functions → ViewModel generation
- Derived properties handling

**Total:** 38 tests, 37 passing (1 failing: test 05 missing imports)

### Running Tests

```bash
# Run all transpiler tests
cargo test --test transpiler_examples_test examples

# Run with output
cargo test --test transpiler_examples_test examples -- --nocapture

# Check specific test
cargo test --test transpiler_examples_test examples -- --nocapture | grep "05-data-binding"
```

### Current Test Status

**Passing:** 37/38 (97.4%)

**Failing:**
- `05-data-binding.md` - Missing import detection for:
  - `androidx.compose.material3.Text` (used inside Button)
  - `androidx.compose.ui.text.input.PasswordVisualTransformation` (for type="password")

This is a minor codegen issue with import collection, not a fundamental problem.

---

## Known Limitations

The transpiler currently has the following limitations:

1. **Import Detection for Nested Components** (Test 05 failure)
   - **Issue:** Text component used inside Button child lambda not detected
   - **Issue:** PasswordVisualTransformation for `type="password"` not imported
   - **Impact:** Minor - affects 1 test out of 38
   - **Status:** Needs enhancement to import collection logic
   - **Effort:** 1-2 hours

2. **Lambda Syntax**: Arrow functions like `(e) => { ... }` need conversion to Kotlin lambda syntax `{ e -> ... }`
   - **Status:** Mostly handled by arrow transformation, but complex cases may need manual adjustment

3. **Kotlin String Interpolation**: `${expression}` syntax not yet supported
   - **Workaround:** Use `{expression}` for now (Whitehall interpolation)

4. **Complex Imports**: Destructuring imports like `import { A, B }` are parsed but may need additional handling
   - **Status:** Basic imports work, complex cases may need enhancement

5. **Val in Function Bodies**: Function-scoped `val` declarations are captured as strings, not parsed structurally
   - **Impact:** Minimal - passes through to Kotlin correctly

These limitations primarily affect more complex examples. The core transpiler works well for component-based code.

---

## Design Principles

These principles emerged from building the transpiler incrementally:

### 1. Depth-First, Not Breadth-First
- Complete each test fully before moving to the next
- Don't implement partial features across multiple tests
- Build proper infrastructure when needed, not minimal hacks

### 2. Test-Driven Development (TDD)
- One test at a time, commit after each passes
- Let test expectations guide implementation
- Tests serve as both validation and documentation

### 3. No Overfitting to Examples
- Write general solutions, not test-specific code
- Transformation rules should be extensible
- Don't hard-code test values

### 4. Reasonable Output Over Spec Compliance
- Test expectations are guidelines, not gospel
- Update test files if transpiler output is more reasonable
- Follow Kotlin/Compose conventions

### 5. Build for Readability
- Generated Kotlin should be clean and debuggable
- Proper indentation and formatting
- Meaningful variable names in output

### 6. Commit Often, Document Changes
- Commit after each test passes
- Write detailed commit messages
- Update docs with progress

### 7. Error Detection Over Undefined Behavior
- Fail fast with clear error messages
- Add safeguards (infinite loop detection, position tracking)
- Include position information in parser errors

---

## Next Steps

### Immediate (Minor Bug Fix)

1. **Fix Test 05 Import Detection** (1-2 hours)
   - Enhance import collection to detect Text inside Button child lambdas
   - Add PasswordVisualTransformation import for `type="password"` TextField
   - **Priority:** Low - workaround is to manually add imports

### Future Enhancements (Optional)

1. **Position Tracking for Better Errors**
   - Track line/column in parser
   - Include in error messages
   - Enable inline error markers in web playground
   - **Effort:** 3-4 hours

2. **Custom Monaco Language Definition**
   - Syntax highlighting for Whitehall in web playground
   - IntelliSense/autocomplete suggestions
   - **Effort:** 4-6 hours

3. **Source Maps**
   - Generate source maps for debugging
   - Map Kotlin errors back to .wh files
   - **Effort:** 6-8 hours

4. **Performance Optimization**
   - Parallel transpilation for large projects
   - Incremental compilation (only changed files)
   - **Effort:** 8-12 hours

---

## Key Files Reference

| File | Lines | Purpose |
|------|-------|---------|
| `src/transpiler/mod.rs` | ~100 | Public API, TranspileResult enum |
| `src/transpiler/parser.rs` | ~1200 | Lexer-free recursive descent parser |
| `src/transpiler/ast.rs` | ~200 | AST definitions |
| `src/transpiler/analyzer.rs` | ~300 | Semantic analysis, store registry |
| `src/transpiler/codegen/compose.rs` | ~3500 | Kotlin/Compose code generation |
| `tests/transpiler_examples_test.rs` | ~150 | Test harness |
| `tests/transpiler-examples/*.md` | ~23 files | Test cases |

---

## Related Documentation

- [REF-OVERVIEW.md](./REF-OVERVIEW.md) - Architecture overview
- [REF-STATE-MANAGEMENT.md](./REF-STATE-MANAGEMENT.md) - @store implementation
- [REF-BUILD-SYSTEM.md](./REF-BUILD-SYSTEM.md) - Build commands

---

*Last Updated: 2025-11-06*
*Version: 1.0*
*Status: Nearly Complete (37/38 tests passing, 97.4%)*
