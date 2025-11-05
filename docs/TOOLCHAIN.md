# Toolchain Management Strategy

**Status:** ✅ Production Ready (All Phases 1-5 Complete)
**Date:** 2025-11-04
**Last Updated:** 2025-11-04
**Goal:** Enable zero-config Android development by bundling required toolchains

## Implementation Status

### ✅ Completed (Phases 1-4)

**Phase 1: Core Toolchain Manager** ✅ DONE
- ✅ `src/toolchain/mod.rs` - Core toolchain management
- ✅ `src/toolchain/defaults.rs` - Default version constants
- ✅ `src/toolchain/platform.rs` - Platform detection (Linux/macOS, x64/aarch64)
- ✅ `src/toolchain/validator.rs` - AGP/Java/Gradle compatibility validation
- ✅ `[toolchain]` section in `whitehall.toml`
- ✅ Version validation prevents incompatible configurations

**Phase 2: Downloader** ✅ DONE
- ✅ HTTP downloads with progress bars (`reqwest` + `indicatif`)
- ✅ Archive extraction (tar.gz for Java/Gradle, zip for Android SDK)
- ✅ Platform-specific URL construction (Adoptium for Java)
- ✅ Android SDK installation via sdkmanager with license acceptance
- ✅ Automatic downloads when toolchains missing

**Phase 3: Integration** ✅ DONE
- ✅ `whitehall exec` - Run commands with toolchain environment
- ✅ `whitehall shell` - Interactive shell with toolchain
- ✅ Updated `run` command to use `toolchain.gradle_cmd()` and `toolchain.adb_cmd()`
- ✅ Updated `init` command to inform users about automatic toolchain downloads
- ✅ Fixed ANDROID_SDK_ROOT conflicts with managed ANDROID_HOME
- ⚠️  Note: `build` command only transpiles (doesn't run Gradle), so no changes needed

**Phase 4: User Commands** ✅ DONE
- ✅ `whitehall toolchain install` - Pre-download toolchains
- ✅ `whitehall toolchain list` - Show installed versions with sizes
- ✅ `whitehall toolchain clean` - Remove all toolchains
- ✅ `whitehall exec` - Run commands with toolchain (top-level command)
- ✅ `whitehall shell` - Interactive shell (top-level command)

**Bonus Features** ✅ DONE
- ✅ Multiple Java/Gradle versions coexist peacefully
- ✅ Automatic version switching per project (like `uv run`)
- ✅ `which java` shows correct binary per project
- ✅ Environment variables set correctly (JAVA_HOME, ANDROID_HOME, etc.)
- ✅ Comprehensive test suite with 6 counter variants
- ✅ Gradle daemon isolation per version

### ✅ All Phases Complete!

**Phase 3 - Build Integration** ✅ COMPLETE
- ✅ `build` command doesn't need changes (only transpiles, doesn't run Gradle)
- ✅ Updated `run` command to use `toolchain.adb_cmd()` and `toolchain.gradle_cmd()`
- ✅ Updated `init` command to inform about toolchain downloads
- ⏳ Test on clean machine without system Java/Gradle/SDK (recommended but optional)

**Phase 5 - Polish & Production Features** ✅ COMPLETE
- ✅ `whitehall doctor` command - Comprehensive health check with toolchain status
- ✅ Retry logic - Prompts user to retry failed downloads (max 3 attempts)
- ✅ Parallel downloads - Java, Gradle, and SDK download simultaneously
- ✅ Checksum verification - Optional SHA256 verification for downloads

### Testing

**Test Coverage:**
- ✅ 16 unit tests passing (platform, validation, defaults)
- ✅ 6 integration test variants in `examples/`:
  - `counter` - Java 21 + Gradle 8.4
  - `counter-java-21-gradle-8.6` - Java 21 + Gradle 8.6
  - `counter-java-17` - Java 17 + Gradle 8.0
  - `counter-java-17-gradle-7` - Java 17 + Gradle 7.6
  - `counter-java-11-gradle-7` - Java 11 + Gradle 7.6
  - `counter-java-11-gradle-8` - Java 11 + Gradle 8.0

