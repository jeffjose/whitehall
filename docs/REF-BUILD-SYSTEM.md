# Whitehall Build System Reference

**Complete guide to build commands and pipeline architecture**

---

## Status

✅ **Fully Implemented** (Phases 1-5 Complete)

All three build commands (`build`, `watch`, `run`) are production-ready.

---

## Quick Summary

The Whitehall build system provides three commands for different workflows:

| Command | Purpose | Behavior | Use Case |
|---------|---------|----------|----------|
| `whitehall init` | Project initialization | Create new project structure | Starting new projects |
| `whitehall compile` | Single-file transpilation | Transpile without Android project | Quick testing, code snippets |
| `whitehall build` | Production build | One-shot transpilation | CI/CD, releases, sharing |
| `whitehall watch` | Development loop | Continuous auto-rebuild | Active development |
| `whitehall run` | Quick test cycle | Build + install + launch | Testing on device |
| `whitehall toolchain` | Toolchain management | Install/list/clean tools | Managing Java/Gradle/SDK |
| `whitehall exec` | Execute with toolchain | Run command with project tools | Running gradle/adb commands |
| `whitehall shell` | Interactive shell | Launch shell with toolchain | Debugging with project environment |
| `whitehall doctor` | System health check | Verify toolchain status | Troubleshooting setup issues |

---

## Command Details

### `whitehall build`

**Philosophy:** "Create a deployable artifact"

**Workflow:**
```bash
whitehall build
# → Transpiles all .wh files to Kotlin
# → Generates complete Gradle/Android project
# → Writes to output directory (default: build/)
# → Exits when complete

cd build/
./gradlew assembleDebug     # Build APK
# or
./gradlew assembleRelease   # Production APK
```

**Characteristics:**
- ✅ One-time execution
- ✅ Clean build (or smart incremental)
- ✅ Generates complete standalone project
- ✅ Exit when done
- ❌ No file watching
- ❌ No device interaction

**Output Structure:**
```
build/                          # Configurable via whitehall.toml
├── settings.gradle.kts         # Generated
├── gradle.properties           # Generated
├── build.gradle.kts            # Generated (root)
├── gradlew                     # Copied from templates
├── gradlew.bat                 # Copied from templates
├── gradle/
│   └── wrapper/
│       ├── gradle-wrapper.jar
│       └── gradle-wrapper.properties
└── app/
    ├── build.gradle.kts        # Generated with dependencies
    ├── src/
    │   └── main/
    │       ├── AndroidManifest.xml
    │       └── kotlin/
    │           └── com/example/myapp/
    │               ├── MainActivity.kt
    │               ├── components/
    │               │   └── *.kt
    │               └── screens/
    │                   └── *.kt
    └── proguard-rules.pro
```

**Flags:**
- `--manifest-path <path>` - Path to whitehall.toml (like cargo)
- `--output-dir <path>` - Override output directory

**Implementation:** `src/commands/build.rs`

---

### `whitehall watch`

**Philosophy:** "Hot development loop with instant feedback"

**Workflow:**
```bash
whitehall watch
# → Initial transpilation
# → Watches src/ for changes
# → Auto-rebuilds on file save
# → Shows errors in real-time
# → Press Ctrl+C to stop
```

**Characteristics:**
- ✅ Long-running process
- ✅ File watching (notify crate)
- ✅ Incremental builds (only changed files)
- ✅ Real-time error feedback
- ✅ Colored output
- ❌ No APK building
- ❌ No device interaction

**Terminal Output:**
```
👀 Watching Whitehall project for changes...
   Press Ctrl+C to stop

🔨 Initial build...
   Transpiled 5 files
✅ Ready! Watching for changes...

📝 Change detected: src/components/Button.wh
   Transpiling Button.wh... ✓
✅ Build successful (42ms)

📝 Change detected: src/screens/HomeScreen.wh
   Transpiling HomeScreen.wh... ✗
❌ Build failed:
   Error in src/screens/HomeScreen.wh:12:5
   Undefined variable 'counte' (did you mean 'count'?)
```

**Watched Paths:**
- `src/**/*.wh` - All Whitehall source files
- `whitehall.toml` - Configuration changes trigger full rebuild

**Implementation:** `src/commands/watch.rs`

---

### `whitehall run`

**Philosophy:** "One command from code to running app"

