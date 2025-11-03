# Whitehall Build System

**Status**: Planning → Implementation
**Last Updated**: 2025-11-03

## Overview

The Whitehall build system provides three commands for different workflows:

| Command | Purpose | Behavior | Use Case |
|---------|---------|----------|----------|
| `whitehall build` | Production build | One-shot transpilation | CI/CD, releases, sharing |
| `whitehall watch` | Development loop | Continuous auto-rebuild | Active development |
| `whitehall run` | Quick test cycle | Build + install + launch | Testing on device |

---

## Command Details

### `whitehall build`

**Philosophy**: "Create a deployable artifact"

**Workflow**:
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

**Characteristics**:
- ✅ One-time execution
- ✅ Clean build (or smart incremental)
- ✅ Generates complete standalone project
- ✅ Exit when done
- ❌ No file watching
- ❌ No device interaction

**Output Structure**:
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

---

### `whitehall watch`

**Philosophy**: "Hot development loop with instant feedback"

**Workflow**:
```bash
whitehall watch
# → Initial transpilation
# → Watches src/ for changes
# → Auto-rebuilds on file save
# → Shows errors in real-time
# → Press Ctrl+C to stop
```

**Characteristics**:
- ✅ Long-running process
- ✅ File watching (notify crate)
- ✅ Incremental builds (only changed files)
- ✅ Real-time error feedback
- ✅ Colored output
- ❌ No APK building
- ❌ No device interaction

**Terminal Output**:
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

**Watched Paths**:
- `src/**/*.wh` - All Whitehall source files
- `whitehall.toml` - Configuration changes trigger full rebuild

---

### `whitehall run`

**Philosophy**: "One command from code to running app"

**Workflow**:
```bash
whitehall run
# → Runs `whitehall build`
# → Runs `./gradlew assembleDebug`
# → Runs `adb install -r app-debug.apk`
# → Runs `adb shell am start -n {package}/.MainActivity`
# → App launches on device
# → Command exits
```

**Characteristics**:
- ✅ Complete automation
- ✅ Device detection
- ✅ APK building via Gradle
- ✅ Installation via ADB
- ✅ App launch
- ❌ No file watching (use `watch` + manual install for that)

**Prerequisites**:
- Android SDK installed (`adb` in PATH)
- Device connected via USB OR emulator running
- USB debugging enabled on device

**Terminal Output**:
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

**Error Handling**:
```bash
# No device connected
❌ No devices connected. Please:
   1. Connect a device via USB, or
   2. Start an emulator with: emulator -avd <name>

# Multiple devices (future: add --device flag)
❌ Multiple devices connected. Please specify:
   whitehall run --device emulator-5554
```

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
│  2. Discover .wh files in src/              │
│  3. For each file:                          │
│     - Determine type (component/screen)     │
│     - Determine package path                │
│     - Transpile to Kotlin                   │
│  4. Generate Android scaffold               │
│  5. Generate MainActivity                   │
│  6. Write all files to output dir           │
│  7. Return BuildResult                      │
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
│   ├── init.rs              # ✅ Already exists
│   ├── build.rs             # NEW: One-shot build
│   ├── watch.rs             # NEW: File watching + loop
│   └── run.rs               # NEW: Build + gradle + adb
├── build_pipeline.rs        # NEW: Shared transpilation logic
├── config.rs                # NEW: Parse whitehall.toml
├── project.rs               # NEW: File discovery & classification
├── android_scaffold.rs      # NEW: Generate Gradle boilerplate
└── transpiler/              # ✅ Already exists (100% complete)
    ├── mod.rs
    ├── ast.rs
    ├── parser.rs
    └── codegen.rs
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
```

### File Type Detection

Whitehall automatically determines component types based on directory structure:

| Source Path | Type | Output Package | Output Path |
|-------------|------|----------------|-------------|
| `src/components/Button.wh` | Component | `{package}.components` | `Button.kt` |
| `src/screens/HomeScreen.wh` | Screen | `{package}.screens` | `HomeScreen.kt` |
| `src/main.wh` | Main | `{package}` | `MainActivity.kt` |

**Package Mapping**:
```
whitehall.toml: package = "com.example.myapp"

