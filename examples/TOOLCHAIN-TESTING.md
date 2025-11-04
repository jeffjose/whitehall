# Toolchain Testing Guide

This directory contains counter app variants with different toolchain configurations to test Whitehall's toolchain management.

## Available Variants

All variants use the same Counter app codebase with different toolchain configurations.

### Java 21 Variants

**counter** - Java 21 + Gradle 8.4 (Modern - Recommended)
```toml
[toolchain]
java = "21"
gradle = "8.4"
agp = "8.2.0"
kotlin = "2.0.0"
```

**counter-java-21-gradle-8.6** - Java 21 + Gradle 8.6 (Newest Gradle)
```toml
[toolchain]
java = "21"
gradle = "8.6"
agp = "8.4.0"
kotlin = "2.0.0"
```

### Java 17 Variants

**counter-java-17** - Java 17 + Gradle 8.0 (Stable LTS)
```toml
[toolchain]
java = "17"
gradle = "8.0"
agp = "8.0.0"
kotlin = "1.9.0"
```

**counter-java-17-gradle-7** - Java 17 + Gradle 7.6 (Java 17 with older Gradle)
```toml
[toolchain]
java = "17"
gradle = "7.6"
agp = "7.4.0"
kotlin = "1.8.0"
```

### Java 11 Variants

**counter-java-11-gradle-7** - Java 11 + Gradle 7.6 (Legacy)
```toml
[toolchain]
java = "11"
gradle = "7.6"
agp = "7.4.0"
kotlin = "1.8.0"
```

**counter-java-11-gradle-8** - Java 11 + Gradle 8.0 (Old Java + New Gradle)
```toml
[toolchain]
java = "11"
gradle = "8.0"
agp = "7.4.0"
kotlin = "1.8.0"
```

## Testing Toolchain Switching

### Test 1: Install Different Versions

```bash
# Install Java 21 toolchain
cd counter
whitehall toolchain install

# Install Java 17 toolchain (coexists with Java 21)
cd ../counter-java-17
whitehall toolchain install

# Install Java 11 toolchain (all three coexist!)
cd ../counter-java-11-gradle-7
whitehall toolchain install

# Verify all are installed
whitehall toolchain list
```

Expected output:
```
Java:
  - 11 (~315 MB)
  - 17 (~315 MB)
  - 21 (~344 MB)

Gradle:
  - 7.6 (~130 MB)
  - 8.0 (~131 MB)
  - 8.4 (~138 MB)
  - 8.6 (~140 MB)

Android SDK:
  - Installed (~390 MB)
```

Total: ~1.7 GB for all variants combined!

### Test 2: Automatic Version Selection

Each project automatically uses its own toolchain:

```bash
# Java 21 project
cd counter
whitehall exec java --version
# → openjdk version "21.0.9"

whitehall exec which java
# → ~/.whitehall/toolchains/java/21/bin/java

# Java 17 project
cd ../counter-java-17
whitehall exec java --version
# → openjdk version "17.x.x"

whitehall exec which java
# → ~/.whitehall/toolchains/java/17/bin/java

# Java 11 project
cd ../counter-java-11-gradle-7
whitehall exec java --version
# → openjdk version "11.x.x"

whitehall exec which gradle
# → ~/.whitehall/toolchains/gradle/7.6/bin/gradle
```

### Test 3: Validation Catches Incompatibilities

Try creating an invalid configuration:

```bash
cd counter
# Edit whitehall.toml and set java = "11" with agp = "8.2.0"

whitehall toolchain install
# → Error: AGP 8.2.0 requires Java 17 or higher
```

### Test 4: Shell Environment

```bash
# Enter shell for Java 21 project
cd counter
whitehall shell
(whitehall:Counter) $ java --version
# → Java 21

(whitehall:Counter) $ echo $JAVA_HOME
# → ~/.whitehall/toolchains/java/21

(whitehall:Counter) $ exit

# Enter shell for Java 17 project
cd ../counter-java-17
whitehall shell
(whitehall:Counter (Java 17)) $ java --version
# → Java 17

(whitehall:Counter (Java 17)) $ exit
```

### Test 5: Clean and Reinstall

```bash
# Remove all toolchains
whitehall toolchain clean

# Verify removed
whitehall toolchain list
# → All empty

# Go to any project and install
cd counter
whitehall exec java --version
# → Automatically downloads and installs Java 21!
```

## Toolchain Combination Matrix

6 variants testing all major combinations:

| Variant | Java | Gradle | AGP | Use Case |
|---------|------|--------|-----|----------|
| `counter` | 21 | 8.4 | 8.2.0 | Modern (recommended) |
| `counter-java-21-gradle-8.6` | 21 | 8.6 | 8.4.0 | Newest Gradle |
| `counter-java-17` | 17 | 8.0 | 8.0.0 | Stable LTS |
| `counter-java-17-gradle-7` | 17 | 7.6 | 7.4.0 | Java 17 + Old Gradle |
| `counter-java-11-gradle-7` | 11 | 7.6 | 7.4.0 | Legacy baseline |
| `counter-java-11-gradle-8` | 11 | 8.0 | 7.4.0 | Old Java + New Gradle |

## What This Tests

- ✅ **Multiple Java versions** coexist peacefully (11, 17, 21)
- ✅ **Multiple Gradle versions** coexist peacefully (7.6, 8.0, 8.4, 8.6)
- ✅ **Automatic version selection** based on project config
- ✅ **Version compatibility validation** prevents invalid configs
- ✅ **Environment isolation** - each project gets correct versions
- ✅ **Shared cache** - download once, use everywhere
- ✅ **Zero config** - just run `whitehall exec`, it downloads automatically
- ✅ **Cross-version mixing** - Old Java + New Gradle works
- ✅ **Same codebase** - Proves toolchain is truly isolated

## Expected Behavior

When you switch directories:

```bash
cd counter                       # whitehall.toml says java=21
whitehall exec java --version   # → Uses Java 21

cd ../counter-java-17           # whitehall.toml says java=17
whitehall exec java --version   # → Uses Java 17 (automatic!)
```

No need to activate/deactivate environments - it's automatic based on which directory you're in!
