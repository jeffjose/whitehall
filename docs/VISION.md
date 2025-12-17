# Vision

## The Dream

Whitehall is a unified, modern toolchain for Android app development - think `cargo` for Android. It brings the developer experience of modern build tools to mobile development.

## Current Status (January 2025)

**✅ Core Features Complete:**
- Transpiler: 100% complete (30 tests passing)
- Routing: File-based routing with type-safe navigation
- State Management: @store implementation with ViewModels
- Build System: All CLI commands working (init, compile, build, watch, run)
- Single-File Mode: Zero-boilerplate prototyping
- Toolchain Management: Zero-config setup (auto-downloads Java/Gradle/SDK)
- Web Playground: Interactive browser-based editor (Phase 1)

**🎯 Next Milestone:** End-to-end testing with real Android devices/emulators

## The Problem

Current Android development is fragmented:
- **Gradle** - Slow, verbose, JVM-based
- **Multiple languages** - Kotlin, XML, Groovy/Kotlin DSL for builds
- **Steep learning curve** - Too many tools and configuration files
- **Poor DX** - Slow builds, unclear error messages, heavyweight IDEs required
- **Setup friction** - Manual installation of Java, Android SDK, Gradle

Web developers have `npm`/`pnpm` + bundlers. Rust has `cargo`. iOS has Swift Package Manager. Android deserves better.

## The Solution

A single Rust-based CLI that handles everything, with zero prerequisites:

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

## What's Working Now

### Transpiler (100% Complete)
- ✅ **30 test cases passing** - All syntax features validated
- ✅ **Component transpilation** - `.wh` → idiomatic Kotlin/Compose
- ✅ **Control flow** - `@if`, `@for`, `@when` directives
- ✅ **Data binding** - `bind:value`, `bind:checked`
- ✅ **Lifecycle hooks** - `$onMount`, `$onDispose` with smart combination
- ✅ **String resources** - i18n support with `R.string.*`
- ✅ **Advanced patterns** - LazyColumn, AsyncImage, modifier chains

### Routing System (100% Complete)
- ✅ **File-based routing** - `src/routes/**/+screen.wh` convention
- ✅ **Type-safe navigation** - Sealed interface Routes generation
- ✅ **Route parameters** - `[id]` folders with automatic extraction
- ✅ **NavHost generation** - Complete MainActivity setup

### State Management (@store - Complete)
- ✅ **Screen-level state** - ViewModel generation from `@store` classes
- ✅ **Reactive properties** - UiState + StateFlow boilerplate generation
- ✅ **Derived properties** - val with getters supported
- ✅ **Auto-wrap suspend** - Functions wrapped in viewModelScope
- ✅ **Hilt integration** - Hybrid auto-detection (`@Inject` or `@hilt`)
- ✅ **Suspend functions** - Dispatcher control (`io`, `cpu`, `main`) and custom scopes

### Build System (All Commands Working)
- ✅ **`whitehall init`** - Project scaffolding with templates
- ✅ **`whitehall compile`** - Single-file transpilation
- ✅ **`whitehall build`** - Full project transpilation + Android scaffold
- ✅ **`whitehall watch`** - File watching with auto-rebuild
- ✅ **`whitehall run`** - Build + install + launch on device

### Single-File Mode (Complete)
- ✅ **Frontmatter config** - TOML metadata in `///` comments
- ✅ **Auto-package generation** - From app name
- ✅ **Content-based caching** - Fast rebuilds with `~/.cache/whitehall/`
- ✅ **Zero boilerplate** - Complete apps in one file

### Toolchain Management (Zero-Config)
- ✅ **Auto-download** - Java, Gradle, Android SDK installed automatically
- ✅ **Parallel downloads** - 3x faster setup
- ✅ **Retry logic** - Handles network failures gracefully
- ✅ **Project-specific versions** - Each project controls its toolchain
- ✅ **`whitehall doctor`** - Comprehensive health check

### Web Playground (Phase 1 Complete)
- ✅ **Monaco editor** - Full-featured code editor
- ✅ **Real-time compilation** - 500ms debounced transpilation
- ✅ **Multiple output tabs** - Kotlin / Errors / AST views
- ✅ **Example snippets** - 5 working examples
- ✅ **URL sharing** - Hash-based code sharing
- ✅ **Keyboard shortcuts** - Ctrl+Enter, Ctrl+S

## Core Principles

1. **Speed** - Rust-powered compilation and caching
2. **Simplicity** - One tool, clear conventions
3. **Modern** - Learn from Svelte, Cargo, and modern web tooling
4. **Scalable** - Works for small apps and large teams
5. **Interoperable** - Generates standard Android artifacts
6. **Zero-config** - No manual setup of Java/Gradle/SDK required

## What's Next

### Immediate (Testing Phase)
- **End-to-end testing** - Validate full pipeline with real devices
- **Bug fixes** - Address issues discovered during testing
- **Example apps** - Todo, blog reader, settings screens
- **Documentation** - Getting started guides and tutorials

### Short-term (Polish Phase)
- **Error messages** - Line/column precision with source context
- **Source maps** - Debug .wh files from Android Studio
- **LSP server** - Editor support with autocomplete
- **Shebang support** - `#!/usr/bin/env whitehall` for scripts

### Medium-term (Developer Experience)
- **Component playground** - Visual component testing
- **Interactive tutorials** - Learn-by-doing in CLI
- **Hot reload** - Live code updates without rebuild
- **Performance optimizations** - RecyclerView for static lists (implemented)

### Long-term (Platform Expansion)
- **Component marketplace** - Community component sharing
- **Multi-platform** - iOS via Compose Multiplatform
- **Web target** - Compose for Web integration
- **CI/CD templates** - GitHub Actions, GitLab CI integration
- **Package manager** - Dependency management and versioning
