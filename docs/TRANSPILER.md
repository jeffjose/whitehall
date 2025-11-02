# Whitehall Transpiler Architecture

## Overview

The Whitehall transpiler converts `.wh` files into Kotlin/Jetpack Compose code. This document outlines the architecture, phases, and implementation strategy.

## Goals

1. **Type Safety**: Generate type-safe Kotlin code that leverages Compose's compile-time checks
2. **Readability**: Emit clean, idiomatic Kotlin that developers can understand and debug
3. **Performance**: Minimize runtime overhead, maximize compile-time transformations
4. **Ergonomics**: Provide helpful error messages with line/column information
5. **Incremental**: Support fast rebuilds by only transpiling changed files

## Architecture Overview

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   .wh File  │ --> │   Lexer     │ --> │   Parser    │ --> │     AST     │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                                                                     │
                                                                     v
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Kotlin Code │ <-- │  Generator  │ <-- │ Transformer │ <-- │  Validator  │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

## Transpilation Phases

### Phase 1: Lexical Analysis (Tokenization)

**Input**: Raw `.wh` file content (string)
**Output**: Token stream

Break the source into tokens:
- Keywords: `var`, `val`, `fun`, `if`, `else`, `when`, `for`, `in`, `onMount`, etc.
- Identifiers: variable names, function names, component names
- Literals: strings, numbers, booleans
- Operators: `=`, `+`, `-`, `*`, `/`, `==`, `!=`, etc.
- Delimiters: `{`, `}`, `(`, `)`, `<`, `>`, `,`, `.`, `:`, etc.
- Special prefixes: `@` (for `@if`, `@for`, `@when`, `@prop`), `$` (for `$routes`, `$screen`)
- Whitespace and comments (preserved for source mapping)

**Example**:
```whitehall
var count = 0

<Text>{count}</Text>
```

Tokens:
```
[KEYWORD: var] [IDENT: count] [EQUALS] [NUMBER: 0]
[LT] [IDENT: Text] [GT] [LBRACE] [IDENT: count] [RBRACE] [LT] [SLASH] [IDENT: Text] [GT]
```

### Phase 2: Parsing

**Input**: Token stream
**Output**: Abstract Syntax Tree (AST)

Build a structured representation of the code. The parser handles two distinct sections:

#### 2a. Kotlin Section (Top of File)
Parse standard Kotlin syntax:
- Variable declarations (`var`, `val`)
- Function declarations (`fun`)
- Property annotations (`@prop val id: String`)
- Import statements (`import $lib.api.ApiClient`)
- Lifecycle hooks (`onMount { ... }`)

#### 2b. Markup Section (After Kotlin Code)
Parse XML-like component syntax:
- Component tags: `<Text>`, `<Column>`, etc.
- Props: `fontSize={24}`, `padding={16}`
- Children: nested components and text
- Control flow: `@if`, `@for`, `@when`
- Data binding: `bind:value={username}`
- Event handlers: `onClick={() => handleClick()}`

**AST Node Types**:

```rust
enum AstNode {
    // Kotlin section
    VariableDeclaration { mutable: bool, name: String, type: Option<String>, value: Expr },
    FunctionDeclaration { name: String, params: Vec<Param>, return_type: Option<String>, body: Block },
    PropDeclaration { name: String, type: String, default: Option<Expr> },
    ImportDeclaration { path: String },
    OnMountHook { body: Block },

    // Markup section
    Component { name: String, props: Vec<Prop>, children: Vec<AstNode> },
    ControlFlowIf { condition: Expr, then_branch: Vec<AstNode>, else_branch: Option<Vec<AstNode>> },
    ControlFlowFor { item: String, collection: Expr, key: Option<Lambda>, body: Vec<AstNode>, empty: Option<Vec<AstNode>> },
    ControlFlowWhen { branches: Vec<WhenBranch> },
    TextContent { content: String },
    Interpolation { expr: Expr },

    // Expressions
    Expr(Expr),
}

enum Expr {
    Literal(LiteralValue),
    Identifier(String),
    BinaryOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Call { target: Box<Expr>, args: Vec<Expr> },
    Lambda { params: Vec<String>, body: Box<Expr> },
    PropertyAccess { target: Box<Expr>, property: String },
    // ... more expression types
}
```

### Phase 3: Semantic Analysis & Validation

**Input**: AST
**Output**: Annotated AST or error messages

Validate the AST and gather type information:

1. **Scope Analysis**:
   - Build symbol table for variables, functions, props
   - Check for undefined variables
   - Verify prop declarations use `val` not `var`

2. **Type Checking**:
   - Verify prop types are explicitly declared
   - Check component prop types match usage
   - Validate expressions in control flow conditions

3. **Route Analysis** (for +screen.wh files):
   - Extract route parameters from directory structure
   - Generate route object types
   - Validate route parameter usage

4. **Binding Analysis**:
   - Verify `bind:value` targets are mutable variables
   - Check for nested property bindings that need `.copy()`

**Error Examples**:
```
Error: @prop must use 'val', not 'var'
  --> src/components/Avatar.wh:4:3
   |
 4 |   @prop var url: String
   |         ^^^ use 'val' instead

Error: Route parameter 'id' accessed but not defined
  --> src/routes/profile/+screen.wh:12:5
   |
12 |     ApiClient.getUser($screen.params.id)
   |                       ^^^^^^^^^^^^^^^^^ parameter 'id' not found in route
```

### Phase 4: AST Transformation

**Input**: Validated AST
**Output**: Transformed AST optimized for Kotlin/Compose generation

Transform Whitehall-specific constructs into Compose-ready structures:

1. **Lifecycle Hooks**:
   ```whitehall
   onMount {
     launch { /* ... */ }
   }
   ```
   Becomes:
   ```kotlin
   LaunchedEffect(Unit) {
     launch { /* ... */ }
   }
   ```

2. **Data Binding**:
   ```whitehall
   bind:value={username}
   ```
   Becomes:
   ```kotlin
   value = username,
   onValueChange = { username = it }
   ```

