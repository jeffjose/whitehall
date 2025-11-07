# Mixed C++ & Rust FFI Example

This example demonstrates using **both C++ and Rust** FFI in the same Whitehall application. You can choose the best language for each part of your native code!

## What This Example Shows

- **Multiple Languages**: Use C++ and Rust together in one project
- **Independent Libraries**: Each language builds its own native library
- **Unified API**: Same seamless FFI experience for both languages
- **Best of Both Worlds**: Use C++ for ecosystems, Rust for safety

## Project Structure

```
ffi-cpp-rust/
├── whitehall.toml          # Enables both C++ and Rust FFI
├── src/
│   ├── main.wh             # Uses both CppMath and RustMath
│   └── ffi/
│       ├── cpp/
│       │   └── cpp_math.cpp    # C++ implementations
│       └── rust/
│           ├── Cargo.toml
│           └── lib.rs          # Rust implementations
```

## The Code

### C++ Implementation (Addition, Multiplication)

```cpp
// @ffi
int add(int a, int b) {
    return a + b;
}

// @ffi
int multiply(int a, int b) {
    return a * b;
}
```

### Rust Implementation (Subtraction, Division)

```rust
#[ffi]
pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

#[ffi]
pub fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        return 0.0;
    }
    a / b
}
```

## Using Both in Whitehall

```whitehall
<script>
  import $ffi.cpp.CppMath
  import $ffi.rust.RustMath

  // Use C++ for addition
  val sum = CppMath.add(20, 15)  // = 35

  // Use Rust for subtraction
  val result = RustMath.subtract(sum, 10)  // = 25
</script>

<Text>Result: {result}</Text>
```

## Configuration

Both languages are enabled in `whitehall.toml`:

```toml
[ffi]
enabled = true

[ffi.cpp]
standard = "17"

[ffi.rust]
profile = "release"
```

## What Gets Generated

Whitehall generates **two separate native libraries**:

### For C++
- `build/kotlin/com/example/fficpprust/CppMath.kt`
- `build/jni/cpp_math_bridge.cpp`
- `libcpp_math.so`

### For Rust
- `build/kotlin/com/example/fficpprust/RustMath.kt`
- Rust JNI bridge in the Rust project
- `librust_math.so`

Both libraries are automatically bundled in the APK and loaded independently.

## When to Use Each Language

### Use C++ When:
- ✅ You need specific C++ libraries (OpenCV, FFmpeg, etc.)
- ✅ Maximum performance is critical (hand-tuned SIMD)
- ✅ Team expertise is in C++
- ✅ Integrating existing C++ codebases

### Use Rust When:
- ✅ Memory safety is critical
- ✅ Building from scratch
- ✅ Want better thread safety guarantees
- ✅ Prefer modern language features
- ✅ Need better error handling

### Use Both When:
- ✅ Different parts have different needs
- ✅ Want to leverage both ecosystems
- ✅ Migrating C++ code to Rust incrementally
- ✅ Performance-critical (C++) + Safety-critical (Rust) sections

## Example Use Cases

**Image Processing App:**
- **C++**: OpenCV for computer vision (ecosystem)
- **Rust**: File parsing and data validation (safety)

**Audio App:**
- **C++**: FFmpeg for codec support (ecosystem)
- **Rust**: Audio buffer management (thread safety)

**Game:**
- **C++**: Physics engine (existing library)
- **Rust**: Save game serialization (data integrity)

## Building

```bash
whitehall build
```

Whitehall will:
1. Discover `@ffi` functions in C++ files
2. Discover `#[ffi]` functions in Rust files
3. Generate Kotlin bindings for both
4. Generate JNI bridges for both
5. Compile both native libraries
6. Bundle both in APK
7. Load both at runtime

## Performance

Both C++ and Rust compile to native code with **zero overhead**:
- No interpreter
- No garbage collector
- No reflection
- Direct function calls
- Same performance as manual JNI

The only overhead is the automatic type conversion, which is the same as manual JNI.

## Key Benefits

✅ **Language Choice** - Pick the right tool for each job
✅ **No Compromises** - Full power of both languages
✅ **Unified Experience** - Same FFI API for both
✅ **Independent Libraries** - No conflicts or coupling
✅ **Easy Migration** - Move code between C++ and Rust incrementally

## Learn More

See the [FFI Documentation](../../docs/FFI.md) for:
- Complete type mapping reference
- Advanced patterns (opaque handles, ByteArray serialization)
- Performance optimization
- Error handling (Phase 5)
- Rust vs C++ trade-offs in detail
