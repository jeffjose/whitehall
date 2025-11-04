# Toolchain Testing Guide

This directory contains counter app variants with different toolchain configurations to test Whitehall's toolchain management.

## Available Variants

### counter (Java 21 - Latest)
**Modern toolchain** - Recommended for new projects
```toml
[toolchain]
java = "21"
gradle = "8.4"
agp = "8.2.0"
kotlin = "2.0.0"
```

### counter-java-17 (Java 17 - Previous LTS)
**Stable toolchain** - Good for production apps
```toml
[toolchain]
java = "17"
gradle = "8.0"
agp = "8.0.0"
kotlin = "1.9.0"
```

### counter-java-11-gradle-7 (Java 11 - Oldest Supported)
**Legacy toolchain** - For older Android projects
```toml
[toolchain]
java = "11"
gradle = "7.6"
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
  - 11 (~300 MB)
  - 17 (~300 MB)
  - 21 (~300 MB)

Gradle:
  - 7.6 (~130 MB)
  - 8.0 (~135 MB)
  - 8.4 (~138 MB)
```

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

## What This Tests

- ✅ **Multiple Java versions** coexist peacefully
- ✅ **Multiple Gradle versions** coexist peacefully
- ✅ **Automatic version selection** based on project config
- ✅ **Version compatibility validation** prevents invalid configs
- ✅ **Environment isolation** - each project gets correct versions
- ✅ **Shared cache** - download once, use everywhere
- ✅ **Zero config** - just run `whitehall exec`, it downloads automatically

## Expected Behavior

When you switch directories:

```bash
cd counter                       # whitehall.toml says java=21
whitehall exec java --version   # → Uses Java 21

cd ../counter-java-17           # whitehall.toml says java=17
whitehall exec java --version   # → Uses Java 17 (automatic!)
```

No need to activate/deactivate environments - it's automatic based on which directory you're in!
