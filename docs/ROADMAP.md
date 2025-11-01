# Roadmap

## Phase 0: Foundation (Current)
**Goal: Prove the concept**

- [ ] Basic CLI structure with `clap`
- [ ] `whitehall init` - Project scaffolding
  - [ ] Create project directory structure
  - [ ] Generate `Whitehall.toml` manifest
  - [ ] Create sample `.wh` file
  - [ ] Generate `.gitignore`
- [ ] Define initial `Whitehall.toml` schema
- [ ] Documentation structure (VISION, ROADMAP, ARCHITECTURE)

**Success metric:** Can run `whitehall init my-app` and get a valid project structure

---

## Phase 1: Validation (v0.1)
**Goal: Parse and validate .whitehall files**

- [ ] `.whitehall` file format specification (syntax design)
- [ ] Lexer for `.wh` files
- [ ] Parser for `.wh` files
- [ ] `whitehall check` - Syntax validation
- [ ] Meaningful error messages
- [ ] Basic LSP support (syntax highlighting)

**Success metric:** Can write `.wh` files and get useful validation feedback

---

## Phase 2: Compilation (v0.2)
**Goal: Generate working Android code**

- [ ] AST → Kotlin transpiler
- [ ] Generate basic Activity/Fragment code
- [ ] Handle UI components
- [ ] `whitehall build` - Transpile to Kotlin
- [ ] Generate Gradle build files
- [ ] Invoke Gradle to create APK

**Success metric:** `whitehall build` produces a working APK

---

## Phase 3: Development Loop (v0.3)
**Goal: Fast iteration**

- [ ] `whitehall run` - Build + deploy to emulator/device
- [ ] File watching and incremental builds
- [ ] Better error reporting with source maps
- [ ] `whitehall clean`

**Success metric:** Can develop a simple app using only Whitehall CLI

---

## Phase 4: Dependencies (v0.4)
**Goal: Reusable components**

- [ ] `whitehall install <dependency>` - Add libraries
- [ ] Support for Maven/Android dependencies
- [ ] Dependency resolution
- [ ] Lock file management

---

## Phase 5: Release & Polish (v0.5)
**Goal: Production-ready**

- [ ] `whitehall build --release` - Optimized builds
- [ ] ProGuard/R8 integration
- [ ] Code signing
- [ ] `whitehall test` - Testing framework
- [ ] CI/CD examples

---

## Future Phases
- Plugin system
- Hot reload
- Component marketplace
- Visual tooling
- Multi-platform support (iOS?)
