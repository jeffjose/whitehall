# Whitehall Examples Restructure Plan

## Why This Restructure?

### Core Problems with Current State

1. **Learning Path is Unclear**
   - 19 examples with no clear simple→complex progression
   - Beginners don't know where to start or what order to follow
   - Advanced users can't find "advanced" examples (no FFI!)

2. **Massive Redundancy**
   - 2 separate todo apps teaching the same CRUD pattern
   - 2 profile examples (display vs edit) that should be one
   - 2 form validation examples teaching identical concepts
   - 2 modal/dialog examples that overlap
   - Users waste time going through repetitive content

3. **Missing Critical Features**
   - **Zero FFI examples** - This is Whitehall's killer feature for performance!
   - FFI (Rust/C++) integration is what sets Whitehall apart from other frameworks
   - Not showing FFI = hiding our best differentiator

4. **Mixing Concepts in Examples**
   - "Weather app" tries to teach async + components + layouts + animations all at once
   - "Settings app" mixes forms + theming + toggles + navigation
   - Cognitive overload - learner can't tell which code is for which concept
   - **Violates "one concept at a time" teaching principle**

5. **No Clear Endpoint**
   - Examples never show "putting it all together"
   - No guidance on building a complete real-world app
   - Learners struggle to go from focused examples → full apps

### What Success Looks Like

✅ **Clear learning path**: Each tier builds on the previous
✅ **One concept per example**: No cognitive overload, laser-focused learning
✅ **No wasted time**: Every example teaches something distinct
✅ **Showcase strengths**: FFI examples demonstrate Whitehall's unique value
✅ **Clear endpoint**: Separate "complete apps" section shows integration

## Core Principle: One Concept Per Example

### ❌ Bad: Mixed Concepts
```
10-weather-app/
  - Teaches: async data + loading states + components +
             layouts + animations + error handling
  Problem: Can't tell which code is for which concept
```

### ✅ Good: Focused Teaching
```
5-async-loading/     - ONLY async data + loading states (simple fetch)
7-animations/        - ONLY animations (fade, slide, scale)
11-components/       - ONLY component composition

THEN later in examples-complete/:
weather-app/         - Combines #5, #7, #11 with comments:
                       "// Uses animations from example 7"
                       "// Uses async patterns from example 5"
```

**Why this works:**
- Learn each concept in isolation
- See it applied in real app later (with references back)
- No confusion about what code does what

## Proposed Structure

### **Tier 1: Fundamentals (1-5)**
*Core reactive concepts needed for any Whitehall app*

1. **counter**
   - **Teaches**: Basic reactive state (var, onClick, state updates)
   - **Complexity**: Single file, ~30 lines
   - **Example**: Button that increments number

2. **todo-list**
   - **Teaches**: Lists, CRUD operations (add, delete)
   - **Complexity**: Single file, ~60 lines
   - **Keeps**: List state management, basic LazyColumn
   - **Avoids**: Multi-file structure, categories, filtering (too much!)

3. **text-input**
   - **Teaches**: TextField, two-way binding (bind:value)
   - **Complexity**: Single file, ~40 lines
   - **Example**: Text input that mirrors to display

4. **form-validation**
   - **Teaches**: Validation patterns, error states, form submission
   - **Complexity**: Single/multi-file, ~80 lines
   - **Example**: Email + password form with validation rules
   - **Avoids**: Multiple forms, profile editing (separate concept)

5. **async-loading**
   - **Teaches**: Suspend functions, loading/error/success states
   - **Complexity**: Multi-file (store + component), ~100 lines
   - **Example**: Fetch simple data, show loading spinner, display results
   - **Avoids**: Complex UI, animations, multiple API calls

### **Tier 2: UI Patterns (6-12)**
*Individual UI patterns - one concept each*

6. **navigation**
   - **Teaches**: Multi-screen navigation, passing data between screens
   - **Complexity**: Multi-file, 2-3 simple screens
   - **Example**: Home → Detail → Settings (just navigation, minimal UI)

7. **animations**
   - **Teaches**: Transitions (fade in/out, slide, scale)
   - **Complexity**: Single file, ~60 lines
   - **Example**: Button that animates different transitions
   - **Avoids**: Real app context (teach mechanics first)

