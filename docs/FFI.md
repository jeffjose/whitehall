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

**Step 2: Build**
```bash
whitehall build
```

**That's it!** Whitehall automatically detects `src/ffi/cpp/` and builds it.

**Step 3: Use in your Whitehall component**
```whitehall
<script>
  import $ffi.cpp.Math

  var result = 0

  $onMount {
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
# Whitehall will auto-generate the ffi macro locally - no external dependencies needed!
```

**Step 3: Write Rust code with `#[ffi]` annotation**

Create `src/ffi/rust/lib.rs`:
```rust
use whitehall::ffi;

#[ffi]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[ffi]
pub fn multiply(a: f64, b: f64) -> f64 {
    a * b
}
```

> **Note:** Whitehall automatically generates a local `ffi_macro/` crate during build.
> No external dependencies or fragile path references needed!

**Step 4: Build**
```bash
whitehall build
```

**That's it!** Whitehall automatically detects `src/ffi/rust/` and builds it.

**Step 5: Use in your component**
```whitehall
<script>
  import $ffi.rust.Math

  var result = 0

  $onMount {
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
use whitehall::ffi;

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

  $onMount {
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
use whitehall::ffi;

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

  $onMount {
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

| Aspect | Simple Types | Complex Types (ByteArray) |
|--------|-------------|---------------|
| **FFI Boundary** | Native types (Int, String, etc.) | `ByteArray` |
| **Whitehall Generates** | All glue code | All glue code |
| **You Write (Native)** | Pure logic | Logic + serialization |
| **You Write (Kotlin)** | Nothing | Wrapper with deserialization |
| **Transparency** | 100% transparent | 1 wrapper function |
| **Best For** | All simple operations | Small/medium data (< 1MB) |

---

## Advanced: Opaque Handles (Zero-Copy Pattern)

For large data or stateful operations, you can use **opaque handles** instead of serialization. A handle is just a `Long` (64-bit integer) that represents a pointer or ID to data stored in native memory.

### Why Use Handles?

**Advantages:**
- ✅ **Zero-copy** - Data stays in native memory
- ✅ **Better performance** - No serialization overhead
- ✅ **Streaming access** - Get parts of data on demand
- ✅ **Stateful operations** - Keep expensive resources loaded (ML models, video decoders, database connections)

**Trade-offs:**
- ❌ **Manual lifecycle** - Must explicitly release handles
- ❌ **Memory leak risk** - Forgetting to release = leak
- ❌ **More complex API** - Object-oriented instead of functional

### C++ Handle Example

**C++ (you write):**
```cpp
// src/ffi/cpp/video-decoder.cpp

// Internal storage
struct VideoDecoder {
    AVFormatContext* format_ctx;
    AVCodecContext* codec_ctx;
    int video_stream_index;
};

std::unordered_map<int64_t, VideoDecoder> g_decoders;
int64_t g_next_handle = 1;

// @ffi
int64_t openVideo(const std::string& path) {
    VideoDecoder decoder;

    // Open video (expensive!)
    if (avformat_open_input(&decoder.format_ctx, path.c_str(), nullptr, nullptr) != 0) {
        return -1;  // Error
    }

    // ... initialize decoder ...

    // Store and return handle
    int64_t handle = g_next_handle++;
    g_decoders[handle] = std::move(decoder);
    return handle;
}

// @ffi
int getFrameCount(int64_t handle) {
    if (g_decoders.find(handle) == g_decoders.end()) {
        return -1;
    }
    return g_decoders[handle].format_ctx->streams[0]->nb_frames;
}

// @ffi
std::vector<uint8_t> getFrame(int64_t handle, int frameIndex) {
    if (g_decoders.find(handle) == g_decoders.end()) {
        return {};
    }

    // Decode specific frame (fast - decoder already open!)
    return decode_frame(g_decoders[handle], frameIndex);
}

// @ffi
void closeVideo(int64_t handle) {
    if (g_decoders.find(handle) == g_decoders.end()) {
        return;
    }

    // Clean up resources
    VideoDecoder& decoder = g_decoders[handle];
    avformat_close_input(&decoder.format_ctx);

    g_decoders.erase(handle);
}
```

**Kotlin wrapper:**
```kotlin
class VideoDecoder(private val handle: Long) : AutoCloseable {
    val frameCount: Int
        get() = VideoFFI.getFrameCount(handle)

    fun getFrame(index: Int): Bitmap {
        val bytes = VideoFFI.getFrame(handle, index)
        return bytes.toBitmap()
    }

    override fun close() {
        VideoFFI.closeVideo(handle)
    }
}

// Factory function
fun openVideo(path: String): VideoDecoder? {
    val handle = VideoFFI.openVideo(path)
    return if (handle >= 0) VideoDecoder(handle) else null
}
```

**Whitehall usage:**
```whitehall
<script>
  @prop val videoPath: String
  var decoder: VideoDecoder? = null
  var currentFrame: Bitmap? = null
  var frameIndex = 0

  $onMount {
    // Open once, keep decoder loaded
    decoder = openVideo(videoPath)
  }

  fun nextFrame() {
    val dec = decoder ?: return
    if (frameIndex < dec.frameCount - 1) {
      frameIndex++
      currentFrame = dec.getFrame(frameIndex)
    }
  }

  fun prevFrame() {
    val dec = decoder ?: return
    if (frameIndex > 0) {
      frameIndex--
      currentFrame = dec.getFrame(frameIndex)
    }
  }

  onUnmount {
    // CRITICAL: Clean up!
    decoder?.close()
  }
</script>

<Column>
  @if (currentFrame != null) {
    <Image bitmap={currentFrame} />
  }

  <Text>Frame {frameIndex + 1} / {decoder?.frameCount ?: 0}</Text>

  <Row>
    <Button onClick={prevFrame}>Previous</Button>
    <Button onClick={nextFrame}>Next</Button>
  </Row>