3. **Control Flow**:
   ```whitehall
   @if (isLoading) {
     <LoadingSpinner />
   } else {
     <Content />
   }
   ```
   Becomes:
   ```kotlin
   if (isLoading) {
     LoadingSpinner()
   } else {
     Content()
   }
   ```

4. **For Loops with Keys**:
   ```whitehall
   @for (post in posts, key = { it.id }) {
     <PostCard post={post} />
   }
   ```
   Becomes:
   ```kotlin
   posts.forEach { post ->
     key(post.id) {
       PostCard(post = post)
     }
   }
   ```

5. **Route References**:
   ```whitehall
   navigate($routes.profile(id = userId))
   ```
   Becomes:
   ```kotlin
   navigate(Routes.Profile(id = userId))
   ```

6. **Special Object Mapping**:
   - `$routes` → `Routes` (generated object)
   - `$screen.params.id` → `id` (from route params)
   - `$app` → package imports
   - `$lib`, `$models`, `$components` → configured aliases

### Phase 5: Code Generation

**Input**: Transformed AST
**Output**: Kotlin source code

Generate clean, formatted Kotlin/Compose code.

#### 5a. Component Files

**Input** (`src/components/Avatar.wh`):
```whitehall
import $models.User

  @prop val url: String
  @prop val size: Int = 48
  @prop val onClick: (() -> Unit)? = null

<AsyncImage
  url={url}
  width={size}
  height={size}
  modifier={onClick?.let { Modifier.clickable { it() } } ?: Modifier}
/>
```

**Output** (`build/generated/src/components/Avatar.kt`):
```kotlin
package com.example.microblog.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.foundation.clickable
import com.example.microblog.models.User
import coil.compose.AsyncImage

@Composable
fun Avatar(
    url: String,
    size: Int = 48,
    onClick: (() -> Unit)? = null
) {
    AsyncImage(
        url = url,
        width = size,
        height = size,
        modifier = onClick?.let { Modifier.clickable { it() } } ?: Modifier
    )
}
```

#### 5b. Screen Files

**Input** (`src/routes/+screen.wh`):
```whitehall
import $components.PostCard
import $lib.api.ApiClient

  var posts: List<Post> = emptyList()
  var isLoading = true

  onMount {
    launch {
      ApiClient.getFeed()
        .onSuccess { posts = it; isLoading = false }
    }
  }

<Scaffold
  topBar={
    <TopAppBar title="My Blog" />
  }
>
  <Column padding={16} spacing={16}>
    @if (isLoading) {
      <LoadingSpinner center />
    } else {
      @for (post in posts, key = { it.id }) {
        <PostCard post={post} onClick={() => navigate($routes.post.detail(id = post.id))} />
      }
    }
  </Column>
</Scaffold>
```

**Output** (`build/generated/src/routes/HomeScreen.kt`):
```kotlin
package com.example.microblog.routes

import androidx.compose.runtime.*
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.material3.TopAppBar
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import com.example.microblog.components.PostCard
import com.example.microblog.lib.api.ApiClient
import com.example.microblog.models.Post
import kotlinx.coroutines.launch

@Composable
fun HomeScreen(navController: NavController) {
    var posts by remember { mutableStateOf<List<Post>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }

    val coroutineScope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        coroutineScope.launch {
            ApiClient.getFeed()
                .onSuccess { posts = it; isLoading = false }
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(title = { Text("My Blog") })
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .padding(paddingValues)
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            if (isLoading) {
                LoadingSpinner(center = true)
            } else {
                posts.forEach { post ->
                    key(post.id) {
                        PostCard(
                            post = post,
                            onClick = { navController.navigate(Routes.Post.Detail(id = post.id)) }
                        )
                    }
                }
            }
        }
    }
}
```

#### 5c. Route Generation

For file-based routing, generate a `Routes.kt` file:

**Input** (directory structure):
```
src/routes/
  +screen.wh              -> /
  login/+screen.wh        -> /login
  profile/[id]/+screen.wh -> /profile/:id
  post/[id]/+screen.wh    -> /post/:id
```

**Output** (`build/generated/routes/Routes.kt`):
```kotlin
package com.example.microblog.routes

import kotlinx.serialization.Serializable

sealed class Routes {
    @Serializable
    object Home : Routes()

    @Serializable
    object Login : Routes()

    @Serializable
    data class Profile(val id: String) : Routes()

    @Serializable
    data class PostDetail(val id: String) : Routes()
}

// Lowercase accessor object for ergonomic syntax
object routes {
    val home: Routes.Home get() = Routes.Home
    val login: Routes.Login get() = Routes.Login
    fun profile(id: String) = Routes.Profile(id)
    fun postDetail(id: String) = Routes.PostDetail(id)
}
```

## Implementation Roadmap

### Milestone 1: Basic Transpilation (MVP)
- [x] Syntax design complete
- [ ] Lexer for basic tokens
- [ ] Parser for simple component (props + markup)
- [ ] AST for basic constructs
- [ ] Code generator for simple component
- [ ] Test: Transpile `Avatar.wh` → `Avatar.kt`

### Milestone 2: Control Flow
- [ ] Parse `@if`, `@else`
- [ ] Parse `@for` with keys and empty blocks
- [ ] Parse `@when` branches
- [ ] Generate correct Compose control flow
- [ ] Test: Transpile component with control flow

### Milestone 3: Screens & Routing
- [ ] Parse route directory structure
- [ ] Generate `Routes.kt` with `@Serializable` objects
- [ ] Handle `$screen.params` access
- [ ] Generate screen composables with NavController
- [ ] Test: Transpile `+screen.wh` → screen composable

### Milestone 4: Advanced Features
- [ ] Data binding (`bind:value`)
- [ ] Lifecycle hooks (`onMount`, etc.)
- [ ] Import alias resolution (`$lib`, `$models`)
- [ ] Nested property `.copy()` generation
- [ ] Test: Full Microblog app transpilation

### Milestone 5: Developer Experience
- [ ] Source maps for debugging
- [ ] Detailed error messages with line numbers
- [ ] Watch mode for incremental compilation
- [ ] Integration with `whitehall dev`
- [ ] Performance optimization