**Real-world Testing:**
- ✅ Downloads work (tested with Java 11, 17, 21)
- ✅ Multiple Gradle versions coexist (7.6, 8.0, 8.4, 8.6)
- ✅ Android SDK installation works
- ✅ License acceptance automated
- ✅ `exec`/`shell` commands work correctly
- ✅ Version switching per project verified

---

## Problem Statement

Whitehall currently requires users to manually install and configure:
- Java/JDK (for Gradle and Android toolchain)
- Android SDK (for building APKs)
- Gradle (build system)
- adb (device deployment - comes with Android SDK)
- Android emulator (optional, for testing)

This creates friction for new users and leads to inconsistent development environments.

## Requirements

1. **Zero prerequisites** - Installing Whitehall should be enough
2. **Deterministic** - Everyone gets the same toolchain versions
3. **Transparent** - Users shouldn't need to know about Java/Gradle/SDK internals
4. **Fast** - Download once, cached forever
5. **Cross-platform** - Support Linux and macOS (x64 + aarch64)
6. **Offline-friendly** - Once downloaded, works without network

## Evaluated Options

### Option 1: mise-en-place (Version Manager)

Use mise to manage Java/Gradle versions:

```toml
# .mise.toml
[tools]
java = "21"
gradle = "8.4"

[env]
ANDROID_HOME = "{{config_root}}/android-sdk"
```

**Pros:**
- Familiar to Rust devs
- Handles Java/Gradle nicely
- Per-project versioning

**Cons:**
- Requires installing mise first (defeats zero-config goal)
- Android SDK support limited
- Adds external dependency
- Emulator setup still manual

### Option 2: sdkmanager (Android's Official Tool)

Create `whitehall setup` command that uses sdkmanager:

**Pros:**
- Official Android tooling
- Handles SDK, build-tools, platform-tools, emulator
- Can install specific API levels

**Cons:**
- Requires JDK first (chicken-egg problem)
- Less elegant
- Manual JDK management

### Option 3: Hybrid (mise + sdkmanager)

Use mise for Java/Gradle, sdkmanager for Android SDK.

**Pros:**
- Best of both worlds
- Follows conventions

**Cons:**
- Two systems to maintain
- Still requires installing mise

### Option 4: Bundled Installer (cargo-mobile2 style)

Bundle everything in Whitehall:

**Pros:**
- Zero external dependencies
- Best UX for newcomers
- Full control

**Cons:**
- Platform-specific complexity
- Large downloads
- Maintenance burden

### Option 5: Check & Guide (Minimal)

Add `whitehall doctor` to check prerequisites:

**Pros:**
- Doesn't prescribe tools
- Works with existing setups
- Low maintenance

**Cons:**
- Manual setup still required
- Inconsistent experiences

## Recommended Approach: Project-Scoped Toolchains

**Inspired by:** `rustup` + `rust-toolchain.toml`, `mise`, `asdf`

**Concept:** Each project specifies its required toolchain versions in `whitehall.toml`. Whitehall manages a shared cache in `~/.whitehall/toolchains/` and downloads versions on-demand.

### Key Design Principle

**Whitehall is a toolchain manager, not a version dictator.** Each project controls its own toolchain requirements, while Whitehall handles downloading, caching, and version selection automatically.

### User Experience

```bash
# Create new project - gets sensible defaults
whitehall init myapp
# → Creates whitehall.toml with [toolchain] section
# → Latest stable versions: Java 21, Gradle 8.4, AGP 8.2.0

# First build - automatic downloads
cd myapp
whitehall build
# → "Downloading Java 21... ✓" (if not cached)
# → "Downloading Gradle 8.4... ✓" (if not cached)
# → "Downloading Android SDK components... ✓"
# → Build succeeds

# Work on legacy project with older toolchain
cd old-project
cat whitehall.toml
# [toolchain]
# java = "17"
# gradle = "8.0"

whitehall build
# → "Downloading Java 17... ✓" (if not cached)
# → Uses Java 17 automatically for this project
# → No conflicts with other projects

# Both projects coexist peacefully
cd ~/myapp && whitehall build      # Uses Java 21
cd ~/old-project && whitehall build # Uses Java 17
```

