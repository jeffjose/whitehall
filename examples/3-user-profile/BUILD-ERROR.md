# Build Error Report: Example 3 - User Profile

## Build Status
❌ **Transpilation Failed** - Stops before Kotlin compilation

## Commands Tested
```bash
cargo run -- compile main.wh  # ❌ Fails at transpilation
cargo run -- build main.wh     # ❌ Fails at transpilation
```

## Error Output
```
error: [Line 6:1] Expected component, found: "data class UserProfile(\n  val username: String,\n  "
```

**Fails at:** Line 11 in main.wh (after frontmatter and comments)

## Root Cause Analysis

### Bug 1: Parser rejects Kotlin imports

**Location:** Line 9 in main.wh

**Code that fails:**
```whitehall
import kotlinx.coroutines.delay
```

**Root Cause:** Parser doesn't recognize Kotlin import statements before component markup.

**Severity:** CRITICAL

---

### Bug 2: Parser rejects data classes (same as Example 2)

**Location:** Line 11 in main.wh

**Code that fails:**
```whitehall
data class UserProfile(
  val username: String,
  val bio: String,
  val avatarUrl: String,
  val followers: Int,
  val following: Int
)
```

**Root Cause:** Same as Example 2 - Parser expects components, not Kotlin declarations.

**Severity:** CRITICAL

---

### Bug 3: Parser would reject sealed classes

**Location:** Line 19 in main.wh (not reached)

**Code that would fail:**
```whitehall
sealed class LoadingState<out T> {
  object Idle : LoadingState<Nothing>()
  object Loading : LoadingState<Nothing>()
  data class Success<T>(val data: T) : LoadingState<T>()
  data class Error(val message: String) : LoadingState<Nothing>()
}
```

**Root Cause:** Same parser limitation - sealed classes are pure Kotlin.

**Severity:** CRITICAL

---

### Bug 4: Parser would reject class definitions for ViewModels

**Location:** Line 26 in main.wh (not reached)

**Code that would fail:**
```whitehall
class ProfileStore {
  var loadingState: LoadingState<UserProfile> = LoadingState.Idle
  var editMode = false

  suspend fun loadProfile(username: String) {
    // ...
  }
}
```

**Root Cause:** ViewModel classes need to be defined before component markup, but parser doesn't allow this.

**According to LANGUAGE-REFERENCE.md line 272-290:** Classes with `var` should auto-generate ViewModels, but this requires parser to accept class definitions.

**Severity:** CRITICAL - Core ViewModel feature unusable

---

## Whitehall Syntax That Would Be Tested (if transpilation worked)

This example tests advanced features:
- ✅ Kotlin imports
- ✅ Data classes
- ✅ Sealed classes with generics
- ✅ Class-based ViewModel generation (auto-detect pattern)
- ✅ Suspend functions in classes
- ✅ `onMount` lifecycle hook
- ✅ `@when` expression with pattern matching
- ✅ `io { }` coroutine dispatcher
- ✅ `launch { }` for coroutines
- ✅ Complex state management patterns
- ✅ Type casting in templates
- ✅ Getters with custom logic (`val profile: UserProfile? get() = ...`)
- ✅ Multiple component composition
- ✅ Nested state and derived values

---

## Language Reference Violations

This example follows LANGUAGE-REFERENCE.md exactly but fails because:

1. **Line 8:** "✅ Any valid Kotlin code is valid Whitehall code"
   - **Violated:** Parser rejects valid Kotlin (imports, data classes, sealed classes, regular classes)

2. **Lines 26-50:** Shows mixing Kotlin and Whitehall freely
   - **Violated:** Can't mix them - Kotlin must come after components

3. **Lines 272-290:** ViewModel generation from classes
   - **Violated:** Can't define classes before components

---

## Potential Fixes

**Parser needs major refactor:**

1. **Two-phase parsing:**
   - Phase 1: Extract and preserve all Kotlin code (imports, classes, functions, etc.)
   - Phase 2: Parse Whitehall-specific syntax (components, @if/@for/@when, bindings)

2. **Kotlin-aware tokenization:**
   - Detect Kotlin language constructs and pass through verbatim
   - Only transform Whitehall-specific syntax

3. **Smart mode switching:**
   - Start in "Kotlin mode" - pass everything through
   - Enter "Whitehall mode" when encountering:
     - Component tags: `<Text>`, `<Column>`, etc.
     - Control flow: `@if`, `@for`, `@when`
     - Lifecycle: `onMount`, `onDispose`
   - Allow switching back to Kotlin mode inside component blocks

**Alternative approach:**
- Use a real Kotlin parser (like kotlinc or IntelliJ's parser)
- Only intercept and transform Whitehall-specific extensions
- Preserve all other Kotlin code exactly as-is
