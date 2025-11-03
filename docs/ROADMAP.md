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

## Phase 2: Routing (Partial - v0.2)
**Goal: File-based routing with type-safe navigation**

- [ ] Scan `src/routes/` directory structure
- [ ] Parse `+screen.wh` files and `[param]` syntax
- [ ] Generate @Serializable route objects (Navigation 2.8+ compatible)
- [ ] Generate sealed interface Routes hierarchy
- [ ] Generate NavHost with `composable<T>` calls
- [x] Type-safe navigation API in components (`$routes.*` references)
- [x] Support dynamic parameters (`$screen.params.id`)
- [ ] Support nested routes

**Success metric:** Can create routes by adding files, navigate with type safety

**Status**: Transpiler handles `$routes.*` and `$screen.params.*` references (tests 07-08), but Routes.kt generation not implemented yet.

---

## Phase 3: Compilation (Partial - v0.3)
**Goal: Generate working Android code**

- [x] AST → Kotlin transpiler (100% complete, 30 tests passing)
- [x] Generate Activity code (MainActivity generation planned)
- [x] Handle UI components (all Compose components supported)
- [ ] `whitehall build` - CLI command to transpile project
- [ ] Generate Gradle build files (scaffold generation planned)
- [ ] Invoke Gradle to create APK

**Success metric:** `whitehall build` produces a working APK

**Status**: Transpiler complete. CLI integration in progress - see `docs/BUILD.md` for implementation plan.

---

## Phase 4: Development Loop (🔄 In Progress - v0.4)
**Goal: Fast iteration**

- [ ] `whitehall build` - Transpile project to Kotlin + generate Gradle scaffold
- [ ] `whitehall watch` - File watching and auto-rebuild
- [ ] `whitehall run` - Build + deploy to emulator/device
- [ ] Better error reporting with source maps
- [ ] `whitehall clean`

**Success metric:** Can develop a simple app using only Whitehall CLI

**Status**: Currently implementing - see `docs/BUILD.md` for detailed plan. Five phases:
- Phase 1: Foundation (shared build pipeline)
- Phase 2: `build` command
- Phase 3: `watch` command
- Phase 4: `run` command
- Phase 5: Polish and documentation

---

## Phase 2.5: Single-File Mode (Deferred - v0.25)
**Goal: Enable zero-config single-file apps**

- [ ] Parse frontmatter configuration (TOML-style comments)
- [ ] Extract inline dependencies from frontmatter
- [ ] `whitehall run <file.wh>` - Single-file execution
- [ ] `whitehall build <file.wh>` - Build APK from single file
- [ ] Temporary project generation in `.whitehall/tmp/`
- [ ] Build caching for single-file apps
- [ ] `whitehall split <file.wh>` - Convert single-file to project

**Success metric:** Can write a complete app in one `.wh` file and run it instantly

**Status**: Deferred until project mode is stable. Focus on project-based development first.

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