### Project Configuration (whitehall.toml)

```toml
[project]
name = "MyApp"
version = "1.0.0"

[android]
min_sdk = 24
target_sdk = 34
package = "com.example.myapp"

[toolchain]
java = "21"           # Java/JDK version
gradle = "8.4"        # Gradle version
agp = "8.2.0"        # Android Gradle Plugin
kotlin = "1.9.20"    # Kotlin compiler version

[build]
output_dir = "build"
```

**Toolchain section is optional.** If omitted, Whitehall uses latest stable defaults.

### Toolchain Cache Structure

```
~/.whitehall/
├── toolchains/
│   ├── java/
│   │   ├── 11/              # For AGP 7.x projects
│   │   ├── 17/              # For AGP 8.x projects
│   │   └── 21/              # For modern AGP 8.2+ projects
│   ├── gradle/
│   │   ├── 7.6/             # Older projects
│   │   ├── 8.0/
│   │   ├── 8.2/
│   │   └── 8.4/             # Latest
│   └── android/
│       ├── cmdline-tools/   # Version-agnostic SDK tools
│       ├── platform-tools/  # adb, fastboot, etc.
│       ├── build-tools/
│       │   ├── 30.0.3/
│       │   ├── 33.0.2/
│       │   └── 34.0.0/
│       ├── platforms/
│       │   ├── android-30/
│       │   ├── android-33/
│       │   └── android-34/
│       └── emulator/        # optional, on-demand
└── config.toml              # metadata about installed versions
```

**Cache is shared across all projects.** Download Java 21 once, use it everywhere.

### Download Sources

#### Java (OpenJDK)

**Recommended: Adoptium/Temurin**
- URL: https://api.adoptium.net/v3/binary/latest/21/ga/{os}/{arch}/jdk/hotspot/normal/eclipse
- Benefits: Official, free, good API, actively maintained
- Platforms: linux/mac, x64/aarch64

**Alternative: Azul Zulu**
- URL: https://www.azul.com/downloads/?package=jdk
- Benefits: Good commercial support
- Platforms: linux/mac, x64/aarch64

#### Gradle

- URL: https://services.gradle.org/distributions/gradle-{version}-bin.zip
- Simple zip extraction

#### Android SDK

1. **Command-line tools:**
   - Linux: https://dl.google.com/android/repository/commandlinetools-linux-9477386_latest.zip
   - macOS: https://dl.google.com/android/repository/commandlinetools-mac-9477386_latest.zip

2. **Then use sdkmanager to install:**
   ```bash
   sdkmanager "platform-tools" "build-tools;34.0.0" "platforms;android-34"
   ```

### Progressive Downloads

Only download what's needed, when needed:

```rust
whitehall init myapp
  → Downloads: Java + cmdline-tools only (~330MB)

whitehall build
  → Downloads: platform-tools, build-tools, platform if missing (~150MB)

whitehall run --emulator
  → Downloads: emulator if missing (~300MB)
```

### Implementation Architecture

#### Core Toolchain Manager

