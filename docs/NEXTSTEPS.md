# Next Steps for Whitehall

**Last Updated**: 2025-11-03

## Current Status

✅ **Transpiler Core: 100% Complete**
- All 23 test cases passing (100% coverage)
- Zero compiler warnings
- Clean, production-ready codebase
- Full feature parity with syntax design

### What's Working
- ✅ Component transpilation (.wh → .kt)
- ✅ All control flow (@if, @for, @when)
- ✅ Data binding (bind:value, bind:checked)
- ✅ Lifecycle hooks (onMount, onDispose)
- ✅ Routing and navigation
- ✅ Import resolution
- ✅ State management (mutableStateOf, derivedStateOf)
- ✅ Advanced patterns (LazyColumn, AsyncImage, modifiers)
- ✅ Internationalization (string resources)

### What's NOT Working
- ❌ CLI integration (`whitehall dev`, `whitehall build`)
- ❌ File watching for auto-rebuild
- ❌ Error messages with line numbers
- ❌ Source maps for debugging
- ❌ Real-world project transpilation

## Next Step Options

### Option 1: CLI Integration (Recommended)
**Goal**: Connect the transpiler to actual CLI commands so users can build real apps

**Estimated Effort**: 4-6 hours

**Tasks**:
1. Implement `whitehall dev` command
   - Watch `.wh` files for changes
   - Trigger transpilation on file change
   - Output generated `.kt` files to build directory
   - Display compilation status

2. Implement `whitehall build` command
   - Transpile all `.wh` files in src/
   - Generate Routes.kt from directory structure
   - Output to configured build directory
   - Report any errors

3. File system integration
   - Recursive directory traversal
   - File type detection (.wh vs +screen.wh)
   - Output path generation (mirror src/ structure)
   - Preserve directory structure