</Column>
```

### Rust Handle Example (Safer!)

Rust's ownership system makes handles much safer:

**Rust (you write):**
```rust
use whitehall::ffi;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Thread-safe global storage
lazy_static! {
    static ref DECODERS: Arc<Mutex<HashMap<i64, VideoDecoder>>> =
        Arc::new(Mutex::new(HashMap::new()));
    static ref NEXT_HANDLE: Arc<Mutex<i64>> = Arc::new(Mutex::new(1));
}

struct VideoDecoder {
    // ... decoder state ...
}

impl Drop for VideoDecoder {
    fn drop(&mut self) {
        // ✅ Automatic cleanup when dropped!
        println!("Cleaning up decoder");
    }
}

#[ffi]
pub fn open_video(path: String) -> i64 {
    let decoder = match VideoDecoder::open(&path) {
        Ok(d) => d,
        Err(_) => return -1,
    };

    let mut decoders = DECODERS.lock().unwrap();
    let mut next = NEXT_HANDLE.lock().unwrap();

    let handle = *next;
    *next += 1;

    decoders.insert(handle, decoder);
    handle
}

#[ffi]
pub fn get_frame_count(handle: i64) -> i32 {
    let decoders = DECODERS.lock().unwrap();
    decoders.get(&handle)
        .map(|d| d.frame_count())
        .unwrap_or(-1)
}

#[ffi]
pub fn get_frame(handle: i64, frame_index: i32) -> Vec<u8> {
    let mut decoders = DECODERS.lock().unwrap();
    decoders.get_mut(&handle)
        .and_then(|d| d.decode_frame(frame_index).ok())
        .unwrap_or_default()
}

#[ffi]
pub fn close_video(handle: i64) {
    let mut decoders = DECODERS.lock().unwrap();
    // ✅ Drop trait automatically cleans up when removed
    decoders.remove(&handle);
}
```

**Rust advantages for handles:**
1. ✅ **Drop trait** - Automatic cleanup, can't forget
2. ✅ **Type safety** - Can't use invalid handles (at compile time with proper design)
3. ✅ **Thread safety** - `Arc<Mutex<T>>` makes it thread-safe
4. ✅ **No undefined behavior** - Borrow checker prevents use-after-free
5. ✅ **Option<T>** - Explicit handling of missing handles

---

## Choosing Between Patterns

Both patterns work with Whitehall automatically (they only use simple types). Choose based on your needs:

### Pattern Comparison

| Aspect | ByteArray Serialization | Opaque Handles |
|--------|------------------------|----------------|
| **FFI Types Used** | `ByteArray` | `Long` + `ByteArray` |
| **Whitehall Support** | ✅ Automatic | ✅ Automatic |
| **Data Copies** | 5 (includes serialization) | 0 (zero-copy) |
| **Memory Owner** | Kotlin (GC) | Native (manual) |
| **Lifecycle** | Automatic | Manual (must call release) |
| **API Style** | Functional (stateless) | Object-oriented (stateful) |
| **Memory Leak Risk** | None | Medium (C++), Low (Rust) |
| **Thread Safety** | Easy | Complex (C++), Easy (Rust) |
| **Performance** | Good (< 1MB) | Excellent (any size) |
| **Code Complexity** | Low | Medium |
| **Best For** | Small/medium data, simple operations | Large data, stateful operations, streaming |

### Decision Guide

**Use ByteArray Pattern When:**
- ✅ Data is small/medium (< 1MB)
- ✅ Stateless operations (function call → result)
- ✅ Want simple, safe API
- ✅ Examples: Image filters, text processing, compression, encryption

**Use Handle Pattern When:**
- ✅ Data is large (> 1MB)
- ✅ Expensive to create/load (ML models, video decoders, databases)
- ✅ Need streaming access (get parts on demand)
- ✅ Zero-copy critical for performance
- ✅ Examples: Video playback, ML inference, real-time processing

**Use Both Together:**

You can mix patterns in the same project!

```cpp
// Handle pattern: Keep expensive resource loaded
// @ffi
int64_t loadModel(const std::string& path) { /* ... */ }

// ByteArray pattern: Process data with cached resource
// @ffi
std::vector<uint8_t> runInference(
    int64_t model_handle,
    const std::vector<uint8_t>& input_tensor
) { /* ... */ }

// Handle pattern: Clean up
// @ffi
void unloadModel(int64_t handle) { /* ... */ }
```

```whitehall
<script>
  var modelHandle: Long? = null

  $onMount {
    // Load expensive model once (handle)
    modelHandle = ML.loadModel("/path/to/model.tflite")
  }

  suspend fun analyze(image: Bitmap) {
    val handle = modelHandle ?: return

    // Pass data in/out efficiently (ByteArray)
    val input = image.toTensor()
    val output = ML.runInference(handle, input)

    return Prediction.fromBytes(output)
  }

  onUnmount {
    // Clean up
    modelHandle?.let { ML.unloadModel(it) }
  }
</script>
```

---

## Rust vs C++ for FFI

### C++ FFI
**Pros:**
- ✅ Rich ecosystem (OpenCV, FFmpeg, etc.)
- ✅ Maximum performance
- ✅ Widely used for Android NDK

**Cons:**
- ❌ Manual memory management (easy to leak)
- ❌ No built-in thread safety
- ❌ Undefined behavior if you mess up
- ❌ Handle pattern requires careful coding

### Rust FFI
**Pros:**
- ✅ **Memory safety enforced** - No leaks, use-after-free, or buffer overflows
- ✅ **Drop trait** - Automatic cleanup (RAII done right)
- ✅ **Thread safety** - `Send`/`Sync` traits prevent data races
- ✅ **Better error handling** - `Result<T, E>` instead of exceptions
- ✅ **Safer handles** - Borrow checker + type system prevent misuse
- ✅ **Modern tooling** - Cargo, built-in testing, great docs

**Cons:**
- ❌ Smaller ecosystem than C++
- ❌ Steeper learning curve
- ❌ Longer compile times

### Rust Makes Handles Safer

**C++ handle pattern risks:**
```cpp
// ❌ Easy mistakes in C++:
void processVideo(const std::string& path) {
    int64_t handle = openVideo(path);

    // Use handle...

    // FORGOT to call closeVideo(handle)!  → LEAK
}