```rust
// src/toolchain/mod.rs
pub struct Toolchain {
    root: PathBuf, // ~/.whitehall/toolchains
}

impl Toolchain {
    /// Ensure Java is installed, download if missing
    /// version: e.g., "11", "17", "21"
    pub fn ensure_java(&self, version: &str) -> Result<PathBuf> {
        let java_home = self.root.join(format!("java/{}", version));
        if !java_home.exists() {
            self.download_java(version)?;
        }
        Ok(java_home)
    }

    /// Ensure Gradle is installed, download if missing
    /// version: e.g., "7.6", "8.0", "8.4"
    pub fn ensure_gradle(&self, version: &str) -> Result<PathBuf> {
        let gradle_home = self.root.join(format!("gradle/{}", version));
        if !gradle_home.exists() {
            self.download_gradle(version)?;
        }
        Ok(gradle_home.join("bin/gradle"))
    }

    /// Ensure Android SDK is installed, download if missing
    pub fn ensure_android_sdk(&self) -> Result<PathBuf> {
        let sdk_root = self.root.join("android");
        if !sdk_root.join("platform-tools/adb").exists() {
            self.download_android_sdk()?;
        }
        Ok(sdk_root)
    }

    /// Get configured adb command
    pub fn adb_cmd(&self) -> Result<Command> {
        let sdk_root = self.ensure_android_sdk()?;
        let adb = sdk_root.join("platform-tools/adb");
        Ok(Command::new(adb))
    }
}
```

#### Transparent Integration

```rust
// src/commands/build.rs
pub fn build() -> Result<()> {
    // Load project config (includes [toolchain] section)
    let config = load_config("whitehall.toml")?;

    // Get toolchain manager
    let toolchain = Toolchain::new()?;

    // Ensure correct versions for THIS project
    let java_home = toolchain.ensure_java(&config.toolchain.java)?;  // e.g., "21"
    let gradle_path = toolchain.ensure_gradle(&config.toolchain.gradle)?;  // e.g., "8.4"
    let android_home = toolchain.ensure_android_sdk()?;

    // Run gradle with project-specific Java version
    let mut gradle = Command::new(gradle_path);
    gradle.arg("assembleDebug")
          .env("JAVA_HOME", java_home)
          .env("ANDROID_HOME", android_home)
          .current_dir(&build_dir);

    gradle.status()?;
    Ok(())
}
```

#### Downloader Module

```rust
// src/toolchain/downloader.rs
pub mod downloader {
    /// Download file with progress bar
    pub fn download_with_progress(url: &str, dest: &Path) -> Result<()>;

    /// Extract tar.gz or zip archive
    pub fn extract_archive(archive: &Path, dest: &Path) -> Result<()>;

    /// Detect current platform (os + arch)
    pub fn detect_platform() -> Platform;

    /// Verify download integrity (SHA256)
    pub fn verify_checksum(file: &Path, expected: &str) -> Result<()>;
}

pub enum Platform {
    LinuxX64,
    LinuxAarch64,
    MacX64,
    MacAarch64,
}
```

### Optional User Commands

```bash
# List installed toolchains and their sizes
whitehall toolchain list

# Clear cache (free up space)
whitehall toolchain clean

# Pre-download specific version
whitehall toolchain install java 21
whitehall toolchain install gradle 8.4

# Check toolchain status
whitehall doctor
```

### Version Management

#### Default Versions

When `whitehall init` creates a new project, it uses current stable defaults:

```rust
// src/toolchain/defaults.rs
pub const DEFAULT_JAVA: &str = "21";       // Java 21 LTS (stable until 2029)
pub const DEFAULT_GRADLE: &str = "8.4";    // Latest stable Gradle
pub const DEFAULT_AGP: &str = "8.2.0";     // Android Gradle Plugin
pub const DEFAULT_KOTLIN: &str = "1.9.20"; // Latest stable Kotlin
```

These defaults are updated with Whitehall releases, but **existing projects are not affected**.

#### Version Compatibility Matrix

Whitehall validates version compatibility when reading `whitehall.toml`:

| AGP Version | Required Gradle | Required Java | Supported |
|-------------|----------------|---------------|-----------|
| 7.4.x | 7.5+ | 11+ | ✅ Yes |
| 8.0.x | 8.0+ | 17+ | ✅ Yes |
| 8.1.x | 8.0+ | 17+ | ✅ Yes |
| 8.2.x | 8.2+ | 17+ | ✅ Yes (recommended) |
| 9.0.x | 8.6+ | 21+ | 🔮 Future |

