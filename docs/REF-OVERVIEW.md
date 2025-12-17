# Whitehall Architecture Overview

**Quick Reference Guide for Understanding the Codebase**

This document provides a high-level map of Whitehall's architecture with pointers to detailed documentation.

---

## What is Whitehall?

Whitehall is a **Kotlin superset for Android development** - it's Kotlin with ergonomic enhancements for Jetpack Compose.

**Core Philosophy:**

1. **Kotlin First** - Any valid Kotlin code is valid Whitehall code
   - Use data classes, sealed classes, extension functions, coroutines, etc.
   - Mix pure Kotlin alongside Whitehall syntax in the same file
   - Zero runtime overhead - transpiles to clean, idiomatic Kotlin/Compose

2. **Additive Syntax** - Whitehall adds convenient features on top:
   - Component markup syntax for UI (`<Text>`, `<Column>`)
   - Automatic state management (var → StateFlow)
   - Data binding shortcuts (`bind:value`, `bind:checked`)
   - UI conveniences (padding shortcuts, color helpers)
   - Lifecycle hooks (`$onMount`, `$onDispose`)

3. **Toolchain Philosophy** - "cargo for Android"
   - Opinionated defaults with project-level control when needed
   - Zero-config setup with automatic toolchain management

---

## Architecture Layers

```
┌─────────────────────────────────────┐
│     CLI Layer (clap)                │  User-facing commands (init, build, run, watch)
├─────────────────────────────────────┤
│     Command Handlers                │  src/commands/{init,build,run,watch}.rs
├─────────────────────────────────────┤
│     Core Services                   │
│  - Transpiler                       │  src/transpiler/ (parser → AST → Kotlin)
│  - Toolchain Manager                │  src/toolchain/ (Java/Gradle/SDK management)
│  - Build Pipeline                   │  src/build_pipeline.rs
├─────────────────────────────────────┤
│     Code Generation                 │
│  - Kotlin Generator                 │  src/transpiler/codegen/compose.rs
│  - Android Scaffold                 │  src/android_scaffold.rs
├─────────────────────────────────────┤
│     Android Integration             │
│  - Gradle Wrapper                   │  Generated per project
│  - ADB Interface                    │  Device deployment
└─────────────────────────────────────┘
```

---

## Key Components Status

| Component | Status | Reference Doc |
|-----------|--------|---------------|
| **Transpiler** | ✅ Complete (38/38 tests, 100%) | [REF-TRANSPILER.md](./REF-TRANSPILER.md) |
| **Pass-Through Architecture** | ✅ Complete (10/10 tests, Phases 0-6) | [PASSTHRU.md](./PASSTHRU.md) |
| **Build System** | ✅ Fully Implemented | [REF-BUILD-SYSTEM.md](./REF-BUILD-SYSTEM.md) |
| **State Management** | ✅ Phase 1.1 Complete (Component inline vars) | [REF-STATE-MANAGEMENT.md](./REF-STATE-MANAGEMENT.md) |
| **Toolchain** | ✅ Fully Implemented (Phases 1-5) | [REF-TOOLCHAIN.md](./REF-TOOLCHAIN.md) |
| **Web Playground** | ✅ Phase 1 Complete | [REF-WEB-PLAYGROUND.md](./REF-WEB-PLAYGROUND.md) |

**Pass-Through Architecture** enables Whitehall as a true Kotlin superset:
- Any Kotlin syntax passes through unchanged (data classes, sealed classes, extension functions, etc.)
- Mix pure Kotlin and Whitehall features in the same file
- Tested with complex patterns: sealed classes, companion objects, extension properties, DSL builders

**Legend:** ✅ Complete | 🔄 In Progress | ⏳ Planned | 🔮 Future

---

## File Structure Guide

```
whitehall/
├── src/
│   ├── main.rs                      # CLI entry point (clap commands)
│   ├── commands/                    # Command implementations
│   │   ├── init.rs                  # Project initialization
│   │   ├── build.rs                 # One-shot transpilation
│   │   ├── watch.rs                 # File watching + auto-rebuild
│   │   └── run.rs                   # Build + install + launch
│   ├── transpiler/                  # Core transpilation engine
│   │   ├── mod.rs                   # Public API
│   │   ├── parser.rs                # Lexer-free recursive descent parser
│   │   ├── ast.rs                   # AST definitions
│   │   ├── analyzer.rs              # Semantic analysis + store registry
│   │   └── codegen/
│   │       ├── compose.rs           # Kotlin/Compose generation
│   │       └── mod.rs
│   ├── toolchain/                   # Toolchain management
│   │   ├── mod.rs                   # Core toolchain manager
│   │   ├── defaults.rs              # Default versions
│   │   ├── platform.rs              # Platform detection
│   │   ├── validator.rs             # Version compatibility
│   │   └── downloader.rs            # HTTP download + extraction
│   ├── build_pipeline.rs            # Shared build logic
│   ├── config.rs                    # Parse whitehall.toml
│   ├── project.rs                   # File discovery
│   └── android_scaffold.rs          # Generate Gradle boilerplate
├── tests/
│   ├── transpiler-examples/         # 38 markdown-based test cases
│   └── transpiler_examples_test.rs  # Test harness
├── tools/
│   └── playground/                  # Web-based IDE
│       ├── backend/                 # Rust + Axum server
│       └── frontend/                # Monaco editor + JS
└── docs/
    ├── refs/                        # Reference documentation (you are here)
    ├── ARCHITECTURE.md              # Original design doc
    ├── BUILD.md                     # Original build system doc
    ├── STORE.md                     # Original state management doc
    ├── TRANSPILER.md                # Original transpiler doc
    └── ...
```