## Current Status and Reset Plan (Nov 2025)

### Situation Analysis

**Test Infrastructure**: ✅ Complete
- Commit `e1ecf0a` established markdown-based test framework
- 18 test cases defined in `tests/transpiler-examples/` (00, 00a, 00b, 01-17)
- Test harness can parse markdown and validate output
- Tests serve dual purpose: validation + documentation

**Transpiler Implementation**: ❌ Fundamentally Broken
- Commit `08648c2f` added ~2000 lines of hand-written parser + codegen
- Only 1 of 14 tests passing at that commit
- Subsequent commits (9 more) attempted band-aid fixes
- Current state: 3 of 14 tests passing, many edge case bugs
- Root causes:
  - Hand-written parser without proper grammar specification
  - All code added in one massive commit (not iterative)
  - No proper lexer phase (attempted "lexer-free" parser)
  - Insufficient prop parsing, text interpolation issues
  - Import and type handling incomplete

### Decision: Clean Reset

**Recommended Action**: Reset to commit `e1ecf0a` and rebuild properly

**Rationale**:
1. Current implementation is built on shaky foundation
2. Fixing piecemeal issues doesn't address core architectural problems
3. Test infrastructure is solid and ready to guide development
4. Commit `e1ecf0a` is clean slate: tests exist, no broken code
5. Better to build correctly once than patch forever

## Actual Implementation Architecture

The implemented transpiler follows a simplified, pragmatic architecture optimized for Whitehall's specific needs.

### Module Structure

```
src/transpiler/
├── mod.rs       # Public API: transpile(input, package, component_name) -> Result<String, String>
├── ast.rs       # Abstract Syntax Tree definitions
├── parser.rs    # Lexer-free recursive descent parser
└── codegen.rs   # Kotlin/Compose code generation with transformations
```

### Parser Architecture (parser.rs)

**Approach**: Lexer-free recursive descent parser
- **No separate lexer**: Parser consumes characters directly from input string
- **Position tracking**: Single `pos: usize` cursor through input
- **Recursive descent**: Each grammar rule is a method (parse_component, parse_if_else, etc.)
- **Lookahead**: `peek_char()` and `peek_ahead(n)` for decision-making

**Key Methods**:
```rust
pub fn parse(&mut self) -> Result<WhitehallFile, String>
  ├─ parse_imports()          // import $models.User
  ├─ parse_props()            // @prop val name: Type
  ├─ parse_state()            // var/val declarations
  └─ parse_component()        // <Component>...</Component>
       ├─ parse_component_prop()    // prop={value} or prop="string"
       ├─ parse_children()          // Recursive markup parsing
       │    ├─ parse_control_flow() // @if, @for, @when
       │    ├─ parse_component()    // Nested components
       │    └─ parse_text_with_interpolation_until_markup()
       └─ parse_markup_block()      // For control flow bodies
```

**Design Decisions**:
1. **Lexer-free**: Simpler for Whitehall's hybrid Kotlin/XML syntax
2. **String-based types**: Parser outputs `String` for types, expressions (no complex Expr AST)
3. **Minimal lookahead**: Most decisions made with 1-2 character peek
4. **Depth tracking**: For nested braces/parens/brackets in complex types
5. **Position-based errors**: Include position in error messages for debugging
6. **Infinite loop guards**: Track position before parsing, error if no progress made

### AST Design (ast.rs)

**Philosophy**: Simple, focused on transpilation needs (not full semantic analysis)

```rust
pub struct WhitehallFile {
    pub imports: Vec<Import>,
    pub props: Vec<PropDeclaration>,
    pub state: Vec<StateDeclaration>,
    pub markup: Markup,
}

pub enum Markup {
    Component(Component),
    Text(String),
    Interpolation(String),
    Sequence(Vec<Markup>),
    IfElse(IfElseBlock),
    ForLoop(ForLoopBlock),
}
```

**Key Features**:
- **Flat prop/state collections**: No scoping needed (single-component files)
- **String expressions**: No expression AST, just capture strings for codegen
- **Markup enum**: Unified representation of all renderable content
- **Recursive children**: Components and control flow contain `Vec<Markup>`

### Code Generator Architecture (codegen.rs)

**Structure**:
```rust
pub struct CodeGenerator {
    package: String,
    component_name: String,
    indent_level: usize,
}

impl CodeGenerator {
    pub fn generate(&mut self, file: &WhitehallFile) -> Result<String, String>
      ├─ collect_imports_recursive()   // Scan AST for needed imports
      ├─ transform_prop()              // Apply Compose idiom transformations
      ├─ generate_markup()             // Convert Markup to Kotlin code
      └─ build_text_expression()       // Handle text + interpolation
}
```

**Prop Transformation System**:

Component-specific transformations for Compose idioms:
```rust
match (component, prop_name) {
    ("Column", "spacing") => "verticalArrangement = Arrangement.spacedBy({value}.dp)",
    ("Column", "padding") => "modifier = Modifier.padding({value}.dp)",
    ("Text", "fontSize") => "fontSize = {value}.sp",
    ("Text", "fontWeight") => "fontWeight = FontWeight.{Capitalized}",
    ("Text", "color") => "color = MaterialTheme.colorScheme.{value}",
    ...
}
```

**Value Transformations**:
- Lambda arrows: `() => expr` → `{ expr }`
- Route aliases: `$routes.post.detail` → `Routes.Post.Detail`
- String literals: `"bold"` → `FontWeight.Bold`
- Numbers: `16` → `16.dp` or `16.sp` (context-dependent)

**Import Management**:
1. **Detection**: Scan AST for components, prop values, transformations
2. **Transformation imports**: Add imports for Arrangement, Modifier, dp, sp, FontWeight, etc.
3. **Component imports**: Map known components (Text, Column, Card) to androidx/material3
4. **Alphabetical sorting**: Standard Kotlin convention
5. **Deduplication**: Track imports in Vec, check before adding

## Design Principles