**Invalid configuration example:**
```toml
[toolchain]
java = "11"
gradle = "8.4"
agp = "8.2.0"    # ERROR: AGP 8.2 requires Java 17+
```

Whitehall will error with:
```
Error: Incompatible toolchain configuration
AGP 8.2.0 requires Java 17 or higher, but java = "11" specified
Suggestion: Update to java = "17" or java = "21"
```

#### Migration Path

To update an old project's toolchain:

```bash
# Option 1: Manual edit
vim whitehall.toml
# [toolchain]
# java = "21"    # Update from 17
# gradle = "8.4" # Update from 8.0

# Option 2: Auto-migrate (future)
whitehall migrate --latest
# Updates whitehall.toml to latest stable versions
# Regenerates build files with new AGP/Kotlin
```

#### Override for Testing

Test your project against different Java versions:

```bash
# Use Java 17 for this build only
whitehall build --java 17

# Test against multiple versions
whitehall build --java 11
whitehall build --java 17
whitehall build --java 21
```

## Benefits

✅ **Zero prerequisites** - Just `cargo install whitehall` and go
✅ **Project autonomy** - Each project controls its own toolchain versions
✅ **No forced upgrades** - Old projects keep working with their specified versions
✅ **Gradual migration** - Update toolchain when ready, not when forced
✅ **Shared cache** - Download Java 21 once, use across all projects
✅ **Team consistency** - Toolchain versions in git (whitehall.toml), everyone uses same
✅ **Fast after first download** - Toolchains cached forever
✅ **Cross-platform** - Handle platform differences internally
✅ **Offline-friendly** - Once downloaded, works offline
✅ **No PATH pollution** - Everything via direct paths, no shims
✅ **Familiar model** - Like `rust-toolchain.toml` or `mise`

## Considerations

### Download Sizes

| Component | Compressed | Uncompressed |
|-----------|-----------|--------------|
| OpenJDK 21 | ~180MB | ~300MB |
| Gradle 8.4 | ~120MB | ~200MB |
| Android cmdline-tools | ~150MB | ~300MB |
| platform-tools | ~15MB | ~30MB |
| build-tools | ~60MB | ~100MB |
| platform (android-34) | ~75MB | ~150MB |
| emulator (optional) | ~300MB | ~600MB |
| **Total (no emulator)** | **~600MB** | **~1.1GB** |

