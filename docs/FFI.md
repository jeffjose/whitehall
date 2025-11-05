# FFI (Foreign Function Interface) Integration

**Status**: 🚧 In Development (Phases 1-2 Complete, Phase 3 Planned)
**Last Updated**: 2025-01-05

---

## Overview

Whitehall supports calling code written in **C++**, **Rust**, or other systems languages through FFI (Foreign Function Interface). This enables performance-critical operations, reuse of existing libraries, and integration with low-level system APIs.

**Philosophy**: Language-agnostic FFI. Write performance-critical code in the best language for the job, import it seamlessly into Whitehall components.

## Table of Contents

- [Why FFI?](#why-ffi)
- [Quick Start](#quick-start)
- [FFI in Standard Kotlin Android Apps](#ffi-in-standard-kotlin-android-apps)
- [FFI in Whitehall](#ffi-in-whitehall)
- [Project Structure](#project-structure)
- [Configuration](#configuration)
- [Type Marshalling Guide](#type-marshalling-guide)
- [Memory Management](#memory-management)
- [Thread Safety](#thread-safety)
- [Error Handling](#error-handling)
- [Complete Examples](#complete-examples)
- [Debugging Native Code](#debugging-native-code)
- [Security Best Practices](#security-best-practices)
- [Testing FFI Code](#testing-ffi-code)
- [Troubleshooting](#troubleshooting)
- [Performance Optimization](#performance-optimization)
- [Implementation Plan](#implementation-plan)

---

## Quick Start

**Want to add FFI to your Whitehall app? Here's the 30-second version:**

### For C++:
```bash
# 1. Create FFI directory
mkdir -p src/ffi

# 2. Add your C++ code
cat > src/ffi/MyNative.cpp << 'EOF'
#include <jni.h>
extern "C" JNIEXPORT jstring JNICALL
Java_com_yourapp_ffi_MyNative_hello(JNIEnv* env, jobject) {
    return env->NewStringUTF("Hello from C++!");
}
EOF

# 3. Add CMakeLists.txt
cat > src/ffi/CMakeLists.txt << 'EOF'
cmake_minimum_required(VERSION 3.22.1)
add_library(mynative SHARED MyNative.cpp)
EOF

# 4. Enable in whitehall.toml
echo '[ffi]
enabled = true' >> whitehall.toml

# 5. Build!
whitehall build
```

### For Rust:
```bash
# 1. Create FFI directory
mkdir -p src/ffi

# 2. Initialize Rust library
cd src/ffi && cargo init --lib && cd ../..

# 3. Configure Cargo.toml
cat >> src/ffi/Cargo.toml << 'EOF'
[lib]
crate-type = ["cdylib"]

[dependencies]
jni = "0.21"
EOF

# 4. Enable in whitehall.toml
echo '[ffi]
enabled = true' >> whitehall.toml

# 5. Build!
whitehall build
```

Now jump to [Complete Examples](#complete-examples) to see real-world usage!

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
   - Rust crates: image, tokio, serde (via Rust-JNI)

3. **Platform-specific optimizations**
   - SIMD instructions (AVX, NEON)
   - Hardware acceleration
   - Direct system calls

---

## FFI in Standard Kotlin Android Apps

### C++ Integration (via JNI)

**1. Write C++ code** (`src/main/cpp/native-lib.cpp`):
```cpp
#include <jni.h>
#include <string>

extern "C" JNIEXPORT jstring JNICALL
Java_com_example_myapp_MainActivity_stringFromJNI(
    JNIEnv* env,
    jobject /* this */) {
    std::string hello = "Hello from C++";
    return env->NewStringUTF(hello.c_str());
}
```

**2. Declare native method in Kotlin**:
```kotlin
class MainActivity : ComponentActivity() {
    external fun stringFromJNI(): String

    companion object {
        init {
            System.loadLibrary("native-lib")
        }
    }
}
```

**3. Configure CMake** (`CMakeLists.txt`):
```cmake
cmake_minimum_required(VERSION 3.22.1)
project("myapp")

add_library(native-lib SHARED native-lib.cpp)
find_library(log-lib log)
target_link_libraries(native-lib ${log-lib})
```

**4. Configure Gradle** (`build.gradle.kts`):
```kotlin
android {
    defaultConfig {
        externalNativeBuild {
            cmake {
                cppFlags += "-std=c++17"
            }
        }
    }
    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
        }
    }
}
```

### Rust Integration (via cargo-ndk)

**1. Write Rust code** (`src/ffi/rust/lib.rs`):
```rust
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

#[no_mangle]
pub extern "system" fn Java_com_example_myapp_RustLib_processImage(
    env: JNIEnv,
    _class: JClass,
    input: JString
) -> jstring {
    // Rust image processing
    let output = env.new_string("Processed by Rust").unwrap();
    output.into_raw()
}
```

**2. Configure Cargo** (`Cargo.toml`):
```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
jni = "0.21"
```

**3. Build with cargo-ndk**:
```bash
cargo ndk -t armeabi-v7a -t arm64-v8a -o ../jniLibs build --release
```

---

## FFI in Whitehall

### Architecture: Auto-Detection

Whitehall automatically detects FFI languages by file extensions and generates appropriate build configuration.

**Detection rules:**
- `*.cpp`, `*.cc`, `*.h`, `*.hpp` + `CMakeLists.txt` → **C++**
- `*.rs` + `Cargo.toml` → **Rust**
- `*.c` + `CMakeLists.txt` → **C**

**Both can coexist in the same project!**

---

## Project Structure

### Basic C++ FFI

```
my-app/
├── whitehall.toml
├── src/
│   ├── components/
│   │   └── VideoPlayer.wh         # Uses FFI
│   └── ffi/                        # FFI code directory
│       ├── CMakeLists.txt
│       ├── video-decoder.cpp
│       └── video-decoder.h
```

### Basic Rust FFI

```
my-app/
├── whitehall.toml
├── src/
│   ├── components/
│   │   └── ImageProcessor.wh
│   └── ffi/
│       ├── Cargo.toml
│       └── lib.rs
```

### Mixed C++ + Rust FFI

```
my-app/
├── whitehall.toml
├── src/
│   ├── components/
│   │   ├── VideoPlayer.wh         # Uses C++ FFI
│   │   └── ImageProcessor.wh      # Uses Rust FFI
│   └── ffi/
│       ├── cpp/                    # C++ subdirectory
│       │   ├── CMakeLists.txt
│       │   └── video-decoder.cpp
│       └── rust/                   # Rust subdirectory
│           ├── Cargo.toml
│           └── lib.rs
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
targets = ["arm64-v8a", "armeabi-v7a"]  # Target architectures
```

---

## Usage Examples

### Example 1: C++ Video Decoder

**src/ffi/CMakeLists.txt:**
```cmake
cmake_minimum_required(VERSION 3.22.1)
project("video-decoder")

add_library(video-decoder SHARED video-decoder.cpp)
find_library(log-lib log)
target_link_libraries(video-decoder ${log-lib})
```

**src/ffi/video-decoder.cpp:**
```cpp
#include <jni.h>
#include <android/bitmap.h>

extern "C" JNIEXPORT jobject JNICALL
Java_com_example_myapp_ffi_VideoDecoder_decodeFrame(
    JNIEnv* env,
    jobject thiz,
    jstring videoUrl,
    jint frameIndex) {

    // Fast C++ video decoding with FFmpeg
    // ... implementation ...

    return processedBitmap;
}
```

**src/components/VideoPlayer.wh:**
```whitehall
import $ffi.VideoDecoder

<script>
  @prop val videoUrl: String
  var currentFrame: Bitmap? = null
  var frameIndex = 0

  fun nextFrame() {
    currentFrame = VideoDecoder.decodeFrame(videoUrl, frameIndex++)
  }

  onMount {
    nextFrame()
  }
</script>

<Column spacing={16}>
  @if (currentFrame != null) {
    <Image bitmap={currentFrame} />
  }

  <Button onClick={nextFrame}>Next Frame</Button>
</Column>
```

### Example 2: Rust Image Processing

**src/ffi/Cargo.toml:**
```toml
[package]
name = "image-processor"
version = "0.1.0"

[lib]
crate-type = ["cdylib"]

[dependencies]
jni = "0.21"
image = "0.24"
```

**src/ffi/lib.rs:**
```rust
use jni::JNIEnv;
use jni::objects::{JClass, JString, JByteArray};
use jni::sys::jbyteArray;
use image::{ImageBuffer, Rgba};

#[no_mangle]
pub extern "system" fn Java_com_example_myapp_ffi_ImageProcessor_applyFilter(
    env: JNIEnv,
    _class: JClass,
    image_bytes: JByteArray,
) -> jbyteArray {
    // Fast Rust image filtering
    let bytes = env.convert_byte_array(image_bytes).unwrap();
    let img = image::load_from_memory(&bytes).unwrap();

    // Apply filter (e.g., gaussian blur)
    let filtered = img.blur(2.0);

    // Convert back to bytes and return
    let output_bytes = filtered.to_bytes();
    env.byte_array_from_slice(&output_bytes).unwrap()
}
```

**src/components/ImageProcessor.wh:**
```whitehall
import $ffi.ImageProcessor

<script>
  @prop val imageUrl: String
  var processed: Bitmap? = null
  var isProcessing = false

  fun applyFilter() {
    isProcessing = true
    launch {
      val original = loadImage(imageUrl)
      val bytes = original.toByteArray()
      val filtered = ImageProcessor.applyFilter(bytes)
      processed = Bitmap.fromByteArray(filtered)
      isProcessing = false
    }
  }

  onMount {
    applyFilter()
  }
</script>

<Column spacing={16}>
  @if (isProcessing) {
    <LoadingSpinner />
  } @else if (processed != null) {
    <Image bitmap={processed} />
  }

  <Button onClick={applyFilter}>Re-apply Filter</Button>
</Column>
```

### Example 3: Mixed C++ and Rust

**Project structure:**
```
src/ffi/
├── cpp/
│   ├── CMakeLists.txt
│   └── video-decoder.cpp
└── rust/
    ├── Cargo.toml
    └── lib.rs
```

**Usage in component:**
```whitehall
import $ffi.cpp.VideoDecoder
import $ffi.rust.ImageProcessor

<script>
  var frame: Bitmap? = null
  var filtered: Bitmap? = null

  onMount {
    launch {
      // Decode with C++ (FFmpeg)
      frame = VideoDecoder.decodeFrame(videoUrl, 0)

      // Process with Rust (image crate)
      filtered = ImageProcessor.applyFilter(frame.toByteArray())
    }
  }
</script>

<Image bitmap={filtered} />
```

---

## Import Resolution

### Syntax

| Import Statement | Resolved Kotlin Path | Notes |
|------------------|---------------------|-------|
| `$ffi.VideoDecoder` | `com.example.app.ffi.VideoDecoder` | Auto-detected language |
| `$ffi.cpp.VideoDecoder` | `com.example.app.ffi.cpp.VideoDecoder` | Explicit C++ |
| `$ffi.rust.ImageProcessor` | `com.example.app.ffi.rust.ImageProcessor` | Explicit Rust |

### Generated Kotlin Bindings

**Current State (Phases 1-2)**: You write Kotlin wrapper code manually.

**Future (Phase 3)**: Auto-generation from annotations or manifest files.

**What Whitehall does now:**
1. ✅ Copies FFI code to build directory
2. ✅ Generates CMake/Cargo build configuration
3. ✅ Resolves `$ffi.*` imports to proper Kotlin package paths
4. ✅ Adds `System.loadLibrary()` calls to generated code

**What you write manually:**
```kotlin
// You create: src/ffi/VideoDecoder.kt
package com.example.app.ffi

import android.graphics.Bitmap

object VideoDecoder {
    external fun decodeFrame(videoUrl: String, frameIndex: Int): Bitmap

    init {
        System.loadLibrary("video-decoder")
    }
}
```

**Import Resolution Rules:**

| Project Structure | Import | Resolves To |
|-------------------|--------|-------------|
| `src/ffi/` (single lang) | `$ffi.MyClass` | `com.example.app.ffi.MyClass` |
| `src/ffi/cpp/` | `$ffi.cpp.MyClass` | `com.example.app.ffi.cpp.MyClass` |
| `src/ffi/rust/` | `$ffi.rust.MyClass` | `com.example.app.ffi.rust.MyClass` |

**Name Conflict Resolution:**

If both C++ and Rust define the same class name:
```
src/ffi/
├── cpp/
│   └── ImageProcessor.kt  # Must use explicit import
└── rust/
    └── ImageProcessor.kt  # Must use explicit import
```

```whitehall
// ❌ Ambiguous - will fail
import $ffi.ImageProcessor

// ✅ Explicit - works
import $ffi.cpp.ImageProcessor
import $ffi.rust.ImageProcessor as RustImageProcessor
```

**Recommendation**: Use explicit paths (`$ffi.cpp.*`, `$ffi.rust.*`) in mixed-language projects.

---

## Requirements

### Minimum Versions

| Component | Minimum Version | Recommended | Notes |
|-----------|----------------|-------------|-------|
| **Android NDK** | 25.0 | 26.0+ | For C/C++ compilation |
| **CMake** | 3.22.1 | 3.28+ | Bundled with Android SDK |
| **Android API** | 24 (Nougat) | 26+ | `min_sdk` in whitehall.toml |
| **Rust** | 1.70 | 1.75+ | For Rust FFI |
| **cargo-ndk** | 3.0 | 3.5+ | Install: `cargo install cargo-ndk` |
| **JNI crate** | 0.21 | Latest | For Rust JNI bindings |

### Installation Check

```bash
# Check NDK
ls $ANDROID_HOME/ndk/

# Check CMake
cmake --version

# Check Rust toolchain
rustc --version
cargo --version

# Install cargo-ndk (if using Rust)
cargo install cargo-ndk

# Add Android targets for Rust
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

### Supported Architectures

Whitehall builds for these Android ABIs by default:

| ABI | CPU Architecture | Notes |
|-----|------------------|-------|
| `arm64-v8a` | ARMv8 64-bit | **Primary target** (95% of devices) |
| `armeabi-v7a` | ARMv7 32-bit | Legacy devices |
| `x86_64` | Intel 64-bit | Emulators only |
| `x86` | Intel 32-bit | Deprecated, emulators |

**Smart Default**: Debug builds only compile for `arm64-v8a` (faster iteration). Release builds compile for all targets.

---

## Type Marshalling Guide

### The Challenge

JNI requires converting between Kotlin/Java types and C/C++/Rust types. This is **the most error-prone part of FFI**.

### Complete Type Mapping Table

| Kotlin Type | JNI Type | C++ Type | Rust Type | Example |
|-------------|----------|----------|-----------|---------|
| `Boolean` | `jboolean` | `uint8_t` | `u8` | `0` or `1` |
| `Byte` | `jbyte` | `int8_t` | `i8` | `-128` to `127` |
| `Char` | `jchar` | `uint16_t` | `u16` | Unicode character |
| `Short` | `jshort` | `int16_t` | `i16` | `-32768` to `32767` |
| `Int` | `jint` | `int32_t` | `i32` | 32-bit integer |
| `Long` | `jlong` | `int64_t` | `i64` | 64-bit integer |
| `Float` | `jfloat` | `float` | `f32` | 32-bit float |
| `Double` | `jdouble` | `double` | `f64` | 64-bit float |
| `String` | `jstring` | `jstring` | `JString` | See [String Handling](#string-handling) |
| `ByteArray` | `jbyteArray` | `jbyteArray` | `JByteArray` | See [Array Handling](#array-handling) |
| `IntArray` | `jintArray` | `jintArray` | `JIntArray` | See [Array Handling](#array-handling) |
| `List<T>` | `jobject` | `jobject` | `JObject` | See [Collections](#collection-handling) |
| `Bitmap` | `jobject` | `jobject` | `JObject` | See [Bitmap Handling](#bitmap-handling) |
| `Unit` (void) | `void` | `void` | `()` | No return value |
| `Any?` (nullable) | `jobject` | `jobject` | `JObject` | Can be null |

### String Handling

#### C++ String Example

```cpp
#include <jni.h>
#include <string>

extern "C" JNIEXPORT jstring JNICALL
Java_com_example_ffi_StringUtils_reverseString(
    JNIEnv* env,
    jobject /* this */,
    jstring input) {

    // ⚠️  ALWAYS check for null!
    if (input == nullptr) {
        return env->NewStringUTF("");
    }

    // Convert jstring to C++ string
    const char* nativeString = env->GetStringUTFChars(input, nullptr);
    if (nativeString == nullptr) {
        return nullptr;  // OutOfMemoryError thrown
    }

    // Process the string
    std::string str(nativeString);
    std::reverse(str.begin(), str.end());

    // ⚠️  CRITICAL: Release the UTF chars!
    env->ReleaseStringUTFChars(input, nativeString);

    // Convert back to jstring
    return env->NewStringUTF(str.c_str());
}
```

#### Rust String Example

```rust
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

#[no_mangle]
pub extern "system" fn Java_com_example_ffi_StringUtils_reverseString(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    // Convert JString to Rust String
    let input_str: String = match env.get_string(&input) {
        Ok(s) => s.into(),
        Err(_) => return JString::default().into_raw(),
    };

    // Process the string
    let reversed: String = input_str.chars().rev().collect();

    // Convert back to JString
    let output = env.new_string(reversed)
        .expect("Couldn't create Java string!");

    output.into_raw()
}
```

### Array Handling

#### C++ Array Example

```cpp
extern "C" JNIEXPORT jint JNICALL
Java_com_example_ffi_ArrayUtils_sumArray(
    JNIEnv* env,
    jobject /* this */,
    jintArray array) {

    if (array == nullptr) {
        return 0;
    }

    // Get array length
    jsize length = env->GetArrayLength(array);

    // Get array elements (copy)
    jint* elements = env->GetIntArrayElements(array, nullptr);
    if (elements == nullptr) {
        return 0;
    }

    // Process array
    jint sum = 0;
    for (jsize i = 0; i < length; i++) {
        sum += elements[i];
    }

    // ⚠️  CRITICAL: Release array!
    // JNI_ABORT = don't copy back changes (we didn't modify)
    env->ReleaseIntArrayElements(array, elements, JNI_ABORT);

    return sum;
}
```

#### Rust Array Example

```rust
use jni::JNIEnv;
use jni::objects::{JClass, JIntArray};
use jni::sys::jint;

#[no_mangle]
pub extern "system" fn Java_com_example_ffi_ArrayUtils_sumArray(
    mut env: JNIEnv,
    _class: JClass,
    array: JIntArray,
) -> jint {
    // Convert to Rust Vec
    let elements: Vec<i32> = match env.get_int_array_elements(&array, jni::objects::ReleaseMode::NoCopyBack) {
        Ok(arr) => arr.to_vec(),
        Err(_) => return 0,
    };

    // Process
    elements.iter().sum()
}
```

### Bitmap Handling

**This is the most requested feature for image/video processing!**

#### Complete C++ Bitmap Example

```cpp
#include <jni.h>
#include <android/bitmap.h>
#include <android/log.h>

#define LOG_TAG "BitmapNative"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

extern "C" JNIEXPORT jobject JNICALL
Java_com_example_ffi_ImageProcessor_grayscale(
    JNIEnv* env,
    jobject /* this */,
    jobject bitmapIn) {

    AndroidBitmapInfo info;
    void* pixels;

    // Get bitmap info
    if (AndroidBitmap_getInfo(env, bitmapIn, &info) < 0) {
        LOGE("Failed to get bitmap info");
        return nullptr;
    }

    // Verify format (RGBA_8888)
    if (info.format != ANDROID_BITMAP_FORMAT_RGBA_8888) {
        LOGE("Bitmap format is not RGBA_8888");
        return nullptr;
    }

    // Lock pixels for reading
    if (AndroidBitmap_lockPixels(env, bitmapIn, &pixels) < 0) {
        LOGE("Failed to lock bitmap pixels");
        return nullptr;
    }

    // Create output bitmap
    jclass bitmapClass = env->FindClass("android/graphics/Bitmap");
    jmethodID createBitmapMethod = env->GetStaticMethodID(
        bitmapClass,
        "createBitmap",
        "(IILandroid/graphics/Bitmap$Config;)Landroid/graphics/Bitmap;"
    );

    jclass configClass = env->FindClass("android/graphics/Bitmap$Config");
    jfieldID rgba8888Field = env->GetStaticFieldID(
        configClass,
        "ARGB_8888",
        "Landroid/graphics/Bitmap$Config;"
    );
    jobject rgba8888 = env->GetStaticObjectField(configClass, rgba8888Field);

    jobject bitmapOut = env->CallStaticObjectMethod(
        bitmapClass,
        createBitmapMethod,
        info.width,
        info.height,
        rgba8888
    );

    void* pixelsOut;
    AndroidBitmap_lockPixels(env, bitmapOut, &pixelsOut);

    // Process pixels (convert to grayscale)
    uint32_t* pixelIn = (uint32_t*)pixels;
    uint32_t* pixelOut = (uint32_t*)pixelsOut;

    for (uint32_t y = 0; y < info.height; y++) {
        for (uint32_t x = 0; x < info.width; x++) {
            uint32_t pixel = pixelIn[y * info.width + x];

            // Extract RGBA components
            uint8_t r = (pixel >> 16) & 0xFF;
            uint8_t g = (pixel >> 8) & 0xFF;
            uint8_t b = pixel & 0xFF;
            uint8_t a = (pixel >> 24) & 0xFF;

            // Grayscale using luminosity method
            uint8_t gray = (uint8_t)(0.299 * r + 0.587 * g + 0.114 * b);

            // Reconstruct pixel
            pixelOut[y * info.width + x] =
                (a << 24) | (gray << 16) | (gray << 8) | gray;
        }
    }

    // ⚠️  CRITICAL: Unlock pixels!
    AndroidBitmap_unlockPixels(env, bitmapIn);
    AndroidBitmap_unlockPixels(env, bitmapOut);

    return bitmapOut;
}
```

#### CMakeLists.txt for Bitmap

```cmake
cmake_minimum_required(VERSION 3.22.1)
project("imageprocessor")

add_library(imageprocessor SHARED ImageProcessor.cpp)

# ⚠️  CRITICAL: Link against jnigraphics for Bitmap support!
find_library(log-lib log)
find_library(jnigraphics-lib jnigraphics)

target_link_libraries(imageprocessor
    ${log-lib}
    ${jnigraphics-lib}
)
```

#### Rust Bitmap Example

Rust doesn't have direct Bitmap API support, so you work with byte arrays:

```rust
use jni::JNIEnv;
use jni::objects::{JClass, JByteArray};
use jni::sys::jbyteArray;

#[no_mangle]
pub extern "system" fn Java_com_example_ffi_ImageProcessor_grayscaleBytes(
    mut env: JNIEnv,
    _class: JClass,
    image_bytes: JByteArray,
    width: i32,
    height: i32,
) -> jbyteArray {
    // Convert to Rust Vec<u8>
    let bytes = env.convert_byte_array(image_bytes)
        .expect("Failed to convert byte array");

    // Process RGBA bytes
    let mut output = Vec::with_capacity(bytes.len());

    for chunk in bytes.chunks(4) {  // RGBA = 4 bytes per pixel
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        let a = chunk[3];

        // Grayscale
        let gray = ((0.299 * r as f32) + (0.587 * g as f32) + (0.114 * b as f32)) as u8;

        output.extend_from_slice(&[gray, gray, gray, a]);
    }

    // Convert back to JByteArray
    env.byte_array_from_slice(&output)
        .expect("Failed to create byte array")
}
```

### Collection Handling

For complex types like `List<String>` or `Map<String, Int>`, you need to use JNI reflection:

```cpp
// Example: Get List<String> from Kotlin/Java
std::vector<std::string> getStringList(JNIEnv* env, jobject list) {
    std::vector<std::string> result;

    jclass listClass = env->GetObjectClass(list);
    jmethodID sizeMethod = env->GetMethodID(listClass, "size", "()I");
    jmethodID getMethod = env->GetMethodID(listClass, "get", "(I)Ljava/lang/Object;");

    jint size = env->CallIntMethod(list, sizeMethod);

    for (jint i = 0; i < size; i++) {
        jstring str = (jstring)env->CallObjectMethod(list, getMethod, i);
        const char* cstr = env->GetStringUTFChars(str, nullptr);
        result.push_back(std::string(cstr));
        env->ReleaseStringUTFChars(str, cstr);
        env->DeleteLocalRef(str);
    }

    return result;
}
```

**Recommendation**: Avoid complex collections in FFI. Prefer simple types (primitives, strings, byte arrays).

---

## Memory Management

### The Golden Rule

**JNI memory is NOT garbage collected automatically!** You must manually manage:
- Local references
- Global references
- String/array buffers

### Reference Types

| Reference Type | Scope | When to Use | How to Delete |
|----------------|-------|-------------|---------------|
| **Local Reference** | Single JNI function call | Most objects | Auto-deleted, or `DeleteLocalRef()` |
| **Global Reference** | Until explicitly deleted | Caching objects across calls | `DeleteGlobalRef()` |
| **Weak Global Reference** | Until explicitly deleted or GC'd | Caching without preventing GC | `DeleteWeakGlobalRef()` |

### Local References

```cpp
// ⚠️  PROBLEM: Memory leak in loop!
extern "C" JNIEXPORT void JNICALL
Java_com_example_Leak_processMany(JNIEnv* env, jobject, jobjectArray items) {
    jsize count = env->GetArrayLength(items);

    for (jsize i = 0; i < count; i++) {
        jobject item = env->GetObjectArrayElement(items, i);

        // Process item...

        // ❌ BAD: Local reference not deleted!
        // If count is large (10,000+), this will run out of memory!
    }
}

// ✅ SOLUTION: Delete local references
extern "C" JNIEXPORT void JNICALL
Java_com_example_Good_processMany(JNIEnv* env, jobject, jobjectArray items) {
    jsize count = env->GetArrayLength(items);

    for (jsize i = 0; i < count; i++) {
        jobject item = env->GetObjectArrayElement(items, i);

        // Process item...

        // ✅ GOOD: Delete local reference
        env->DeleteLocalRef(item);
    }
}
```

### Global References

```cpp
// Example: Cache a class reference for repeated use
jclass gBitmapClass = nullptr;

extern "C" JNIEXPORT void JNICALL
Java_com_example_Native_init(JNIEnv* env, jobject) {
    // Find class and create global reference
    jclass localClass = env->FindClass("android/graphics/Bitmap");
    gBitmapClass = (jclass)env->NewGlobalRef(localClass);
    env->DeleteLocalRef(localClass);  // Delete local after creating global
}

extern "C" JNIEXPORT void JNICALL
Java_com_example_Native_cleanup(JNIEnv* env, jobject) {
    // ⚠️  CRITICAL: Delete global reference when done!
    if (gBitmapClass != nullptr) {
        env->DeleteGlobalRef(gBitmapClass);
        gBitmapClass = nullptr;
    }
}
```

### Common Memory Leaks

#### Leak 1: Unreleased String UTF Chars

```cpp
// ❌ BAD
jstring getName(JNIEnv* env, jstring input) {
    const char* name = env->GetStringUTFChars(input, nullptr);
    std::string processed = std::string(name) + " processed";
    // ❌ MEMORY LEAK: Never released!
    return env->NewStringUTF(processed.c_str());
}

// ✅ GOOD
jstring getName(JNIEnv* env, jstring input) {
    const char* name = env->GetStringUTFChars(input, nullptr);
    std::string processed = std::string(name) + " processed";
    env->ReleaseStringUTFChars(input, name);  // ✅ Released!
    return env->NewStringUTF(processed.c_str());
}
```

#### Leak 2: Unreleased Array Elements

```cpp
// ❌ BAD
void processArray(JNIEnv* env, jintArray arr) {
    jint* elements = env->GetIntArrayElements(arr, nullptr);
    // Process...
    // ❌ MEMORY LEAK: Never released!
}

// ✅ GOOD
void processArray(JNIEnv* env, jintArray arr) {
    jint* elements = env->GetIntArrayElements(arr, nullptr);
    // Process...
    env->ReleaseIntArrayElements(arr, elements, 0);  // ✅ Released!
}
```

### Rust Memory Management

Good news: Rust's RAII handles most of this automatically!

```rust
use jni::JNIEnv;
use jni::objects::JString;

// ✅ Rust automatically manages memory
#[no_mangle]
pub extern "system" fn Java_com_example_Native_process(
    mut env: JNIEnv,
    _: JClass,
    input: JString
) -> jstring {
    // get_string() borrows, automatically released when dropped
    let rust_str: String = env.get_string(&input)
        .expect("Failed to get string")
        .into();

    let output = format!("{} processed", rust_str);

    // new_string() creates a new JString, returned as raw pointer
    env.new_string(output)
        .expect("Failed to create string")
        .into_raw()
}  // ✅ Rust's Drop trait handles cleanup
```

**Recommendation**: Use Rust for FFI if memory safety is a concern!

---

## Thread Safety

### The Critical Rule

**`JNIEnv*` is thread-local!** You CANNOT:
- Store `JNIEnv*` in a global variable
- Pass `JNIEnv*` to another thread
- Use `JNIEnv*` from a thread you created

### Calling Java from Native Threads

If you create threads in C++ (e.g., for async processing), you must attach them to the JVM:

```cpp
#include <jni.h>
#include <pthread.h>

// Global JavaVM pointer (thread-safe)
static JavaVM* g_jvm = nullptr;

// Called once when library loads
extern "C" JNIEXPORT jint JNICALL
JNI_OnLoad(JavaVM* vm, void* reserved) {
    g_jvm = vm;  // Cache JavaVM pointer
    return JNI_VERSION_1_6;
}

// Background thread function
void* backgroundWork(void* arg) {
    JNIEnv* env = nullptr;

    // ⚠️  CRITICAL: Attach thread to JVM!
    jint result = g_jvm->AttachCurrentThread(&env, nullptr);
    if (result != JNI_OK) {
        // Handle error
        return nullptr;
    }

    // Now you can use JNI calls
    jclass cls = env->FindClass("com/example/MyClass");
    // ... do work ...

    // ⚠️  CRITICAL: Detach when done!
    g_jvm->DetachCurrentThread();

    return nullptr;
}

// Start background thread
extern "C" JNIEXPORT void JNICALL
Java_com_example_Native_startBackground(JNIEnv* env, jobject) {
    pthread_t thread;
    pthread_create(&thread, nullptr, backgroundWork, nullptr);
    pthread_detach(thread);
}
```

### Calling Kotlin from C++ Callbacks

```cpp
// Example: Call Kotlin callback from C++ async operation
jobject g_callback = nullptr;  // Global reference to callback

extern "C" JNIEXPORT void JNICALL
Java_com_example_Native_setCallback(JNIEnv* env, jobject, jobject callback) {
    // Store global reference to callback
    if (g_callback != nullptr) {
        env->DeleteGlobalRef(g_callback);
    }
    g_callback = env->NewGlobalRef(callback);
}

void* asyncOperation(void* arg) {
    JNIEnv* env = nullptr;
    g_jvm->AttachCurrentThread(&env, nullptr);

    // Simulate work
    sleep(2);

    // Call Kotlin callback
    if (g_callback != nullptr) {
        jclass callbackClass = env->GetObjectClass(g_callback);
        jmethodID onCompleteMethod = env->GetMethodID(
            callbackClass,
            "onComplete",
            "(Ljava/lang/String;)V"
        );

        jstring result = env->NewStringUTF("Work completed!");
        env->CallVoidMethod(g_callback, onCompleteMethod, result);
        env->DeleteLocalRef(result);
    }

    g_jvm->DetachCurrentThread();
    return nullptr;
}
```

### Rust Thread Safety

```rust
use jni::{JavaVM, JNIEnv};
use std::thread;
use std::sync::Arc;

// Store JavaVM (thread-safe)
static mut JAVA_VM: Option<Arc<JavaVM>> = None;

#[no_mangle]
pub extern "system" fn JNI_OnLoad(vm: JavaVM, _reserved: *mut std::ffi::c_void) -> jint {
    unsafe {
        JAVA_VM = Some(Arc::new(vm));
    }
    jni::sys::JNI_VERSION_1_6
}

#[no_mangle]
pub extern "system" fn Java_com_example_Native_startBackground(
    _env: JNIEnv,
    _class: JClass
) {
    thread::spawn(move || {
        let jvm = unsafe { JAVA_VM.as_ref().unwrap().clone() };

        // Attach to current thread
        let mut env = jvm.attach_current_thread()
            .expect("Failed to attach thread");

        // Do JNI work...

        // env is automatically detached when dropped
    });
}
```

---

## Error Handling

### JNI Exception Handling

**Critical**: JNI functions can fail silently! You must check for exceptions.

```cpp
#include <jni.h>
#include <android/log.h>

#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, "Native", __VA_ARGS__)

// ❌ BAD: No error handling
jstring bad_function(JNIEnv* env, jstring input) {
    const char* str = env->GetStringUTFChars(input, nullptr);
    // What if this fails? Returns nullptr, but no error handling!
    return env->NewStringUTF(str);
}

// ✅ GOOD: Proper error handling
jstring good_function(JNIEnv* env, jstring input) {
    // Check for pending exception first
    if (env->ExceptionCheck()) {
        env->ExceptionDescribe();  // Print to logcat
        env->ExceptionClear();
        return nullptr;
    }

    // Check for null input
    if (input == nullptr) {
        // Throw Java exception
        jclass exClass = env->FindClass("java/lang/NullPointerException");
        env->ThrowNew(exClass, "Input string cannot be null");
        return nullptr;
    }

    // Get string
    const char* str = env->GetStringUTFChars(input, nullptr);
    if (str == nullptr) {
        // OutOfMemoryError already thrown by JVM
        return nullptr;
    }

    // Create output
    jstring result = env->NewStringUTF(str);
    env->ReleaseStringUTFChars(input, str);

    if (result == nullptr) {
        LOGE("Failed to create output string");
        return nullptr;
    }

    return result;
}
```

### Throwing Exceptions to Kotlin

```cpp
void throwException(JNIEnv* env, const char* className, const char* message) {
    jclass exClass = env->FindClass(className);
    if (exClass != nullptr) {
        env->ThrowNew(exClass, message);
    }
}

extern "C" JNIEXPORT jint JNICALL
Java_com_example_Native_divide(JNIEnv* env, jobject, jint a, jint b) {
    if (b == 0) {
        throwException(env, "java/lang/ArithmeticException", "Division by zero");
        return 0;  // Return value is ignored when exception is thrown
    }
    return a / b;
}
```

### Rust Error Handling

```rust
use jni::JNIEnv;
use jni::objects::JClass;

#[no_mangle]
pub extern "system" fn Java_com_example_Native_divide(
    mut env: JNIEnv,
    _class: JClass,
    a: i32,
    b: i32
) -> i32 {
    if b == 0 {
        // Throw exception to Kotlin
        env.throw_new(
            "java/lang/ArithmeticException",
            "Division by zero"
        ).expect("Failed to throw exception");
        return 0;
    }
    a / b
}

// Better: Use Result type
fn divide_safe(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

#[no_mangle]
pub extern "system" fn Java_com_example_Native_divideChecked(
    mut env: JNIEnv,
    _class: JClass,
    a: i32,
    b: i32
) -> i32 {
    match divide_safe(a, b) {
        Ok(result) => result,
        Err(msg) => {
            env.throw_new("java/lang/ArithmeticException", msg)
                .expect("Failed to throw exception");
            0
        }
    }
}
```

### Error Handling Best Practices

1. **Always check for null** before dereferencing
2. **Always check exceptions** after calling Java methods
3. **Use logging** (`__android_log_print` in C++, `android_logger` in Rust)
4. **Throw meaningful exceptions** to Kotlin/Java layer
5. **Document error conditions** in function comments

---

## Implementation Plan

### Phase 1: C++ Support (4-6 weeks)

**Goal**: Basic C++ FFI working end-to-end

**Tasks**:
1. ✅ **Config parsing** (`src/config.rs`)
   - Parse `[ffi]` section from `whitehall.toml`
   - Validate FFI configuration
   - Time: 2 hours

2. ✅ **File discovery** (`src/project.rs`)
   - Detect `src/ffi/` directory
   - Scan for `*.cpp` files and `CMakeLists.txt`
   - Classify as FFI code
   - Time: 2 hours

3. ✅ **FFI code copying** (`src/build_pipeline.rs`)
   - Copy `src/ffi/` → `build/app/src/main/cpp/`
   - Preserve directory structure
   - Time: 1 hour

4. ✅ **Gradle integration** (`src/android_scaffold.rs`)
   - Generate `externalNativeBuild` block in `build.gradle.kts`
   - Add CMake configuration
   - Time: 2-3 hours

5. ✅ **Kotlin binding generation** (NEW module: `src/ffi_bindings.rs`)
   - Parse C++ function signatures from comments or separate manifest
   - Generate Kotlin `object` with `external fun` declarations
   - Add `System.loadLibrary()` calls
   - Time: 8-12 hours

6. ✅ **Import resolution** (`src/transpiler/codegen.rs`)
   - Resolve `$ffi.*` to `{package}.ffi.*`
   - Add imports to generated Kotlin
   - Time: 2 hours

7. ✅ **Testing**
   - Create example project with C++ FFI
   - Test `whitehall build`
   - Test APK installation and execution
   - Time: 4-6 hours

**Milestone**: Can use C++ FFI in Whitehall components, APK builds and runs.

---

### Phase 2: Rust Support (2-3 weeks)

**Goal**: Add Rust FFI support via cargo-ndk

**Tasks**:
1. ✅ **Detect Rust code**
   - Scan for `Cargo.toml` in `src/ffi/`
   - Time: 1 hour

2. ✅ **Install cargo-ndk check**
   - Verify `cargo-ndk` is installed
   - Provide helpful error if missing
   - Time: 1 hour

3. ✅ **Build integration**
   - Run `cargo ndk build` during `whitehall build`
   - Copy built `.so` files to `build/app/src/main/jniLibs/`
   - Time: 4-6 hours

4. ✅ **Kotlin binding generation**
   - Parse Rust `#[no_mangle]` functions
   - Generate Kotlin wrappers
   - Time: 6-8 hours

5. ✅ **Testing**
   - Create example with Rust FFI
   - Test end-to-end
   - Time: 3-4 hours

**Milestone**: Can use both C++ and Rust FFI in same project.

---

### Phase 3: Auto-Generated Bindings (4-6 weeks)

**Goal**: Reduce boilerplate, auto-generate JNI glue code

**Option A: Annotation-based (simpler)**
```cpp
// src/ffi/video-decoder.cpp

/// @whitehall-export
/// fun decodeFrame(videoUrl: String, frameIndex: Int): Bitmap
extern "C" JNIEXPORT jobject JNICALL
Java_com_example_myapp_ffi_VideoDecoder_decodeFrame(...) {
    // ...
}
```

Whitehall parses `@whitehall-export` comments and generates Kotlin wrapper.

**Option B: Manifest file (more flexible)**
```yaml
# src/ffi/bindings.yaml
library: video-decoder

functions:
  - name: decodeFrame
    params:
      - name: videoUrl
        type: String
      - name: frameIndex
        type: Int
    returns: Bitmap
```

**Option C: Use existing tools**
- C++: Use SWIG or djinni
- Rust: Use mozilla/uniffi-rs

**Recommendation**: Start with Option A (comments), consider Option C later.

---

## Design Decisions

### 1. Why `src/ffi/` directory?

- Clear separation from Kotlin/Whitehall code
- Android convention: `src/main/cpp/` → we use `src/ffi/`
- Easy to exclude from Whitehall transpiler
- Mirrors output structure

### 2. Why auto-detection instead of explicit `[ffi.cpp]` vs `[ffi.rust]`?

- Simpler for users (drop in files, it works)
- Can mix C++ and Rust without config changes
- File extensions are unambiguous
- Build system detects and handles appropriately

### 3. Why not transpile FFI code?

- C++ and Rust are already mature, optimized languages
- Whitehall adds no value by transforming them
- Let CMake/Cargo do what they do best
- Whitehall's job: copy files, generate glue code, build integration

### 4. Why generate Kotlin bindings?

- Manual JNI boilerplate is error-prone
- Reduces friction for FFI adoption
- Type-safe Kotlin signatures from FFI metadata
- Users focus on FFI logic, not glue code

---

## Challenges & Solutions

### Challenge 1: JNI Function Naming

**Problem**: JNI requires specific function names
```cpp
Java_com_example_myapp_ffi_VideoDecoder_decodeFrame
```

**Solution**:
- Whitehall generates correct package-based names
- Provide template or generator tool
- Document naming convention clearly

---

### Challenge 2: Type Marshalling

**Problem**: Converting between Kotlin/Java types and C/C++/Rust types

**Solution**:
- Provide helper library for common conversions (Bitmap, String, arrays)
- Document type mapping table
- Use existing tools (JNI for C++, jni-rs for Rust)

| Kotlin Type | JNI Type | C++ Type | Rust Type |
|-------------|----------|----------|-----------|
| `String` | `jstring` | `jstring` | `JString` |
| `Int` | `jint` | `int32_t` | `i32` |
| `ByteArray` | `jbyteArray` | `jbyteArray` | `JByteArray` |
| `Bitmap` | `jobject` | `jobject` | `JObject` |

---

### Challenge 3: Multiple Architectures

**Problem**: Android supports multiple CPU architectures (arm64-v8a, armeabi-v7a, x86, x86_64)

**Solution**:
- CMake handles C++ compilation for all architectures automatically
- cargo-ndk handles Rust compilation with `-t` flag
- Generated APK includes all architectures

---

### Challenge 4: Build Performance

**Problem**: Compiling C++/Rust for multiple architectures is slow

**Solution**:
- Cache compiled `.so` files (only rebuild if source changes)
- Parallel compilation (CMake `-j`, cargo `-j`)
- Debug builds: compile for single architecture (arm64-v8a only)
- Release builds: compile for all architectures

---

## Success Criteria

### Phase 1 Complete When:
- ✅ Can write C++ code in `src/ffi/`
- ✅ `whitehall build` copies C++ code to correct location
- ✅ Generated Gradle includes `externalNativeBuild`
- ✅ Can import `$ffi.*` in Whitehall components
- ✅ APK builds successfully with embedded `.so`
- ✅ C++ functions callable from Whitehall components
- ✅ Example project demonstrating video/image processing

### Phase 2 Complete When:
- ✅ Same as above, but with Rust code
- ✅ Can mix C++ and Rust in same project
- ✅ Both `$ffi.cpp.*` and `$ffi.rust.*` imports work

### Phase 3 Complete When:
- ✅ Minimal manual JNI boilerplate required
- ✅ Kotlin bindings auto-generated from metadata
- ✅ Type conversions handled automatically
- ✅ Documentation and examples are comprehensive

---

## Future Enhancements

### Short-term
- 🔜 **Hot reload for FFI** - Detect C++/Rust changes, rebuild, reinstall
- 🔜 **FFI templates** - `whitehall ffi add cpp-image-processor`
- 🔜 **Prebuilt libraries** - Link against system OpenCV, FFmpeg, etc.
- 🔜 **Debug symbols** - Proper debugging for C++/Rust code

### Medium-term
- 🔜 **WASM support** - Compile Rust to WASM, run in WebView
- 🔜 **Cross-compilation** - Build on Linux for Android (Docker container)
- 🔜 **FFI marketplace** - Share FFI modules (OpenCV wrapper, ML inference, crypto)
- 🔜 **Automatic binding generation** - Zero-boilerplate FFI

### Long-term
- 🔜 **Multi-platform FFI** - Same FFI code for Android and iOS (via Kotlin Multiplatform)
- 🔜 **FFI benchmarking** - Built-in performance profiling
- 🔜 **FFI testing** - Unit test FFI functions from Whitehall tests

---

## Resources

### Learning JNI
- [Android NDK Documentation](https://developer.android.com/ndk/guides)
- [JNI Tips and Tricks](https://developer.android.com/training/articles/perf-jni)
- [Example JNI Projects](https://github.com/android/ndk-samples)

### Rust JNI
- [jni-rs crate](https://github.com/jni-rs/jni-rs)
- [cargo-ndk](https://github.com/bbqsrc/cargo-ndk)
- [Rust on Android Tutorial](https://mozilla.github.io/firefox-browser-architecture/experiments/2017-09-21-rust-on-android.html)

### Tools
- [SWIG](http://www.swig.org/) - Generate JNI bindings from C/C++
- [djinni](https://github.com/dropbox/djinni) - Cross-language interface generator
- [uniffi](https://mozilla.github.io/uniffi-rs/) - Rust FFI bindings generator

---

## Example Projects (Future)

```bash
# Create example with C++ FFI
whitehall init video-player --template cpp-ffi

# Create example with Rust FFI
whitehall init image-processor --template rust-ffi

# Create example with both
whitehall init multimedia-app --template mixed-ffi
```

---

**FFI support makes Whitehall a complete solution for high-performance Android apps!** 🚀