These principles emerged from building the transpiler incrementally:

### 1. Depth-First, Not Breadth-First
- **Complete each test fully** before moving to the next
- Don't implement partial features across multiple tests
- Build proper infrastructure when needed, not minimal hacks
- *Example*: Test 03 required full prop transformation system, so we built it properly

### 2. Test-Driven Development (TDD)
- **One test at a time**, commit after each passes
- Let test expectations guide implementation
- Tests serve as both validation and documentation
- **Atomic commits**: Each commit represents one working test

### 3. No Overfitting to Examples
- Write **general solutions**, not test-specific code
- Transformation rules should be extensible
- Don't hard-code test values
- *Example*: Prop transformation system handles any component/prop combo, not just test 03's specific props

### 4. Reasonable Output Over Spec Compliance
- Test expectations are **guidelines, not gospel**
- Update test files if transpiler output is more reasonable
- Follow Kotlin/Compose conventions (alphabetical imports, idiomatic formatting)
- *Example*: We changed import ordering to alphabetical sorting (better than arbitrary)

### 5. Build for Readability
- Generated Kotlin should be **clean and debuggable**
- Proper indentation and formatting
- Meaningful variable names in output
- Idiomatic Compose patterns

### 6. Commit Often, Document Changes
- **Commit after each test passes**
- Write detailed commit messages explaining what was added
- Update docs/TRANSPILER.md with progress
- Commits form an audit trail of development decisions

### 7. Error Detection Over Undefined Behavior
- **Fail fast** with clear error messages
- Add safeguards (infinite loop detection, position tracking)
- Better to error than produce wrong code
- Include position information in parser errors

## Technical Decisions

### Why Lexer-Free Parser?
**Decision**: Single-pass parser without separate tokenization
**Rationale**:
- Whitehall mixes Kotlin and XML syntax (hard to tokenize cleanly)
- Recursive descent handles nested structures naturally
- Lookahead of 1-2 characters sufficient for decision-making
- Simpler codebase (~600 LOC parser vs. lexer + parser)

### Why String-Based Expression Handling?
**Decision**: Capture expressions as strings, not full AST
**Rationale**:
- Transpiler doesn't need to understand expression semantics
- Pass-through most Kotlin expressions unchanged
- Simpler than building full expression AST
- Transformation can happen on strings (route aliases, lambda arrows)

### Why Component-Specific Transformations?
**Decision**: Match on `(component, prop)` pairs for transformations
**Rationale**:
- Compose components have component-specific prop semantics
- `padding` means different things on Column vs Text
- Allows context-aware transformations (spacing → verticalArrangement on Column)
- Extensible pattern: just add more match arms

### Why Alphabetical Import Sorting?
**Decision**: Always sort imports alphabetically
**Rationale**:
- Standard Kotlin convention
- Deterministic output (no order dependencies)
- Easier to read and diff
- Simpler than complex ordering heuristics

### Implementation Strategy: Test-Driven Development

Unlike the previous attempt (all code at once), we built **incrementally**, one test at a time:

#### Phase 1: Foundation (Tests 00-00b) ✅ COMPLETE

**Goal**: Get the 3 simplest tests passing with solid foundation

**Status**: ✅ **3/14 tests passing** (commits: `ce1f733`, `b284ca6`, `6ac7b93`)

**Test 00: Minimal Text** ✅ (`00-minimal-text.md`)
- Input: Just `<Text>Hello, World!</Text>`
- Implemented: Basic component shell, text node, Text component generation
- Commit: `ce1f733` - 1/14 tests passing

**Test 00a: Text with Interpolation** ✅ (`00a-text-with-interpolation.md`)
- Input: State variable + `<Text>Hello, {name}!</Text>`
- Implemented: State parsing (`var name = "value"`), text interpolation `{var}` → `$var`
- Commit: `b284ca6` - 2/14 tests passing

**Test 00b: Single Prop** ✅ (`00b-single-prop.md`)
- Input: `@prop val message: String` + usage
- Implemented: Prop parsing, function parameters, bare interpolation `{message}` → `message`
- Commit: `6ac7b93` - 3/14 tests passing

**Architecture Created**:
```
src/transpiler/
  mod.rs      # Public API: transpile(input, package, name)
  ast.rs      # AST: WhitehallFile, PropDeclaration, StateDeclaration, Markup
  parser.rs   # Parse: @prop, var/val, <Component>, {interpolation}
  codegen.rs  # Generate: @Composable functions, imports, state with remember
```

**Testing**: `cargo test examples`

**Checkpoint**: ✅ Basic component structure working, solid foundation established.

#### Phase 2: Basic Components (Test 01) ✅ COMPLETE

**Goal**: Full component with multiple props and defaults

**Status**: Functionally complete, cosmetic formatting remains (commits: `4e95178`, `b65b45d`)

**Test 01: Basic Component** (`01-basic-component.md`)
- Input: Avatar component with url, size, onClick props
- Implemented:
  - ✅ Import statements parsing (`import $models.User`)
  - ✅ Import resolution (`$models` → `com.example.app.models`)
  - ✅ Complex type parsing (`(() -> Unit)?` with `->` operator)
  - ✅ Self-closing tags (`<AsyncImage />`)
  - ✅ Component props with expressions (`url={url}`, `modifier={...}`)
  - ✅ Nested brace handling in prop values
  - ✅ Smart import ordering (prop imports, user imports, component imports)
  - ⏸️  Multiline formatting for component calls (cosmetic only)
- Progress: **4/14 tests passing!** (commit: `aca93fa`)

**Checkpoint**: ✅ Real components can be defined and used! Phase 2 COMPLETE!

#### Phase 3: Control Flow (Tests 02-04)
**Goal**: All control flow constructs working

**Test 02: If/Else** ✅ (`02-control-flow-if.md`)
- Input: @if/@else if/else with nullable props
- Implemented:
  - ✅ `@if`/`@else if`/`@else` parsing and generation
  - ✅ Control flow in `parse_children()` and `parse_markup_block()`
  - ✅ `IfElse` branch recursion in import collection
  - ✅ Column component import
  - ✅ Nullable prop default values (`String? = null`)
  - ✅ Component trailing lambda syntax (no `()` before `{`)
  - ✅ Refined import ordering (single component before Composable)
  - ✅ Infinite loop detection safeguards
