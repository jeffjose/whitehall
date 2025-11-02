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
- 14 test cases defined in `tests/transpiler-examples/`
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

### Implementation Strategy: Test-Driven Development

Unlike the previous attempt (all code at once), we'll build **incrementally**, one test at a time:

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

#### Phase 2: Basic Components (Test 01) ⏳ IN PROGRESS

**Goal**: Full component with multiple props and defaults

**Status**: Infrastructure added, parser debugging needed (commit: `4e95178`)

**Test 01: Basic Component** (`01-basic-component.md`)
- Input: Avatar component with url, size, onClick props
- Implemented (but not yet working):
  - ✅ Import statements parsing (`import $models.User`)
  - ✅ Import resolution (`$models` → `com.example.app.models`)
  - ✅ Self-closing tags (`<AsyncImage />`)
  - ✅ Component props with expressions (`url={url}`, `modifier={...}`)
  - ✅ Nested brace handling in expressions
  - ❌ Parser issue: not finding component after imports/props (to debug)
- Target: 4/14 tests passing

**Checkpoint**: Real components can be defined and used.

#### Phase 3: Control Flow (Tests 02-04)
**Goal**: All control flow constructs working

**Test 02: If/Else** (`02-control-flow-if.md`)
- Implement: `@if`, `@else` parsing and generation
- Verify: 5/14 tests passing

**Test 03: For Loops** (`03-control-flow-for.md`)
- Implement: `@for` with keys and empty blocks
- Verify: 6/14 tests passing

**Test 04: When** (`04-control-flow-when.md`)
- Implement: `@when` expression branches
- Verify: 7/14 tests passing

**Checkpoint**: All control flow working correctly.

#### Phase 4: Advanced Features (Tests 05-06)
**Goal**: Data binding and lifecycle hooks

**Test 05: Data Binding** (`05-data-binding.md`)
- Implement: `bind:value={var}` transformation
- Verify: 8/14 tests passing

**Test 06: Lifecycle Hooks** (`06-lifecycle-hooks.md`)
- Implement: `onMount` → `LaunchedEffect` transformation
- Verify: 9/14 tests passing

**Checkpoint**: State management fully functional.

#### Phase 5: Routing (Tests 07-08)
**Goal**: Navigation and route parameters

**Test 07: Routing Simple** (`07-routing-simple.md`)
- Implement: `$routes` alias, navigation
- Verify: 10/14 tests passing

**Test 08: Routing Params** (`08-routing-params.md`)
- Implement: `$screen.params` access
- Verify: 11/14 tests passing

**Checkpoint**: Routing system complete.

#### Phase 6: Composition (Tests 09-10)
**Goal**: Imports and nesting

**Test 09: Imports** (`09-imports.md`)
- Implement: Full import alias system (`$lib`, `$models`, etc.)
- Verify: 12/14 tests passing

**Test 10: Nested Components** (`10-nested-components.md`)
- Implement: Deep component trees
- Verify: 13/14 tests passing

#### Phase 7: Complex Examples (Test 11+)
**Goal**: Real-world complexity

**Test 11+**: Complex state management, etc.
- Verify: 14/14 tests passing

**Checkpoint**: ✅ Full transpiler complete!

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

- `01-basic-component.md` - Simple component with props
- `02-control-flow-if.md` - `@if/@else` conditional rendering
- `03-control-flow-for.md` - `@for` loops with keys and empty blocks
- `04-control-flow-when.md` - `@when` expressions
- `05-data-binding.md` - `bind:value` two-way binding
- `06-lifecycle-hooks.md` - `onMount` and lifecycle hooks
- `07-routing-simple.md` - Basic `$routes` navigation
- `08-routing-params.md` - Route parameters via `$screen.params`
- `09-imports.md` - Import aliases (`$lib`, `$models`, etc.)
- `10-nested-components.md` - Deep component trees
- ... and more as needed

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