4. Basic error reporting
   - Catch transpilation errors
   - Display filename and error message
   - Continue on error (don't crash on one bad file)

**Why This?**
- Makes the transpiler actually usable for building apps
- Unblocks real-world testing
- Required before any production use
- Foundation for other features (watch mode needs this)

**Starting Point**:
- `src/cli/dev.rs` (create)
- `src/cli/build.rs` (create)
- Update `src/main.rs` to add commands

### Option 2: Developer Experience Improvements
**Goal**: Make transpiler errors helpful and debuggable

**Estimated Effort**: 6-8 hours

**Tasks**:
1. Position tracking in parser
   - Track line/column for every parsed element
   - Include position in AST nodes
   - Preserve position through transformations

2. Error messages with context
   - Show error location (file:line:col)
   - Display relevant source code snippet
   - Highlight problematic token/expression
   - Suggest fixes when possible

3. Source maps (optional)
   - Generate .map files linking Kotlin back to .wh
   - Enable debugging in Android Studio
   - Show original .wh code in stack traces

4. Watch mode with live reload
   - Use `notify` crate for file watching
   - Incremental compilation (only changed files)
   - Show compilation time and status
   - Color-coded output (errors in red, etc.)

**Why This?**
- Essential for good developer experience
- Makes debugging much easier
- Users will appreciate helpful error messages
- Watch mode increases productivity significantly

**Starting Point**:
- Enhance `src/transpiler/parser.rs` with position tracking
- Create `src/transpiler/error.rs` for formatted errors
- Add `notify` dependency for file watching

### Option 3: Real-World Testing
**Goal**: Validate transpiler on actual applications

**Estimated Effort**: 2-4 hours (testing) + bug fixes as needed

**Tasks**:
1. Create sample applications
   - Todo app (state management, lists)
   - Blog reader (API calls, routing, images)
   - Settings screen (forms, persistence)

2. Transpile and test
   - Write apps in Whitehall syntax
   - Transpile to Kotlin
   - Build with Gradle
   - Run on Android/Desktop
   - Fix any discovered bugs

3. Edge case discovery
   - Complex component nesting
   - Large files (500+ lines)
   - Unusual prop combinations
   - Performance edge cases

4. Create integration tests
   - Add new test cases for discovered issues
   - Ensure bugs don't regress
   - Document edge cases in tests

**Why This?**
- Discover bugs before users do
- Build confidence in production readiness
- Create example apps for documentation
- Validate design decisions with real use cases

**Starting Point**:
- Create `examples/` directory
- Build simple apps in .wh format
- Test transpilation and compilation

### Option 4: Route Generation System
**Goal**: Generate Routes.kt from file-based routing structure

**Estimated Effort**: 4-5 hours

**Tasks**:
1. Directory structure analysis
   - Scan src/routes/ for +screen.wh files
   - Extract route parameters from [param] directories
   - Build route tree data structure
   - Detect nested routes and layouts

2. Routes.kt generation
   - Generate sealed class hierarchy
   - Add @Serializable annotations
   - Create route builder functions
   - Generate lowercase accessor object

3. NavHost generation
   - Generate NavHost composable
   - Map routes to screens
   - Handle route parameters
   - Support nested navigation

4. Integration with screens
   - Pass route params to screen functions
   - Transform $routes.* references
   - Type-safe navigation calls

**Why This?**
- Complete the routing story
- Enable type-safe navigation
- Critical for multi-screen apps
- Already designed, just needs implementation

**Starting Point**:
- Create `src/transpiler/routes.rs`
- Implement directory scanning
- Generate sealed class structure

### Option 5: Additional Test Coverage (If Needed)
**Goal**: Add more test cases for edge cases and patterns

**Estimated Effort**: 1-2 hours per test

**Potential Tests**:
- Event handlers (multiple types: onClick, onLongClick, etc.)
- Animations and transitions
- Gestures and modifiers
- Custom components
- Complex nested layouts
- Performance edge cases
- Error boundary patterns

**Why This?**
- Only if gaps are discovered during real-world testing
- Depends on what patterns users actually need
- Can add incrementally as needs arise

**Starting Point**:
- Add new .md files to tests/transpiler-examples/
- Follow existing test format
- Implement features to make tests pass

## Recommendation

**Start with Option 1 (CLI Integration)**

**Rationale**:
1. **Unblocks everything else**: Can't do real-world testing without CLI
2. **Immediate value**: Makes transpiler actually usable
3. **Foundation for DX**: Watch mode and error reporting need CLI first
4. **Quick win**: 4-6 hours to basic working version
5. **User-facing**: Developers can start building apps

**After CLI Integration**:
1. Do Option 3 (Real-World Testing) to discover issues
2. Add Option 5 (Additional Tests) for any discovered gaps
3. Enhance with Option 2 (Developer Experience)
4. Complete with Option 4 (Route Generation)

## What's NOT Needed Yet

These can wait until there's real demand:

- 🔜 Package management (dependencies, versions)
- 🔜 Plugin system for custom transformations
- 🔜 Optimization passes (dead code elimination, etc.)
- 🔜 Multi-platform targeting (iOS, Web)
- 🔜 Hot reload / hot module replacement
- 🔜 LSP (Language Server Protocol) for editor support
- 🔜 Formatter and linter

## How to Resume Work

### Quick Start (CLI Integration)

```bash
# 1. Create CLI module structure
mkdir -p src/cli
touch src/cli/mod.rs src/cli/dev.rs src/cli/build.rs

# 2. Add file watching dependency
cargo add notify

# 3. Start implementing in src/cli/build.rs:
# - Scan src/ directory for .wh files
# - Call transpile() for each file
# - Write output to build/generated/

# 4. Test with a simple project
whitehall init test-app
cd test-app
# Add some .wh files
whitehall build
# Check build/generated/ for output

# 5. Iterate until working
```

### Design Decisions to Make

**For CLI Integration**:
- Where to output generated files? (`build/generated/`? `app/src/main/kotlin/`?)
- How to mirror directory structure? (keep src/components/ → .../components/?)
- What to do on error? (stop build? continue with warnings?)
- How to show progress? (spinner? progress bar? simple log?)

**For Watch Mode**:
- Which directories to watch? (just src/? include tests/?)
- Debounce time? (300ms? 500ms?)
- How to handle rapid changes? (batch transpilation?)
- Clear terminal between rebuilds?

## Success Criteria

**CLI Integration Complete When**:
- ✅ `whitehall build` transpiles all .wh files successfully
- ✅ Output directory mirrors input structure
- ✅ Generated Kotlin compiles without errors
- ✅ Can build and run a simple app end-to-end
- ✅ Reasonable error messages on transpilation failure

**Developer Experience Complete When**:
- ✅ Errors show file:line:col location
- ✅ Watch mode rebuilds on file change
- ✅ Compilation time displayed
- ✅ Source snippets shown for errors
- ✅ Color-coded terminal output

**Production Ready When**:
- ✅ Real-world apps (100+ components) transpile successfully
- ✅ Performance acceptable (<1s for typical project)
- ✅ Error messages are helpful and actionable
- ✅ Documentation exists for all features
- ✅ Example apps demonstrate patterns

---

**The transpiler core is complete and rock-solid. Time to make it usable!** 🚀