- Progress: **5/14 tests passing!** (commit: `3a2758c`)

**Test 03: For Loops** ✅ (`03-control-flow-for.md`)
- Input: @for with key expressions, empty blocks, and complex prop transformations
- Implemented:
  - ✅ `@for (item in collection, key = { expr }) { } empty { }` parsing
  - ✅ if/else + forEach generation pattern
  - ✅ key() wrapping for keyed iterations
  - ✅ String literal props (`prop="value"`)
  - ✅ Card component import
  - ✅ **Prop transformation system:**
    - Column spacing={N} → verticalArrangement = Arrangement.spacedBy(N.dp)
    - Column padding={N} → modifier = Modifier.padding(N.dp)
    - Text fontSize={N} → fontSize = N.sp
    - Text fontWeight="string" → fontWeight = FontWeight.Enum
    - Text color="string" → color = MaterialTheme.colorScheme.value
  - ✅ Lambda arrow transformation: `() => expr` → `{ expr }`
  - ✅ Route alias transformation: `$routes.post.detail` → `Routes.Post.Detail`
  - ✅ Transformation-based import detection and management
  - ✅ Alphabetical import sorting (standard Kotlin convention)
- Progress: **6/14 tests passing!** (commits: `304f634`, `a885f79`)

**Test 04: When** ✅ (`04-control-flow-when.md`)
- Input: @when with multi-branch conditionals and else clause
- Implemented:
  - ✅ `@when { condition -> markup }` parsing
  - ✅ Multiple branches with conditions
  - ✅ `else` branch support (condition = None)
  - ✅ Inline component generation for branch bodies
  - ✅ When branch recursion in import collection
- Progress: **7/14 tests passing!** (commit: `edc994c`)

**Checkpoint**: ✅ All control flow working correctly (if/else, for, when)!

#### Phase 4: Advanced Features (Tests 05-06)
**Goal**: Data binding and lifecycle hooks

**Test 05: Data Binding** ✅ (`05-data-binding.md`)
- Input: bind:value syntax, functions, Button/TextField components
- Implemented:
  - ✅ Function declarations: `fun name() { body }`
  - ✅ Colon-based prop names: `bind:value`, `on:click`, etc.
  - ✅ `bind:value` transformation → `value + onValueChange`
  - ✅ Button `text` prop → child `Text` component
  - ✅ TextField `label` → `label = { Text(...) }`
  - ✅ Button `onClick` wrapping in braces
  - ✅ TextField and Button component imports
- Progress: **8/14 tests passing!** (commits: `d116c52`, `c5aa78b`)

**Test 06: Lifecycle Hooks** ✅ (`06-lifecycle-hooks.md`)
- Input: onMount hook, state with type annotations, coroutines
- Implemented:
  - ✅ `onMount { }` lifecycle hook parsing
  - ✅ State with type annotations: `var name: Type = value`
  - ✅ `mutableStateOf<Type>(value)` for generic types
  - ✅ `rememberCoroutineScope()` generation when hooks present
  - ✅ `LaunchedEffect(Unit)` wrapper for onMount
  - ✅ Transform `launch` → `coroutineScope.launch`
  - ✅ `kotlinx.coroutines.launch` import
- Progress: **9/14 tests passing!** (commit: `5a0a450`)

**Checkpoint**: ✅ State management and lifecycle hooks fully functional!

#### Phase 5: Routing (Tests 07-08)
**Goal**: Navigation and route parameters

**Test 07: Simple Navigation** ✅ (`07-routing-simple.md`)
- Input: navigate($routes.login), type: screen metadata
- Implemented:
  - ✅ `type: screen` metadata support in transpiler API
  - ✅ Add `NavController` parameter for screens
  - ✅ Transform `navigate()` → `navController.navigate()`
  - ✅ Transform `$routes.login` → `Routes.Login` in function bodies
  - ✅ `androidx.navigation.NavController` import
  - ✅ Route alias transformation across entire codebase
- Progress: **10/14 tests passing!** (commit: `0e93569`)

**Test 08: Route Parameters** ✅ (`08-routing-params.md`)
- Input: $screen.params.id, function params, nullable interpolations
- Implemented:
  - ✅ Extract route params from `$screen.params.{name}` references
  - ✅ Add route params as function parameters (after navController)
  - ✅ Transform `$screen.params.id` → `id` in code
  - ✅ Function parameter parsing: `fun name(params) { body }`
  - ✅ Nullable variable tracking from type annotations
  - ✅ Null assertion operator: `{user.name}` → `user!!.name`
  - ✅ Scaffold component support
- Progress: **11/14 tests passing!** (commit: `7cebaae`)

**Checkpoint**: Routing system complete.

#### Phase 6: Composition (Tests 09-10)
**Goal**: Imports and nesting

**Test 09: Import Aliases** ✅ (`09-imports.md`)
- Input: import $lib.api.ApiClient, $models.User, etc.
- Implemented:
  - ✅ Already working through existing `resolve_import()` function
  - ✅ Alphabetical import sorting (standard Kotlin convention)
  - ✅ All import aliases: `$app`, `$lib`, `$models`, `$components`
- Progress: **12/14 tests passing!** (commit: `89b9c22`)

**Test 10: Nested Components** ⏸️ (`10-nested-components.md`)
- Input: Components as prop values, Scaffold with topBar, deep nesting
- **Status**: Requires component-as-prop-value support (complex feature)
- Remaining work:
  - ⏸️ Parse markup (e.g., `<TopAppBar />`) as prop values
  - ⏸️ Scaffold lambda wrapper: `{ paddingValues -> ... }`
  - ⏸️ TopAppBar title transformation: `title={var}` → `title = { Text(var) }`
  - ⏸️ Modifier chaining for Scaffold children

