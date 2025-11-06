# REF-* Documentation Review Plan

**Goal:** Verify all REF-* documentation against actual code implementation to ensure accuracy.

**Methodology:** For each claim in the docs, trace through actual code to verify behavior.

**Status:** Created 2025-11-06

---

## Review Checklist

### ✅ REF-STATE-MANAGEMENT.md - REVIEWED (2025-11-06)

**Status:** Systematic review complete - all issues fixed
- ✅ Fixed @store vs ViewModels confusion
- ✅ Verified class with var → ViewModel
- ✅ Verified @store object → Singleton
- ✅ Verified @Inject/@hilt → @HiltViewModel
- ✅ Fixed Phase 1.1 status contradictions (line 11, line 105)
- ✅ Updated StoreInfo struct documentation with missing fields
- ✅ Removed misleading @store annotations from Hilt examples

**Code verified:**
- `src/transpiler/analyzer.rs:75-83` - StoreInfo struct definition
- `src/transpiler/analyzer.rs:397-430` - collect_stores logic
- `src/transpiler/codegen/compose.rs:3020-3120` - ViewModel vs Singleton generation
- `tests/transpiler-examples/27-29.md` - Store test cases
- `tests/transpiler-examples/30-32.md` - Component inline vars (all passing)

---

### ✅ REF-TRANSPILER.md - NEEDS REVIEW

**Claims to verify:**

1. **Test count and status**
   - ✅ Doc says: 37/38 tests (97.4%)
   - ✅ Code check: `cargo test --test transpiler_examples_test examples`
   - ✅ Verified: Accurate

2. **Parser architecture: "Lexer-free recursive descent"**
   - [ ] Code check: `src/transpiler/parser.rs`
   - [ ] Verify: Is it actually lexer-free? Or does it tokenize first?
   - [ ] Verify: Recursive descent implementation?

3. **AST structure claims**
   - [ ] Code check: `src/transpiler/ast.rs`
   - [ ] Verify: WhitehallFile structure matches doc
   - [ ] Verify: Markup enum variants match doc
   - [ ] Verify: PropValue enum matches doc

4. **Code generator architecture**
   - [ ] Code check: `src/transpiler/codegen/compose.rs`
   - [ ] Verify: Component-specific transformations table
   - [ ] Verify: Import management claims
   - [ ] Verify: Value transformations (lambda arrows, route aliases, etc.)

5. **Store registry claims**
   - [ ] Code check: `src/transpiler/analyzer.rs`
   - [ ] Verify: StoreRegistry structure
   - [ ] Verify: Cross-file detection mechanism
   - [ ] Verify: Hilt detection logic

6. **Multi-file output (Phase 1.1)**
   - [ ] Code check: `src/transpiler/mod.rs` TranspileResult enum
   - [ ] Verify: Single vs Multiple variants
   - [ ] Verify: API methods (primary_content, is_multiple, files)

7. **Test categories breakdown**
   - [ ] Verify: Foundation (00-00e) - 6 tests exists
   - [ ] Verify: Core Features (01-06) - 6 tests
   - [ ] Verify: Routing (07-08) - 2 tests
   - [ ] Verify: Composition (09-11) - 3 tests
   - [ ] Verify: Extended Patterns (12-17) - 6 tests
   - [ ] Verify: Advanced Features (18-26) - 9 tests
   - [ ] Verify: Stores (27-29) - 3 tests
   - [ ] Verify: Phase 1.1 (30-32) - 3 tests

**Files to check:**
- `src/transpiler/parser.rs` (~1200 lines)
- `src/transpiler/ast.rs` (~200 lines)
- `src/transpiler/analyzer.rs` (~300 lines)
- `src/transpiler/codegen/compose.rs` (~3500 lines)
- `tests/transpiler-examples/*.md` (38 files)

---

### ⏳ REF-BUILD-SYSTEM.md - NEEDS REVIEW

**Claims to verify:**

1. **Command availability**
   - ✅ Doc says: 9 commands (init, compile, build, watch, run, toolchain, exec, shell, doctor)
   - ✅ Code check: `src/main.rs` Commands enum
   - ✅ Verified: Accurate

2. **`whitehall build` behavior**
   - [ ] Code check: `src/commands/build.rs`
   - [ ] Verify: One-shot transpilation claim
   - [ ] Verify: Output directory structure
   - [ ] Verify: Generates complete Gradle project claim

3. **`whitehall watch` behavior**
   - [ ] Code check: `src/commands/watch.rs`
   - [ ] Verify: File watching implementation (notify crate)
   - [ ] Verify: Incremental builds claim
   - [ ] Verify: 100ms detection claim
   - [ ] Verify: Debouncing logic

4. **`whitehall run` behavior**
   - [ ] Code check: `src/commands/run.rs`
   - [ ] Verify: Build + gradle + adb pipeline
   - [ ] Verify: Device detection logic
   - [ ] Verify: APK installation process

5. **`whitehall init` behavior**
   - [ ] Code check: `src/commands/init.rs`
   - [ ] Verify: Project structure generation
   - [ ] Verify: whitehall.toml template
   - [ ] Verify: Example file creation