src/components/Button.wh → com.example.myapp.components.Button
src/screens/Home.wh      → com.example.myapp.screens.Home
src/main.wh              → com.example.myapp.MainActivity
```

---

## Implementation Phases

### Phase 1: Foundation (Shared Core) - 7-9 hours
**Goal**: Build the shared infrastructure that all commands depend on

**Status**: ⏳ Not started

**Tasks**:
1. ✅ **Config parsing** (`src/config.rs`)
   - Parse `whitehall.toml`
   - Validate Android package name
   - Default values for optional fields
   - Time: 1 hour

2. ✅ **File discovery** (`src/project.rs`)
   - Scan `src/` recursively for `.wh` files
   - Classify files (component/screen/main)
   - Determine package paths
   - Time: 1-2 hours

3. ✅ **Build pipeline** (`src/build_pipeline.rs`)
   - Core `execute_build()` function
   - Transpile each file
   - Handle errors gracefully
   - Return `BuildResult` struct
   - Time: 3-4 hours

4. ✅ **Android scaffold** (`src/android_scaffold.rs`)
   - Generate `settings.gradle.kts`
   - Generate `build.gradle.kts` (root + app)
   - Generate `AndroidManifest.xml`
   - Generate default `MainActivity.kt`
   - Copy/generate Gradle wrapper files
   - Time: 2-3 hours

**Milestone**: Foundation complete when:
- Can parse config ✓
- Can discover files ✓
- Can transpile all files ✓
- Can generate complete Android project ✓

**Testing**:
```bash
# Unit tests
cargo test config::tests
cargo test project::tests
cargo test build_pipeline::tests

# Integration test
cargo test test_build_pipeline_integration
```

---

### Phase 2: `whitehall build` Command - 2-3 hours
**Goal**: Implement the simplest command first to validate foundation

**Status**: ⏳ Not started

**Tasks**:
1. ✅ **Implement `commands/build.rs`**
   - Call `build_pipeline::execute_build()`
   - Format output (success/error messages)
   - Handle build errors
   - Time: 1 hour

2. ✅ **Update CLI** (`src/main.rs`)
   - Add `Build` subcommand
   - Wire to `commands::build::execute()`
   - Time: 30 minutes

3. ✅ **Manual testing**
   - Create test project
   - Run `whitehall build`
   - Verify output structure
   - Attempt `./gradlew assembleDebug`
   - Time: 1 hour

4. ✅ **Documentation**
   - Update templates if needed
   - Add build command help text
   - Time: 30 minutes

**Milestone**: `build` command complete when:
- `whitehall build` successfully transpiles project ✓
- Generated Gradle project builds with `./gradlew` ✓
- APK can be installed on device ✓
- Error messages are clear ✓

**Testing**:
```bash
whitehall init test-app
cd test-app
# Add some .wh files
whitehall build
cd build
./gradlew assembleDebug
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

---

### Phase 3: `whitehall watch` Command - 3-4 hours
**Goal**: Add file watching for development workflow

**Status**: ⏳ Not started

**Dependencies**: Phase 2 complete

**Tasks**:
1. ✅ **Add notify dependency**
   - Update `Cargo.toml`
   - Test file watching basics
   - Time: 30 minutes

2. ✅ **Implement `commands/watch.rs`**
   - Initial build
   - Set up file watcher for `src/` and `whitehall.toml`
   - Rebuild loop on changes
   - Debouncing (avoid rebuilding twice for one save)
   - Colored output (errors in red, success in green)
   - Time: 2-3 hours

3. ✅ **Update CLI**
   - Add `Watch` subcommand
   - Wire to `commands::watch::execute()`
   - Time: 30 minutes

4. ✅ **Manual testing**
   - Run `whitehall watch`
   - Edit files, verify auto-rebuild
   - Introduce error, verify error display
   - Fix error, verify recovery
   - Time: 1 hour

**Milestone**: `watch` command complete when:
- Detects file changes within 100ms ✓
- Rebuilds only changed files (incremental) ✓
- Shows clear error messages ✓
- Handles Ctrl+C gracefully ✓
- Debounces rapid changes ✓

**Testing**:
```bash
# Terminal 1
whitehall watch

# Terminal 2
echo '<Text>Changed!</Text>' > src/components/Test.wh
# Verify Terminal 1 shows rebuild

# Introduce syntax error
echo '<Text>Missing closing tag' > src/components/Test.wh
# Verify error displayed

# Fix error
echo '<Text>Fixed</Text>' > src/components/Test.wh
# Verify build recovers
```

---

### Phase 4: `whitehall run` Command - 3-4 hours
**Goal**: Complete automation from code to running app

**Status**: ⏳ Not started

**Dependencies**: Phase 2 complete (Phase 3 optional)

