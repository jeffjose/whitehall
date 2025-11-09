# Whitehall Examples

This directory contains **15 focused examples** that teach Whitehall concepts **one at a time**. Each example demonstrates exactly one concept without cognitive overload.

## 📚 Learning Path

Start from example 01 and work through sequentially. Each builds on concepts from previous examples.

### Fundamentals (01-04)

**01-button-counter** - Buttons and State
- Teaches: `var` state, `Button` onClick, `@if` conditionals
- Complexity: Single file, ~60 lines
- **Start here** if new to Whitehall

**02-task-list** - List Operations
- Teaches: `mutableListOf`, add/remove operations, `@for` loops
- Complexity: Single file, ~70 lines
- Builds on: State from example 01

**03-text-input** - TextField Binding
- Teaches: `TextField`, two-way binding with `value`/`onValueChange`
- Complexity: Single file, ~45 lines
- Builds on: State management

**04-form-validation** - Form Validation
- Teaches: Input validation, error messages, form submission
- Complexity: Single file, ~70 lines
- Builds on: Text input from example 03

### UI Patterns (05-10)

**05-navigation** - Screen Switching
- Teaches: Changing screens with state variables
- Complexity: Single file, ~75 lines
- Builds on: State and conditionals

**06-async-data** - Async Loading
- Teaches: `suspend` functions, loading states, `delay`
- Complexity: Single file, ~50 lines
- **First async example**

**07-dialogs** - Dialog Windows
- Teaches: `AlertDialog`, modal interactions, `onDismissRequest`
- Complexity: Single file, ~75 lines
- Builds on: State for show/hide

**08-animations** - Animated Visibility
- Teaches: `AnimatedVisibility`, `fadeIn`/`fadeOut`, `animateContentSize`
- Complexity: Single file, ~60 lines
- Imports: androidx.compose.animation.*

**09-theming** - Dark Mode
- Teaches: `MaterialTheme`, `darkColorScheme`/`lightColorScheme`, theme switching
- Complexity: Single file, ~70 lines
- Imports: androidx.compose.material3.* theming

**10-tabs** - Tab Switching
- Teaches: `TabRow`, `Tab` selection, content switching
- Complexity: Single file, ~60 lines
- Imports: androidx.compose.material3.TabRow

### Advanced Patterns (11-12)

**11-lazy-lists** - LazyColumn Performance
- Teaches: `LazyColumn` for efficient list rendering
- Complexity: Single file, ~40 lines
- Builds on: Lists from example 02
- **Performance focused**

**12-component-patterns** - Reusable UI Patterns
- Teaches: Extracting repeated UI patterns for reusability
- Complexity: Single file, ~55 lines
- Builds on: All previous concepts

### FFI - Foreign Function Interface (13-15)

**13-ffi-rust-basic** - Rust FFI Basics
- Teaches: Calling Rust code from Whitehall using `#[ffi]` macro
- Complexity: Multi-file (src/ffi/rust/), ~80 lines total
- **First FFI example** - zero JNI boilerplate
- Demonstrates: Simple Rust functions (math, factorial, prime check)

**14-ffi-rust-json** - Rust FFI with serde_json
- Teaches: Using popular Rust libraries (serde_json) via FFI
- Complexity: Multi-file with Rust dependencies, ~90 lines
- **Real-world FFI** - demonstrates external crate usage
- Demonstrates: JSON parsing, creation, manipulation with serde

**15-ffi-cpp-basic** - C++ FFI Basics
- Teaches: Calling C++ code from Whitehall using `@ffi` annotation
- Complexity: Multi-file (src/ffi/cpp/), ~85 lines total
- **C++ FFI** - string utilities in C++
- Demonstrates: String operations, STL usage

## 🎯 Design Principles

### One Concept Per Example

Each example teaches **exactly one thing**:
- ✅ 03-text-input: TextField only
- ✅ 13-ffi-rust-basic: Basic Rust FFI only
- ❌ Not: TextField + validation + forms + database

### No Cognitive Overload

Examples are **deliberately simple**:
- Single file (except FFI and multi-file concepts)
- 40-90 lines
- No mixing of unrelated concepts
- Clear focus stated in comments

### Progressive Complexity

Concepts build logically:
```
State → Lists → Input → Validation
  ↓
Navigation → Async → Dialogs
  ↓
Animations → Theming → Tabs
  ↓
Performance → Patterns
  ↓
FFI: Rust (basic) → Rust (libraries) → C++
```

## 🚀 Next Steps

After completing all 15 examples, move to **`examples-complete/`** to see how these concepts combine into real applications:

- **weather-app** - Combines examples 06 (async) + 08 (animations) + 11 (lazy lists)
- **notes-app** - Combines examples 02 (lists) + 04 (validation) + 05 (navigation)
- **calculator** - State management patterns
- **settings-app** - Combines examples 09 (theming) + 10 (tabs) + 04 (forms)
- **profile-editor** - Form management and validation

## 🔨 Building Examples

Build all numbered examples:
```bash
bash scripts/build-example-apps.sh
```

Build a single example:
```bash
# Single-file examples
cargo run -- build examples/01-button-counter/main.wh

# FFI examples (project structure - requires Android NDK)
cargo run -- build examples/13-ffi-rust-basic
```

**Note:** FFI examples (13-15) require Android NDK (~1GB download) for native compilation. The build script will automatically install it on first FFI build. Non-FFI examples (01-12) build without NDK.

## 📖 Documentation

- **LANGUAGE-REFERENCE.md** - Complete Whitehall language reference
- **examples-complete/README.md** - Integration patterns guide

## 💡 Tips for Learning

1. **Read the file header** - Every example starts with comments explaining what it teaches
2. **Build and run** - See the app in action on Android
3. **Modify and experiment** - Change values, add buttons, break things!
4. **One concept at a time** - Don't skip ahead. Master each before moving on
5. **Check examples-complete/** - See how concepts combine in real apps
6. **FFI is optional** - Examples 13-15 show advanced features, not required for basic Whitehall

## 🦀 FFI Examples Explained

### Why FFI?

FFI (Foreign Function Interface) lets you call Rust and C++ code from Whitehall for:
- **Performance-critical operations** - image processing, cryptography
- **Using existing libraries** - serde_json, OpenCV, etc.
- **Code reuse** - leverage existing Rust/C++ codebases

### The Magic

**Rust:**
```rust
#[ffi]  // ← This macro generates all JNI boilerplate!
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**C++:**
```cpp
// @ffi  ← This annotation generates all JNI boilerplate!
int add(int a, int b) {
    return a + b;
}
```

**Whitehall:**
```whitehall
import $ffi.rust.Math
val result = Math.add(5, 3)  // Calls Rust seamlessly!
```

No manual JNI code needed!

## 🤝 Contributing

When adding new examples, follow these rules:

1. **One concept only** - If teaching two things, make two examples
2. **Keep it simple** - 40-90 lines maximum (FFI can be larger due to structure)
3. **Single file** - Unless multi-file IS the concept (like FFI)
4. **Clear header** - Explain what, why, and focus
5. **Test it builds** - Use `scripts/build-example-apps.sh`