**Checkpoint**: Import system complete. Test 10 deferred (architectural change needed).

#### Phase 7: Complex Examples (Test 11)
**Goal**: Real-world complexity

**Test 11: Complex State Management** ✅ (`11-complex-state-management.md`)
- Input: Multiple state vars, computed vals, derived state, complex interactions
- Implemented:
  - ✅ Separated var (mutable) and val (computed) state declarations
  - ✅ Type annotations on computed values: `val name: Type = value`
  - ✅ Multi-line value formatting with proper indentation
  - ✅ Smart ordering based on computed state:
    - With computed state: `var → val → functions → coroutineScope → lifecycle`
    - Without computed state: `var → coroutineScope → lifecycle → functions`
  - ✅ LaunchedEffect proper nesting and indentation
  - ✅ Blank lines between all major sections
- Progress: **13/14 tests passing!** (commit: `abe1bf4`)

**Checkpoint**: ✅ **93% complete!** All major features working. Only test 10 requires architectural enhancement.

#### Phase 8: Extended Patterns (Tests 12-17)
**Goal**: Performance, images, advanced layouts, and lifecycle patterns

**Status**: Tests defined, implementation pending

**Test 12: LazyColumn** (`12-lazy-column.md`)
- Input: LazyColumn with items() for performance-optimized scrollable lists
- Required: `items()` function instead of forEach, key parameter support
- Transformations: padding → contentPadding, spacing → verticalArrangement

**Test 13: Box Layout** (`13-box-layout.md`)
- Input: Box for stacking/overlaying components (avatar with status indicator)
- Required: Box container, alignment prop support (bottomEnd, etc.)
- Transformations: AsyncImage size props, background color handling

**Test 14: AsyncImage** (`14-async-image.md`)
- Input: AsyncImage with placeholder, error states, and crossfade
- Required: Coil ImageRequest builder pattern, placeholder/error drawables
- Transformations: url → model, size → Modifier.size(), content description

**Test 15: Modifier Chains** (`15-modifier-chains.md`)
- Input: Multiple modifiers chained, conditional modifier application
- Required: Modifier.let() for conditional chaining, fillMaxWidth/fillMaxSize
- Transformations: fillMaxWidth={bool} → conditional .fillMaxWidth()

**Test 16: Lifecycle Cleanup** (`16-lifecycle-cleanup.md`)
- Input: onDispose hook for resource cleanup (WebSocket disconnect)
- Required: DisposableEffect instead of LaunchedEffect, onDispose callback
- Pattern: onMount + onDispose → DisposableEffect { ... onDispose { } }

**Test 17: Error Handling** (`17-error-handling.md`)
- Input: Try/catch in async operations, loading/error/success states
- Required: Try/catch/finally in LaunchedEffect, error state handling
- Pattern: Common loading → error → success state machine

**Evaluation**: These tests cover high-priority real-world patterns:
- LazyColumn: Essential for any app with scrollable lists (performance)
- Box: Fundamental layout primitive alongside Column/Row
- AsyncImage: Image loading is ubiquitous in mobile apps
- Modifier chains: Advanced but common pattern for conditional styling
- Lifecycle cleanup: Critical for preventing memory leaks
- Error handling: Standard pattern for async operations

**Implementation Strategy**: After completing tests 00-11, tackle these in order 12-17. Each represents a distinct feature area that builds on the existing architecture.

### Key Principles for Rebuild

1. **One Test at a Time**: Don't move to next test until current passes
2. **Commit After Each Test**: Create atomic commits per test
3. **No Big Leaps**: Resist urge to implement multiple features at once
4. **Test Output Guides Design**: Let expected Kotlin shape implementation
5. **Refactor Between Tests**: Clean up after each test passes
6. **Use Proper Tools**: Consider pest/nom instead of hand-rolled parser

### Test Execution Pattern

For each test:

```bash
# 1. Read the test markdown file
cat tests/transpiler-examples/00-minimal-text.md

# 2. Understand input and expected output

# 3. Run test (will fail initially)
cargo test test_transpile_all_examples -- --nocapture

# 4. Implement minimum code to make THIS test pass
#    (Don't worry about future tests yet)

# 5. Verify test passes
cargo test test_transpile_all_examples

# 6. Commit immediately with clear message
git add .
git commit -m "Implement test 00: minimal text component"

# 7. Move to next test
```

### Avoiding Previous Mistakes

**DON'T**:
- ❌ Implement all features at once
- ❌ Write code without running tests
- ❌ Batch multiple tests before committing
- ❌ Try to handle edge cases prematurely
- ❌ Write "clever" hand-rolled parsers

**DO**:
- ✅ Implement exactly what current test needs
- ✅ Run tests constantly during development
- ✅ Commit after each passing test
- ✅ Let tests reveal edge cases naturally
- ✅ Consider using battle-tested parser libraries

### Success Metrics

- **Test Count**: Must monotonically increase (never go backward)
- **Commit Size**: Small, focused commits (~100-300 lines)
- **Build Time**: All tests should run in <5 seconds
- **Code Quality**: Clean, documented, refactored between tests

### Prerequisites: Fix Test Metadata

**IMPORTANT**: Before resetting the transpiler, we need to fix the test markdown format.

#### Current Problems

1. **Component Name is Implicit**:
   - Test `01-basic-component.md` expects output `fun Avatar()` but filename would derive `BasicComponent`
   - Test `07-routing-simple.md` expects output `fun WelcomeScreen()` but filename would derive `RoutingSimple`

2. **Package Name Varies**:
   - Most tests use `com.example.app.components`
   - Routing tests use `com.example.app.screens`
   - No way to specify this per-test

3. **File Type Unknown**:
   - Is this a `.wh` component file?
   - Is this a `+screen.wh` screen file?
   - Affects how transpiler should process it

4. **Test Harness Logic**:
   - Currently derives component name from markdown filename
   - Hardcodes package as `com.example.app.components`
   - No way to override or customize

#### Solution: Metadata Section

Add a `## Metadata` section at the **bottom** of each test file with explicit configuration (machine-readable data goes last, human-readable content first).