// ❌ Use after free:
int64_t handle = openVideo(path);
closeVideo(handle);
getFrame(handle, 0);  // CRASH! Handle is invalid
```

**Rust prevents these at compile time:**
```rust
// ✅ Rust enforces cleanup:
pub fn process_video(path: &str) {
    let handle = open_video(path);

    // Use handle...

    // Compiler ERROR if you don't call close_video()
    // (if using proper RAII types)
}

// ✅ Can encode handle validity in type system:
pub struct VideoHandle(i64);

impl Drop for VideoHandle {
    fn drop(&mut self) {
        close_video(self.0);  // Always cleaned up!
    }
}

// Now impossible to forget cleanup or use freed handle!
```

### Recommendation

- **Use C++** if you need specific libraries (OpenCV, FFmpeg) or maximum performance
- **Use Rust** if memory safety is critical or you're building from scratch
- **Use both!** Different parts of your app can use different languages

**Both work identically from Whitehall's perspective** - the choice is based on your native code needs.

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
│           └── src/
│               └── lib.rs            # Your code with #[ffi]
└── build/                            # Generated by Whitehall
    ├── kotlin/
    │   └── com/example/ffi/
    │       └── ImageProcessor.kt     # Generated binding
    └── jni/
        └── rust_bridge.rs            # Generated JNI glue
```

