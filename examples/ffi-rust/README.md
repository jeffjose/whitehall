# Rust FFI Example

This example demonstrates how to call Rust code from a Whitehall application using the FFI (Foreign Function Interface) system.

## What This Example Shows

- **Automatic Rust Discovery**: The `#[ffi]` attribute marks functions for export
- **Zero Boilerplate**: No manual JNI code needed
- **Type Safety**: Automatic type conversions between Rust, JNI, and Kotlin
- **Memory Safety**: Rust's ownership system + automatic JNI management
- **Naming Conventions**: Automatic snake_case → camelCase conversion

## Project Structure

```
ffi-rust/
├── whitehall.toml          # Project configuration with FFI enabled
├── src/
│   ├── main.wh             # Main Whitehall component
│   └── ffi/
│       └── rust/
│           ├── Cargo.toml  # Rust project configuration
│           └── lib.rs      # Rust code with #[ffi] attributes
```

## The Rust Code

```rust
#[ffi]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

That's it! The `#[ffi]` attribute tells Whitehall to:
1. Generate Rust JNI bridge code
2. Generate Kotlin bindings
3. Configure Cargo build for Android targets
4. Compile and bundle the native library

## Using in Whitehall

```whitehall
<script>
  import $ffi.rust.Math

  var result = Math.add(5, 3)  // Calls Rust code!
</script>

<Text>Result: {result}</Text>
```

## Naming Convention

Rust uses `snake_case`, Kotlin uses `camelCase`. Whitehall converts automatically:

```rust
#[ffi]
pub fn is_positive(n: i32) -> bool {
    n > 0
}
```

Becomes in Kotlin:
```kotlin
external fun isPositive(n: Int): Boolean
```

## Building

```bash
whitehall build
```

Whitehall will automatically:
- Discover `#[ffi]` annotated functions
- Generate `build/kotlin/com/example/ffirust/Math.kt`
- Generate Rust JNI bridge code
- Configure Cargo for Android targets (aarch64, armv7, etc.)
- Compile Rust library
- Bundle in APK

## What Gets Generated

### Kotlin Binding
```kotlin
package com.example.ffirust

object Math {
    external fun add(a: Int, b: Int): Int
    external fun multiply(a: Int, b: Int): Int
    external fun divide(a: Double, b: Double): Double
    external fun isPositive(n: Int): Boolean

    init {
        System.loadLibrary("math")
    }
}
```

### Rust JNI Bridge
```rust
#[no_mangle]
pub extern "system" fn Java_com_example_ffirust_Math_add(
    _env: JNIEnv,
    _class: JClass,
    a: jint,
    b: jint,
) -> jint {
    crate::add(a, b)
}
```

All generated automatically!

## Supported Types

- **Primitives**: `i32`, `i64`, `f32`, `f64`, `bool`, `()` (void)
- **Strings**: `String`
- **Arrays**: `Vec<i32>`, `Vec<i64>`, `Vec<f32>`, `Vec<f64>`, `Vec<bool>`, `Vec<String>`

## Why Rust?

✅ **Memory Safety** - No leaks, no use-after-free, no undefined behavior
✅ **Thread Safety** - Data races prevented at compile time
✅ **Performance** - As fast as C++, often faster
✅ **Modern Tooling** - Cargo, built-in testing, great docs
✅ **Better Error Handling** - Result<T, E> instead of exceptions
✅ **Safer FFI** - Drop trait ensures cleanup, type system prevents misuse

## Key Benefits Over C++

- **Memory safety enforced by compiler** - Leaks are nearly impossible
- **Drop trait** - Automatic resource cleanup (RAII done right)
- **Send/Sync traits** - Thread safety guaranteed at compile time
- **Better error handling** - Result<T, E> is more ergonomic than exceptions
- **Modern language** - Pattern matching, iterators, async/await

## Dependencies

The Rust project uses:
```toml
[dependencies]
jni = "0.21"
```

Whitehall handles the `jni` crate automatically for JNI bridge generation.

## Learn More

See the [FFI Documentation](../../docs/FFI.md) for complete details on:
- Advanced type mappings
- Result<T, E> error handling (Phase 5)
- Complex types with ByteArray serialization
- Performance optimization
- Rust vs C++ trade-offs