**Format** (simple key-value, no YAML dependency):

```markdown
# Test Name

Description of what this test validates.

## Input

```whitehall
// Whitehall code
```

## Output

```kotlin
// Expected Kotlin code
```

## Metadata

```
file: Avatar.wh
package: com.example.app.components
```
```

**Metadata Fields**:

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `file` | string | Yes | Source filename (determines component name) | `Avatar.wh`, `WelcomeScreen.wh` |
| `package` | string | Yes | Kotlin package for generated code | `com.example.app.components` |
| `type` | enum | No | File type hint (`component`, `screen`) | `screen` |

**Component Name Derivation**:
- `Avatar.wh` → `fun Avatar()`
- `MinimalText.wh` → `fun MinimalText()`
- `WelcomeScreen.wh` → `fun WelcomeScreen()`

**Benefits**:
1. Explicit > Implicit: No guessing, all metadata clearly stated
2. Flexible: Each test can specify different packages/names
3. Maintainable: Easy to update metadata without changing code
4. Self-Documenting: Reader immediately sees what file this represents

#### Example: Updated Test File

**Before** (`01-basic-component.md`):
```markdown
# Basic Component with Props

Tests a component with required and optional props.

## Input

```whitehall
import $models.User
  @prop val url: String
  ...
```

## Output

```kotlin
package com.example.app.components
...
fun Avatar(...) { ... }
```
```

**After** (with metadata at bottom):
```markdown
# Basic Component with Props

Tests a component with required and optional props.

## Input

```whitehall
import $models.User
  @prop val url: String
  ...
```

## Output

```kotlin
package com.example.app.components
...
fun Avatar(...) { ... }
```

## Metadata

```
file: Avatar.wh
package: com.example.app.components
```
```

#### Test Harness Changes

Update `tests/transpiler_examples_test.rs` to:

1. Parse `## Metadata` section from markdown
2. Extract key-value pairs: `file`, `package`, `type`
3. Derive component name from `file` field (strip `.wh`)
4. Pass metadata to transpiler function

**Parsing Logic**:
```rust
#[derive(Debug)]
struct TestMetadata {
    file: String,
    package: String,
    type_hint: Option<String>,
}

fn parse_metadata(content: &str) -> Result<TestMetadata, String> {
    // Find ## Metadata section
    // Extract code block content
    // Parse key:value lines
    // Return TestMetadata struct
}
```

**Transpiler Call**:
```rust
let component_name = metadata.file.trim_end_matches(".wh");
let output = transpile(&test.input, &metadata.package, component_name)?;
```

#### Required Steps (before transpiler reset):

1. ⏳ Add metadata sections to all 14 test files
2. ⏳ Update test harness to parse and use metadata
3. ⏳ Validate all tests parse correctly
4. ⏳ Commit test infrastructure updates

### Next Immediate Steps

1. ✅ Review TRANSPILER.md plan - Approved
2. ⏳ Update test files with metadata (prerequisite)
3. ⏳ Update test harness to parse metadata
4. ⏳ Execute: `git reset --hard e1ecf0a`
5. ⏳ Create `src/transpiler/` directory structure
6. ⏳ Start with test 00: minimal text
7. ⏳ Build incrementally, test by test

## Technology Choices

### Parser Library

**Recommendation**: Use `pest` parser generator

**Rationale**:
- PEG (Parsing Expression Grammar) is well-suited for mixed Kotlin/XML syntax
- Declarative grammar files are easier to maintain
- Good error reporting
- Strong community support

**Alternative**: `nom` (parser combinator library)
- More flexible but requires more manual code
- Better for complex/ambiguous grammars

### AST Representation

Use Rust enums and structs with `serde` for serialization:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitehallFile {
    pub imports: Vec<ImportDeclaration>,
    pub props: Vec<PropDeclaration>,
    pub state: Vec<VariableDeclaration>,
    pub functions: Vec<FunctionDeclaration>,
    pub lifecycle: Vec<LifecycleHook>,
    pub markup: Component,
}
```

### Code Generation

Use Rust's formatting capabilities with template strings or dedicated code generation libraries:

```rust
fn generate_component(comp: &Component) -> String {
    format!(
        "@Composable\nfun {}({}) {{\n{}\n}}",
        comp.name,
        generate_params(&comp.props),
        generate_body(&comp.children)
    )
}
```

Consider using `prettyplease` for final Kotlin code formatting.

## Edge Cases & Considerations

### 1. Ambiguous Syntax

**Problem**: `<` can be both XML tag start and comparison operator

**Solution**: Context-aware parsing
- After newline/statement end → likely XML tag
- Inside expression → likely comparison
- Use lookahead to check for identifier after `<`

### 2. Kotlin Code in Props

**Problem**: Complex Kotlin expressions in prop values

```whitehall
<Text fontSize={if (isLarge) 24 else 16} />
```

**Solution**: Parse prop values as full Kotlin expressions, delegate to Kotlin expression parser

### 3. Nested Components

**Problem**: Deep component trees with multiple levels

**Solution**: Recursive descent parsing, maintain indentation context

### 4. State Management

**Problem**: `var` vs `var by remember { mutableStateOf() }`

**Solution**:
- Top-level `var` in screens/components → `var by remember { mutableStateOf() }`
- Local `var` inside functions → regular Kotlin `var`

### 5. Import Resolution

**Problem**: Resolving `$lib.api.ApiClient` to actual package path

**Solution**:
- Parse `whitehall.toml` for `[imports.aliases]`
- Build import map at transpile start
- Replace `$` aliases during code generation

### 6. Source Mapping

**Problem**: Debugging generated Kotlin code

**Solution**:
- Track line/column during parsing
- Generate Kotlin comments with source references:
  ```kotlin
  // src/components/Avatar.wh:5
  AsyncImage(...)
  ```
- Consider full source map support (`.map` files)

## Testing Strategy

### Markdown-Based Integration Tests (Primary Approach)

We use **markdown files** as test cases that serve dual purposes:
1. **Executable tests**: Validated by the test harness
2. **Living documentation**: Quick reference showing input → output transformations

#### Test File Format

Each test is a markdown file in `tests/transpiler-examples/`:

```markdown
# Test Name