**Workflow:**
```bash
whitehall run
# → Runs `whitehall build`
# → Runs `./gradlew assembleDebug`
# → Runs `adb install -r app-debug.apk`
# → Runs `adb shell am start -n {package}/.MainActivity`
# → App launches on device
# → Command exits
```

**Characteristics:**
- ✅ Complete automation
- ✅ Device detection
- ✅ APK building via Gradle
- ✅ Installation via ADB
- ✅ App launch
- ❌ No file watching (use `watch` + manual install for that)

**Prerequisites:**
- Android SDK installed (`adb` in PATH) - Or use bundled toolchain
- Device connected via USB OR emulator running
- USB debugging enabled on device

**Terminal Output:**
```
🚀 Building and running Whitehall app...

🔨 Step 1/4: Building...
   Transpiled 5 files
✅ Build complete

📱 Step 2/4: Checking for connected devices...
   Found 1 device(s)

🔧 Step 3/4: Building APK with Gradle...
   BUILD SUCCESSFUL in 12s

📲 Step 4/4: Installing and launching...
   Installing app... ✓
   Launching app... ✓

✅ App running on device!
```

**Error Handling:**
```bash
# No device connected
❌ No devices connected. Please:
   1. Connect a device via USB, or
   2. Start an emulator with: emulator -avd <name>

# Multiple devices (future: add --device flag)

---

### `whitehall init`

**Philosophy:** "Instant project scaffolding"

**Workflow:**
```bash
whitehall init my-app
cd my-app
# → Creates project structure
# → Generates whitehall.toml
# → Creates src/ directory with example .wh file
```

**Characteristics:**
- ✅ Zero-config setup
- ✅ Generates whitehall.toml with sensible defaults
- ✅ Creates basic project structure
- ✅ Includes starter example

**Implementation:** `src/commands/init.rs`

---

### `whitehall compile`

**Philosophy:** "Quick transpilation without full Android project"

**Workflow:**
```bash
whitehall compile src/components/Button.wh
# → Transpiles single file to Kotlin
# → Prints to stdout
# → No Android project generation

whitehall compile Button.wh --package com.myapp.components
# → Custom package name

whitehall compile Button.wh --no-package
# → Omit package declaration (for pasting into existing files)
```

**Characteristics:**
- ✅ Fast transpilation (no project overhead)
- ✅ Useful for testing syntax
- ✅ Can output to stdout or file
- ✅ Works on single .wh files

**Use Cases:**
- Quick syntax validation
- Testing transpiler output
- Generating code snippets
- Learning Whitehall syntax

**Implementation:** `src/commands/compile.rs`

---

### `whitehall toolchain`

**Philosophy:** "Explicit toolchain management"

**Subcommands:**

#### `whitehall toolchain install`
```bash
whitehall toolchain install
# → Reads whitehall.toml
# → Downloads required Java/Gradle/Android SDK
# → Caches in ~/.whitehall/toolchains/
```

#### `whitehall toolchain list`
```bash
whitehall toolchain list
# → Shows all installed toolchains
# → Displays versions and disk usage
```

#### `whitehall toolchain clean`
```bash
whitehall toolchain clean
# → Removes all cached toolchains
# → Frees disk space
```

**Characteristics:**
- ✅ Project-scoped (like rust-toolchain.toml)
- ✅ Automatic downloads on first build
- ✅ Shared cache across projects
- ✅ No system-wide pollution

**Learn more:** [REF-TOOLCHAIN.md](./REF-TOOLCHAIN.md)

**Implementation:** `src/commands/toolchain.rs`

---

### `whitehall exec`

**Philosophy:** "Run commands with project's toolchain"

**Workflow:**
```bash
whitehall exec gradle assembleDebug
# → Runs gradle with project's Java/Gradle versions
# → Sets JAVA_HOME, ANDROID_HOME automatically

whitehall exec adb devices
# → Uses project's Android SDK platform-tools

whitehall exec -- gradle tasks --all
# → Use -- to pass flags that start with -
```

**Characteristics:**
- ✅ Automatic environment setup
- ✅ No manual PATH modifications
- ✅ Works with any command

**Use Cases:**
- Running Gradle tasks directly
- Using ADB from project toolchain
- Custom build scripts

**Implementation:** `src/commands/exec.rs`

---

### `whitehall shell`

**Philosophy:** "Interactive shell with toolchain environment"

**Workflow:**
```bash
whitehall shell
# → Launches interactive shell (bash/zsh)
# → Sets JAVA_HOME, ANDROID_HOME, PATH
# → All commands use project's toolchain

