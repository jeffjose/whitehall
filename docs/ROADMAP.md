# Roadmap

## Phase 0: Foundation (✓ Completed)
**Goal: Prove the concept**

- [x] Basic CLI structure with `clap`
- [x] `whitehall init` - Project scaffolding
  - [x] Create project directory structure
  - [x] Generate `whitehall.toml` manifest
  - [x] Create sample `.wh` file
  - [x] Generate `.gitignore`
- [x] Define initial `whitehall.toml` schema
- [x] Documentation structure (VISION, ROADMAP, ARCHITECTURE)

**Success metric:** ✓ Can run `whitehall init my-app` and get a valid project structure

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

## Phase 2: Routing (v0.2)
**Goal: File-based routing with type-safe navigation**

- [ ] Scan `src/routes/` directory structure
- [ ] Parse `+screen.wh` files and `[param]` syntax
- [ ] Generate @Serializable route objects (Navigation 2.8+ compatible)
- [ ] Generate sealed interface Routes hierarchy
- [ ] Generate NavHost with `composable<T>` calls
- [ ] Type-safe navigation API in components
- [ ] Support dynamic parameters `[id]`
- [ ] Support nested routes

**Success metric:** Can create routes by adding files, navigate with type safety

---

## Phase 3: Compilation (v0.3)
**Goal: Generate working Android code**

- [ ] AST → Kotlin transpiler
- [ ] Generate basic Activity/Fragment code
- [ ] Handle UI components
- [ ] `whitehall build` - Transpile to Kotlin
- [ ] Generate Gradle build files
- [ ] Invoke Gradle to create APK

**Success metric:** `whitehall build` produces a working APK

---

## Phase 2.5: Single-File Mode (v0.25)
**Goal: Enable zero-config single-file apps**

- [ ] Parse frontmatter configuration (TOML-style comments)
- [ ] Extract inline dependencies from frontmatter
- [ ] `whitehall run <file.wh>` - Single-file execution
- [ ] `whitehall build <file.wh>` - Build APK from single file
- [ ] Temporary project generation in `.whitehall/tmp/`
- [ ] Build caching for single-file apps
- [ ] `whitehall split <file.wh>` - Convert single-file to project

**Success metric:** Can write a complete app in one `.wh` file and run it instantly

---

## Phase 4: Development Loop (v0.4)
**Goal: Fast iteration**

- [ ] `whitehall run` - Build + deploy to emulator/device
- [ ] File watching and incremental builds
- [ ] Better error reporting with source maps
- [ ] `whitehall clean`

**Success metric:** Can develop a simple app using only Whitehall CLI

---

## Phase 5: Dependencies (v0.5)
**Goal: Reusable components**

- [ ] `whitehall install <dependency>` - Add libraries
- [ ] Support for Maven/Android dependencies
- [ ] Dependency resolution
- [ ] Lock file management

---

## Phase 6: Release & Polish (v0.6)
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
