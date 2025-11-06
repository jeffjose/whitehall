# Whitehall Toolchain Management Reference

**Complete guide to zero-config Android development with bundled toolchains**

---

## Status

✅ **Production Ready** (All Phases 1-5 Complete)

Whitehall manages Java, Gradle, and Android SDK automatically with zero user setup required.

---

## Quick Summary

**Problem:** Android development traditionally requires manually installing:
- Java/JDK (for Gradle and Android toolchain)
- Android SDK (for building APKs)
- Gradle (build system)
- adb (device deployment)

**Solution:** Whitehall bundles everything automatically.

**Architecture:** Project-scoped toolchains (like `rust-toolchain.toml` or `mise`)

```bash
# Just install Whitehall and go
cargo install whitehall
whitehall init myapp    # Downloads Java, Gradle, SDK automatically (first time)
whitehall build         # Uses cached toolchains
whitehall run           # Just works™
```

---

## Architecture

### Toolchain Cache Structure

```
~/.whitehall/toolchains/
├── java/
│   ├── 11/              # For AGP 7.x projects
│   ├── 17/              # For AGP 8.x projects
│   └── 21/              # For modern AGP 8.2+ projects
├── gradle/
│   ├── 7.6/             # Older projects
│   ├── 8.0/
│   ├── 8.2/
│   └── 8.4/             # Latest
└── android/
    ├── cmdline-tools/   # Version-agnostic SDK tools
    ├── platform-tools/  # adb, fastboot, etc.
    ├── build-tools/
    │   ├── 30.0.3/
    │   ├── 33.0.2/
    │   └── 34.0.0/
    ├── platforms/
    │   ├── android-30/
    │   ├── android-33/
    │   └── android-34/
    └── emulator/        # Optional, on-demand
```

**Key Principle:** Cache is shared across all projects. Download Java 21 once, use it everywhere.

---

## Project Configuration

### whitehall.toml

```toml
[project]
name = "my-app"
version = "0.1.0"

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

### Default Versions

**Location:** `src/toolchain/defaults.rs`

```rust
pub const DEFAULT_JAVA: &str = "21";       // Java 21 LTS (stable until 2029)
pub const DEFAULT_GRADLE: &str = "8.4";    // Latest stable Gradle
pub const DEFAULT_AGP: &str = "8.2.0";     // Android Gradle Plugin
pub const DEFAULT_KOTLIN: &str = "2.0.0";  // Kotlin compiler version
```

These defaults are updated with Whitehall releases, but **existing projects are not affected**.

---

## User Experience

### First Run (Automatic Download)

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
```

### Legacy Projects (Automatic Version Switching)

```bash
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

---

## Version Management

### Version Compatibility Matrix

Whitehall validates version compatibility when reading `whitehall.toml`:

| AGP Version | Required Gradle | Required Java | Supported |
|-------------|----------------|---------------|--------------|
| 7.4.x | 7.5+ | 11+ | ✅ Yes |
| 8.0.x | 8.0+ | 17+ | ✅ Yes |
| 8.1.x | 8.0+ | 17+ | ✅ Yes |
| 8.2.x | 8.2+ | 17+ | ✅ Yes (recommended) |
| 8.3.x | 8.4+ | 17+ | ✅ Yes |
| 8.4.x | 8.6+ | 17+ | ✅ Yes |
| 9.0.x | 8.6+ | 21+ | ✅ Yes |

**Invalid Configuration Example:**

```toml
[toolchain]
java = "11"
gradle = "8.4"
agp = "8.2.0"    # ERROR: AGP 8.2 requires Java 17+
```

**Whitehall Error:**

```
Error: Incompatible toolchain configuration
AGP 8.2.0 requires Java 17 or higher, but java = "11" specified
Suggestion: Update to java = "17" or java = "21"
```

### Multiple Versions Coexist

**Key Feature:** Different projects can use different toolchain versions without conflicts.

**How it works:**
- Each toolchain version installed in separate directory
- Whitehall sets `JAVA_HOME`, `ANDROID_HOME`, etc. per-project
- No PATH pollution (direct binary paths)
- Gradle daemon isolation per version

---

## Implementation

### Module Structure

```
src/toolchain/
├── mod.rs           # Core toolchain manager
├── defaults.rs      # Default version constants
├── platform.rs      # Platform detection (Linux/macOS, x64/aarch64)
├── validator.rs     # AGP/Java/Gradle compatibility validation
└── downloader.rs    # HTTP download + extraction
```

### Core Toolchain Manager

**Location:** `src/toolchain/mod.rs`

```rust
pub struct Toolchain {
    root: PathBuf, // ~/.whitehall/toolchains
}