# Now you can run:
gradle --version
adb devices
java -version
# All use project-specific versions
```

**Characteristics:**
- ✅ Full toolchain environment
- ✅ Persistent for entire session
- ✅ No manual setup

**Use Cases:**
- Interactive debugging
- Running multiple commands
- Exploring toolchain setup

**Implementation:** `src/commands/shell.rs`

---

### `whitehall doctor`

**Philosophy:** "Verify system health and toolchain status"

**Workflow:**
```bash
whitehall doctor
# → Checks whitehall.toml exists
# → Verifies toolchain installation
# → Tests Java/Gradle/Android SDK
# → Checks for common issues
# → Provides fix suggestions
```

**Sample Output:**
```
🔍 Checking Whitehall setup...

✅ whitehall.toml found
✅ Java 21 installed (~/.whitehall/toolchains/java/21/)
✅ Gradle 8.4 installed (~/.whitehall/toolchains/gradle/8.4/)
✅ Android SDK installed (~/.whitehall/toolchains/android/)
⚠️  No devices connected (run 'adb devices' or start emulator)

Overall: Ready for development
```

**Characteristics:**
- ✅ Comprehensive health check
- ✅ Actionable error messages
- ✅ Suggests fixes

**Use Cases:**
- Troubleshooting setup issues
- Verifying installation
- Checking device connectivity

**Implementation:** `src/commands/doctor.rs`
❌ Multiple devices connected. Please specify:
   whitehall run --device emulator-5554
```

**Implementation:** `src/commands/run.rs`

---

## Architecture

### Shared Build Pipeline

All three commands use the same core transpilation logic:

```
┌──────────────────────────────────────────────┐
│      Shared Build Pipeline                   │
│      (src/build_pipeline.rs)                 │
│                                              │
│  fn execute_build(config, clean) -> Result  │
│                                              │
│  1. Parse whitehall.toml                    │
│  2. Build store registry (cross-file)       │
│  3. Discover .wh files in src/              │
│  4. For each file:                          │
│     - Determine type (component/screen)     │
│     - Determine package path                │
│     - Transpile to Kotlin                   │
│  5. Generate Android scaffold               │
│  6. Generate MainActivity                   │
│  7. Write all files to output dir           │
│  8. Return BuildResult                      │
└──────────────────────────────────────────────┘
                   ▲
                   │
      ┌────────────┼────────────┐
      │            │            │
┌─────▼─────┐ ┌───▼────┐ ┌─────▼─────┐
│   build   │ │ watch  │ │    run    │
│           │ │        │ │           │
│ • Call    │ │ • Call │ │ • Call    │
│   once    │ │   loop │ │   once    │
│ • Report  │ │ • Watch│ │ • gradlew │
│ • Exit    │ │   files│ │ • adb     │
└───────────┘ │ • Auto │ └───────────┘
              │   build│
              └────────┘
```

### Module Structure

```
src/
├── main.rs                  # CLI entry point (clap)
├── lib.rs                   # Module exports
├── commands/
│   ├── mod.rs
│   ├── init.rs              # ✅ Project initialization
│   ├── build.rs             # ✅ One-shot build
│   ├── watch.rs             # ✅ File watching + loop
│   └── run.rs               # ✅ Build + gradle + adb
├── build_pipeline.rs        # ✅ Shared transpilation logic
├── config.rs                # ✅ Parse whitehall.toml
├── project.rs               # ✅ File discovery & classification
├── android_scaffold.rs      # ✅ Generate Gradle boilerplate
└── transpiler/              # ✅ Core transpiler (23/23 tests passing)
    ├── mod.rs
    ├── parser.rs
    ├── ast.rs
    ├── analyzer.rs
    └── codegen/
        └── compose.rs
```

---

## Configuration

### whitehall.toml

```toml
[project]
name = "my-app"
version = "0.1.0"

[android]
min_sdk = 24
target_sdk = 34
package = "com.example.myapp"

[build]
output_dir = "build"        # Where to generate Android project
optimize_level = "default"  # "default" or "aggressive" (future)

[toolchain]
java = "21"
gradle = "8.4"
agp = "8.2.0"
```