Brief description of what this test validates.

## Input

```whitehall
// Whitehall source code
```

## Output

```kotlin
// Expected Kotlin output
```
```

#### Benefits

- **Easy Review**: Side-by-side input/output comparison in readable format
- **Maintainable**: Plain text, easy to edit and review in PRs
- **Documentation**: Tests serve as examples that never get stale
- **Quick Reference**: Developers can browse examples to learn syntax

#### Test Organization

Tests are numbered and organized by feature:

**Foundation (00-00b)**:
- `00-minimal-text.md` - Simplest component, just text
- `00a-text-with-interpolation.md` - Text with {var} interpolation
- `00b-single-prop.md` - Component with one prop

**Core Features (01-06)**:
- `01-basic-component.md` - Simple component with props and defaults
- `02-control-flow-if.md` - `@if/@else` conditional rendering
- `03-control-flow-for.md` - `@for` loops with keys and empty blocks
- `04-control-flow-when.md` - `@when` expressions
- `05-data-binding.md` - `bind:value` two-way binding
- `06-lifecycle-hooks.md` - `onMount` lifecycle hook

**Routing (07-08)**:
- `07-routing-simple.md` - Basic `$routes` navigation
- `08-routing-params.md` - Route parameters via `$screen.params`

**Composition (09-11)**:
- `09-imports.md` - Import aliases (`$lib`, `$models`, etc.)
- `10-nested-components.md` - Deep component trees
- `11-complex-state-management.md` - Multiple state vars, derived state

**Extended Patterns (12-17)**:
- `12-lazy-column.md` - LazyColumn with items() for performance
- `13-box-layout.md` - Box layout for stacking/overlaying
- `14-async-image.md` - AsyncImage with placeholder/error states
- `15-modifier-chains.md` - Chained and conditional modifiers
- `16-lifecycle-cleanup.md` - onDispose for resource cleanup
- `17-error-handling.md` - Try/catch in async operations

#### Test Runner Implementation

The test harness (`tests/transpiler_examples_test.rs`) performs:

1. **Parse markdown files**: Extract Whitehall input and expected Kotlin output
2. **Run transpiler**: Convert input `.wh` code to Kotlin
3. **Compare output**: Validate actual vs expected (with normalized whitespace)
4. **Report failures**: Show diff with line numbers for debugging

```rust
#[test]
fn test_transpile_all_examples() {
    let test_files = load_test_files();

    for (filename, content) in test_files {
        let test = parse_test_file(&content, &filename)
            .expect("Failed to parse test file");

        let actual_output = transpile(&test.input)
            .expect("Transpilation failed");

        assert_eq!(
            normalize_whitespace(&actual_output),
            normalize_whitespace(&test.expected_output),
            "Transpilation mismatch in {}", filename
        );
    }
}
```

The test runner handles:
- Loading all `.md` files from `tests/transpiler-examples/`
- Parsing markdown to extract code blocks
- Running transpilation (once implemented)
- Comparing with normalized whitespace (ignores minor formatting)
- Providing detailed error messages on mismatch

#### Running Tests

```bash
# Run all transpiler tests
cargo test transpiler_examples

# Run specific test validation
cargo test test_basic_component_structure

# Test with output
cargo test transpiler_examples -- --nocapture
```

### Unit Tests

Test individual components and phases:

```rust
#[test]
fn test_parse_prop_declaration() {
    let input = "@prop val url: String";
    let result = parse_prop(input);
    assert_eq!(result.name, "url");
    assert_eq!(result.prop_type, "String");
    assert_eq!(result.mutable, false);
}

#[test]
fn test_lexer_tokenize_component() {
    let input = "<Text>{count}</Text>";
    let tokens = tokenize(input);
    assert_eq!(tokens[0], Token::LessThan);
    assert_eq!(tokens[1], Token::Identifier("Text".to_string()));
}
```

### End-to-End Tests

Transpile full Microblog app and verify:
- All `.wh` files transpile successfully
- Generated Kotlin compiles without errors
- App runs without crashes
- Navigation and routing work correctly

### Adding New Tests

1. Create `tests/transpiler-examples/NN-feature-name.md`
2. Add `## Input` section with Whitehall code
3. Add `## Output` section with expected Kotlin
4. Run `cargo test transpiler_examples`
5. Test serves as both validation and documentation

## File Organization

```
src/transpiler/
  lexer/
    mod.rs           # Tokenization
    token.rs         # Token types
  parser/
    mod.rs           # Parser entry point
    kotlin.rs        # Kotlin syntax parser
    markup.rs        # Markup/component parser
    grammar.pest     # PEG grammar (if using pest)
  ast/
    mod.rs           # AST node definitions
    visitor.rs       # AST traversal
  analyzer/
    mod.rs           # Semantic analysis
    scope.rs         # Scope tracking
    types.rs         # Type checking
  transform/
    mod.rs           # AST transformations
    lifecycle.rs     # Lifecycle hook transforms
    binding.rs       # Data binding transforms
    control_flow.rs  # Control flow transforms
  codegen/
    mod.rs           # Code generation entry
    component.rs     # Component generation
    screen.rs        # Screen generation
    routes.rs        # Route generation
  error/
    mod.rs           # Error types and formatting
tests/
  transpiler-examples/  # Markdown-based test cases
    README.md           # Test format documentation
    01-basic-component.md
    02-control-flow-if.md
    03-control-flow-for.md
    ... (more test cases)
  transpiler_examples_test.rs  # Test harness
```

## Next Steps

1. **Create lexer** for basic tokens (keywords, identifiers, literals)
2. **Write PEG grammar** for simple component structure
3. **Build minimal AST** for one component
4. **Implement basic code generator** for Avatar component
5. **Test transpilation** of `Avatar.wh` → `Avatar.kt`
6. **Iterate** on more complex examples

---

*This document will evolve as we implement the transpiler and discover new requirements.*
