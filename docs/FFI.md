# FFI (Foreign Function Interface) Integration

**Status**: 🔄 Design Phase
**Last Updated**: 2025-01-04

---

## Overview

Whitehall supports calling code written in **C++**, **Rust**, or other systems languages through FFI (Foreign Function Interface). This enables performance-critical operations, reuse of existing libraries, and integration with low-level system APIs.

**Philosophy**: Language-agnostic FFI. Write performance-critical code in the best language for the job, import it seamlessly into Whitehall components.

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

Whitehall doesn't generate JNI wrapper code automatically (you write it manually in C++/Rust). It only:
1. Copies FFI code to build directory
2. Generates CMake/Cargo build configuration
3. Resolves `$ffi.*` imports to proper Kotlin package paths
4. Adds `System.loadLibrary()` calls

**You write the glue code:**
```kotlin
// Generated by whitehall in build/app/.../ffi/VideoDecoder.kt
package com.example.app.ffi

object VideoDecoder {
    external fun decodeFrame(videoUrl: String, frameIndex: Int): Bitmap

    init {
        System.loadLibrary("video-decoder")
    }
}
```

**Wait, actually we should generate this!** See [Implementation Plan](#implementation-plan) below.

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