**Tasks**:
1. ✅ **Device detection** (`commands/run.rs`)
   - Check `adb devices`
   - Verify at least one device connected
   - Handle multiple devices (error for now)
   - Time: 1 hour

2. ✅ **Gradle integration**
   - Execute `./gradlew assembleDebug` in output dir
   - Capture and display build output
   - Handle Gradle errors
   - Time: 1 hour

3. ✅ **ADB integration**
   - Install APK via `adb install -r`
   - Launch activity via `adb shell am start`
   - Handle installation errors
   - Time: 1 hour

4. ✅ **Implement full command**
   - Wire all steps together
   - Progress reporting
   - Error recovery
   - Time: 1 hour

5. ✅ **Update CLI**
   - Add `Run` subcommand
   - Wire to `commands::run::execute()`
   - Time: 30 minutes

**Milestone**: `run` command complete when:
- Detects no device → clear error ✓
- Builds → installs → launches successfully ✓
- Handles errors at each step ✓
- Displays progress clearly ✓

**Testing**:
```bash
# Test with no device
adb kill-server
whitehall run
# Should show clear error

# Test with emulator
emulator -avd Pixel_5_API_34 &
sleep 10  # Wait for boot
whitehall run
# Should build, install, and launch

# Verify app appears on screen
```

---

### Phase 5: Polish & Documentation - 1-2 hours
**Goal**: Production-ready experience

**Status**: ⏳ Not started

**Dependencies**: Phases 2, 3, 4 complete

**Tasks**:
1. ✅ **Error messages**
   - Review all error messages
   - Add helpful suggestions
   - Include file:line:col for transpiler errors
   - Time: 1 hour

2. ✅ **Help text**
   - Improve command descriptions
   - Add examples to `--help`
   - Time: 30 minutes

3. ✅ **Documentation**
   - Update README.md
   - Update NEXTSTEPS.md
   - Add troubleshooting section
   - Time: 1 hour

4. ✅ **Integration tests**
   - Test full workflows
   - Test error scenarios
   - Time: 1 hour

**Milestone**: Polish complete when:
- All commands have clear help text ✓
- Error messages are actionable ✓
- Documentation is complete ✓
- Integration tests pass ✓

---

## Total Time Estimate

| Phase | Tasks | Time |
|-------|-------|------|
| Phase 1: Foundation | Config, Discovery, Pipeline, Scaffold | 7-9 hours |
| Phase 2: `build` | Command implementation + testing | 2-3 hours |
| Phase 3: `watch` | File watching + auto-rebuild | 3-4 hours |
| Phase 4: `run` | Device integration + automation | 3-4 hours |
| Phase 5: Polish | Error messages, docs, tests | 1-2 hours |
| **Total** | | **16-22 hours** |

**Minimum viable**: Phases 1 + 2 = 9-12 hours (just `build` command)
**Complete system**: All phases = 16-22 hours (all three commands)

---

## Dependencies

### Cargo.toml additions

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
walkdir = "2.5"
notify = "6.1"  # For watch command
```

### External tools required

- **Android SDK** (for `adb`, required by `run` command)
- **Gradle** (generated wrapper, user runs it)
- **Java 17+** (for Gradle, user installs)

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

## Future Enhancements

### Short-term (after initial implementation)
- 🔜 **Incremental builds**: Hash-based change detection (only rebuild changed files)
- 🔜 **Progress indicators**: Spinner or progress bar for long operations
- 🔜 **Build profiles**: Debug vs Release configurations
- 🔜 **Source maps**: Map Kotlin errors back to .wh files
- 🔜 **Parallel transpilation**: Speed up large projects

### Medium-term
- 🔜 **`whitehall clean`**: Remove build directory
- 🔜 **`whitehall run --watch`**: Combined watch + auto-install
- 🔜 **Multiple device support**: `--device` flag for `run`
- 🔜 **Hot reload**: Partial app updates without full rebuild
- 🔜 **Build caching**: Speed up clean builds

### Long-term
- 🔜 **Remote builds**: Build on server, stream to device
- 🔜 **Bundle optimization**: Analyze and reduce APK size
- 🔜 **Custom build steps**: Plugin system for transformations

---

## Testing Strategy

### Unit Tests
```rust
// src/config.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_valid_config() { ... }

    #[test]
    fn test_invalid_package_name() { ... }
}

// src/project.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_discover_files() { ... }

    #[test]
    fn test_classify_component_file() { ... }
}
```

### Integration Tests
```rust
// tests/integration_test.rs
#[test]
fn test_build_simple_project() {
    let temp = TempDir::new().unwrap();
    // Create minimal project
    // Run build command
    // Verify output structure
}