**Mitigation:**
- Progressive downloads (only what's needed)
- Show progress bars (like uv does)
- One-time cost
- Optional `whitehall toolchain clean` to reclaim space

### Platform Detection

Need to detect and handle:
- **OS:** Linux, macOS
- **Architecture:** x86_64, aarch64 (Apple Silicon)
- **Libc:** glibc vs musl (Linux only)

Use existing crates:
- `target-lexicon` for platform detection
- `platform-info` for detailed system info

### Offline Support

For air-gapped systems:
```bash
# On internet-connected machine
whitehall toolchain install
tar -czf whitehall-toolchains.tar.gz ~/.whitehall/toolchains/

# On offline machine
tar -xzf whitehall-toolchains.tar.gz -C ~/
whitehall build  # Works offline!
```

### Updates

- Toolchain versions pinned per Whitehall version
- Update via `whitehall self-update` (or re-install)
- Old toolchains can coexist (version in path)

## Comparison with Alternatives

| Aspect | mise (external) | uv-style (bundled) |
|--------|----------------|-------------------|
| User setup | Install mise | None |
| Config files | .mise.toml visible | Transparent |
| Version flexibility | User chooses | Fixed (simpler) |
| Cache location | ~/.local/share/mise | ~/.whitehall |
| Shared with other tools | Yes | Just Whitehall |
| Updates | mise update | whitehall update |
| Complexity | User-facing | Internal only |
| Android SDK | Manual | Automatic |
| Mental model | "mise + whitehall" | "just whitehall" |

## Implementation Phases

### Phase 1: Core Toolchain Manager (Foundation)

**Goal:** Build the basic infrastructure to locate and ensure toolchains exist.

**Implementation:**
```rust
// src/toolchain/mod.rs
Toolchain::ensure_java()         // Check if Java exists, return path or error
Toolchain::ensure_android_sdk()  // Check if SDK exists, return path or error
Toolchain::gradle_cmd()          // Return configured gradle Command
Toolchain::adb_cmd()             // Return configured adb Command
Platform detection               // Linux/Mac/Windows, x64/aarch64
```

**Tasks:**
- [ ] Create `src/toolchain/mod.rs`
- [ ] Add `[toolchain]` section to `src/config.rs` (ToolchainConfig struct)
- [ ] Add default version constants in `src/toolchain/defaults.rs`
- [ ] Implement `ensure_java(version)` - Check `~/.whitehall/toolchains/java/{version}`
- [ ] Implement `ensure_gradle(version)` - Check `~/.whitehall/toolchains/gradle/{version}`
- [ ] Implement `ensure_android_sdk()` - Check `~/.whitehall/toolchains/android`
- [ ] Implement `adb_cmd()` with proper ANDROID_HOME
- [ ] Platform detection using `std::env::consts`
- [ ] Version compatibility validation (AGP vs Java/Gradle)

**Deliverable:** Basic toolchain manager that can check and report what's installed for a given project

**Estimated time:** 2-3 days

---

### Phase 2: Downloader (The Hard Part)

**Goal:** Implement the actual downloading and extraction of toolchains.

**Implementation:**
```rust
// src/toolchain/downloader.rs
download_with_progress()  // HTTP download with progress bar (reqwest + indicatif)
extract_archive()         // Unzip/untar (zip crate, tar crate)
verify_checksum()         // SHA256 verification
retry logic               // Handle network failures gracefully
```

**Dependencies:**
- `reqwest` - HTTP client
- `indicatif` - Progress bars
- `zip` - ZIP extraction
- `tar` + `flate2` - TAR.GZ extraction
- `sha2` - Checksum verification

**Tasks:**
- [ ] HTTP download with progress bar (reqwest + indicatif)
- [ ] Archive extraction (tar.gz, zip) for Java and Gradle
- [ ] Version-aware URL construction (e.g., Java 11/17/21, Gradle 7.x/8.x)
- [ ] Checksum verification (optional but recommended)
- [ ] Error handling and retries (exponential backoff)
- [ ] Platform-specific URL selection (Linux/Mac/Windows, x64/aarch64)
- [ ] Android SDK installation via sdkmanager (after Java is available)

**Deliverable:** Can actually download and install any supported toolchain version

**Estimated time:** 3-5 days

---

### Phase 3: Integration (Wire it Up)

**Goal:** Replace all existing system assumptions with bundled toolchain.

**Before:**
```rust
// Assumes user has gradle in PATH
Command::new("gradle").arg("assembleDebug").status()?;
```

**After:**
```rust
// Uses bundled gradle, downloads if needed
let mut gradle = toolchain.gradle_cmd()?;
gradle.arg("assembleDebug").status()?;
```

**Tasks:**
- [ ] Update `build` command to use `toolchain.gradle_cmd()`
- [ ] Update `run` command to use `toolchain.adb_cmd()`
- [ ] Update `init` command to trigger toolchain downloads on first use
- [ ] Remove assumptions about system Java/SDK
- [ ] Test on clean machine without Java/SDK installed

**Deliverable:** Whitehall works without system Java/Gradle/SDK installed

**Estimated time:** 2-3 days

---

### Phase 4: User Commands (Visibility & Control)

**Goal:** Give users explicit control over toolchain management.

**Commands:**
```bash
whitehall toolchain list      # Show what's installed, versions, sizes
whitehall toolchain clean     # Delete cached toolchains (free space)
whitehall toolchain install   # Pre-download (e.g., for offline use)
whitehall doctor              # Enhanced health check with toolchain info
```

**Tasks:**
- [ ] `whitehall toolchain list` - Show installed components
- [ ] `whitehall toolchain clean` - Remove cache
- [ ] `whitehall toolchain install` - Pre-download all
- [ ] `whitehall doctor` improvements - Include toolchain status

**Deliverable:** Users can inspect and manage toolchains explicitly

**Estimated time:** 1-2 days

---

### Phase 5: Polish (Production Ready) ✅ COMPLETE

**Goal:** Production-quality experience with edge case handling.

**Implemented Features:**
- ✅ **Doctor command** - Comprehensive system health check
- ✅ **Better errors** - "Java download failed, retry? [y/n]" with user prompts
- ✅ **Retry logic** - Automatic retry with max 3 attempts
- ✅ **Parallel downloads** - Java, Gradle, and SDK downloaded simultaneously
- ✅ **Checksum verification** - Optional SHA256 verification for integrity

**Tasks:**
- ✅ `whitehall doctor` command with toolchain status
- ✅ Better error messages with retry prompts
- ✅ Parallel downloads (Java + Gradle + SDK simultaneously)
- ✅ Checksum verification enabled

**Deliverable:** Production-quality experience with edge cases handled

**Actual time:** 2 days

---

### MVP Definition

**Phases 1-3 constitute the MVP.** After Phase 3, Whitehall is truly zero-config:

```bash
cargo install whitehall
whitehall init myapp    # Downloads Java, SDK, Gradle automatically (first time)
whitehall build         # Uses cached toolchains
whitehall run           # Just works™
```

Phases 4-5 are enhancements for better UX, debugging, and control.

**MVP Timeline:** ~1-2 weeks
**Complete Implementation:** ~2-3 weeks

## Reference Implementations

**Study these projects for inspiration:**

1. **uv** (Python): https://github.com/astral-sh/uv
   - `uv python install 3.12` downloads to `~/.local/share/uv/python/`
   - Uses direct paths, no shims
   - Excellent progress bars and UX

2. **rustup** (Rust): https://github.com/rust-lang/rustup
   - Downloads toolchains to `~/.rustup/toolchains/`
   - Manages multiple versions elegantly
   - Proxy binary system

3. **cargo-binstall**: https://github.com/cargo-bins/cargo-binstall
   - Downloads pre-built binaries
   - Good platform detection logic

4. **bun** (JavaScript): https://github.com/oven-sh/bun
   - Bundles everything needed
   - Single binary philosophy

5. **deno** (JavaScript): https://github.com/denoland/deno
   - Self-contained runtime
   - Manages dependencies internally

## Related Documentation

- [VISION.md](./VISION.md) - Overall project vision
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture
- [BUILD.md](./BUILD.md) - Current build pipeline
- [ROADMAP.md](./ROADMAP.md) - Feature roadmap

## Open Questions

1. **Emulator:** Should we auto-install emulator images? Which ones? System images are large (~1GB each).
2. **Gradle wrapper:** Still generate wrapper in project build dir, or always use managed gradle?
3. **Telemetry:** Track download success/failures (opt-in) to improve reliability?
4. **Mirror support:** Allow custom download mirrors for enterprises/China?
5. **Minimum versions:** What's the oldest Java/Gradle/AGP we'll support? (Suggest: AGP 7.4+)
6. **Version aliases:** Support "lts", "stable", "latest" instead of explicit numbers?

## Conclusion

The **project-scoped toolchain approach** provides the best balance for Whitehall:

- **Flexibility:** Each project controls its toolchain, like `rust-toolchain.toml`
- **Simplicity:** Shared cache, automatic downloads, no manual setup
- **Reliability:** Team consistency via version pinning in git
- **Gradual migration:** Update when ready, no forced upgrades
- **Familiar:** Follows rustup/mise patterns that developers know

This approach honors Whitehall's vision as **"cargo for Android"** - opinionated defaults with project-level control when needed.

**Next step:** Implement Phase 1 (core toolchain manager with version support).