---

## Compilation Pipeline

```
.wh files → Parser → AST → Semantic Analysis → Code Generator → .kt files
              ↓                   ↓
         Pass-Through         Store Registry
         (Pure Kotlin)        Type Checking
                              Validation
```

**Key Phases:**

1. **Parsing** (`parser.rs`) - Hybrid parsing strategy:
   - Whitehall-specific syntax (props, markup, directives) → Parsed and transformed
   - Pure Kotlin syntax (data classes, extension functions, etc.) → Passed through unchanged
   - Maintains source order and position for accurate error reporting

2. **Semantic Analysis** (`analyzer.rs`) - Build store registry, validate types

3. **Code Generation** (`codegen/compose.rs`) - Transform AST to idiomatic Kotlin/Compose
   - Whitehall features → Kotlin/Compose equivalents
   - Pass-through blocks → Output unchanged

4. **Android Scaffold** (`android_scaffold.rs`) - Generate Gradle project structure

**Pass-Through Architecture:**
- Enables Whitehall as a true Kotlin superset
- Kotlin blocks captured with context tracking (strings, comments, braces)
- Tested with complex patterns: sealed classes, companion objects, extension properties, DSL builders
- Learn more: [PASSTHRU.md](./PASSTHRU.md)

**Learn more:** [REF-TRANSPILER.md](./REF-TRANSPILER.md)

---

## State Management Architecture

Whitehall provides multiple state management patterns:

| Pattern | Syntax | Status |
|---------|--------|--------|
| Local state (inline) | `var count = 0` | ✅ Supported (remember/mutableStateOf) |
| Local state (complex) | `var count = 0` (auto-ViewModel) | ✅ Phase 1.1 Complete |
| Props | `@prop val name: String` | ✅ Supported |
| Two-way binding | `bind:value={email}` | ✅ Supported |
| Screen-level stores | `@store class UserProfile { ... }` | ✅ Complete (Phases 0-5) |
| Suspend functions | `suspend fun save()` | ✅ Complete (auto-wrap in viewModelScope) |
| Coroutine dispatchers | `io { }`, `cpu { }`, `main { }` | ✅ Complete |
| Lifecycle hooks | `$onMount`, `$onDispose` | ✅ Complete |
| Hilt integration | `@Inject` or `@hilt` | ✅ Complete (hybrid auto-detection) |

**Learn more:** [REF-STATE-MANAGEMENT.md](./REF-STATE-MANAGEMENT.md)

---

## Build System

Three commands for different workflows:

| Command | Purpose | Use Case |
|---------|---------|----------|
| `whitehall init` | Create new project | Project initialization |
| `whitehall compile` | Transpile single file | Quick testing without Android project |
| `whitehall build` | One-shot transpilation | CI/CD, releases |
| `whitehall watch` | Continuous auto-rebuild | Active development |
| `whitehall run` | Build + install + launch | Testing on device |
| `whitehall toolchain` | Manage toolchains | Install/list/clean Java/Gradle/SDK |
| `whitehall exec` | Run command with toolchain | Execute gradle/adb with project tools |
| `whitehall shell` | Interactive shell | Debug with project environment |
| `whitehall doctor` | Check system health | Verify toolchain status |

**Status:** ✅ Fully Implemented (Phases 1-5)

**Learn more:** [REF-BUILD-SYSTEM.md](./REF-BUILD-SYSTEM.md)

---

## Toolchain Management

Whitehall bundles required toolchains (Java, Gradle, Android SDK) for zero-config setup.

**Architecture:** Project-scoped toolchains (like `rust-toolchain.toml`)

```
~/.whitehall/toolchains/
├── java/
│   ├── 11/              # For AGP 7.x projects
│   ├── 17/              # For AGP 8.x projects
│   └── 21/              # For modern AGP 8.2+ projects
├── gradle/
│   ├── 7.6/
│   ├── 8.0/
│   └── 8.4/
└── android/
    ├── platform-tools/  # adb, fastboot
    ├── build-tools/
    └── platforms/
```

**Key Feature:** Each project specifies its toolchain in `whitehall.toml`, Whitehall downloads on-demand.

**Learn more:** [REF-TOOLCHAIN.md](./REF-TOOLCHAIN.md)

---

## Web Playground

Web-based IDE for learning and experimenting with Whitehall syntax.

**Status:** ✅ Phase 1 Complete (Nov 4, 2025)

