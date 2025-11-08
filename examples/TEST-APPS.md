# Whitehall Test Apps - Complexity Series

Progressive test suite to validate Whitehall transpiler against idiomatic syntax from LANGUAGE-REFERENCE.md

## Overview

| # | App | Complexity | Transpilation | Build | Key Features Tested |
|---|-----|------------|---------------|-------|---------------------|
| 1 | Button Counter | Beginner | ✅ | ❌ | Basic state, events, control flow |
| 2 | Task List | Intermediate | ❌ | ❌ | Data classes, arrays, @for loops |
| 3 | User Profile | Advanced | ❌ | ❌ | ViewModels, sealed classes, suspend |

## Test Results Summary

### Example 1: Button Counter ✅❌
**Directory:** `examples/1-button-counter/`

**Status:** Transpiles but Kotlin compilation fails

**Bugs Found:**
1. ❌ Padding shortcut `p={24}` not transformed (generates invalid `p = 24` parameter)
2. ❌ Lambda onClick syntax broken (wraps and immediately invokes: `{ {...}() }`)

**Syntax Tested:**
- ✅ Simple state (`var count = 0`)
- ✅ Text styling (`fontSize`, `fontWeight`, `color`)
- ✅ Hex colors (`#FF5722`)
- ✅ Theme colors (`color="primary"`)
- ✅ @if/@else if/@else control flow
- ✅ Spacer component
- ❌ Padding shortcuts (`p={24}`)
- ❌ Lambda event handlers (`onClick={() => ...}`)

**See:** `examples/1-button-counter/BUILD-ERROR.md`

---

### Example 2: Task List ❌❌
**Directory:** `examples/2-task-list/`

**Status:** Transpilation fails - doesn't reach Kotlin compilation

**Bugs Found:**
1. ❌ Parser rejects Kotlin data classes before component markup

**Error:**
```
error: [Line 4:1] Expected component, found: "data class Task(..."
```

**Root Cause:** Parser doesn't allow pure Kotlin code (data classes, sealed classes, etc.) before components, violating core language promise: "Any valid Kotlin code is valid Whitehall code"

**Syntax That Would Be Tested:**
- ❌ Data classes (blocked)
- ❌ Array literals `[...]`
- ❌ Data binding (`bind:value`, `bind:checked`)
- ❌ @for with key
- ❌ @for with empty block
- ❌ Derived state
- ❌ LazyColumn
- ❌ Modifier.weight()
- ❌ Complex state mutations

**See:** `examples/2-task-list/BUILD-ERROR.md`

---

### Example 3: User Profile ❌❌
**Directory:** `examples/3-user-profile/`

**Status:** Transpilation fails - doesn't reach Kotlin compilation

**Bugs Found:**
1. ❌ Parser rejects Kotlin imports
2. ❌ Parser rejects data classes (same as Example 2)
3. ❌ Parser rejects sealed classes
4. ❌ Parser rejects class definitions (breaks ViewModel generation feature)

**Error:**
```
error: [Line 6:1] Expected component, found: "data class UserProfile(..."
```

**Root Cause:** Same parser limitation as Example 2, but affects more advanced Kotlin features

**Critical Impact:** ViewModel auto-generation feature (LANGUAGE-REFERENCE.md lines 272-290) is completely unusable because classes can't be defined before components.

**Syntax That Would Be Tested:**
- ❌ Kotlin imports (blocked)
- ❌ Data classes (blocked)
- ❌ Sealed classes with generics (blocked)
- ❌ Class-based ViewModels (blocked)
- ❌ Suspend functions (blocked)
- ❌ onMount lifecycle hook (blocked)
- ❌ @when pattern matching (blocked)
- ❌ io { } dispatcher (blocked)
- ❌ launch { } coroutines (blocked)

**See:** `examples/3-user-profile/BUILD-ERROR.md`

---

## Critical Bugs Summary

### Bug #1: Padding Shortcuts Not Working
**Severity:** HIGH
**Component:** Prop transformer
**Example:** 1

Padding shortcuts like `p={24}`, `px={20}`, `py={8}` don't transform to `Modifier.padding()`. Generated code has invalid parameters.

**Fix Needed:** Transform padding props to Modifier in prop transformer.

---

### Bug #2: Lambda onClick Syntax Broken
**Severity:** CRITICAL
**Component:** Lambda transformer
**Example:** 1

Arrow function syntax `onClick={() => expr}` generates invalid `onClick = { {expr}() }` that immediately invokes.

**Fix Needed:** Map `() => {...}` to `{ ... }` without extra wrapping/invocation.

---

### Bug #3: Parser Rejects Kotlin Code Before Components
**Severity:** CRITICAL
**Component:** Parser core
**Examples:** 2, 3

Parser expects component markup immediately and rejects:
- Kotlin imports
- Data classes
- Sealed classes
- Regular classes
- Type aliases
- Extension functions

**Why Critical:** Violates core language promise "Any valid Kotlin code is valid Whitehall code" (LANGUAGE-REFERENCE.md line 8)

**Fix Needed:** Complete parser refactor to:
1. Allow arbitrary Kotlin code before components
2. Two-phase parsing (Kotlin extraction + Whitehall transformation)
3. Smart mode switching between Kotlin and Whitehall syntax

---

## Testing These Apps

```bash
# Test transpilation only
cargo run -- compile examples/1-button-counter/main.wh
cargo run -- compile examples/2-task-list/main.wh
cargo run -- compile examples/3-user-profile/main.wh

# Test full build (transpile + Kotlin compile + APK)
cargo run -- build examples/1-button-counter/main.wh
cargo run -- build examples/2-task-list/main.wh
cargo run -- build examples/3-user-profile/main.wh
```

---

## Next Steps

1. **Fix Bug #3 first** - This is blocking Examples 2 & 3 completely
   - Refactor parser to accept Kotlin code before components
   - This enables all Kotlin superset features

2. **Fix Bug #2** - Lambda syntax is breaking all event handlers
   - Fix arrow function transformation
   - Critical for interactive apps

3. **Fix Bug #1** - Padding shortcuts
   - Add prop transformation for `p`, `px`, `py`, etc.
   - Important for ergonomic syntax

Once these bugs are fixed, all three examples should build and run successfully.