**Note:** Current implementation writes `jni_bridge.rs` to `src/ffi/rust/src/` and modifies your `lib.rs` to add `mod jni_bridge;`. This is a known issue - generated code should stay in `build/`, not pollute `src/`. See [Issue: Generated Code in src/](#current-implementation-issue) below.

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
│           └── src/
│               └── lib.rs
└── build/
    ├── kotlin/
    │   └── com/example/ffi/
    │       ├── cpp/
    │       │   └── VideoDecoder.kt
    │       └── rust/
    │           └── ImageProcessor.kt
    └── jni/
        ├── cpp_bridge.cpp
        └── rust_bridge.rs
```

---

## Rust FFI Implementation Details

**✅ Fixed: Generated Code Now in `build/`**

As of the latest version, Rust FFI now correctly generates code to `build/` instead of polluting `src/`:

### How It Works

**Current behavior (`src/ffi_build.rs:392-463`):**
1. Generates full `jni_bridge.rs` into `build/generated/jni/rust/jni_bridge.rs` ✅
2. Auto-generates `build.rs` to copy bridge to `OUT_DIR` during compilation ✅
3. Creates minimal stub `src/ffi/rust/src/jni_bridge.rs` with just `include!` ✅
4. Adds single line `mod jni_bridge;` to `lib.rs` (acceptable one-time modification) ✅

**Why this is clean:**
- ✅ All meaningful generated code stays in `build/`
- ✅ Consistent with C++ FFI pattern
- ✅ `src/` only contains tiny stub (5 lines) that includes from `OUT_DIR`
- ✅ Standard Rust pattern using `build.rs` + `include!` macro

**Generated files:**
```
build/
└── generated/
    └── jni/
        └── rust/
            └── jni_bridge.rs           # Full generated JNI bridge

src/ffi/rust/
├── build.rs                            # Auto-generated (copies bridge to OUT_DIR)
├── src/
│   ├── lib.rs                          # User code + "mod jni_bridge;"
│   └── jni_bridge.rs                   # Minimal stub: include!(concat!(env!("OUT_DIR"), "/jni_bridge.rs"))
└── Cargo.toml
```

**What gets committed to git:**
- ✅ User's FFI code (`lib.rs` with your functions)
- ✅ Tiny stub file `jni_bridge.rs` (5 lines of comments + 1 line include!)
- ✅ Auto-generated `build.rs` (15 lines, handles copying)
- ✅ Auto-generated `ffi_macro/` crate (self-contained, no external dependencies)
- ❌ Full generated bridge (stays in `build/` ← gitignored)

### FFI Macro Generation

**The `#[ffi]` attribute is auto-generated locally - no external dependencies!**

When you run `whitehall build`, the system automatically generates a local `ffi_macro/` crate:

```
src/ffi/rust/
├── ffi_macro/                          # Auto-generated by Whitehall
│   ├── Cargo.toml                     # proc-macro = true
│   └── src/
│       └── lib.rs                     # Simple pass-through macro
├── build.rs
├── src/
│   ├── lib.rs
│   └── jni_bridge.rs
└── Cargo.toml                         # Updated with: whitehall = { path = "./ffi_macro" }
```

**Benefits:**
- ✅ No fragile relative paths like `../../../../../whitehall-ffi-macro`
- ✅ Self-contained - each Rust FFI project has its own macro
- ✅ Works out of the box - no manual setup needed
- ✅ Version controlled with your project

The macro itself is trivial (just a marker for the parser):
```rust
#[proc_macro_attribute]
pub fn ffi(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item  // Pass through unchanged
}
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
use whitehall::ffi;

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

  $onMount {
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
# Whitehall auto-generates the ffi macro - no external dependency needed
image = "0.24"
```

**`src/ffi/rust/lib.rs`:**
```rust
use whitehall::ffi;
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

<Column gap={16}>
  <Row>
    <Image bitmap={originalImage} width={200} height={200} />
    @if (processedImage != null) {
      <Image bitmap={processedImage} width={200} height={200} />
    }
  </Row>

  @if (isProcessing) {
    <LoadingSpinner />
  } @else {
    <Row gap={8}>
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

  $onMount {
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

<Column gap={16}>
  @if (currentFrame != null) {
    <Image bitmap={currentFrame} />
  }

  <Text>Frame {frameIndex + 1} of {totalFrames}</Text>

  <Row gap={8}>
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

## Implementation Roadmap

This section outlines the concrete, actionable phases for implementing FFI support in Whitehall.

---

## Phase 0: Foundation (Current)

**Goal:** Design and document the FFI system architecture.

**Status:** ✅ **COMPLETE**

**Tasks:**
- [x] Design annotation-based FFI approach (native code is source of truth)
- [x] Define simple types (primitives, String, arrays)
- [x] Define complex type patterns (ByteArray serialization, opaque handles)
- [x] Document complete architecture
- [x] Create examples for C++ and Rust

**Deliverables:**
- ✅ Comprehensive FFI.md documentation
- ✅ Clear design decisions
- ✅ Examples for all patterns

---

## Phase 1: Minimal Viable FFI (C++ Primitives Only)

**Goal:** Get the simplest possible FFI working end-to-end.

**Success Criteria:**
```cpp
// @ffi
int add(int a, int b) { return a + b; }
```
→ Works in Whitehall component with zero manual setup.

### Tasks

#### 1.1: C++ Annotation Parsing
**File:** `src/ffi_parser/cpp.rs`

```rust
pub struct CppFfiFunction {
    pub name: String,
    pub params: Vec<(String, CppType)>,
    pub return_type: CppType,
    pub source_file: PathBuf,
}

pub enum CppType {
    Int,
    Long,
    Float,
    Double,
    Bool,
}

pub fn discover_cpp_ffi(ffi_dir: &Path) -> Result<Vec<CppFfiFunction>> {
    // 1. Find all .cpp files in src/ffi/cpp/
    // 2. Parse each file for // @ffi annotations
    // 3. Extract function signatures
    // 4. Return list of discovered functions
}
```

**Implementation:**
- Use regex to find `// @ffi` followed by function signature
- Parse function signature (can start with simple regex, improve later)
- Support only primitives in Phase 1
- Return error for unsupported types

**Test:**
```rust
#[test]
fn test_parse_simple_function() {
    let cpp = r#"
        // @ffi
        int add(int a, int b) {
            return a + b;
        }
    "#;

    let functions = parse_cpp_ffi_from_string(cpp).unwrap();
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].name, "add");
}
```

---

#### 1.2: Kotlin Binding Generation
**File:** `src/ffi_codegen/kotlin_binding.rs`

```rust
pub fn generate_kotlin_binding(
    function: &CppFfiFunction,
    package: &str,
) -> String {
    // Generate:
    // external fun add(a: Int, b: Int): Int
}

pub fn generate_kotlin_object(
    functions: &[CppFfiFunction],
    package: &str,
    library_name: &str,
) -> String {
    // Generate:
    // object Math {
    //     external fun add(a: Int, b: Int): Int
    //     init { System.loadLibrary("math") }
    // }
}
```

**Output Example:**
```kotlin
// Generated: build/kotlin/com/example/ffi/Math.kt
package com.example.ffi

object Math {
    external fun add(a: Int, b: Int): Int
    external fun multiply(a: Int, b: Int): Int

    init {
        System.loadLibrary("math")
    }
}
```

---

#### 1.3: JNI Bridge Generation
**File:** `src/ffi_codegen/jni_bridge.rs`

```rust
pub fn generate_jni_bridge(
    function: &CppFfiFunction,
    package: &str,
) -> String {
    // Generate C++ JNI wrapper
}
```

**Output Example:**
```cpp
// Generated: build/jni/math_bridge.cpp
#include <jni.h>

// Forward declarations from user code
int add(int a, int b);
int multiply(int a, int b);

extern "C" JNIEXPORT jint JNICALL
Java_com_example_ffi_Math_add(
    JNIEnv* env,
    jobject thiz,
    jint a,
    jint b
) {
    return add(a, b);
}

extern "C" JNIEXPORT jint JNICALL
Java_com_example_ffi_Math_multiply(
    JNIEnv* env,
    jobject thiz,
    jint a,
    jint b
) {
    return multiply(a, b);
}
```

**Key Points:**
- Primitives map directly (jint → int)
- No error handling needed yet (Phase 2)
- No memory management needed (primitives are values)

---

#### 1.4: CMake Generation
**File:** `src/ffi_codegen/cmake.rs`

```rust
pub fn generate_cmake(
    library_name: &str,
    source_files: &[PathBuf],
    bridge_file: &Path,
) -> String {
    // Generate CMakeLists.txt
}
```

**Output Example:**
```cmake
# Generated: build/cmake/CMakeLists.txt
cmake_minimum_required(VERSION 3.22.1)
project("math")

add_library(math SHARED
    ${CMAKE_SOURCE_DIR}/src/ffi/cpp/math.cpp
    ${CMAKE_CURRENT_BINARY_DIR}/math_bridge.cpp
)

find_library(log-lib log)
target_link_libraries(math ${log-lib})

set(CMAKE_CXX_STANDARD 17)
```

---

#### 1.5: Build Integration
**File:** `src/build/ffi.rs`

```rust
pub fn build_ffi(config: &Config) -> Result<()> {
    if !config.ffi.enabled {
        return Ok(());
    }

    // 1. Discover FFI functions
    let functions = discover_cpp_ffi(&config.ffi_dir)?;

    // 2. Generate Kotlin bindings
    let kotlin_code = generate_kotlin_bindings(&functions, &config.package)?;
    write_file("build/kotlin/.../Math.kt", &kotlin_code)?;

    // 3. Generate JNI bridge
    let jni_code = generate_jni_bridge(&functions, &config.package)?;
    write_file("build/jni/math_bridge.cpp", &jni_code)?;

    // 4. Generate CMakeLists.txt
    let cmake_code = generate_cmake("math", &["src/ffi/cpp/math.cpp"], "build/jni/math_bridge.cpp")?;
    write_file("build/cmake/CMakeLists.txt", &cmake_code)?;

    // 5. Run CMake + Make
    run_cmake_build(&config)?;

    Ok(())
}
```

---

#### 1.6: Import Resolution
**File:** `src/transpiler/imports.rs`

```rust
// Resolve $ffi.cpp.Math to com.example.ffi.Math
pub fn resolve_ffi_import(import: &str, package: &str) -> Option<String> {
    if import.starts_with("$ffi.") {
        // $ffi.cpp.Math → com.example.ffi.Math
        let rest = &import[5..]; // Remove "$ffi."
        return Some(format!("{}.ffi.{}", package, rest));
    }
    None
}
```

---

### Phase 1 Test Case

**Input: `src/ffi/cpp/math.cpp`**
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

**Input: `src/components/Calculator.wh`**
```whitehall
<script>
  import $ffi.cpp.Math

  var result = 0

  $onMount {
    result = Math.add(5, 3)  // Should equal 8
  }
</script>

<Text>Result: {result}</Text>
```

**Command:**
```bash
whitehall build
```

**Expected Output:**
- Generates `build/kotlin/com/example/ffi/Math.kt`
- Generates `build/jni/math_bridge.cpp`
- Generates `build/cmake/CMakeLists.txt`
- Compiles `libmath.so`
- Bundles in APK
- App runs, displays "Result: 8"

**Phase 1 Complete When:** Code generation works! ✅

---

## Phase 1.6: Native Compilation ✅ COMPLETE

**Goal:** Compile C++ and Rust code into native `.so` libraries for all Android architectures.

**Current Status:**
- ✅ Code generation (Kotlin, JNI bridge, CMakeLists.txt, Cargo setup)
- ✅ **Actual native compilation (CMake/Cargo execution)**
- ✅ `.so` file bundling into APK
- ✅ Created `whitehall-ffi-macro` crate for #[ffi] attribute support

**Implementation Complete:**
Phase 1.6 has been fully implemented. Native compilation now works for both C++ (via CMake/NDK) and Rust (via cargo-ndk) for all Android architectures. The `ffi_only` setting is no longer required for FFI projects - native libraries are automatically compiled and bundled during the build process.

### Architecture

```
Native Compilation Pipeline
├── C++ Build (CMake + NDK)
│   ├── arm64-v8a     (aarch64)   ← Build in parallel
│   ├── armeabi-v7a   (armv7)     ← Build in parallel
│   ├── x86_64        (x86_64)    ← Build in parallel
│   └── x86           (i686)      ← Build in parallel
│
├── Rust Build (Cargo + NDK)
│   ├── aarch64-linux-android      ← Build in parallel
│   ├── armv7-linux-androideabi    ← Build in parallel
│   ├── x86_64-linux-android       ← Build in parallel
│   └── i686-linux-android         ← Build in parallel
│
└── Copy → build/jniLibs/{arch}/lib{name}.so
```

**Concurrency Model:**
- C++ and Rust builds run in parallel (if both present)
- Within each language, all architectures build in parallel
- Use Rayon or tokio for async parallel builds

### Tasks

#### 1.6.1: NDK Detection and Installation
**File:** `src/toolchain/mod.rs`

```rust
impl Toolchain {
    /// Ensure Android NDK is installed
    ///
    /// Downloads NDK if not present (~1GB download)
    pub fn ensure_ndk(&self) -> Result<PathBuf> {
        let sdk_root = self.ensure_android_sdk()?;
        let ndk_version = DEFAULT_NDK; // "26.1.10909125"
        let ndk_path = sdk_root.join("ndk").join(ndk_version);

        if ndk_path.exists() {
            return Ok(ndk_path);
        }

        // Install using sdkmanager
        // sdkmanager "ndk;26.1.10909125"
    }
}
```

**Requirements:**
- Android NDK 26+ (LTS with good CMake/Rust support)
- CMake 3.22+ (bundled with NDK)
- Rust toolchains for Android targets (if using Rust FFI)

#### 1.6.2: CMake Build (C++)
**File:** `src/ffi_build/native.rs` (new module)

```rust
/// Build C++ native library for a specific Android architecture
fn build_cpp_arch(
    ndk_path: &Path,
    cmake_dir: &Path,
    library_name: &str,
    arch: &str,  // "arm64-v8a", "armeabi-v7a", "x86_64", "x86"
) -> Result<PathBuf> {
    let build_dir = cmake_dir.join("build").join(arch);

    // 1. Configure CMake
    let android_abi = arch;  // arm64-v8a, armeabi-v7a, x86_64, x86
    let toolchain_file = ndk_path.join("build/cmake/android.toolchain.cmake");

    Command::new("cmake")
        .arg("-B").arg(&build_dir)
        .arg("-S").arg(cmake_dir)
        .arg(format!("-DCMAKE_TOOLCHAIN_FILE={}", toolchain_file.display()))
        .arg(format!("-DANDROID_ABI={}", android_abi))
        .arg("-DANDROID_PLATFORM=android-24")
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .status()?;

    // 2. Build
    Command::new("cmake")
        .arg("--build").arg(&build_dir)
        .arg("--config").arg("Release")
        .status()?;

    // 3. Find .so file
    let so_file = build_dir.join(format!("lib{}.so", library_name));
    Ok(so_file)
}

/// Build C++ for all architectures in parallel
pub fn build_cpp_all_archs(
    ndk_path: &Path,
    cmake_dir: &Path,
    library_name: &str,
) -> Result<HashMap<String, PathBuf>> {
    let archs = vec!["arm64-v8a", "armeabi-v7a", "x86_64", "x86"];

    // Build all archs in parallel using rayon
    let results: Vec<_> = archs
        .par_iter()
        .map(|arch| {
            println!("Building C++ for {}...", arch);
            let so_path = build_cpp_arch(ndk_path, cmake_dir, library_name, arch)?;
            Ok((arch.to_string(), so_path))
        })
        .collect();

    // Check for errors
    results.into_iter().collect()
}
```

**CMake Android Build Variables:**
- `CMAKE_TOOLCHAIN_FILE`: NDK's `android.toolchain.cmake`
- `ANDROID_ABI`: Target architecture (arm64-v8a, armeabi-v7a, x86_64, x86)
- `ANDROID_PLATFORM`: Min SDK (android-24)
- `CMAKE_BUILD_TYPE`: Release

#### 1.6.3: Cargo Build (Rust)
**File:** `src/ffi_build/native.rs`

```rust
/// Build Rust native library for a specific Android target
fn build_rust_target(
    cargo_dir: &Path,
    target: &str,  // "aarch64-linux-android", etc.
) -> Result<PathBuf> {
    // 1. Build with cargo
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target").arg(target)
        .current_dir(cargo_dir)
        .env("CARGO_TARGET_DIR", cargo_dir.join("target"))
        .status()?;

    // 2. Find .so file
    let so_file = cargo_dir
        .join("target")
        .join(target)
        .join("release")
        .join("lib{name}.so");

    Ok(so_file)
}

/// Build Rust for all targets in parallel
pub fn build_rust_all_targets(
    cargo_dir: &Path,
) -> Result<HashMap<String, PathBuf>> {
    let targets = vec![
        ("aarch64-linux-android", "arm64-v8a"),
        ("armv7-linux-androideabi", "armeabi-v7a"),
        ("x86_64-linux-android", "x86_64"),
        ("i686-linux-android", "x86"),
    ];

    // Build all targets in parallel
    let results: Vec<_> = targets
        .par_iter()
        .map(|(target, arch)| {
            println!("Building Rust for {} ({})...", arch, target);
            let so_path = build_rust_target(cargo_dir, target)?;
            Ok((arch.to_string(), so_path))
        })
        .collect();

    results.into_iter().collect()
}
```

**Rust Target Setup:**
```bash
# User must have Rust Android targets installed:
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android
```

**Cargo Configuration:**
Create `.cargo/config.toml` in project root:
```toml
[target.aarch64-linux-android]
linker = "<ndk>/toolchains/llvm/prebuilt/<host>/bin/aarch64-linux-android24-clang"

[target.armv7-linux-androideabi]
linker = "<ndk>/toolchains/llvm/prebuilt/<host>/bin/armv7a-linux-androideabi24-clang"

[target.x86_64-linux-android]
linker = "<ndk>/toolchains/llvm/prebuilt/<host>/bin/x86_64-linux-android24-clang"

[target.i686-linux-android]
linker = "<ndk>/toolchains/llvm/prebuilt/<host>/bin/i686-linux-android24-clang"
```

#### 1.6.4: Copy .so Files to jniLibs
**File:** `src/ffi_build/native.rs`

```rust
/// Copy compiled .so files to jniLibs directory
///
/// Gradle expects native libraries in:
/// build/jniLibs/{arch}/lib{name}.so
pub fn copy_to_jnilibs(
    so_files: &HashMap<String, PathBuf>,
    build_dir: &Path,
) -> Result<()> {
    for (arch, so_path) in so_files {
        let dest_dir = build_dir.join("jniLibs").join(arch);
        fs::create_dir_all(&dest_dir)?;

        let dest_file = dest_dir.join(so_path.file_name().unwrap());
        fs::copy(so_path, &dest_file)?;

        println!("  ✓ {} → {}", arch, dest_file.display());
    }
    Ok(())
}
```

**jniLibs Directory Structure:**
```
build/
└── jniLibs/
    ├── arm64-v8a/
    │   ├── libmath.so
    │   └── libvideo_encoder.so
    ├── armeabi-v7a/
    │   ├── libmath.so
    │   └── libvideo_encoder.so
    ├── x86_64/
    │   └── libmath.so
    └── x86/
        └── libmath.so
```

#### 1.6.5: Integration with Build Pipeline
**File:** `src/ffi_build.rs`

```rust
pub fn build_ffi(config: &Config, project_root: &Path) -> Result<()> {
    let ffi_dir = project_root.join("src/ffi");
    let build_dir = project_root.join(&config.build.output_dir);

    // Skip if ffi_only mode
    if config.ffi.ffi_only {
        // Only generate code, don't compile natives
        return Ok(());
    }

    // Get NDK
    let toolchain = Toolchain::new()?;
    let ndk_path = toolchain.ensure_ndk()?;

    // Parallel: Build C++ and Rust simultaneously
    let (cpp_result, rust_result) = rayon::join(
        || {
            if ffi_dir.join("cpp").exists() {
                println!("Building C++ native libraries...");
                build_cpp_all_archs(&ndk_path, &build_dir.join("cmake"), &library_name)
            } else {
                Ok(HashMap::new())
            }
        },
        || {
            if ffi_dir.join("rust").exists() {
                println!("Building Rust native libraries...");
                build_rust_all_targets(&ffi_dir.join("rust"))
            } else {
                Ok(HashMap::new())
            }
        },
    );

    let mut all_so_files = cpp_result?;
    all_so_files.extend(rust_result?);

    // Copy all .so files to jniLibs
    copy_to_jnilibs(&all_so_files, &build_dir)?;

    println!("Native libraries built successfully!");
    Ok(())
}
```

### Success Criteria

**Phase 1.6 Complete When:**
1. `whitehall build` (without `ffi_only = true`) successfully:
   - Detects/installs NDK
   - Runs CMake for C++ code (all archs in parallel)
   - Runs Cargo for Rust code (all targets in parallel)
   - Copies `.so` files to `build/jniLibs/{arch}/`
   - Gradle bundles `.so` files into APK
   - App loads native libraries successfully

2. Examples work without `ffi_only = true`:
   - `examples/ffi-cpp` builds full APK
   - `examples/ffi-rust` builds full APK
   - `examples/ffi-cpp-rust` builds full APK with both libraries

3. Build is fast:
   - Parallel compilation (all archs/targets at once)
   - Incremental builds (only rebuild if source changed)
   - NDK cached (~5 minutes first time, instant after)

### Dependencies

**Cargo.toml additions:**
```toml
[dependencies]
rayon = "1.7"  # Parallel iteration for multi-arch builds
```

**System Requirements (User):**
- Rust toolchains: `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android`
- CMake 3.22+ (bundled with NDK, handled automatically)

**System Requirements (Whitehall):**
- Android NDK 26+ (auto-downloaded by Whitehall)
- ~1GB disk space for NDK
- ~100MB per architecture for build artifacts

---

## Phase 2: String Support

**Goal:** Add String type marshalling with automatic memory management.

**Success Criteria:**
```cpp
// @ffi
std::string greet(const std::string& name) {
    return "Hello, " + name;
}
```
→ Works automatically with proper memory management.

### Tasks

#### 2.1: Extend Type System
```rust
pub enum CppType {
    Int, Long, Float, Double, Bool,
    String,  // ← New
}
```

#### 2.2: JNI Bridge for Strings
**Template:**
```cpp
extern "C" JNIEXPORT jstring JNICALL
Java_..._greet(JNIEnv* env, jobject thiz, jstring j_name) {
    // 1. Null check
    if (j_name == nullptr) {
        return env->NewStringUTF("");
    }

    // 2. Convert to C++ string
    const char* c_name = env->GetStringUTFChars(j_name, nullptr);
    if (c_name == nullptr) {
        return nullptr;
    }

    // 3. Call user function
    std::string result = greet(std::string(c_name));

    // 4. Release memory
    env->ReleaseStringUTFChars(j_name, c_name);

    // 5. Convert result
    return env->NewStringUTF(result.c_str());
}
```

**Code Generator:**
```rust
fn generate_string_param_conversion(param_name: &str) -> String {
    format!(r#"
    if ({0} == nullptr) {{
        // Handle null - throw exception or return default
    }}
    const char* c_{0} = env->GetStringUTFChars({0}, nullptr);
    if (c_{0} == nullptr) {{
        return nullptr;  // OutOfMemoryError
    }}
    std::string cpp_{0}(c_{0});
    env->ReleaseStringUTFChars({0}, c_{0});
    "#, param_name)
}

fn generate_string_return(expr: &str) -> String {
    format!(r#"
    std::string result = {0};
    jstring j_result = env->NewStringUTF(result.c_str());
    if (j_result == nullptr) {{
        return nullptr;  // OutOfMemoryError
    }}
    return j_result;
    "#, expr)
}
```

#### 2.3: Test Case
```cpp
// @ffi
std::string greet(const std::string& name) {
    return "Hello, " + name;
}

// @ffi
std::string toUpper(const std::string& text) {
    std::string result = text;
    for (char& c : result) c = toupper(c);
    return result;
}
```

```whitehall
<script>
  import $ffi.cpp.Text

  var greeting = Text.greet("Alice")  // "Hello, Alice"
  var upper = Text.toUpper("hello")    // "HELLO"
</script>
```

**Phase 2 Complete When:** String marshalling works with no memory leaks! ✅

---

## Phase 3: Array Support

**Goal:** Add array type marshalling (IntArray, FloatArray, ByteArray).

**Success Criteria:**
```cpp
// @ffi
std::vector<int32_t> doubleArray(const std::vector<int32_t>& arr) {
    std::vector<int32_t> result;
    for (int val : arr) result.push_back(val * 2);
    return result;
}
```
→ Works with automatic memory management.

### Tasks

#### 3.1: Extend Type System
```rust
pub enum CppType {
    Int, Long, Float, Double, Bool, String,
    IntArray,     // ← New
    FloatArray,   // ← New
    ByteArray,    // ← New
}
```

#### 3.2: JNI Bridge for Arrays
**Template for IntArray:**
```cpp
extern "C" JNIEXPORT jintArray JNICALL
Java_..._doubleArray(JNIEnv* env, jobject thiz, jintArray j_arr) {
    if (j_arr == nullptr) {
        return env->NewIntArray(0);
    }

    jsize length = env->GetArrayLength(j_arr);
    jint* elements = env->GetIntArrayElements(j_arr, nullptr);
    if (elements == nullptr) {
        return nullptr;
    }

    std::vector<int32_t> vec(elements, elements + length);
    std::vector<int32_t> result = doubleArray(vec);

    env->ReleaseIntArrayElements(j_arr, elements, JNI_ABORT);

    jintArray j_result = env->NewIntArray(result.size());
    env->SetIntArrayRegion(j_result, 0, result.size(), result.data());

    return j_result;
}
```

**Phase 3 Complete When:** All array types work! ✅

---

## Phase 4: Rust Support

**Goal:** Add Rust FFI with same functionality as C++.

### Tasks

#### 4.1: Rust Annotation Parsing
**File:** `src/ffi_parser/rust.rs`

Use `syn` crate to parse Rust AST:
```rust
pub fn discover_rust_ffi(ffi_dir: &Path) -> Result<Vec<RustFfiFunction>> {
    // 1. Find all .rs files
    // 2. Parse with syn
    // 3. Find functions with #[ffi] attribute
    // 4. Extract signatures
}
```

#### 4.2: Rust JNI Bridge Generation
Generate Rust code instead of C++:
```rust
// Generated bridge
use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::jint;

#[no_mangle]
pub extern "system" fn Java_com_example_ffi_Math_add(
    _env: JNIEnv,
    _class: JClass,
    a: jint,
    b: jint,
) -> jint {
    crate::add(a, b)
}
```

#### 4.3: Cargo Integration
Generate/modify `Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
jni = "0.21"
```

Build with `cargo ndk`.

**Phase 4 Complete When:** Rust FFI works identically to C++! ✅

---

## Phase 5: Error Handling

**Goal:** Automatic exception propagation across FFI boundary.

### Tasks

#### 5.1: Exception Mapping
```rust
fn exception_type_for_cpp(cpp_exception: &str) -> &str {
    match cpp_exception {
        "std::invalid_argument" => "java/lang/IllegalArgumentException",
        "std::runtime_error" => "java/lang/RuntimeException",
        "std::exception" => "java/lang/RuntimeException",
        _ => "java/lang/RuntimeException",
    }
}
```

#### 5.2: Try-Catch Generation
Wrap all user function calls:
```cpp
extern "C" JNIEXPORT jint JNICALL
Java_..._divide(JNIEnv* env, jobject thiz, jint a, jint b) {
    try {
        return divide(a, b);
    } catch (const std::invalid_argument& e) {
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

#### 5.3: Rust Result<T, E> Support
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

Bridge converts `Err` to exception automatically.

**Phase 5 Complete When:** Exceptions propagate correctly from native to Kotlin! ✅

---

## Phase 6: Documentation & Polish

**Goal:** Production-ready FFI system.

### Tasks

- [ ] Write comprehensive error messages
- [ ] Add validation for unsupported types
- [ ] Improve parser error reporting
- [ ] Add debugging flags (`WHITEHALL_FFI_DEBUG=1`)
- [ ] Performance profiling
- [ ] Memory leak detection in tests
- [ ] Complete user guide with troubleshooting
- [ ] Add CI tests for FFI

---

## Future Phases (Post-MVP)

### Phase 7: Advanced Features
- [ ] Async FFI (suspend functions)
- [ ] Callback support (native → Kotlin)
- [ ] Struct/dataclass helpers
- [ ] Proc macro for Rust (better DX)

### Phase 8: Optimization
- [ ] Zero-copy string views
- [ ] Direct buffer access
- [ ] JNI local frame management
- [ ] Parallel JNI calls

### Phase 9: Ecosystem
- [ ] Common serialization helpers (JSON, Protobuf)
- [ ] Image conversion utilities
- [ ] Audio buffer helpers
- [ ] Video frame utilities

---

## Current Status

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 0: Foundation | ✅ Complete | 100% |
| Phase 1: C++ Code Generation | ✅ Complete | 100% |
| Phase 1.6: Native Compilation | ✅ Complete | 100% |
| Phase 2: Strings | ✅ Complete | 100% |
| Phase 3: Arrays | ✅ Complete | 100% |
| Phase 4: Rust | ✅ Complete | 100% |
| Phase 5: Error Handling | ✅ Complete | 100% |
| Phase 6: Polish | ⏸️ Not Started | 0% |

### Phase 1, 2, 3, 4, & 5 Complete! 🎉

**Phase 1: C++ Primitives**
- ✅ C++ Annotation Parser
- ✅ Kotlin Binding Generator
- ✅ JNI Bridge Generator
- ✅ CMake Generator
- ✅ Build System Integration
- ✅ End-to-End Testing

**Phase 2: String Support**
- ✅ Extended CppType enum with String support
- ✅ Updated Kotlin bindings for String type
- ✅ JNI string conversions with memory management
- ✅ Null safety checks
- ✅ Comprehensive test coverage

**Phase 3: Array Support**
- ✅ Added array types (IntArray, LongArray, FloatArray, DoubleArray, BoolArray, StringArray)
- ✅ Template type parsing (`std::vector<T>` and `const std::vector<T>&`)
- ✅ JNI array conversions with proper memory management
- ✅ Automatic vector construction from JNI arrays
- ✅ Array return value conversions
- ✅ String array support (jobjectArray)
- ✅ Array size checks and null safety
- ✅ Comprehensive test coverage

**Phase 4: Rust Support**
- ✅ Rust FFI parser using `syn` crate for AST parsing
- ✅ #[ffi] attribute macro detection
- ✅ Rust type mapping (i32, i64, f32, f64, bool, String, Vec<T>)
- ✅ Rust JNI bridge generator with proper `jni` crate usage
- ✅ Automatic snake_case to camelCase conversion for Kotlin conventions
- ✅ Full support for primitives, strings, and arrays in Rust
- ✅ Proper memory management with JNI string and array conversions
- ✅ Integration with existing Kotlin binding generation
- ✅ Comprehensive test coverage for all Rust FFI types

**Phase 5: Error Handling**
- ✅ **C++ Exception Handling**:
  - Automatic try-catch wrapping for all C++ function calls
  - Exception type mapping (invalid_argument, out_of_range, runtime_error, exception)
  - Maps to appropriate Java exceptions (IllegalArgumentException, IndexOutOfBoundsException, RuntimeException)
  - Preserves error messages with e.what()
  - Safe default return values
  - Zero overhead when no exceptions thrown

- ✅ **Rust Result<T, E> Support**:
  - Parser detects Result<T, E> return types
  - Extracts inner type T for Kotlin bindings
  - Generates match expressions in Rust JNI bridge
  - Ok(value) → returns value
  - Err(e) → throws RuntimeException with error message
  - Works with all base types (primitives, strings, arrays)
  - Idiomatic Rust error handling with automatic Java exception conversion

**What Works Now:**
- C++ functions with `@ffi` annotations are automatically discovered
- Rust functions with `#[ffi]` attribute are automatically discovered
- Kotlin bindings are generated automatically for both C++ and Rust
- JNI bridge code is generated with proper type conversions and memory management
- CMake configuration is generated for C++
- Cargo configuration is supported for Rust
- **Supported types (C++):**
  - Primitives: int, long, float, double, bool, void
  - Strings: std::string (with automatic JNI conversion & memory management)
  - Arrays: std::vector<int>, std::vector<long>, std::vector<float>, std::vector<double>, std::vector<bool>, std::vector<std::string>
- **Supported types (Rust):**
  - Primitives: i32, i64, f32, f64, bool, () (unit/void)
  - Strings: String (with automatic JNI conversion & memory management)
  - Arrays: Vec<i32>, Vec<i64>, Vec<f32>, Vec<f64>, Vec<bool>, Vec<String>
- Automatic snake_case to camelCase conversion for Rust functions
- Full integration with `whitehall build` command
- Comprehensive test coverage

---

## Next Actions

**Phase 5 Complete! Next: Phase 6 - Polish & Production Readiness**

To start Phase 6:

1. Write comprehensive, helpful error messages in parsers
2. Add validation for unsupported types with suggestions
3. Improve parser error reporting with better context
4. Add debugging flags (WHITEHALL_FFI_DEBUG=1)
5. Performance profiling and optimization
6. Memory leak detection in tests
7. Complete troubleshooting guide with common issues
8. Add CI tests for FFI system

**Current Milestone:** Phase 6 - Production-ready FFI system with excellent developer experience

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
