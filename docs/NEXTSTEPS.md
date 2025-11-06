# Next Steps for Whitehall

**Last Updated**: 2025-11-06

> **Note:** For completed features and overall roadmap, see ROADMAP.md

---

## Quick Context

**Where we are:**
- Transpiler: 38/38 tests passing (100%)
- CLI: All 9 commands working
- Toolchain: Zero-config auto-downloads
- Playground: 18 examples, multi-file support
- Phase 1.1: Auto-ViewModel generation complete

**What needs testing:**
- Real Android device/emulator workflows
- Multi-component production apps
- RecyclerView optimizations in compiled code

---

## Immediate Next Steps

### Option 1: End-to-End Testing (RECOMMENDED) 🎯
**Goal**: Verify the entire pipeline works by building a real app

**Why this matters:**
- Validates toolchain downloads work on clean machines
- Tests `whitehall run` with real devices
- Verifies generated Android projects compile with Gradle
- Finds edge cases in real usage

**Tasks**:
1. Create a simple multi-component app (Counter + List + Form)
2. Run `whitehall build` and verify Gradle compilation
3. Run `whitehall run` on physical device/emulator
4. Test `whitehall watch` with live file changes
5. Verify RecyclerView optimization in APK
6. Document any issues found

**Estimated Effort**: 2-4 hours + bug fixes

---

### Option 2: Developer Experience Improvements
**Goal**: Make Whitehall more pleasant to use

**Tasks**:
1. **Error messages with line numbers**
   - Track parser position for every token
   - Return structured errors with line/column
   - Enable inline errors in playground Monaco editor
   - **Effort**: 3-4 hours

2. **Shebang support for single files**
   ```bash
   #!/usr/bin/env whitehall run
   # counter.wh
   var count = 0
   <Button onClick={() => count++}>Count: {count}</Button>
   ```
   - Makes .wh files directly executable
   - **Effort**: 1-2 hours

3. **Source maps for debugging**
   - Map generated Kotlin back to .wh source
   - Better stack traces in Android Studio
   - **Effort**: 4-6 hours

**Estimated Effort**: 8-12 hours total

---

### Option 3: Quality of Life Features
**Goal**: Implement high-priority DX improvements from QOL.md

See `docs/QOL.md` for full list. Top priorities:

1. **Alignment shortcuts** - `align="center"` vs `horizontalAlignment="CenterHorizontally"`
2. **onClick on any component** - Auto-wrap in clickable modifier
3. **Boolean props** - `enabled` vs `enabled={true}`
4. **Divider component** - Common UI element

**Estimated Effort**: 2-3 hours per feature

---

### Option 4: Web Playground Enhancements
**Goal**: Improve playground developer experience

**Tasks**:
1. **Parser position tracking** (prerequisite for inline errors)
   - Track line/column in parser
   - Return structured error objects
   - Enable red squiggly lines in Monaco
   - **Effort**: 3-4 hours

2. **Deploy to production**
   - Backend: Fly.io (Rust-friendly)
   - Frontend: Vercel (static hosting)
   - Domain: play.whitehall.dev
   - **Effort**: 2-3 hours

3. **Visual preview (Phase 2)**
   - AST to HTML converter
   - Material3 CSS approximation
   - Preview tab (non-interactive)
   - **Effort**: 4-6 hours

**Estimated Effort**: 9-13 hours total

---

### Option 5: Community Readiness
**Goal**: Prepare for public release

**Tasks**:
1. **Documentation polish**
   - Quick start guide
   - Tutorial series (beginner → advanced)
   - API reference
   - Migration guide from Android Views/XML

2. **Example projects**
   - Counter (minimal)
   - Todo list (state management)
   - Blog reader (routing + API)
   - E-commerce (complex app)

3. **Template system**
   - `whitehall create` with interactive prompts
   - Built-in templates
   - Community template registry

4. **Website**
   - Landing page (whitehall.dev)
   - Documentation site
   - Playground link
   - GitHub integration

**Estimated Effort**: 20-30 hours

---

## Decision Framework

**If you want to:**
- ✅ **Validate everything works** → Option 1 (End-to-End Testing)
- 🎨 **Improve developer experience** → Option 2 (DX Improvements)
- 🚀 **Add popular features quickly** → Option 3 (QOL Features)
- 🌐 **Polish the playground** → Option 4 (Playground)
- 📢 **Prepare for launch** → Option 5 (Community)

**Recommended path:**
1. Option 1 (verify it works) → 2-4 hours
2. Option 2 or 3 (make it better) → 8-12 hours
3. Option 4 (showcase it) → 9-13 hours
4. Option 5 (go public) → 20-30 hours

---

## Known Gaps

**Not critical but worth noting:**

1. **Import system edge cases**
   - Circular imports not detected
   - Import path resolution could be more robust

2. **Type inference limitations**
   - Some complex expressions need manual type hints
   - Derives types from usage, not always accurate

3. **Compose compatibility**
   - Targets Compose 1.5.x
   - May need updates for Compose 2.0

4. **Performance**
   - Transpilation is fast (<100ms for small files)
   - Large projects (100+ files) not tested

5. **Windows support**
   - Toolchain management only supports Linux/macOS
   - Transpiler should work on Windows (untested)

---

## Future Phases (Long-term)

See ROADMAP.md for detailed phase breakdown:
- **Phase 2**: Advanced state management (@store enhancements)
- **Phase 3**: Build system improvements (incremental compilation)
- **Phase 4**: IDE integration (VS Code extension)
- **Phase 5**: Community ecosystem (plugins, themes)

---

**Note:** This document focuses on immediate next steps (1-2 weeks of work). For long-term vision, see ROADMAP.md and VISION.md.
