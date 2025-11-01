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

### 3. Project Structure

A Whitehall project looks like:

```
my-app/
├── Whitehall.toml          # Project manifest
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

### 4. Compilation Pipeline

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

### 5. The .whitehall File Format

Initial concept (subject to evolution):

```whitehall
component LoginScreen {
  state {
    email: String = ""
    password: String = ""
  }

  view {
    Column(padding = 16) {
      TextField(
        value = email,
        onChanged = (value) => { email = value }
      )

      Button(
        text = "Login",
        onClick = () => { handleLogin() }
      )
    }
  }

  fn handleLogin() {
    // Logic here
  }
}
```

This transpiles to idiomatic Kotlin with Jetpack Compose.

### 6. Scaling Considerations

**From the start:**
- Modular crate structure (workspace)
- Clear separation of concerns
- Plugin-friendly architecture
- Extensible code generation

**Future:**
- Plugin system using WASM or dynamic libraries
- Custom optimization passes
- Alternative backends (Swift for iOS?)

### 7. Configuration: Whitehall.toml

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

### 8. Error Handling Philosophy

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