### File Type Detection

Whitehall automatically determines component types based on directory structure:

| Source Path | Type | Output Package | Output Path |
|-------------|------|----------------|-------------|
| `src/components/Button.wh` | Component | `{package}.components` | `Button.kt` |
| `src/screens/HomeScreen.wh` | Screen | `{package}.screens` | `HomeScreen.kt` |
| `src/stores/UserProfile.wh` | Store | `{package}.stores` | `UserProfile.kt` |
| `src/main.wh` | Main | `{package}` | `MainActivity.kt` |

**Package Mapping:**
```
whitehall.toml: package = "com.example.myapp"

src/components/Button.wh → com.example.myapp.components.Button
src/screens/Home.wh      → com.example.myapp.screens.Home
src/stores/UserProfile.wh → com.example.myapp.stores.UserProfile
src/main.wh              → com.example.myapp.MainActivity
```

---

## Implementation Phases

### Phase 1: Foundation (Shared Core) ✅ COMPLETE
**Goal:** Build the shared infrastructure that all commands depend on

**Status:** ✅ Complete (7-9 hours actual)

**Tasks:**
1. ✅ Config parsing (`src/config.rs`)
2. ✅ File discovery (`src/project.rs`)
3. ✅ Build pipeline (`src/build_pipeline.rs`)
4. ✅ Android scaffold (`src/android_scaffold.rs`)

**Milestone:** Foundation complete when:
- ✅ Can parse config
- ✅ Can discover files
- ✅ Can transpile all files
- ✅ Can generate complete Android project

---

### Phase 2: `whitehall build` Command ✅ COMPLETE
**Goal:** Implement the simplest command first to validate foundation

**Status:** ✅ Complete (2-3 hours actual)

**Tasks:**
1. ✅ Implement `commands/build.rs`
2. ✅ Update CLI (`src/main.rs`)
3. ✅ Manual testing
4. ✅ Documentation

**Milestone:** `build` command complete when:
- ✅ `whitehall build` successfully transpiles project
- ✅ Generated Kotlin code is correct and idiomatic
- ✅ Android project scaffold generated correctly
- ✅ `--manifest-path` flag works (like cargo)
- ✅ Error messages are clear

**Known Pending:**
- ⏳ Gradle wrapper generation (users run `gradle wrapper` manually for now)
- ⏳ Gradle build verification (`./gradlew assembleDebug`)
- ⏳ APK installation and device testing

---

### Phase 3: `whitehall watch` Command ✅ COMPLETE
**Goal:** Add file watching for development workflow

**Status:** ✅ Complete (3-4 hours actual)

**Dependencies:** Phase 2 complete

**Tasks:**
1. ✅ Add notify dependency
2. ✅ Implement `commands/watch.rs`
3. ✅ Update CLI
4. ✅ Manual testing

**Milestone:** `watch` command complete when:
- ✅ Detects file changes within 100ms
- ✅ Rebuilds only changed files (incremental)
- ✅ Shows clear error messages
- ✅ Handles Ctrl+C gracefully
- ✅ Debounces rapid changes

---

### Phase 4: `whitehall run` Command ✅ COMPLETE
**Goal:** Complete automation from code to running app

**Status:** ✅ Complete (3-4 hours actual)

**Dependencies:** Phase 2 complete (Phase 3 optional)

**Tasks:**
1. ✅ Device detection (`commands/run.rs`)
2. ✅ Gradle integration
3. ✅ ADB integration
4. ✅ Implement full command
5. ✅ Update CLI

**Milestone:** `run` command complete when:
- ✅ Detects no device → clear error
- ✅ Builds → installs → launches successfully
- ✅ Handles errors at each step
- ✅ Displays progress clearly

---

### Phase 5: Polish & Documentation ✅ COMPLETE
**Goal:** Production-ready experience

**Status:** ✅ Complete (1-2 hours actual)

**Dependencies:** Phases 2, 3, 4 complete

**Tasks:**
1. ✅ Error messages
2. ✅ Help text
3. ✅ Documentation
4. ✅ Integration tests

**Milestone:** Polish complete when:
- ✅ All commands have clear help text
- ✅ Error messages are actionable
- ✅ Documentation is complete
- ✅ Integration tests pass

---

## User Workflows