#[test]
fn test_transpilation_error_handling() {
    // Create project with invalid .wh file
    // Run build
    // Verify clear error message
}
```

### Manual Testing Checklist

**Phase 1 (Foundation)**:
- [ ] Parse valid `whitehall.toml`
- [ ] Reject invalid package names
- [ ] Discover files in nested directories
- [ ] Classify components vs screens correctly
- [ ] Generate complete Android project structure
- [ ] All Gradle files have correct syntax

**Phase 2 (`build` command)**:
- [ ] Build project with single component
- [ ] Build project with multiple components
- [ ] Build project with screens
- [ ] Handle transpilation errors gracefully
- [ ] Output directory is configurable
- [ ] Generated Kotlin compiles with `./gradlew`
- [ ] APK installs and runs on device

**Phase 3 (`watch` command)**:
- [ ] Initial build succeeds
- [ ] Detects .wh file changes
- [ ] Detects whitehall.toml changes
- [ ] Rebuilds automatically
- [ ] Shows errors clearly
- [ ] Recovers from errors
- [ ] Handles Ctrl+C gracefully
- [ ] Doesn't rebuild for unrelated files

**Phase 4 (`run` command)**:
- [ ] Detects no device → error
- [ ] Detects device successfully
- [ ] Builds APK with Gradle
- [ ] Installs APK via adb
- [ ] Launches app on device
- [ ] Handles errors at each step
- [ ] Works with emulator
- [ ] Works with physical device

---

## Success Criteria

### Phase 1 Complete When:
- ✅ Can parse `whitehall.toml` successfully
- ✅ Can discover all `.wh` files in project
- ✅ Can transpile each file to Kotlin
- ✅ Can generate complete Android project scaffold
- ✅ Generated project structure is correct

### Phase 2 Complete When:
- ✅ `whitehall build` transpiles entire project
- ✅ Generated Kotlin code compiles without errors
- ✅ `./gradlew assembleDebug` succeeds
- ✅ APK can be installed on device
- ✅ App launches without crashes

### Phase 3 Complete When:
- ✅ `whitehall watch` performs initial build
- ✅ Detects file changes within 100ms
- ✅ Auto-rebuilds on save
- ✅ Shows clear error messages
- ✅ Handles Ctrl+C gracefully

### Phase 4 Complete When:
- ✅ `whitehall run` detects devices
- ✅ Builds, installs, and launches app
- ✅ Works with emulators and physical devices
- ✅ Shows clear progress at each step
- ✅ Handles errors gracefully

### All Phases Complete When:
- ✅ All three commands work end-to-end
- ✅ Error messages are helpful and actionable
- ✅ Documentation is complete and accurate
- ✅ Integration tests pass
- ✅ Can build a real app from scratch to running

---

## Notes

### Design Decisions

1. **Why separate `watch` and `run`?**
   - `watch` is for continuous feedback during editing
   - `run` is for quick "test on device" cycles
   - Combining them (`run --watch`) is a future enhancement
   - Separation keeps each command simple and focused

2. **Why not incremental builds initially?**
   - Full rebuilds are simpler to implement correctly
   - For small projects (<50 files), full rebuild is fast (<1s)
   - Incremental builds add complexity (hashing, dependency tracking)
   - Can add as optimization later without breaking API

3. **Why generate Gradle wrapper?**
   - Users shouldn't need to install Gradle separately
   - Ensures consistent Gradle version across machines
   - Standard Android practice

4. **Why not build APK in `build` command?**
   - `build` focuses on transpilation only
   - APK building is expensive (10-30s)
   - Users might want to inspect Kotlin code without building
   - `run` command handles full automation when needed

### Potential Issues

1. **Gradle wrapper distribution**
   - Need to bundle wrapper files in templates
   - Or generate on first `whitehall build`
   - Alternative: Document that users run `gradle wrapper` once

2. **Multiple devices**
   - `adb` requires device selector for multiple devices
   - For now: error if >1 device connected
   - Future: add `--device` flag

3. **File watching performance**
   - Large projects might have slow watch startup
   - Solution: Use `RecursiveMode::Recursive` only on `src/`
   - Ignore `build/`, `.git/`, etc.

4. **Windows compatibility**
   - Path separators (`/` vs `\`)
   - Use `std::path::PathBuf` everywhere
   - Test on Windows in CI

---

**Ready to implement!** 🚀
