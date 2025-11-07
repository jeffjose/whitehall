# C++ FFI Example

This example demonstrates how to call C++ code from a Whitehall application using the FFI (Foreign Function Interface) system.

## What This Example Shows

- **Automatic C++ Discovery**: The `@ffi` annotation marks functions for export
- **Zero Boilerplate**: No manual JNI code needed
- **Type Safety**: Automatic type conversions between C++, JNI, and Kotlin
- **Simple Integration**: Just write C++ code and use it in Whitehall components

## Project Structure

```
ffi-cpp/
├── whitehall.toml          # Project configuration with FFI enabled
├── src/
│   ├── main.wh             # Main Whitehall component
│   └── ffi/
│       └── cpp/
│           └── math.cpp    # C++ code with @ffi annotations
```

## The C++ Code

```cpp
// @ffi
int add(int a, int b) {
    return a + b;
}
```

That's it! The `@ffi` annotation tells Whitehall to:
1. Generate JNI bridge code
2. Generate Kotlin bindings
3. Configure CMake build
4. Compile and bundle the native library

## Using in Whitehall

```whitehall
<script>
  import $ffi.cpp.Math

  var result = Math.add(5, 3)  // Calls C++ code!
</script>

<Text>Result: {result}</Text>
```

## Building

```bash
whitehall build
```

Whitehall will automatically:
- Discover `@ffi` annotated functions
- Generate `build/kotlin/com/example/fficpp/Math.kt`
- Generate `build/jni/math_bridge.cpp`
- Generate `build/cmake/CMakeLists.txt`
- Compile `libmath.so`
- Bundle in APK

## What Gets Generated

### Kotlin Binding
```kotlin
package com.example.fficpp

object Math {
    external fun add(a: Int, b: Int): Int
    external fun multiply(a: Int, b: Int): Int
    external fun divide(a: Double, b: Double): Double

    init {
        System.loadLibrary("math")
    }
}
```

### JNI Bridge
```cpp
extern "C" JNIEXPORT jint JNICALL
Java_com_example_fficpp_Math_add(
    JNIEnv* env,
    jobject thiz,
    jint a,
    jint b
) {
    return add(a, b);
}
```

All generated automatically!

## Supported Types

- **Primitives**: `int`, `long`, `float`, `double`, `bool`
- **Strings**: `std::string`
- **Arrays**: `std::vector<int>`, `std::vector<long>`, `std::vector<float>`, `std::vector<double>`, `std::vector<bool>`, `std::vector<std::string>`

## Custom CMake Configuration (Optional)

Whitehall auto-generates `build/cmake/CMakeLists.txt` for you. However, if you need custom build configuration:

**Create `src/ffi/cpp/CMakeLists.txt`** and Whitehall will use it instead:

```cmake
# Custom CMakeLists.txt with additional dependencies
cmake_minimum_required(VERSION 3.22.1)
project("math")

add_library(math SHARED
    ${CMAKE_SOURCE_DIR}/src/ffi/cpp/math.cpp
    ${CMAKE_SOURCE_DIR}/build/generated/jni/math_bridge.cpp
)

set(CMAKE_CXX_STANDARD 20)  # Use C++20
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Add custom flags
target_compile_options(math PRIVATE -O3 -march=native)

# Link custom libraries
find_library(log-lib log)
target_link_libraries(math ${log-lib} your-custom-lib)
```

**Note**: If you provide your own CMakeLists.txt, you must include both:
- Your source files: `${CMAKE_SOURCE_DIR}/src/ffi/cpp/*.cpp`
- The generated JNI bridge: `${CMAKE_SOURCE_DIR}/build/generated/jni/*_bridge.cpp`

## Key Benefits

✅ **No JNI Boilerplate** - Just write clean C++ code
✅ **Automatic Memory Management** - No leaks, no manual cleanup
✅ **Type Safety** - Compile-time type checking
✅ **Simple** - 3 lines of C++ vs 150+ lines of manual JNI
✅ **Fast** - Native performance with zero overhead

## Learn More

See the [FFI Documentation](../../docs/FFI.md) for complete details on:
- Advanced type mappings
- Error handling
- Complex types with ByteArray serialization
- Performance optimization