### Workflow 1: Getting Started
```bash
whitehall init my-app
cd my-app
# Edit src/main.wh
whitehall run
```

### Workflow 2: Active Development
```bash
# Terminal 1: Continuous feedback
whitehall watch

# Terminal 2: Edit files
vim src/components/Button.wh
# Save → see instant rebuild in Terminal 1

# When ready to test on device
cd build && ./gradlew installDebug
```

### Workflow 3: Quick Testing
```bash
# Edit some files
vim src/screens/HomeScreen.wh

# Test immediately
whitehall run
```

### Workflow 4: CI/CD
```bash
whitehall build
cd build
./gradlew assembleRelease --no-daemon
# Upload to Play Store
```

---

## Dependencies

### Cargo.toml

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
walkdir = "2.5"
notify = "6.1"  # For watch command
```

### External Tools

**Managed by Whitehall (bundled toolchain):**
- Java/JDK (for Gradle and Android toolchain)
- Gradle (build system)
- Android SDK (for building APKs)
- adb (device deployment - comes with Android SDK)

**Optional:**
- Android emulator (for testing without physical device)

---

## Known Gaps

### Pending Enhancements

1. **Gradle Wrapper Generation**
   - **Status:** Users run `gradle wrapper` manually
   - **Impact:** Low - one-time setup per project
   - **Effort:** 1-2 hours
   - **Priority:** Low

2. **Incremental Builds**
   - **Status:** Full rebuilds on every `build` call
   - **Impact:** Medium - slow for large projects (>50 files)
   - **Effort:** 4-6 hours
   - **Priority:** Medium
   - **Design:** Hash-based change detection, only rebuild changed files

3. **Build Profiles**
   - **Status:** Always builds debug
   - **Impact:** Low - can manually run `./gradlew assembleRelease`
   - **Effort:** 2-3 hours
   - **Priority:** Low
   - **Design:** `--release` flag, separate build configurations

4. **Source Maps**
   - **Status:** No source mapping
   - **Impact:** Medium - debugging shows Kotlin line numbers, not .wh
   - **Effort:** 6-8 hours
   - **Priority:** Medium
   - **Design:** Generate .map files, map Kotlin errors back to .wh

5. **Parallel Transpilation**
   - **Status:** Sequential file processing
   - **Impact:** Low-Medium - slow for large projects (>100 files)
   - **Effort:** 4-6 hours
   - **Priority:** Low
   - **Design:** Use rayon for parallel file transpilation

---

## Next Steps

### Short-term (After Current Priorities)
- 🔜 Build profiles (debug vs release)
- 🔜 Incremental builds with hash-based change detection
- 🔜 Progress indicators (spinner or progress bar)

### Medium-term
- 🔜 `whitehall clean` command (remove build directory)
- 🔜 `whitehall run --watch` (combined watch + auto-install)
- 🔜 Multiple device support (`--device` flag)
- 🔜 Build caching for faster clean builds

### Long-term
- 🔜 Source maps for debugging
- 🔜 Hot reload (partial app updates without full rebuild)
- 🔜 Remote builds (build on server, stream to device)
- 🔜 Bundle optimization (analyze and reduce APK size)
- 🔜 Custom build steps (plugin system for transformations)

---

## Key Files Reference

| File | Lines | Purpose |
|------|-------|---------|
| `src/commands/build.rs` | ~150 | Build command implementation |
| `src/commands/watch.rs` | ~200 | Watch command with file monitoring |
| `src/commands/run.rs` | ~300 | Run command with device integration |
| `src/build_pipeline.rs` | ~400 | Shared build logic |
| `src/config.rs` | ~150 | Parse whitehall.toml |
| `src/project.rs` | ~200 | File discovery and classification |
| `src/android_scaffold.rs` | ~500 | Generate Gradle boilerplate |

---

## Related Documentation

- [REF-OVERVIEW.md](./REF-OVERVIEW.md) - Architecture overview
- [REF-TRANSPILER.md](./REF-TRANSPILER.md) - Transpiler details
- [REF-TOOLCHAIN.md](./REF-TOOLCHAIN.md) - Toolchain management
- [REF-STATE-MANAGEMENT.md](./REF-STATE-MANAGEMENT.md) - State management patterns

---

*Last Updated: 2025-11-06*
*Version: 1.1*
*Status: Fully Implemented (Phases 1-5 Complete)*
