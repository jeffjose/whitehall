# Whitehall Example Apps

This directory contains real-world example applications to test and demonstrate Whitehall features.

## Examples

### 1. Counter (Minimal)
**Location**: `counter/`
**Features**: Basic state management, button clicks, simple UI
**Complexity**: ⭐ Beginner

### 2. Todo List
**Location**: `todo-list/`
**Features**: Lists, components, dynamic state, conditional rendering
**Complexity**: ⭐⭐ Intermediate

### 3. Weather App
**Location**: `weather-app/`
**Features**: Cards, loading states, simulated data
**Complexity**: ⭐⭐ Intermediate

### 4. Profile Card
**Location**: `profile-card/`
**Features**: Reusable components, props, conditional text
**Complexity**: ⭐⭐ Intermediate

### 5. Settings Screen
**Location**: `settings-screen/`
**Features**: Forms, switches, complex layouts
**Complexity**: ⭐⭐⭐ Advanced

## FFI Examples

### 6. C++ FFI
**Location**: `ffi-cpp/`
**Features**: C++ native code, `@ffi` annotations, automatic JNI bridge
**Complexity**: ⭐⭐ Intermediate
**What you'll learn**: How to call C++ functions from Whitehall with zero boilerplate

### 7. Rust FFI
**Location**: `ffi-rust/`
**Features**: Rust native code, `#[ffi]` attributes, memory safety, automatic naming conversion
**Complexity**: ⭐⭐ Intermediate
**What you'll learn**: How to call Rust functions from Whitehall, snake_case → camelCase conversion

### 8. Mixed C++ & Rust FFI
**Location**: `ffi-cpp-rust/`
**Features**: Both C++ and Rust in one app, multiple native libraries
**Complexity**: ⭐⭐⭐ Advanced
**What you'll learn**: When to use C++ vs Rust, combining multiple FFI languages

## Testing Examples

Build any example:
\`\`\`bash
cd examples/counter
whitehall build
\`\`\`

Build from root with --manifest-path:
\`\`\`bash
whitehall build --manifest-path examples/counter/whitehall.toml
\`\`\`
