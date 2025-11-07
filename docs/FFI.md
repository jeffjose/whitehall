# FFI (Foreign Function Interface) Integration

**Status**: 🚧 In Development (Design Phase)
**Last Updated**: 2025-01-07

---

## Overview

Whitehall supports calling code written in **C++**, **Rust**, or other systems languages through FFI (Foreign Function Interface). This enables performance-critical operations, reuse of existing libraries, and integration with low-level system APIs.

**Philosophy**: Write performance-critical code in the best language for the job. Annotate functions with `@ffi` or `#[ffi]`, and Whitehall automatically generates all the JNI glue code and Kotlin bindings.

**Key Innovation**: You write clean C++/Rust code with simple annotations. Whitehall handles all the complex JNI boilerplate, memory management, type conversions, and error handling automatically.

## Table of Contents

- [Why FFI?](#why-ffi)
- [Quick Start](#quick-start)
- [The Whitehall FFI Advantage](#the-whitehall-ffi-advantage)
- [How It Works](#how-it-works)
- [Simple Types (Automatic)](#simple-types-automatic)
- [Complex Types (Manual Serialization)](#complex-types-manual-serialization)
- [Project Structure](#project-structure)
- [Configuration](#configuration)
- [Complete Examples](#complete-examples)
- [Type Mapping Reference](#type-mapping-reference)
- [Understanding JNI Complexity](#understanding-jni-complexity)
- [Memory Management](#memory-management)
- [Thread Safety](#thread-safety)
- [Error Handling](#error-handling)
- [Debugging Native Code](#debugging-native-code)
- [Security Best Practices](#security-best-practices)
- [Testing FFI Code](#testing-ffi-code)
- [Troubleshooting](#troubleshooting)

---

## Why FFI?

### Use Cases

1. **Performance-critical operations**
   - Image/video processing (OpenCV, FFmpeg)
   - Audio processing (real-time DSP)
   - Machine learning inference (TensorFlow Lite, ONNX)
   - Cryptography (native crypto libraries)

2. **Reusing existing libraries**
   - C/C++ libraries: OpenCV, SQLite, libpng, zlib
   - Rust crates: image, tokio, serde

3. **Platform-specific optimizations**
   - SIMD instructions (AVX, NEON)
   - Hardware acceleration
   - Direct system calls

---

## Quick Start

### C++ Example (30 seconds)

**Step 1: Write your C++ code with `@ffi` annotation**

Create `src/ffi/cpp/math.cpp`:
```cpp
// @ffi
int add(int a, int b) {
    return a + b;
}

// @ffi
double multiply(double a, double b) {
    return a * b;
}
```

**Step 2: Enable FFI in `whitehall.toml`**
```toml
[ffi]
enabled = true
```

**Step 3: Build**
```bash
whitehall build
```

**Step 4: Use in your Whitehall component**
```whitehall
<script>
  import $ffi.cpp.Math

  var result = 0

  onMount {
    result = Math.add(5, 3)  // Calls C++!
  }
</script>

<Text>Result: {result}</Text>
```

**That's it!** Whitehall automatically:
- ✅ Discovers your `@ffi` annotated functions
- ✅ Generates JNI bridge code
- ✅ Generates Kotlin bindings
- ✅ Configures CMake build
- ✅ Compiles and bundles the native library

---

### Rust Example (30 seconds)

**Step 1: Initialize Rust project**
```bash
mkdir -p src/ffi/rust
cd src/ffi/rust
cargo init --lib
```

**Step 2: Configure `Cargo.toml`**
```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
whitehall-ffi = "0.1"
```

**Step 3: Write Rust code with `#[ffi]` annotation**

Create `src/ffi/rust/lib.rs`:
```rust
use whitehall_ffi::ffi;

#[ffi]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[ffi]
pub fn multiply(a: f64, b: f64) -> f64 {
    a * b
}
```

**Step 4: Build and use**
```bash
whitehall build
```

```whitehall
<script>
  import $ffi.rust.Math

  var result = 0

  onMount {
    result = Math.add(10, 7)  // Calls Rust!
  }
</script>

<Text>Result: {result}</Text>
```

---

## The Whitehall FFI Advantage

### Traditional Android JNI (Manual Hell)

**You must write:**

1. **C++ with JNI boilerplate** (100+ lines):
```cpp
#include <jni.h>
#include <string>

extern "C" JNIEXPORT jint JNICALL
Java_com_example_myapp_NativeLib_add(
    JNIEnv* env,
    jobject /* this */,
    jint a,
    jint b) {
    return a + b;
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_example_myapp_NativeLib_greet(
    JNIEnv* env,
    jobject /* this */,
    jstring name) {

    // Manual null check
    if (name == nullptr) {
        return env->NewStringUTF("");
    }

    // Manual type conversion
    const char* nativeName = env->GetStringUTFChars(name, nullptr);
    if (nativeName == nullptr) {
        return nullptr;
    }

    // Actual logic
    std::string result = "Hello, " + std::string(nativeName);

    // Manual cleanup (memory leak if forgotten!)
    env->ReleaseStringUTFChars(name, nativeName);

    return env->NewStringUTF(result.c_str());
}
```

2. **CMakeLists.txt**:
```cmake
cmake_minimum_required(VERSION 3.22.1)
add_library(native-lib SHARED native-lib.cpp)
find_library(log-lib log)
target_link_libraries(native-lib ${log-lib})
```

3. **Kotlin external declarations**:
```kotlin
class NativeLib {
    external fun add(a: Int, b: Int): Int
    external fun greet(name: String): String

    companion object {
        init {
            System.loadLibrary("native-lib")
        }
    }
}
```

4. **Gradle configuration**:
```kotlin
android {
    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
        }
    }
}
```

**Total: ~150 lines of boilerplate for 2 simple functions!**

---

### Whitehall FFI (Automatic)

**You write:**

```cpp
// src/ffi/cpp/math.cpp

// @ffi
int add(int a, int b) {
    return a + b;
}

// @ffi
std::string greet(const std::string& name) {
    return "Hello, " + name;
}
```

**That's it! 10 lines total.**

Whitehall generates all 150+ lines of boilerplate automatically.

---

## How It Works

### Build Process

```
┌─────────────────────────────────────────────────────────────┐
│  1. Developer writes C++/Rust with @ffi annotations         │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  2. Whitehall scans src/ffi/ for annotations                │
│     - Finds: @ffi int add(int, int)                         │
│     - Finds: #[ffi] pub fn multiply(f64, f64) -> f64        │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  3. Generates Kotlin bindings                               │
│     → build/kotlin/com/example/ffi/Math.kt                  │
│       object Math {                                         │
│         external fun add(a: Int, b: Int): Int               │
│       }                                                     │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  4. Generates JNI bridge code                               │
│     → build/jni/math_bridge.cpp                             │
│       extern "C" JNIEXPORT jint JNICALL                     │
│       Java_com_example_ffi_Math_add(...) {                  │
│         return add(a, b); // Calls your function            │
│       }                                                     │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  5. Compiles native code                                    │
│     → libmath.so                                            │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  6. Whitehall components import and use                     │
│     import $ffi.cpp.Math                                    │
│     Math.add(5, 3)                                          │
└─────────────────────────────────────────────────────────────┘
```

### What Gets Generated

For this simple C++ function:
```cpp
// @ffi
int add(int a, int b) {
    return a + b;
}
```

Whitehall generates:

**1. Kotlin binding** (`build/kotlin/com/example/ffi/Math.kt`):
```kotlin
package com.example.ffi

object Math {
    external fun add(a: Int, b: Int): Int

    init {
        System.loadLibrary("math")
    }
}
```

**2. JNI bridge** (`build/jni/math_bridge.cpp`):
```cpp
#include <jni.h>
#include "math.cpp"

extern "C" JNIEXPORT jint JNICALL
Java_com_example_ffi_Math_add(
    JNIEnv* env,
    jobject thiz,
    jint a,
    jint b
) {
    // Auto-generated type conversions
    // Auto-generated null checks
    // Auto-generated error handling

    return add(a, b);

    // Auto-generated cleanup
}
```

**3. CMake configuration** (`build/cmake/CMakeLists.txt`):
```cmake
cmake_minimum_required(VERSION 3.22.1)
project("math")

add_library(math SHARED
    ${CMAKE_SOURCE_DIR}/src/ffi/cpp/math.cpp
    ${CMAKE_CURRENT_BINARY_DIR}/math_bridge.cpp
)

find_library(log-lib log)
target_link_libraries(math ${log-lib})
```

---

## Simple Types (Automatic)

Whitehall automatically handles these types with zero boilerplate:

### Supported Types

| Whitehall/Kotlin | C++ | Rust | Notes |
|------------------|-----|------|-------|
| `Int` | `int32_t` / `int` | `i32` | 32-bit integer |
| `Long` | `int64_t` / `long long` | `i64` | 64-bit integer |
| `Float` | `float` | `f32` | 32-bit float |
| `Double` | `double` | `f64` | 64-bit float |
| `Boolean` | `bool` | `bool` | True/false |
| `String` | `std::string` | `String` | UTF-8 text |
| `ByteArray` | `std::vector<uint8_t>` | `Vec<u8>` | Binary data |
| `IntArray` | `std::vector<int32_t>` | `Vec<i32>` | Integer array |
| `FloatArray` | `std::vector<float>` | `Vec<f32>` | Float array |

### Example: All Simple Types

**C++:**
```cpp
// @ffi
int addInts(int a, int b) {
    return a + b;
}

// @ffi
double addDoubles(double a, double b) {
    return a + b;
}

// @ffi
bool isPositive(int n) {
    return n > 0;
}

// @ffi
std::string toUpper(const std::string& text) {
    std::string result = text;
    for (char& c : result) c = toupper(c);
    return result;
}

// @ffi
std::vector<int32_t> doubleArray(const std::vector<int32_t>& arr) {
    std::vector<int32_t> result;
    for (int val : arr) result.push_back(val * 2);
    return result;
}
```

**Rust:**
```rust
use whitehall_ffi::ffi;

#[ffi]
pub fn add_ints(a: i32, b: i32) -> i32 {
    a + b
}

#[ffi]
pub fn add_doubles(a: f64, b: f64) -> f64 {
    a + b
}

#[ffi]
pub fn is_positive(n: i32) -> bool {
    n > 0
}

#[ffi]
pub fn to_upper(text: String) -> String {
    text.to_uppercase()
}

#[ffi]
pub fn double_array(arr: Vec<i32>) -> Vec<i32> {
    arr.iter().map(|x| x * 2).collect()
}
```

**Usage:**
```whitehall
<script>
  import $ffi.cpp.Math

  onMount {
    val sum = Math.addInts(5, 3)              // 8
    val dsum = Math.addDoubles(1.5, 2.5)      // 4.0
    val pos = Math.isPositive(-3)             // false
    val upper = Math.toUpper("hello")         // "HELLO"
    val doubled = Math.doubleArray(intArrayOf(1, 2, 3))  // [2, 4, 6]
  }
</script>
```

**Zero boilerplate!** Just write your logic, Whitehall handles everything else.

---

## Complex Types (Manual Serialization)

For custom types (data classes, complex objects), use `ByteArray` for the FFI boundary and handle serialization yourself.

### Why This Approach?

**Problem:** Complex types don't map cleanly across FFI boundaries:
```kotlin
data class Face(
  val bounds: Rect,
  val confidence: Float,
  val landmarks: List<Point>  // Variable size!
)
```

Automatically marshalling this would require:
- JVM object allocation from C++
- Reflection-based constructor calls
- Recursive handling of nested objects
- Complex memory management

**Solution:** Use `ByteArray` as the FFI boundary. You control serialization.

### Example: Image Processing with Custom Types

**Step 1: Define FFI boundary with ByteArray**

**C++:**
```cpp
// src/ffi/cpp/vision.cpp

// @ffi
std::vector<uint8_t> detectFacesRaw(const std::vector<uint8_t>& imageBytes) {
    // 1. Deserialize image
    cv::Mat image = cv::imdecode(imageBytes, cv::IMREAD_COLOR);

    // 2. Detect faces (your logic)
    std::vector<cv::Rect> faces = detectFaces(image);

    // 3. Serialize results to bytes
    return serializeFaces(faces);
}

// Helper: Serialize faces to bytes
std::vector<uint8_t> serializeFaces(const std::vector<cv::Rect>& faces) {
    std::vector<uint8_t> bytes;

    // Write count
    int count = faces.size();
    bytes.insert(bytes.end(),
                 reinterpret_cast<uint8_t*>(&count),
                 reinterpret_cast<uint8_t*>(&count) + sizeof(count));

    // Write each face
    for (const auto& face : faces) {
        bytes.insert(bytes.end(),
                     reinterpret_cast<const uint8_t*>(&face),
                     reinterpret_cast<const uint8_t*>(&face) + sizeof(face));
    }

    return bytes;
}
```

**Rust:**
```rust
use whitehall_ffi::ffi;

#[ffi]
pub fn detect_faces_raw(image_bytes: Vec<u8>) -> Vec<u8> {
    // 1. Deserialize image
    let image = image::load_from_memory(&image_bytes).unwrap();

    // 2. Detect faces (your logic)
    let faces = detect_faces(&image);

    // 3. Serialize results
    serialize_faces(&faces)
}

fn serialize_faces(faces: &[Face]) -> Vec<u8> {
    let mut bytes = Vec::new();

    // Write count
    bytes.extend_from_slice(&(faces.len() as i32).to_le_bytes());

    // Write each face
    for face in faces {
        bytes.extend_from_slice(&face.x.to_le_bytes());
        bytes.extend_from_slice(&face.y.to_le_bytes());
        bytes.extend_from_slice(&face.width.to_le_bytes());
        bytes.extend_from_slice(&face.height.to_le_bytes());
        bytes.extend_from_slice(&face.confidence.to_le_bytes());
    }

    bytes
}
```

**Step 2: Create Kotlin wrapper with serialization**

```kotlin
// Kotlin data class
data class Face(
    val x: Int,
    val y: Int,
    val width: Int,
    val height: Int,
    val confidence: Float
) {
    companion object {
        fun listFromBytes(bytes: ByteArray): List<Face> {
            val buffer = ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)
            val count = buffer.int

            return (0 until count).map {
                Face(
                    x = buffer.int,
                    y = buffer.int,
                    width = buffer.int,
                    height = buffer.int,
                    confidence = buffer.float
                )
            }
        }
    }

    fun toBytes(): ByteArray {
        return ByteBuffer.allocate(20)
            .order(ByteOrder.LITTLE_ENDIAN)
            .putInt(x)
            .putInt(y)
            .putInt(width)
            .putInt(height)
            .putFloat(confidence)
            .array()
    }
}

// Helper function wrapping the FFI call
fun detectFaces(bitmap: Bitmap): List<Face> {
    // Convert Bitmap to ByteArray
    val stream = ByteArrayOutputStream()
    bitmap.compress(Bitmap.CompressFormat.PNG, 100, stream)
    val imageBytes = stream.toByteArray()

    // Call FFI (Whitehall handles the glue)
    val resultBytes = Vision.detectFacesRaw(imageBytes)

    // Deserialize results
    return Face.listFromBytes(resultBytes)
}
```

**Step 3: Use in Whitehall component**

```whitehall
<script>
  import $ffi.cpp.Vision

  @prop val image: Bitmap
  var faces: List<Face> = emptyList()
  var isProcessing = false

  suspend fun detectFaces() {
    isProcessing = true

    // High-level API (handles serialization)
    faces = detectFaces(image)

    isProcessing = false
  }

  onMount {
    launch { detectFaces() }
  }
</script>

<Column>
  @if (isProcessing) {
    <LoadingSpinner />
  } @else {
    <Text>Found {faces.size} faces</Text>

    @for (face in faces) {
      <Text>
        Face at ({face.x}, {face.y})
        confidence: {face.confidence}
      </Text>
    }
  }

  <Button onClick={launch { detectFaces() }}>
    Detect Again
  </Button>
</Column>
```

### Summary: Two-Tier System

| Aspect | Simple Types | Complex Types |
|--------|-------------|---------------|
| **FFI Boundary** | Native types (Int, String, etc.) | `ByteArray` |
| **Whitehall Generates** | All glue code | All glue code |
| **You Write (Native)** | Pure logic | Logic + serialization |
| **You Write (Kotlin)** | Nothing | Wrapper with deserialization |
| **Transparency** | 100% transparent | 1 wrapper function |

---

## Project Structure

### Single Language (C++)

```
my-app/
├── whitehall.toml
├── src/
│   ├── components/
│   │   └── VideoPlayer.wh
│   └── ffi/
│       └── cpp/
│           ├── video-decoder.cpp     # Your code with @ffi
│           └── video-decoder.h
└── build/                             # Generated by Whitehall
    ├── kotlin/
    │   └── com/example/ffi/
    │       └── VideoDecoder.kt        # Generated binding
    ├── jni/
    │   └── video_decoder_bridge.cpp   # Generated JNI glue
    └── cmake/
        └── CMakeLists.txt             # Generated build config
```

### Single Language (Rust)

```
my-app/
├── whitehall.toml
├── src/
│   ├── components/
│   │   └── ImageProcessor.wh
│   └── ffi/
│       └── rust/
│           ├── Cargo.toml            # You create once
│           └── lib.rs                # Your code with #[ffi]
└── build/                            # Generated by Whitehall
    ├── kotlin/
    │   └── com/example/ffi/
    │       └── ImageProcessor.kt     # Generated binding
    └── jni/
        └── rust_bridge.rs            # Generated JNI glue
```

### Mixed C++ + Rust

```
my-app/
├── whitehall.toml
├── src/
│   ├── components/
│   │   ├── VideoPlayer.wh            # Uses C++ FFI
│   │   └── ImageProcessor.wh         # Uses Rust FFI
│   └── ffi/
│       ├── cpp/
│       │   └── video-decoder.cpp
│       └── rust/
│           ├── Cargo.toml
│           └── lib.rs
└── build/
    └── kotlin/
        └── com/example/ffi/
            ├── cpp/
            │   └── VideoDecoder.kt
            └── rust/
                └── ImageProcessor.kt
```

---

## Configuration

### whitehall.toml

```toml
[project]
name = "my-app"
version = "0.1.0"

[android]
package = "com.example.myapp"
min_sdk = 24
target_sdk = 34

[ffi]
enabled = true                      # Enable FFI support

# Optional: C++ configuration
[ffi.cpp]
standard = "17"                     # C++ standard (11, 14, 17, 20)
flags = ["-O3", "-Wall"]            # Additional compiler flags
libraries = ["opencv", "ffmpeg"]    # System libraries to link

# Optional: Rust configuration
[ffi.rust]
profile = "release"                 # Build profile (debug, release)
targets = [                         # Target architectures
    "arm64-v8a",
    "armeabi-v7a"
]
```

### Annotation Syntax

#### C++ (Comment-Based)

```cpp
// @ffi
int myFunction(int a, int b) {
    return a + b;
}

// @ffi(name = "custom_name")  // Optional: custom export name
int internalFunction(int x) {
    return x * 2;
}
```

#### Rust (Attribute Macro)

```rust
use whitehall_ffi::ffi;

#[ffi]
pub fn my_function(a: i32, b: i32) -> i32 {
    a + b
}

#[ffi(name = "custom_name")]  // Optional: custom export name
pub fn internal_function(x: i32) -> i32 {
    x * 2
}
```

---

## Complete Examples

### Example 1: Math Library (C++)

**`src/ffi/cpp/math.cpp`:**
```cpp
#include <cmath>
#include <vector>
#include <algorithm>

// @ffi
int add(int a, int b) {
    return a + b;
}

// @ffi
int multiply(int a, int b) {
    return a * b;
}

// @ffi
double sqrt(double n) {
    return std::sqrt(n);
}

// @ffi
int sum(const std::vector<int32_t>& numbers) {
    return std::accumulate(numbers.begin(), numbers.end(), 0);
}

// @ffi
std::vector<int32_t> fibonacci(int n) {
    std::vector<int32_t> result;
    if (n <= 0) return result;

    result.push_back(0);
    if (n == 1) return result;

    result.push_back(1);
    for (int i = 2; i < n; i++) {
        result.push_back(result[i-1] + result[i-2]);
    }

    return result;
}
```

**Usage:**
```whitehall
<script>
  import $ffi.cpp.Math

  var result = 0
  var fibNumbers: IntArray = intArrayOf()

  fun calculate() {
    result = Math.add(Math.multiply(3, 4), 5)  // (3 * 4) + 5 = 17

    val numbers = intArrayOf(1, 2, 3, 4, 5)
    val total = Math.sum(numbers)  // 15

    fibNumbers = Math.fibonacci(10)  // [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
  }

  onMount {
    calculate()
  }
</script>

<Column>
  <Text>Result: {result}</Text>
  <Text>Fibonacci: {fibNumbers.joinToString(", ")}</Text>
  <Button onClick={calculate}>Calculate</Button>
</Column>
```

### Example 2: Image Processing (Rust)

**`src/ffi/rust/Cargo.toml`:**
```toml
[package]
name = "image-processor"
version = "0.1.0"

[lib]
crate-type = ["cdylib"]

[dependencies]
whitehall-ffi = "0.1"
image = "0.24"
```

**`src/ffi/rust/lib.rs`:**
```rust
use whitehall_ffi::ffi;
use image::{ImageBuffer, Rgba, DynamicImage, GenericImageView};

#[ffi]
pub fn blur(image_bytes: Vec<u8>, radius: f32) -> Vec<u8> {
    // Decode image
    let img = image::load_from_memory(&image_bytes).unwrap();

    // Apply blur
    let blurred = img.blur(radius);

    // Encode back to bytes
    let mut bytes = Vec::new();
    blurred.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
        .unwrap();

    bytes
}

#[ffi]
pub fn grayscale(image_bytes: Vec<u8>) -> Vec<u8> {
    let img = image::load_from_memory(&image_bytes).unwrap();
    let gray = img.grayscale();

    let mut bytes = Vec::new();
    gray.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
        .unwrap();

    bytes
}

#[ffi]
pub fn resize(image_bytes: Vec<u8>, width: i32, height: i32) -> Vec<u8> {
    let img = image::load_from_memory(&image_bytes).unwrap();
    let resized = img.resize(width as u32, height as u32, image::imageops::FilterType::Lanczos3);

    let mut bytes = Vec::new();
    resized.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
        .unwrap();

    bytes
}
```

**Helper (Kotlin):**
```kotlin
// Bitmap conversion helpers
fun Bitmap.toByteArray(): ByteArray {
    val stream = ByteArrayOutputStream()
    compress(Bitmap.CompressFormat.PNG, 100, stream)
    return stream.toByteArray()
}

fun ByteArray.toBitmap(): Bitmap {
    return BitmapFactory.decodeByteArray(this, 0, size)
}
```

**Usage:**
```whitehall
<script>
  import $ffi.rust.ImageProcessor

  @prop val originalImage: Bitmap
  var processedImage: Bitmap? = null
  var isProcessing = false

  suspend fun applyBlur() {
    isProcessing = true

    val imageBytes = originalImage.toByteArray()
    val blurred = ImageProcessor.blur(imageBytes, 5.0f)
    processedImage = blurred.toBitmap()

    isProcessing = false
  }

  suspend fun applyGrayscale() {
    isProcessing = true

    val imageBytes = originalImage.toByteArray()
    val gray = ImageProcessor.grayscale(imageBytes)
    processedImage = gray.toBitmap()

    isProcessing = false
  }

  suspend fun resize() {
    isProcessing = true

    val imageBytes = originalImage.toByteArray()
    val resized = ImageProcessor.resize(imageBytes, 512, 512)
    processedImage = resized.toBitmap()

    isProcessing = false
  }
</script>

<Column spacing={16}>
  <Row>
    <Image bitmap={originalImage} width={200} height={200} />
    @if (processedImage != null) {
      <Image bitmap={processedImage} width={200} height={200} />
    }
  </Row>

  @if (isProcessing) {
    <LoadingSpinner />
  } @else {
    <Row spacing={8}>
      <Button onClick={launch { applyBlur() }}>Blur</Button>
      <Button onClick={launch { applyGrayscale() }}>Grayscale</Button>
      <Button onClick={launch { resize() }}>Resize</Button>
    </Row>
  }
</Column>
```

### Example 3: Video Decoder (C++ with FFmpeg)

**`src/ffi/cpp/video-decoder.cpp`:**
```cpp
extern "C" {
#include <libavformat/avformat.h>
#include <libavcodec/avcodec.h>
#include <libswscale/swscale.h>
}

#include <vector>
#include <string>

// @ffi
std::vector<uint8_t> decodeFrame(const std::string& videoPath, int frameIndex) {
    AVFormatContext* formatCtx = nullptr;

    // Open video file
    if (avformat_open_input(&formatCtx, videoPath.c_str(), nullptr, nullptr) != 0) {
        return {};
    }

    if (avformat_find_stream_info(formatCtx, nullptr) < 0) {
        avformat_close_input(&formatCtx);
        return {};
    }

    // Find video stream
    int videoStream = -1;
    for (unsigned i = 0; i < formatCtx->nb_streams; i++) {
        if (formatCtx->streams[i]->codecpar->codec_type == AVMEDIA_TYPE_VIDEO) {
            videoStream = i;
            break;
        }
    }

    if (videoStream == -1) {
        avformat_close_input(&formatCtx);
        return {};
    }

    // Decode frame at index
    // ... FFmpeg decoding logic ...

    // Convert to RGBA bytes
    std::vector<uint8_t> frameBytes;
    // ... conversion logic ...

    avformat_close_input(&formatCtx);
    return frameBytes;
}

// @ffi
int getFrameCount(const std::string& videoPath) {
    AVFormatContext* formatCtx = nullptr;

    if (avformat_open_input(&formatCtx, videoPath.c_str(), nullptr, nullptr) != 0) {
        return -1;
    }

    if (avformat_find_stream_info(formatCtx, nullptr) < 0) {
        avformat_close_input(&formatCtx);
        return -1;
    }

    int videoStream = -1;
    for (unsigned i = 0; i < formatCtx->nb_streams; i++) {
        if (formatCtx->streams[i]->codecpar->codec_type == AVMEDIA_TYPE_VIDEO) {
            videoStream = i;
            break;
        }
    }

    int frameCount = formatCtx->streams[videoStream]->nb_frames;

    avformat_close_input(&formatCtx);
    return frameCount;
}
```

**Usage:**
```whitehall
<script>
  import $ffi.cpp.VideoDecoder

  @prop val videoPath: String
  var currentFrame: Bitmap? = null
  var frameIndex = 0
  var totalFrames = 0

  suspend fun loadFrame(index: Int) {
    val frameBytes = VideoDecoder.decodeFrame(videoPath, index)
    if (frameBytes.isNotEmpty()) {
      currentFrame = frameBytes.toBitmap()
      frameIndex = index
    }
  }

  onMount {
    totalFrames = VideoDecoder.getFrameCount(videoPath)
    launch { loadFrame(0) }
  }

  fun nextFrame() {
    if (frameIndex < totalFrames - 1) {
      launch { loadFrame(frameIndex + 1) }
    }
  }

  fun prevFrame() {
    if (frameIndex > 0) {
      launch { loadFrame(frameIndex - 1) }
    }
  }
</script>

<Column spacing={16}>
  @if (currentFrame != null) {
    <Image bitmap={currentFrame} />
  }

  <Text>Frame {frameIndex + 1} of {totalFrames}</Text>

  <Row spacing={8}>
    <Button onClick={prevFrame} enabled={frameIndex > 0}>
      Previous
    </Button>
    <Button onClick={nextFrame} enabled={frameIndex < totalFrames - 1}>
      Next
    </Button>
  </Row>
</Column>
```

---

## Type Mapping Reference

### Complete Type Mapping Table

| Kotlin Type | JNI Type | C++ Type | Rust Type | Auto-Marshalled | Notes |
|-------------|----------|----------|-----------|-----------------|-------|
| `Unit` | `void` | `void` | `()` | ✅ | No return value |
| `Boolean` | `jboolean` | `bool` | `bool` | ✅ | True/false |
| `Byte` | `jbyte` | `int8_t` | `i8` | ✅ | -128 to 127 |
| `Short` | `jshort` | `int16_t` | `i16` | ✅ | -32768 to 32767 |
| `Int` | `jint` | `int32_t` / `int` | `i32` | ✅ | 32-bit integer |
| `Long` | `jlong` | `int64_t` / `long long` | `i64` | ✅ | 64-bit integer |
| `Float` | `jfloat` | `float` | `f32` | ✅ | 32-bit float |
| `Double` | `jdouble` | `double` | `f64` | ✅ | 64-bit float |
| `String` | `jstring` | `std::string` | `String` | ✅ | UTF-8 text |
| `ByteArray` | `jbyteArray` | `std::vector<uint8_t>` | `Vec<u8>` | ✅ | Binary data |
| `IntArray` | `jintArray` | `std::vector<int32_t>` | `Vec<i32>` | ✅ | Integer array |
| `LongArray` | `jlongArray` | `std::vector<int64_t>` | `Vec<i64>` | ✅ | Long array |
| `FloatArray` | `jfloatArray` | `std::vector<float>` | `Vec<f32>` | ✅ | Float array |
| `DoubleArray` | `jdoubleArray` | `std::vector<double>` | `Vec<f64>` | ✅ | Double array |
| `Bitmap` | - | - | - | ❌ | Use ByteArray + conversion |
| `List<T>` | - | - | - | ❌ | Use ByteArray + serialization |
| Custom types | - | - | - | ❌ | Use ByteArray + serialization |

### Conversion Examples

**Primitives (Direct mapping):**
```cpp
// @ffi
int add(int a, int b) { return a + b; }
```
```kotlin
Math.add(5, 3)  // Direct call, no conversion
```

**Strings (Automatic):**
```cpp
// @ffi
std::string toUpper(const std::string& text) {
    std::string result = text;
    for (char& c : result) c = toupper(c);
    return result;
}
```
```kotlin
val upper = Text.toUpper("hello")  // "HELLO"
```

Whitehall generates:
- `jstring` → `const char*` → `std::string`
- `std::string` → `const char*` → `jstring`
- Automatic memory management

**Arrays (Automatic):**
```cpp
// @ffi
std::vector<int32_t> doubleValues(const std::vector<int32_t>& input) {
    std::vector<int32_t> result;
    for (int val : input) result.push_back(val * 2);
    return result;
}
```
```kotlin
val doubled = Arrays.doubleValues(intArrayOf(1, 2, 3))  // [2, 4, 6]
```

Whitehall generates:
- `jintArray` → `jint*` → `std::vector<int32_t>`
- `std::vector<int32_t>` → `jint*` → `jintArray`
- Automatic memory management

---

## Understanding JNI Complexity

This section explains **why** FFI is complex, so you appreciate what Whitehall automates for you.

### The Problem: Two Different Worlds

**Kotlin/Java side:**
- Garbage collected memory
- Objects on the heap
- Immutable strings
- Exceptions for errors

**C++/Rust side:**
- Manual memory management
- Raw pointers
- Null-terminated char arrays
- Return codes or crashes

### What Makes JNI Hard

#### 1. Manual Type Conversions

**Without Whitehall:**
```cpp
// Every string requires 3+ operations
extern "C" JNIEXPORT jstring JNICALL
Java_..._process(JNIEnv* env, jobject, jstring input) {
    // 1. Check null
    if (input == nullptr) return nullptr;

    // 2. Convert to C string
    const char* cStr = env->GetStringUTFChars(input, nullptr);
    if (cStr == nullptr) return nullptr;  // OutOfMemoryError

    // 3. Do work
    std::string result = process(cStr);

    // 4. MUST release (or memory leak!)
    env->ReleaseStringUTFChars(input, cStr);

    // 5. Convert back
    return env->NewStringUTF(result.c_str());
}
```

**With Whitehall:**
```cpp
// @ffi
std::string process(const std::string& input) {
    return doWork(input);  // Just your logic!
}
```

#### 2. Memory Management Hell

**Without Whitehall:**
```cpp
// ❌ MEMORY LEAK: Unreleased local references
for (int i = 0; i < 10000; i++) {
    jobject item = env->GetObjectArrayElement(array, i);
    // Process...
    // FORGOT: env->DeleteLocalRef(item);  // Leak!
}
```

**With Whitehall:**
- Automatic `DeleteLocalRef` calls
- Automatic `Release*` calls for arrays/strings
- Guaranteed cleanup even on errors

#### 3. Thread Safety Nightmare

**Without Whitehall:**
```cpp
JNIEnv* env = /* ... */;

std::thread([env]() {  // ❌ CRASH: env is thread-local!
    env->CallVoidMethod(...);
}).detach();

// Must manually attach thread:
JNIEnv* newEnv;
g_jvm->AttachCurrentThread(&newEnv, nullptr);
// ... do work ...
g_jvm->DetachCurrentThread();
```

**With Whitehall:**
- Handles thread attachment automatically
- Manages `JavaVM*` globally
- Safe callback mechanisms

#### 4. Name Mangling

**Without Whitehall:**
```cpp
// Must match EXACTLY:
extern "C" JNIEXPORT jint JNICALL
Java_com_example_myapp_ffi_Math_add(...)
//    └─ package ──┘ └─ class ─┘ └─ method
```

One typo = `UnsatisfiedLinkError` at runtime.

**With Whitehall:**
- Automatic name generation
- Compile-time verification
- No manual string munging

#### 5. Silent Failures

**Without Whitehall:**
```cpp
// ❌ If NewStringUTF fails, returns nullptr but no exception!
jstring str = env->NewStringUTF(data);
return str;  // Might be null!

// Must check EVERYTHING:
if (env->ExceptionCheck()) {
    env->ExceptionDescribe();
    env->ExceptionClear();
    return nullptr;
}
```

**With Whitehall:**
- Automatic exception checks
- Automatic error propagation
- Guaranteed error handling

### What Whitehall Automates

For every `@ffi` function, Whitehall generates:

1. ✅ **JNI function signature** with correct name mangling
2. ✅ **Type conversions** (jstring ↔ std::string, etc.)
3. ✅ **Null safety checks**
4. ✅ **Memory management** (Get* + Release* pairs)
5. ✅ **Exception handling**
6. ✅ **Thread safety** (when needed)
7. ✅ **Build configuration** (CMake/Cargo)
8. ✅ **Kotlin bindings** (external declarations)
9. ✅ **Library loading** (System.loadLibrary)

**Result:** You write 10 lines of clean C++/Rust, Whitehall generates 150+ lines of bulletproof JNI glue.

---

## Memory Management

### The Golden Rule

**JNI memory is NOT garbage collected!** You must manually manage:
- Local references
- Global references
- String/array buffers

**Whitehall handles all of this automatically.**

### What Whitehall Does

For every FFI function, Whitehall generates proper cleanup:

**Example: String parameter**
```cpp
// Your code:
// @ffi
std::string reverse(const std::string& input) {
    return std::string(input.rbegin(), input.rend());
}

// Whitehall generates:
extern "C" JNIEXPORT jstring JNICALL
Java_..._reverse(JNIEnv* env, jobject, jstring input) {
    if (input == nullptr) {
        return env->NewStringUTF("");
    }

    const char* cStr = env->GetStringUTFChars(input, nullptr);
    if (cStr == nullptr) {
        return nullptr;
    }

    std::string result = reverse(std::string(cStr));

    // ✅ Auto-generated: Release string
    env->ReleaseStringUTFChars(input, cStr);

    return env->NewStringUTF(result.c_str());
}
```

**Example: Array parameter**
```cpp
// Your code:
// @ffi
int sum(const std::vector<int32_t>& numbers) {
    return std::accumulate(numbers.begin(), numbers.end(), 0);
}

// Whitehall generates:
extern "C" JNIEXPORT jint JNICALL
Java_..._sum(JNIEnv* env, jobject, jintArray array) {
    if (array == nullptr) {
        return 0;
    }

    jsize length = env->GetArrayLength(array);
    jint* elements = env->GetIntArrayElements(array, nullptr);
    if (elements == nullptr) {
        return 0;
    }

    std::vector<int32_t> vec(elements, elements + length);
    int result = sum(vec);

    // ✅ Auto-generated: Release array
    // JNI_ABORT = don't copy back (we didn't modify)
    env->ReleaseIntArrayElements(array, elements, JNI_ABORT);

    return result;
}
```

### Common Memory Leaks (That Whitehall Prevents)

#### Leak 1: Unreleased String UTF Chars
```cpp
// ❌ Manual JNI: Easy to forget release
const char* name = env->GetStringUTFChars(input, nullptr);
return env->NewStringUTF(name);  // LEAK!

// ✅ Whitehall: Always releases
```

#### Leak 2: Unreleased Array Elements
```cpp
// ❌ Manual JNI: Easy to forget release
jint* elements = env->GetIntArrayElements(arr, nullptr);
// ... process ...
return result;  // LEAK!

// ✅ Whitehall: Always releases
```

#### Leak 3: Local References in Loops
```cpp
// ❌ Manual JNI: Leaks in large loops
for (int i = 0; i < 10000; i++) {
    jobject item = env->GetObjectArrayElement(array, i);
    // LEAK! (JVM has limited local reference slots)
}

// ✅ Whitehall: Auto-deletes local refs
```

---

## Thread Safety

### The Critical Rule

**`JNIEnv*` is thread-local!** You CANNOT:
- Store `JNIEnv*` in a global variable
- Pass `JNIEnv*` to another thread
- Use `JNIEnv*` from a C++ thread you created

### Whitehall's Thread Safety

**Current design: Simple functions are single-threaded**

If you need async/threaded operations:

**Option 1: Return immediately, process in Kotlin coroutines**
```cpp
// @ffi
std::vector<uint8_t> processImage(const std::vector<uint8_t>& image) {
    // Fast C++ processing (single-threaded)
    return result;
}
```

```kotlin
// Handle async in Kotlin
suspend fun processImageAsync(bitmap: Bitmap): Bitmap = withContext(Dispatchers.Default) {
    val bytes = bitmap.toByteArray()
    val processed = ImageProcessor.processImage(bytes)  // FFI call
    processed.toBitmap()
}
```

**Option 2: Future support for async FFI**
```cpp
// Future syntax (not implemented yet)
// @ffi(async = true)
std::future<std::vector<uint8_t>> processImageAsync(const std::vector<uint8_t>& image) {
    return std::async([image]() {
        // Whitehall handles thread attachment
        return processInBackground(image);
    });
}
```

---

## Error Handling

### Exception Propagation

Whitehall automatically propagates exceptions across the FFI boundary.

**C++:**
```cpp
// @ffi
int divide(int a, int b) {
    if (b == 0) {
        throw std::invalid_argument("Division by zero");
    }
    return a / b;
}
```

**Whitehall generates:**
```cpp
extern "C" JNIEXPORT jint JNICALL
Java_..._divide(JNIEnv* env, jobject, jint a, jint b) {
    try {
        return divide(a, b);
    } catch (const std::invalid_argument& e) {
        // Auto-generated: Throw to Kotlin
        jclass exClass = env->FindClass("java/lang/IllegalArgumentException");
        env->ThrowNew(exClass, e.what());
        return 0;
    } catch (const std::exception& e) {
        jclass exClass = env->FindClass("java/lang/RuntimeException");
        env->ThrowNew(exClass, e.what());
        return 0;
    }
}
```

**Kotlin:**
```kotlin
try {
    val result = Math.divide(10, 0)
} catch (e: IllegalArgumentException) {
    // Exception from C++!
    println("Error: ${e.message}")  // "Division by zero"
}
```

**Rust:**
```rust
#[ffi]
pub fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}
```

Whitehall converts `Result<T, String>` to exceptions automatically.

---

## Debugging Native Code

### Logging from Native Code

**C++:**
```cpp
#include <android/log.h>

#define LOG_TAG "MyNative"
#define LOGD(...) __android_log_print(ANDROID_LOG_DEBUG, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

// @ffi
int processData(const std::vector<uint8_t>& data) {
    LOGD("Processing %zu bytes", data.size());

    int result = compute(data);

    if (result < 0) {
        LOGE("Processing failed with code %d", result);
    }

    return result;
}
```

**Rust:**
```rust
use android_logger::{Config, FilterBuilder};
use log::{debug, error, info};

#[ffi]
pub fn process_data(data: Vec<u8>) -> i32 {
    debug!("Processing {} bytes", data.len());

    let result = compute(&data);

    if result < 0 {
        error!("Processing failed with code {}", result);
    }

    result
}
```

### Using Android Studio Debugger

1. **Set breakpoints** in C++/Rust source files
2. **Attach LLDB debugger**: Run → Debug
3. **Step through code** and inspect variables
4. **View native stack traces**

### Command-Line Debugging

```bash
# Capture crash log
adb logcat > crash.log

# Symbolicate with ndk-stack
$ANDROID_NDK/ndk-stack -sym build/intermediates/cmake/debug/obj/arm64-v8a -dump crash.log
```

### Memory Leak Detection

**Enable AddressSanitizer:**
```toml
# whitehall.toml
[ffi.cpp]
flags = ["-fsanitize=address", "-fno-omit-frame-pointer"]
```

ASan will detect:
- Memory leaks
- Buffer overflows
- Use-after-free
- Double-free

---

## Security Best Practices

### Input Validation

Always validate at the FFI boundary:

```cpp
// @ffi
std::string processFile(const std::string& path) {
    // Validate path
    if (path.empty()) {
        throw std::invalid_argument("Path cannot be empty");
    }

    // Check for path traversal
    if (path.find("..") != std::string::npos) {
        throw std::invalid_argument("Path traversal detected");
    }

    // Check allowed directory
    if (path.find("/data/data/com.example") != 0) {
        throw std::invalid_argument("Access denied");
    }

    // Safe to proceed
    return processFileImpl(path);
}
```

### Buffer Overflow Prevention

**C++ (Use std::string and std::vector):**
```cpp
// ✅ SAFE: Bounds-checked
std::vector<uint8_t> data(size);
data[i] = value;  // Throws if out of bounds in debug

// ❌ DANGEROUS: Manual buffer
uint8_t buffer[64];
strcpy(buffer, input);  // Buffer overflow!
```

**Rust (Automatic bounds checking):**
```rust
// ✅ Rust panics on out-of-bounds access
let mut data = vec![0u8; size];
data[i] = value;  // Safe
```

### Secure Data Handling

**Zero sensitive data after use:**
```cpp
// @ffi
bool verifyPassword(const std::string& password) {
    bool valid = checkPassword(password);

    // Zero memory
    std::fill(const_cast<char*>(password.data()),
              const_cast<char*>(password.data()) + password.size(),
              0);

    return valid;
}
```

---

## Testing FFI Code

### Strategy

1. **Unit test native code** (C++ with Google Test, Rust with cargo test)
2. **Integration test** FFI boundary (Kotlin JUnit)
3. **Mock FFI** in UI tests

### C++ Unit Testing

```cpp
// tests/math_test.cpp
#include <gtest/gtest.h>
#include "math.cpp"

TEST(MathTest, Addition) {
    EXPECT_EQ(add(2, 3), 5);
    EXPECT_EQ(add(-1, 1), 0);
}

TEST(MathTest, Division) {
    EXPECT_THROW(divide(10, 0), std::invalid_argument);
}
```

### Rust Unit Testing

```rust
// lib.rs
#[ffi]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }
}
```

Run: `cargo test`

### Integration Testing (Kotlin)

```kotlin
@Test
fun testNativeAdd() {
    val result = Math.add(2, 3)
    assertEquals(5, result)
}

@Test(expected = IllegalArgumentException::class)
fun testNativeDivideByZero() {
    Math.divide(10, 0)  // Should throw
}
```

---

## Troubleshooting

### "Library not found: libmath.so"

**Cause:** Library not built or not included in APK.

**Solution:**
```bash
# Check if library exists
find build -name "*.so"

# Verify ffi.enabled in whitehall.toml
[ffi]
enabled = true

# Clean rebuild
whitehall build --clean
```

### "UnsatisfiedLinkError: No implementation found"

**Cause:** JNI function name mismatch.

**Solution:** This shouldn't happen with Whitehall auto-generation. If it does, file a bug report with:
- Your `@ffi` annotation
- The function signature
- Build logs

### "SIGSEGV (Segmentation fault)"

**Cause:** Memory corruption (null pointer, buffer overflow, use-after-free).

**Debug:**
```bash
# Use ndk-stack
adb logcat | $ANDROID_NDK/ndk-stack -sym build/obj/arm64-v8a

# Or enable AddressSanitizer
[ffi.cpp]
flags = ["-fsanitize=address"]
```

### "cargo-ndk: command not found"

**Solution:**
```bash
cargo install cargo-ndk

# Add Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
```

---

## Requirements

### Minimum Versions

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| **Android NDK** | 25.0 | 26.0+ |
| **CMake** | 3.22.1 | 3.28+ |
| **Android API** | 24 | 26+ |
| **Rust** | 1.70 | 1.75+ |
| **cargo-ndk** | 3.0 | 3.5+ |

### Installation

```bash
# Check NDK
ls $ANDROID_HOME/ndk/

# Install Rust toolchain
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi

# Install cargo-ndk
cargo install cargo-ndk
```

---

## Implementation Status

### Phase 1: Design & Documentation ✅
- [x] Design FFI annotation approach
- [x] Define supported types
- [x] Document architecture
- [ ] Finalize annotation syntax

### Phase 2: Basic Code Generation (Planned)
- [ ] Parse C++ `@ffi` comments
- [ ] Parse Rust `#[ffi]` attributes
- [ ] Generate Kotlin bindings
- [ ] Generate JNI bridge code
- [ ] Generate CMake configuration

### Phase 3: Type Marshalling (Planned)
- [ ] Primitives (Int, Long, Float, etc.)
- [ ] String conversion
- [ ] Array conversion
- [ ] Error handling

### Phase 4: Advanced Features (Future)
- [ ] Async FFI
- [ ] Callback support
- [ ] Complex type serialization helpers
- [ ] Performance profiling

---

## Summary

### What You Write

**C++:**
```cpp
// @ffi
int add(int a, int b) {
    return a + b;
}
```

**Rust:**
```rust
#[ffi]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Whitehall:**
```whitehall
<script>
  import $ffi.cpp.Math

  val result = Math.add(5, 3)
</script>
```

### What Whitehall Generates

- ✅ JNI bridge code (150+ lines per function)
- ✅ Kotlin bindings
- ✅ Type conversions
- ✅ Memory management
- ✅ Error handling
- ✅ Build configuration
- ✅ Library loading

### The Result

**Write performance-critical code in C++/Rust. Use it seamlessly in Whitehall. Zero JNI boilerplate.**

---

**Questions or feedback?** File an issue at [whitehall/issues](https://github.com/yourusername/whitehall/issues)
