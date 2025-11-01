# Vision

## The Dream

Whitehall is a unified, modern toolchain for Android app development - think `cargo` for Android. It brings the developer experience of modern build tools to mobile development.

## The Problem

Current Android development is fragmented:
- **Gradle** - Slow, verbose, JVM-based
- **Multiple languages** - Kotlin, XML, Groovy/Kotlin DSL for builds
- **Steep learning curve** - Too many tools and configuration files
- **Poor DX** - Slow builds, unclear error messages, heavyweight IDEs required

Web developers have `npm`/`pnpm` + bundlers. Rust has `cargo`. iOS has Swift Package Manager. Android deserves better.

## The Solution

A single Rust-based CLI that handles everything:

```bash
# Single-file mode (like rustc or Python's uv)
whitehall run app.wh            # Run a complete app from one file
whitehall build app.wh          # Build APK from single file

# Project mode (like cargo)
whitehall init my-app           # Scaffold new project
whitehall build                 # Compile .whitehall → Kotlin/Java
whitehall build --release       # Production builds
whitehall run                   # Build + run on emulator/device
whitehall test                  # Run tests
whitehall install <dependency>  # Add dependencies
whitehall publish               # Publish to Play Store
```

### Two Modes, One Tool

**Single-file mode** - Perfect for learning, prototyping, and sharing:
- Complete Android apps in a single `.wh` file
- Inline dependency declarations (like Python's uv)
- Zero boilerplate - just write your app
- Instant sharing - paste a file, run it

**Project mode** - For production apps:
- Structured file organization
- File-based routing
- Shared components and assets
- Team-ready architecture

## Core Principles

1. **Speed** - Rust-powered compilation and caching
2. **Simplicity** - One tool, clear conventions
3. **Modern** - Learn from Svelte, Cargo, and modern web tooling
4. **Scalable** - Works for small apps and large teams
5. **Interoperable** - Generates standard Android artifacts

## The Future

- Custom `.whitehall` file format for component definitions
- Smart compilation to Kotlin/Java
- Built-in hot reload
- Component marketplace
- First-class testing and debugging
- CI/CD integration out of the box