8. **theming**
   - **Teaches**: Dark/light mode toggle, MaterialTheme colors
   - **Complexity**: Single file, ~50 lines
   - **Example**: Toggle button that switches theme, shows colored cards
   - **Avoids**: Full settings UI (that's mixing concepts)

9. **tabs**
   - **Teaches**: TabRow, switching content between tabs
   - **Complexity**: Single file, ~70 lines
   - **Example**: 3 tabs with different content panels

10. **dialogs**
    - **Teaches**: AlertDialog, modal patterns, dismiss handling
    - **Complexity**: Single file, ~80 lines
    - **Example**: Buttons that show different dialog types (info, confirm, form)

11. **lazy-lists**
    - **Teaches**: LazyColumn with many items, performance
    - **Complexity**: Single file, ~60 lines
    - **Example**: Generate 1000 items, scroll smoothly

12. **search-filter**
    - **Teaches**: Filtering a list, search input
    - **Complexity**: Single file, ~90 lines
    - **Example**: List of items with search box to filter
    - **Avoids**: Categories, complex data (just filtering)

### **Tier 3: Architecture Patterns (13-15)**
*How to structure larger apps*

13. **viewmodel-pattern**
    - **Teaches**: ViewModel/Store class, separating logic from UI
    - **Complexity**: Multi-file (store + view), ~100 lines
    - **Example**: Counter but with ViewModel class (compare to #1)

14. **derived-state**
    - **Teaches**: Computed values, reactive derivation
    - **Complexity**: Single file, ~60 lines
    - **Example**: firstName + lastName → fullName automatically updates

15. **component-composition**
    - **Teaches**: Breaking UI into reusable components
    - **Complexity**: Multi-file (multiple components), ~120 lines
    - **Example**: ProfileCard = Avatar + Name + Bio + Stats (all separate components)

### **Tier 4: Advanced - FFI (16-17)**
*High-performance native code integration*

16. **rust-ffi-math**
    - **Teaches**: Rust FFI basics, calling Rust from Whitehall
    - **Complexity**: Multi-file (rust + jni + ui), ~150 lines
    - **Example**: Simple math operations (add, multiply, fibonacci)
    - **File structure**:
      - `src/ffi/rust/src/lib.rs` - Rust functions with #[ffi]
      - `src/ffi/rust/src/jni_bridge.rs` - Auto-generated JNI
      - `src/main.wh` - UI calling Rust functions
      - `whitehall.toml` with `[ffi.rust]` config

17. **cpp-ffi-strings**
    - **Teaches**: C++ FFI, string manipulation across FFI boundary
    - **Complexity**: Multi-file (cpp + jni + ui), ~150 lines
    - **Example**: String operations (uppercase, reverse, word count)
    - **File structure**:
      - `src/ffi/cpp/string_utils.cpp` - C++ functions with @ffi
      - `src/ffi/cpp/jni_bridge.cpp` - Auto-generated JNI
      - `src/main.wh` - UI calling C++ functions
      - `whitehall.toml` with `[ffi.cpp]` config

## Complete Apps - Separate Section

### Where "Complete Apps" Go

**NOT in main examples/** - that's for focused learning

**Instead create**: `examples-complete/` or `docs/tutorials/building-complete-apps/`

### Complete App Examples

These show how to **combine** concepts from focused examples:

1. **weather-dashboard/**
   - Combines: async-loading (#5) + animations (#7) + lazy-lists (#11)
   - Shows: Real API, error handling, loading animations, forecast list
   - Comments reference back: `// Uses animation patterns from example 7`

2. **notes-app/**
   - Combines: todo-list (#2) + search-filter (#12) + viewmodel (#13)
   - Shows: Full CRUD, categories, filtering, proper architecture

3. **settings-app/**
   - Combines: theming (#8) + form-validation (#4) + tabs (#9)
   - Shows: Organized settings, theme toggle, preference forms

4. **calculator/**
   - Combines: viewmodel (#13) + component-composition (#15)
   - Shows: State machine, operation sequencing, clean architecture

5. **ecommerce-product-browser/**
   - Combines: navigation (#6) + search-filter (#12) + async-loading (#5)
   - Shows: Product list → detail screen, shopping cart, search

## Current Examples - What to Do

### Map Current → New Structure

**Keep (with focus improvements):**
- 1-button-counter → 1-counter ✅
- 6-async-data → 5-async-loading (simplify: just data fetch, remove complexity)
- 7-dialogs-snackbars → 10-dialogs
- 8-animations → 7-animations (keep standalone!)
- 9-theming → 8-theming (keep standalone!)
- 10-tabs-navigation → 9-tabs
- 12-search-filter → 12-search-filter (simplify: just filtering)

**Merge (simplify to single concept):**
- 2-task-list + 14-todo-app → 2-todo-list (simple CRUD, no multi-file complexity)
- 3-user-profile + 17-profile-settings → Move to examples-complete/
- 4-form-validation + 11-complex-forms → 4-form-validation (one good example)

**Move to examples-complete/:**
- 15-weather-dashboard → examples-complete/weather-app/
- 16-settings-app → examples-complete/settings-app/
- 18-calculator → examples-complete/calculator/
- 19-notes-app → examples-complete/notes-app/
- 3+17 merged → examples-complete/profile-editor/

**Delete (redundant or teaches same concept):**
- 5-navigation (if it's not good quality - replace with new focused one)
- 13-bottom-sheet (covered by dialogs #10)

**Create new:**
- 3-text-input (new - focused TextField example)
- 11-lazy-lists (new - focused LazyColumn with performance)
- 13-viewmodel-pattern (new - architecture pattern)
- 14-derived-state (new - computed values)
- 15-component-composition (new - reusable components)
- 16-rust-ffi-math (new - simple Rust FFI)
- 17-cpp-ffi-strings (new - simple C++ FFI)

## Implementation Plan

### Phase 1: Create New Focused Examples
*Add missing focused examples first*

1. Create `3-text-input` - TextField basics
2. Create `11-lazy-lists` - LazyColumn performance
3. Create `13-viewmodel-pattern` - ViewModel architecture
4. Create `14-derived-state` - Computed values
5. Create `15-component-composition` - Reusable components
6. Create `16-rust-ffi-math` - Simple Rust FFI
7. Create `17-cpp-ffi-strings` - Simple C++ FFI

**Commit**: "Add focused examples for core concepts"

### Phase 2: Simplify Existing Examples
*Remove complexity, focus on one concept*

1. Simplify `2-todo-list` - Just add/delete, no categories
2. Simplify `5-async-loading` - Just fetch + display, minimal UI
3. Simplify `12-search-filter` - Just search box + filtering
4. Ensure `7-animations` is standalone (not mixed with app)
5. Ensure `8-theming` is standalone (not mixed with settings)

**Commit**: "Simplify examples to teach one concept each"

### Phase 3: Move Complete Apps
*Separate focused learning from integration*

1. Create `examples-complete/` directory
2. Move 15-weather-dashboard → `examples-complete/weather-app/`
3. Move 18-calculator → `examples-complete/calculator/`
4. Move 19-notes-app → `examples-complete/notes-app/`
5. Move 16-settings-app → `examples-complete/settings-app/`
6. Create `examples-complete/profile-editor/` (merge 3+17)

Add README explaining: "These combine concepts from examples/ - study those first"

**Commit**: "Move complete apps to examples-complete/"

### Phase 4: Delete Redundant Examples
*Clean up what's now covered*

1. Delete `13-bottom-sheet` (covered by 10-dialogs)
2. Delete `14-todo-app` (merged into simplified 2-todo-list)
3. Delete `17-profile-settings` (merged into examples-complete/profile-editor/)
4. Delete old `4-form-validation` if keeping 11's version
5. Delete `3-user-profile` (merged into examples-complete/profile-editor/)

**Commit**: "Remove redundant examples"

### Phase 5: Renumber
*Create clean 1-17 sequence*

Renumber to achieve clean progression:
```
examples/
  1-counter/
  2-todo-list/
  3-text-input/
  4-form-validation/
  5-async-loading/
  6-navigation/
  7-animations/
  8-theming/
  9-tabs/
  10-dialogs/
  11-lazy-lists/
  12-search-filter/
  13-viewmodel-pattern/
  14-derived-state/
  15-component-composition/
  16-rust-ffi-math/
  17-cpp-ffi-strings/

examples-complete/
  weather-app/
  notes-app/
  calculator/
  settings-app/
  profile-editor/
```

**Commit**: "Renumber examples for clear progression"

### Phase 6: Documentation
*Guide users through the learning path*

1. Create `examples/README.md`:
   ```markdown
   # Whitehall Examples

   Learn Whitehall one concept at a time.

   ## Learning Path

   ### Tier 1: Fundamentals (1-5)
   Start here if you're new to Whitehall...

   ### Tier 2: UI Patterns (6-12)
   Learn individual UI patterns...

   ### Tier 3: Architecture (13-15)
   Structure larger applications...

   ### Tier 4: Advanced (16-17)
   High-performance FFI integration...

   ## Next Steps

   Once you've completed these examples, see `examples-complete/`
   for full applications that combine multiple concepts.
   ```

2. Create `examples-complete/README.md`:
   ```markdown
   # Complete Application Examples

   These applications combine concepts from `examples/`.
   Study the focused examples first!

   Each app includes comments like:
   "// Uses animation patterns from example 7"
   "// Uses async loading from example 5"

   ## Applications
   - weather-app/ - Combines: async-loading + animations + lists
   - notes-app/ - Combines: CRUD + filtering + viewmodel
   ...
   ```

3. Update `docs/LANGUAGE-REFERENCE.md` to reference new structure

**Commit**: "Add comprehensive examples documentation"

## Success Criteria

✅ **17 focused examples** - Each teaches ONE clear concept
✅ **No cognitive overload** - Learner knows exactly what each example teaches
✅ **Clear progression** - Tier 1 → 2 → 3 → 4
✅ **FFI coverage** - 2 examples (Rust + C++) demonstrating key differentiator
✅ **Integration shown** - examples-complete/ shows how to combine concepts
✅ **No redundancy** - Every example is distinct and necessary

## Timeline Estimate

- **Phase 1** (Create new): 4-6 hours (7 new examples)
- **Phase 2** (Simplify): 2-3 hours (modify 5 examples)
- **Phase 3** (Move complete): 1 hour (move + README)
- **Phase 4** (Delete): 30 min
- **Phase 5** (Renumber): 1 hour
- **Phase 6** (Docs): 1-2 hours

**Total**: 9-13 hours of focused work

## Open Questions

1. **FFI build requirements** - Too heavy for examples?
   - Requires NDK (~1GB), Rust toolchain
   - **Proposal**: Provide source code, make actual building optional
   - Document setup in examples-complete/ for those who want to try

2. **Single-file vs Multi-file in Tier 1?**
   - 1-3 could be single-file (simplicity)
   - 4-5 could be multi-file (showing structure)
   - **Proposal**: Use single-file until concept requires separation

3. **Should we have examples-complete/ or just docs/tutorials/?**
   - examples-complete/ = actual runnable code
   - docs/tutorials/ = step-by-step guides
   - **Proposal**: Both - tutorials reference examples-complete/ code

4. **Navigation example (#6) - keep existing or rebuild?**
   - Current example 5 exists
   - **Decision needed**: Evaluate quality, improve or replace

## Migration Guide for Users

### For Tutorial Writers

**Old example references:**
```
"See example 14-todo-app" → "See example 2-todo-list"
"See example 15-weather" → "See examples-complete/weather-app/"
```

**Mapping:**
- Examples 1-5 → Core fundamentals (some renumbered)
- Examples 6-12 → UI patterns (some new, some renumbered)
- Examples 13-15 → Architecture patterns (all new)
- Examples 16-17 → FFI (all new)
- Old "complete" examples → examples-complete/

### Breaking Changes

- Example numbers 1-19 → 1-17 (renumbered)
- Some examples moved to examples-complete/
- Some examples deleted (redundant)
- New focused examples added

### What Stays the Same

- Core concepts still taught
- Code patterns remain consistent
- Whitehall syntax unchanged
- Just better organized for learning