impl Toolchain {
    /// Ensure Java is installed, download if missing
    pub fn ensure_java(&self, version: &str) -> Result<PathBuf> {
        let java_home = self.root.join(format!("java/{}", version));
        if !java_home.exists() {
            self.download_java(version)?;
        }
        Ok(java_home)
    }

    /// Ensure Gradle is installed, download if missing
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

    /// Get configured gradle command
    pub fn gradle_cmd(&self, version: &str) -> Result<Command> {
        let gradle_path = self.ensure_gradle(version)?;
        let java_home = self.ensure_java(&self.config.toolchain.java)?;
        let android_home = self.ensure_android_sdk()?;

        let mut cmd = Command::new(gradle_path);
        cmd.env("JAVA_HOME", java_home);
        cmd.env("ANDROID_HOME", android_home);
        Ok(cmd)
    }
}
```

### Downloader Module

**Location:** `src/toolchain/downloader.rs`

**Features:**
- HTTP download with progress bars (reqwest + indicatif)
- Archive extraction (tar.gz for Java/Gradle, zip for Android SDK)
- Platform-specific URL construction (Adoptium for Java)
- Android SDK installation via sdkmanager with license acceptance
- Retry logic with user prompts (max 3 attempts)
- Parallel downloads (Java, Gradle, SDK simultaneously)
- Optional SHA256 checksum verification

**Download Sources:**

**Java (OpenJDK):**
- **Provider:** Adoptium/Temurin
- **URL:** `https://api.adoptium.net/v3/binary/latest/{version}/ga/{os}/{arch}/jdk/hotspot/normal/eclipse`
- **Platforms:** Linux/macOS, x64/aarch64
- **Benefits:** Official, free, good API, actively maintained

**Gradle:**
- **URL:** `https://services.gradle.org/distributions/gradle-{version}-bin.zip`
- **Simple zip extraction**

**Android SDK:**
1. **Command-line tools:**
   - Linux: `https://dl.google.com/android/repository/commandlinetools-linux-9477386_latest.zip`
   - macOS: `https://dl.google.com/android/repository/commandlinetools-mac-9477386_latest.zip`

2. **Then use sdkmanager to install:**
   ```bash
   sdkmanager "platform-tools" "build-tools;34.0.0" "platforms;android-34"
   ```

### Progressive Downloads

Only download what's needed, when needed:

```
whitehall init myapp
  → Downloads: Java + cmdline-tools only (~330MB)

whitehall build
  → Downloads: platform-tools, build-tools, platform if missing (~150MB)

whitehall run --emulator
  → Downloads: emulator if missing (~300MB)
```

---

## Integration with Build Commands

### build command

```rust
// src/commands/build.rs
pub fn build() -> Result<()> {
    let config = load_config("whitehall.toml")?;
    let toolchain = Toolchain::new()?;

    // Ensure correct versions for THIS project
    let java_home = toolchain.ensure_java(&config.toolchain.java)?;
    let gradle_path = toolchain.ensure_gradle(&config.toolchain.gradle)?;
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

### run command

```rust
// src/commands/run.rs
pub fn run() -> Result<()> {
    let toolchain = Toolchain::new()?;

    // Use toolchain-managed gradle
    let gradle = toolchain.gradle_cmd(config.toolchain.gradle)?;
    gradle.arg("assembleDebug").status()?;

    // Use toolchain-managed adb
    let mut adb = toolchain.adb_cmd()?;
    adb.arg("install").arg("-r").arg("app-debug.apk");
    adb.status()?;
}
```

---

## User Commands

### whitehall toolchain list

**Purpose:** Show installed toolchains and their sizes

```bash
whitehall toolchain list
```

**Output:**
```
Installed Toolchains (~/.whitehall/toolchains):

Java:
  ✓ 11  (297 MB)
  ✓ 17  (312 MB)
  ✓ 21  (325 MB)

Gradle:
  ✓ 7.6  (184 MB)
  ✓ 8.0  (197 MB)
  ✓ 8.4  (211 MB)

Android SDK:
  ✓ platform-tools (31 MB)
  ✓ build-tools 34.0.0 (98 MB)
  ✓ platforms android-34 (147 MB)

Total: 1.8 GB
```

**Implementation:** `src/commands/toolchain.rs`

---

### whitehall toolchain clean

**Purpose:** Remove all cached toolchains (free space)

```bash
whitehall toolchain clean
```

**Output:**
```
⚠️  This will delete all cached toolchains (~/.whitehall/toolchains)
   Total size: 1.8 GB

Are you sure? [y/N] y

Removing Java 11... ✓
Removing Java 17... ✓
Removing Java 21... ✓
Removing Gradle 7.6... ✓
Removing Gradle 8.0... ✓
Removing Gradle 8.4... ✓
Removing Android SDK... ✓