6. **`whitehall compile` behavior**
   - [ ] Code check: `src/commands/compile.rs`
   - [ ] Verify: Single-file transpilation
   - [ ] Verify: --package flag
   - [ ] Verify: --no-package flag

7. **`whitehall toolchain` subcommands**
   - [ ] Code check: `src/commands/toolchain.rs`
   - [ ] Verify: install subcommand
   - [ ] Verify: list subcommand
   - [ ] Verify: clean subcommand

8. **`whitehall exec` behavior**
   - [ ] Code check: `src/commands/exec.rs`
   - [ ] Verify: Environment variable setup
   - [ ] Verify: JAVA_HOME, ANDROID_HOME handling

9. **`whitehall shell` behavior**
   - [ ] Code check: `src/commands/shell.rs`
   - [ ] Verify: Interactive shell launch
   - [ ] Verify: Toolchain environment setup

10. **`whitehall doctor` behavior**
    - [ ] Code check: `src/commands/doctor.rs`
    - [ ] Verify: Health check items
    - [ ] Verify: Error detection and suggestions

11. **Build pipeline architecture**
    - [ ] Code check: `src/build_pipeline.rs`
    - [ ] Verify: Shared pipeline diagram accuracy
    - [ ] Verify: File discovery logic
    - [ ] Verify: Package path mapping

12. **File type detection**
    - [ ] Code check: `src/project.rs`
    - [ ] Verify: components/ → .components package
    - [ ] Verify: screens/ → .screens package
    - [ ] Verify: stores/ → .stores package

**Files to check:**
- `src/main.rs` - CLI definition
- `src/commands/*.rs` - All command implementations
- `src/build_pipeline.rs` - Shared build logic
- `src/project.rs` - File discovery

---

### ⏳ REF-TOOLCHAIN.md - NEEDS REVIEW

**Claims to verify:**

1. **Cache structure**
   - [ ] Code check: `src/toolchain/mod.rs`
   - [ ] Verify: ~/.whitehall/toolchains/ structure
   - [ ] Verify: java/, gradle/, android/ subdirectories
   - [ ] Verify: Version-specific subdirectories

2. **Toolchain detection and download**
   - [ ] Code check: `src/toolchain/downloader.rs`
   - [ ] Verify: Download sources for Java/Gradle/SDK
   - [ ] Verify: Extraction logic
   - [ ] Verify: Verification after download

3. **Version compatibility**
   - [ ] Code check: `src/toolchain/validator.rs`
   - [ ] Verify: AGP version requirements
   - [ ] Verify: Java version compatibility matrix
   - [ ] Verify: Gradle version requirements

4. **Platform detection**
   - [ ] Code check: `src/toolchain/platform.rs`
   - [ ] Verify: OS detection (Linux, macOS, Windows)
   - [ ] Verify: Architecture detection (x86_64, aarch64)
   - [ ] Verify: Platform-specific download URLs

5. **Default versions**
   - [ ] Code check: `src/toolchain/defaults.rs`
   - [ ] Verify: Default Java version
   - [ ] Verify: Default Gradle version
   - [ ] Verify: Default AGP version

6. **Environment variable setup**
   - [ ] Verify: JAVA_HOME setup
   - [ ] Verify: ANDROID_HOME setup
   - [ ] Verify: PATH modifications

**Files to check:**
- `src/toolchain/mod.rs` - Core manager
- `src/toolchain/defaults.rs` - Default versions
- `src/toolchain/platform.rs` - Platform detection
- `src/toolchain/validator.rs` - Version compatibility
- `src/toolchain/downloader.rs` - Download logic

---

### ⏳ REF-WEB-PLAYGROUND.md - NEEDS REVIEW

**Claims to verify:**

1. **Status: "Phase 1 Complete (Nov 4, 2025)"**
   - [ ] Verify: Actual completion date (seems wrong - says 2025)
   - [ ] Verify: Is Phase 1 actually complete?

2. **Backend tech stack**
   - [ ] Code check: `tools/playground/backend/Cargo.toml`
   - [ ] Verify: Axum framework
   - [ ] Verify: Tower-HTTP CORS
   - [ ] Verify: Port 3000

3. **Frontend tech stack**
   - [ ] Code check: `tools/playground/frontend/`
   - [ ] Verify: Monaco Editor via CDN
   - [ ] Verify: Tailwind CSS via CDN
   - [ ] Verify: Vanilla JavaScript (no build step)

4. **API endpoint: POST /api/compile**
   - [ ] Code check: `tools/playground/backend/src/main.rs`
   - [ ] Verify: Request/Response format
   - [ ] Verify: Error handling
   - [ ] Verify: Success response structure

5. **Features claimed**
   - [ ] Verify: Real-time compilation with 500ms debounce
   - [ ] Verify: Multiple output tabs (Kotlin / Errors / AST)
   - [ ] Verify: 5 example snippets exist
   - [ ] Verify: URL hash state for sharing
   - [ ] Verify: Copy/format/clear buttons
   - [ ] Verify: Keyboard shortcuts (Ctrl+Enter, Ctrl+S)
   - [ ] Verify: Mobile responsive layout

