# Architecture

## Design Principles

### 1. Rust for the Toolchain
**Why Rust:**
- Performance - Fast compilation and file operations
- Reliability - Catch bugs at compile time
- Ecosystem - Excellent libraries (`clap`, `serde`, `syn`)
- Cross-platform - Single codebase for Linux/Mac/Windows

### 2. Layered Architecture

```
┌─────────────────────────────────────┐
│     CLI Layer (clap)                │  User-facing commands
├─────────────────────────────────────┤
│     Command Handlers                │  init, build, run, etc.
├─────────────────────────────────────┤
│     Core Services                   │
│  - Project Manager                  │
│  - Parser & Compiler                │
│  - Dependency Resolver              │
├─────────────────────────────────────┤
│     Code Generation                 │
│  - Kotlin Generator                 │
│  - Gradle Generator                 │
├─────────────────────────────────────┤
│     Android Integration             │
│  - Gradle Wrapper                   │
│  - ADB Interface                    │
└─────────────────────────────────────┘
```

### 3. Execution Modes: Single-File vs Project

**Status**: Currently implementing project mode only. Single-file mode is planned for future.

Whitehall will eventually support two distinct execution modes, inspired by the `rustc` vs `cargo` distinction:

#### Single-File Mode (🔜 Future)

For learning, prototyping, and sharing. A complete Android app in one `.wh` file.

**Example: `counter.wh`**
```whitehall
/// [app]
/// name = "Counter"
/// package = "com.example.counter"
/// minSdk = 24
///
/// [dependencies]
/// androidx.compose.material3 = "1.2.0"

component App()

<script>
  state { count = 0 }
</script>

<Column>
  <Text>{count}</Text>
  <Button onClick={() => count++}>Increment</Button>
</Column>
```

**Usage:**
```bash
whitehall run counter.wh        # Compile & run (future)
whitehall build counter.wh      # Build APK (future)
```

**Planned implementation:**
1. Parse TOML frontmatter (lines starting with `///`)
2. Extract app config and dependencies
3. Generate temporary project structure in `.whitehall/tmp/<hash>/`
4. Create synthetic `whitehall.toml` from frontmatter
5. Compile using normal project pipeline
6. Cache builds for fast re-runs
7. Clean up on completion (keep cache)

#### Project Mode (🔄 Current Focus)

For production apps with multiple screens, shared components, and team development.

**Usage:**
```bash
whitehall init my-app           # ✅ Implemented
cd my-app
whitehall build                 # 🔄 In progress (see docs/BUILD.md)
whitehall watch                 # 🔄 In progress
whitehall run                   # 🔄 In progress
```

**When to use each:**
- **Single-file (future):** Tutorials, examples, quick experiments, sharing code snippets
- **Project (current):** All development (this is what we're building first)

### 4. Project Structure

A Whitehall project looks like:

```
my-app/
├── whitehall.toml          # Project manifest
├── src/
│   ├── main.wh             # Entry point
│   └── components/         # Reusable components
│       └── Button.wh
├── .whitehall/             # Generated/cache (gitignored)
│   ├── build/
│   └── cache/
└── android/                # Generated Android project
    ├── app/
    │   ├── build.gradle
    │   └── src/
    │       └── main/
    │           └── kotlin/  # Generated from .wh files
    └── settings.gradle
```

### 5. Compilation Pipeline

```
.wh files → Lexer → Tokens → Parser → AST → Analyzer → IR → Kotlin Generator → .kt files
                                              ↓
                                      Type Checking
                                      Validation
```

**Key decisions:**
- **AST-based:** Preserve source information for good error messages
- **Incremental:** Only recompile changed files
- **Cached:** Store compilation artifacts

### 6. The .whitehall File Format

**Current syntax** (see `docs/TRANSPILER.md` for full specification and test examples in `tests/transpiler-examples/`):

**LoginScreen.wh:**
```whitehall
<script>
  @prop val title: String = "Login"

  var email = ""
  var password = ""

  fun handleLogin() {
    // Authentication logic
  }
</script>

<Column padding={16} spacing={8}>
  <Text fontSize={24}>{title}</Text>

  <Input
    bind:value={email}
    label="Email"
  />

  <Input
    bind:value={password}
    label="Password"
    type="password"
  />

  <Button
    text="Login"
    onClick={handleLogin}
  />
</Column>
```

**Key features:**
- Markup-based UI (inspired by Svelte)
- No `<script>` section - Kotlin code at top, markup below
- `@prop val` for component props (Kotlin-native)
- `var`/`val` for state (Kotlin keywords)
- Filename determines component name
- Full feature set: control flow (`@if`, `@for`, `@when`), data binding (`bind:value`), lifecycle hooks (`onMount`, `onDispose`)

This transpiles to idiomatic Kotlin with Jetpack Compose. See `docs/TRANSPILER.md` for complete feature list and examples.

### 7. Scaling Considerations

**From the start:**
- Modular crate structure (workspace)
- Clear separation of concerns
- Plugin-friendly architecture
- Extensible code generation

**Future:**
- Plugin system using WASM or dynamic libraries
- Custom optimization passes
- Alternative backends (Swift for iOS?)

### 8. Configuration: whitehall.toml

Inspired by `Cargo.toml`:

```toml
[project]
name = "my-app"
version = "0.1.0"
authors = ["You <you@example.com>"]

[android]
min_sdk = 24
target_sdk = 34
package = "com.example.myapp"

[dependencies]
# Future: Whitehall components
# Future: Android libraries

[build]
optimize_level = "default"  # or "aggressive"
```

### 9. Error Handling Philosophy

Learn from Rust compiler:
- Precise error locations
- Helpful suggestions
- Color-coded output
- Example fixes

```
error: Component state must specify type
  ┌─ src/main.wh:3:5
  │
3 │     username = ""
  │     ^^^^^^^^ missing type annotation
  │
  = help: add type annotation: `username: String = ""`
```

## Technology Stack

- **CLI:** `clap` v4
- **Config:** `serde` + `toml`
- **File operations:** `std::fs`, `walkdir`
- **Process execution:** `std::process::Command` (for Gradle/ADB)
- **Parsing:** TBD - possibly `pest` or hand-written recursive descent
- **Testing:** `cargo test` + integration tests