✅ Cleaned 1.8 GB
```

**Note:** Toolchains will be re-downloaded on next build.

---

### whitehall toolchain install

**Purpose:** Pre-download toolchains (e.g., for offline use)

```bash
whitehall toolchain install
```

**Output:**
```
📥 Pre-downloading toolchains for current project...

Downloading Java 21... ✓ (297 MB)
Downloading Gradle 8.4... ✓ (211 MB)
Downloading Android SDK... ✓ (276 MB)

✅ All toolchains installed
```

**Flags:**
- `--java <version>` - Install specific Java version
- `--gradle <version>` - Install specific Gradle version
- `--all` - Install all versions (for offline development)

---

### whitehall doctor

**Purpose:** Comprehensive health check with toolchain status

```bash
whitehall doctor
```

**Output:**
```
🔍 Whitehall Health Check

Project:
  ✓ whitehall.toml found
  ✓ Valid package name: com.example.myapp
  ✓ Valid directory structure

Toolchain:
  ✓ Java 21 installed (297 MB)
  ✓ Gradle 8.4 installed (211 MB)
  ✓ Android SDK installed (276 MB)
  ✓ Version compatibility: AGP 8.2.0 ↔ Gradle 8.4 ↔ Java 21

Environment:
  ✓ adb accessible
  ✗ No devices connected (run `adb devices`)

Build:
  ✓ Transpiler working
  ✓ Generated Kotlin compiles

✅ Everything looks good!
```

**Implementation:** `src/commands/doctor.rs`

---

## Offline Support

For air-gapped systems:

```bash
# On internet-connected machine
whitehall toolchain install --all
tar -czf whitehall-toolchains.tar.gz ~/.whitehall/toolchains/

# On offline machine
tar -xzf whitehall-toolchains.tar.gz -C ~/
whitehall build  # Works offline!
```

---

## Testing

### Test Coverage

**Unit Tests:** 16 tests passing
- Platform detection
- Version validation
- Default versions
- Compatibility checks

**Integration Tests:** 6 counter variants
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

## Known Gaps

### Pending Enhancements

1. **Emulator Support**
   - **Status:** Not yet implemented
   - **Question:** Auto-install emulator images? Which ones? System images are large (~1GB each)
   - **Priority:** Low - users can install manually

2. **Gradle Wrapper vs Managed Gradle**
   - **Current:** Generate wrapper in project build dir
   - **Question:** Always use managed gradle instead?
   - **Priority:** Low - current approach works

3. **Telemetry**
   - **Question:** Track download success/failures (opt-in) to improve reliability?
   - **Priority:** Low - privacy concerns

4. **Mirror Support**
   - **Question:** Allow custom download mirrors for enterprises/China?
   - **Priority:** Medium - important for international users
   - **Effort:** 4-6 hours

5. **Version Aliases**
   - **Question:** Support "lts", "stable", "latest" instead of explicit numbers?
   - **Priority:** Low - explicit versions better for reproducibility
   - **Effort:** 2-3 hours

---

## Next Steps

### Short-term
- 🔜 Test on clean machine without system Java/Gradle/SDK
- 🔜 Add support for mirror URLs (enterprise use case)
- 🔜 Improve error messages for network failures

### Medium-term
- 🔜 Emulator installation and management
- 🔜 Version aliases ("lts", "stable", "latest")
- 🔜 Cache size limits and automatic cleanup

### Long-term
- 🔜 Telemetry (opt-in) for reliability metrics
- 🔜 Custom toolchain sources
- 🔜 Offline bundle generation

---

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

---

## Key Files Reference

| File | Lines | Purpose |
|------|-------|---------|
| `src/toolchain/mod.rs` | ~1000 | Core toolchain manager |
| `src/toolchain/defaults.rs` | ~60 | Default version constants |
| `src/toolchain/platform.rs` | ~130 | Platform detection |
| `src/toolchain/validator.rs` | ~270 | Version compatibility |
| `src/toolchain/downloader.rs` | ~480 | HTTP download + extraction |
| `src/commands/toolchain.rs` | ~240 | User commands (list/clean/install) |
| `src/commands/doctor.rs` | ~220 | Health check |

---

## Related Documentation

- [REF-OVERVIEW.md](./REF-OVERVIEW.md) - Architecture overview
- [REF-BUILD-SYSTEM.md](./REF-BUILD-SYSTEM.md) - Build commands
- [REF-STATE-MANAGEMENT.md](./REF-STATE-MANAGEMENT.md) - State management

---

*Last Updated: 2025-01-06*
*Version: 1.0*
*Status: Production Ready (Phases 1-5 Complete)*