6. **File structure**
   - [ ] Verify: tools/playground/backend/ exists
   - [ ] Verify: tools/playground/frontend/ exists
   - [ ] Verify: README.md exists

**Files to check:**
- `tools/playground/backend/src/main.rs`
- `tools/playground/backend/Cargo.toml`
- `tools/playground/frontend/index.html`
- `tools/playground/frontend/style.css`
- `tools/playground/frontend/app.js`

---

### ⏳ REF-OVERVIEW.md - NEEDS REVIEW

**Claims to verify:**

1. **Component status table**
   - [ ] Verify: Transpiler 37/38 tests
   - [ ] Verify: Build System "Fully Implemented"
   - [ ] Verify: State Management "Phase 1.1 Complete"
   - [ ] Verify: Toolchain "Fully Implemented"
   - [ ] Verify: Web Playground "Phase 1 Complete"

2. **Architecture layers diagram**
   - [ ] Verify: Actual module structure matches diagram
   - [ ] Verify: Layer separation accurate

3. **File structure guide**
   - [ ] Verify: Directory structure matches reality
   - [ ] Verify: File paths are correct
   - [ ] Verify: Line counts are accurate
   - [ ] Verify: Test count (38 not 23)

4. **State management patterns table**
   - [ ] Verify: All patterns are actually implemented
   - [ ] Verify: Status indicators are accurate

5. **CLI command table**
   - [ ] Verify: All 9 commands exist
   - [ ] Verify: Descriptions match actual behavior

6. **Test categories**
   - [ ] Verify: Test file count per category
   - [ ] Verify: Total test count = 38

7. **Compilation pipeline diagram**
   - [ ] Verify: Pipeline stages match code
   - [ ] Verify: Module flow is accurate

**Files to check:**
- All REF-* files (cross-references)
- `src/` directory structure
- `tests/transpiler-examples/` count

---

## Review Process

For each REF-* file:

1. **Read documentation section by section**
2. **Identify concrete claims** (numbers, behavior, structure)
3. **Trace claim to source code**
4. **Run code/tests if needed** to verify behavior
5. **Note discrepancies** in this file
6. **Fix documentation** if wrong
7. **Update code** if docs are right but code is wrong
8. **Mark section as reviewed** with checkmark

## Discrepancies Found

### REF-STATE-MANAGEMENT.md (Fixed)
- **Issue:** Claimed @store class creates ViewModels
- **Reality:** class with var creates ViewModels (no @store needed)
- **Issue:** Didn't document @store object for singletons
- **Reality:** @store object → Singleton with StateFlow
- **Fixed:** 2025-11-06 (initial commits 40d86d5, 0b9e066)
- **Issue:** Line 11 said "underway" but status header said "Phase 1.1 Complete"
- **Reality:** Phase 1.1 is complete with 38/38 tests passing
- **Issue:** Line 105 status showed "🔄 In Progress"
- **Reality:** Should be "✅ Complete"
- **Issue:** StoreInfo struct documentation missing 2 fields
- **Reality:** Missing has_vars: bool and route_params: Vec<String>
- **Issue:** Misleading @store annotations in Hilt examples
- **Reality:** var triggers ViewModel, not @store; @store only for object singletons
- **Fixed:** 2025-11-06 (commit 91a3bb5)

### REF-TRANSPILER.md
- **Issue:** Doc claimed 37/38 tests (97.4%), test 05 failing
- **Reality:** 38/38 tests passing (100%) - test 05 has been fixed
- **Issue:** Doc claimed ~600 LOC parser
- **Reality:** 1595 lines actual
- **Issue:** Component struct props type wrong in doc
- **Reality:** props is Vec<ComponentProp>, not Vec<(String, PropValue)>, also has self_closing field
- **Fixed:** 2025-11-06 (updated test status, line counts, AST docs)
- (To be filled as we review)

### REF-BUILD-SYSTEM.md
- (To be filled as we review)

### REF-TOOLCHAIN.md
- (To be filled as we review)

### REF-WEB-PLAYGROUND.md
- (To be filled as we review)

### REF-OVERVIEW.md
- (To be filled as we review)

---

## Priority Order

1. ✅ **REF-STATE-MANAGEMENT.md** - DONE (had major issues, now fixed)
2. **REF-TRANSPILER.md** - HIGH (core functionality, complex claims)
3. **REF-BUILD-SYSTEM.md** - HIGH (user-facing commands)
4. **REF-OVERVIEW.md** - MEDIUM (high-level, references other docs)
5. **REF-TOOLCHAIN.md** - MEDIUM (well-defined scope)
6. **REF-WEB-PLAYGROUND.md** - LOW (separate tool, less critical)

---

## How to Use This File

1. Pick a REF-* file to review
2. Go through each claim in the checklist
3. Check the corresponding code
4. Mark items with ✅ if accurate, ❌ if wrong
5. Document discrepancies in "Discrepancies Found" section
6. Fix documentation or code as needed
7. Update status at top when file review is complete

---

*Created: 2025-11-06*
*Last Updated: 2025-11-06*