**Features:**
- Monaco editor with real-time compilation
- Multiple output tabs (Kotlin / Errors / AST)
- Example snippets
- URL hash state for sharing
- Keyboard shortcuts

**Location:** `tools/playground/`

**Learn more:** [REF-WEB-PLAYGROUND.md](./REF-WEB-PLAYGROUND.md)

---

## Testing Strategy

### Transpiler Tests

**Approach:** Markdown-based test cases (serve dual purpose: validation + documentation)

**Location:** `tests/transpiler-examples/`

**Status:** 38/38 tests passing (100% coverage)

**Test Categories:**
- Foundation (00-00e): Basic text, interpolation, props, variables, lists, arrays
- Core Features (01-06): Components, control flow, data binding, lifecycle
- Routing (07-08): Navigation, route parameters
- Composition (09-11): Imports, nesting, complex state
- Extended Patterns (12-17): LazyColumn, Box, modifiers, error handling
- Advanced Features (18-26): String resources, checkboxes, derived state, colors, padding shortcuts, escape braces, inline lambdas, spacer shortcuts, function return types
- Stores (27-29): Hilt stores, explicit Hilt, stores without Hilt
- Component Inline Vars / Phase 1.1 (30-32): Basic inline vars, suspend functions, derived properties

**Run tests:** `cargo test --test transpiler_examples_test examples`

---

## Project Configuration

### whitehall.toml

```toml
[project]
name = "my-app"
version = "0.1.0"

[android]
min_sdk = 24
target_sdk = 34
package = "com.example.myapp"

[toolchain]
java = "21"           # Java/JDK version
gradle = "8.4"        # Gradle version
agp = "8.2.0"        # Android Gradle Plugin
kotlin = "1.9.20"    # Kotlin compiler version

[build]
output_dir = "build"
```

---

## Known Gaps & Next Steps

### Completed Recently

**Phase 1.1: Component Inline Vars → ViewModel** ✅
- Auto-detect `var` in component `<script>` blocks
- Generate ViewModel automatically for complex components
- Smart heuristic: suspend functions, lifecycle hooks, or >=3 functions
- Multi-file output: ComponentViewModel.kt + Component.kt

**Status:** Complete - tests 30-32 passing, TranspileResult::Multiple implemented

**Learn more:** [REF-STATE-MANAGEMENT.md](./REF-STATE-MANAGEMENT.md) → Phase 1.1

### Future Enhancements

**Phase 1.2:** Imported classes with `var` - Auto-detect at usage sites
**Phase 1.3:** Global singletons - `@store object` pattern
**Phase 2.0:** Advanced features - Store composition, DevTools, persistence middleware

---

## Common Workflows

### Creating a New Project

```bash
whitehall init my-app
cd my-app
# → Generates project structure with whitehall.toml
# → Downloads toolchains on first build (Java, Gradle, SDK)
```

### Development Loop

```bash
# Terminal 1: Continuous feedback
whitehall watch

# Terminal 2: Edit files
vim src/components/Button.wh
# Save → see instant rebuild in Terminal 1

# When ready to test on device
whitehall run
```

### CI/CD Build

```bash
whitehall build
cd build
./gradlew assembleRelease --no-daemon
# Upload to Play Store
```

---

## Error Messages Philosophy

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

---

## Related Documentation

### Reference Docs (Detailed)
- [REF-TRANSPILER.md](./REF-TRANSPILER.md) - Transpiler architecture and implementation
- [REF-BUILD-SYSTEM.md](./REF-BUILD-SYSTEM.md) - Build commands and pipeline
- [REF-STATE-MANAGEMENT.md](./REF-STATE-MANAGEMENT.md) - State management patterns
- [REF-TOOLCHAIN.md](./REF-TOOLCHAIN.md) - Toolchain management
- [REF-WEB-PLAYGROUND.md](./REF-WEB-PLAYGROUND.md) - Web-based IDE


## For LLMs: Quick Navigation

**To understand:**
- **Overall architecture** → You are here (OVERVIEW.md)
- **How transpilation works** → [REF-TRANSPILER.md](./REF-TRANSPILER.md)
- **How state management works** → [REF-STATE-MANAGEMENT.md](./REF-STATE-MANAGEMENT.md)
- **How builds work** → [REF-BUILD-SYSTEM.md](./REF-BUILD-SYSTEM.md)
- **How toolchains work** → [REF-TOOLCHAIN.md](./REF-TOOLCHAIN.md)
- **Web playground** → [REF-WEB-PLAYGROUND.md](./REF-WEB-PLAYGROUND.md)

**To find code:**
- **Transpiler implementation** → `src/transpiler/`
- **Build commands** → `src/commands/`
- **Toolchain management** → `src/toolchain/`
- **Test cases** → `tests/transpiler-examples/`

**Current work:**
- **Minor bug fixes** → Test 05 import detection
- **Future phases** → Phase 1.2 (imported classes), Phase 1.3 (global singletons)

---

*Last Updated: 2025-11-06*
*Version: 1.1*
*Status: Verified Against Codebase*
