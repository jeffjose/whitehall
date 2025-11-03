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

## Phase 1: Validation (✓ Completed - v0.1)
**Goal: Parse and validate .whitehall files**

- [x] `.whitehall` file format specification (syntax design)
- [x] Lexer-free recursive descent parser for `.wh` files
- [x] Parser for `.wh` files (handles all syntax features)
- [x] Meaningful error messages from transpiler
- [ ] `whitehall check` - Syntax validation CLI command (planned)
- [ ] Basic LSP support (syntax highlighting) (future)

**Success metric:** ✓ Can write `.wh` files and transpiler validates them (30/30 tests passing)

**Status**: Transpiler core is **100% complete** - all 30 test cases passing (28 transpiler + 2 optimization examples). See `docs/TRANSPILER.md` for details.

---

## Phase 2: Routing (✓ Completed - v0.2)
**Goal: File-based routing with type-safe navigation**

- [x] Scan `src/routes/` directory structure
- [x] Parse `+screen.wh` files and `[param]` syntax
- [x] Generate @Serializable route objects (Navigation 2.8+ compatible)
- [x] Generate sealed interface Routes hierarchy
- [x] Generate NavHost with `composable<T>` calls
- [x] Type-safe navigation API in components (`$routes.*` references)
- [x] Support dynamic parameters (`$screen.params.id`)
- [ ] Support nested routes (future enhancement)

**Success metric:** ✓ Can create routes by adding files, navigate with type safety

**Status**: **100% complete** - File-based routing fully implemented:
- File discovery recognizes `src/routes/**/+screen.wh`
- Derives screen names from paths (login/+screen.wh → LoginScreen)
- Extracts route params from [id] folders
- Generates Routes.kt sealed interface
- Generates MainActivity with complete NavHost setup
- All 7 microblog screens transpile correctly

---

## Phase 3: Compilation (✓ Completed - v0.3)
**Goal: Generate working Android code**

- [x] AST → Kotlin transpiler (100% complete, 30 tests passing)
- [x] Generate Activity code (MainActivity with NavHost)
- [x] Handle UI components (all Compose components supported)
- [x] `whitehall build` - CLI command to transpile project
- [x] Generate Gradle build files (complete scaffold generation)
- [x] Invoke Gradle to create APK (via `whitehall run`)

**Success metric:** ✓ `whitehall build` produces a working Android project

**Status**: **100% complete** - Full compilation pipeline working:
- Transpiles all .wh files to idiomatic Kotlin
- Generates complete Gradle project structure
- Creates MainActivity with routing setup
- All CLI commands implemented (build, watch, run)

---

## Phase 4: Development Loop (✓ Completed - v0.4)
**Goal: Fast iteration**

- [x] `whitehall build` - Transpile project to Kotlin + generate Gradle scaffold
- [x] `whitehall watch` - File watching and auto-rebuild
- [x] `whitehall run` - Build + deploy to emulator/device
- [ ] Better error reporting with source maps
- [ ] `whitehall clean`

**Success metric:** Can develop a simple app using only Whitehall CLI

**Status**: ✓ All core CLI commands implemented and working:
- ✅ `whitehall init` - Creates project structure
- ✅ `whitehall build` - Transpiles .wh → .kt + generates Android scaffold
- ✅ `whitehall watch` - File watching with auto-rebuild (notify crate)
- ✅ `whitehall run` - Builds, runs Gradle, installs APK, launches app

**Next**: End-to-end testing with real apps to verify the complete workflow

---

## Phase 2.5: Single-File Mode (Planned - v0.25)
**Goal: Enable zero-config single-file apps** (Like `uv` for Python, `rust-script` for Rust)

- [ ] Parse frontmatter configuration (`///` TOML comments)
- [ ] Extract inline dependencies from frontmatter
- [ ] `whitehall run <file.wh>` - Single-file execution
- [ ] `whitehall build <file.wh>` - Build APK from single file
- [ ] Temporary project generation in `.whitehall/cache/{hash}/`
- [ ] Build caching for single-file apps (content-based hashing)
- [ ] `whitehall split <file.wh>` - Convert single-file to project
- [ ] Shebang support (`#!/usr/bin/env whitehall`)
- [ ] Size limits and warnings (>500 lines → suggest split)

**Success metric:** Can write a complete app in one `.wh` file and run it instantly

**Status**: Design complete (see `docs/SINGLE-FILE-MODE.md`). Implementation pending after end-to-end testing.

**Priority**: Medium (enables rapid prototyping and learning)

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
