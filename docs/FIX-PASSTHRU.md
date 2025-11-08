# Fix Pass-Through Function Handling

**Status:** 🔴 Planning
**Created:** 2025-11-08
**Issue:** Pass-through tests failing for extension functions and top-level functions after store class

---

## Problem Statement

Pass-through architecture is partially working but fails for files containing:
1. A store class (e.g., `class UserStore`)
2. Followed by top-level functions or extension functions

**Current Behavior:**
```
Error: [Line 23:1] Expected component, found: "fun User.displayName(): String = \"$name <$email>\""
```

**Expected Behavior:**
Functions after store class should pass through unchanged as Kotlin blocks.

**Failing Tests:**
- `04-typealias-and-helpers.md` - Extension function `fun User.displayName()`
- `05-mixed-constructs.md` - Generic extension `fun <T> ApiResult<T>.isSuccess()`
- `09-kotlin-edge-cases.md` - SAM constructor `fun createFilter()`
- `10-real-world-patterns.md` - Extension function `fun User.toDisplayString()`

**Passing Tests:**
- ✅ `01-data-class.md` - data classes work
- ✅ `02-sealed-class.md` - sealed classes work
- ✅ `03-enum-class.md` - enum classes work
- ✅ `06-string-literals.md` - string literals work
- ✅ `07-comments.md` - comments work
- ✅ `08-complex-nested.md` - complex nesting works

---

## Root Cause Analysis

### Parser Flow for Failing Cases

File structure:
```kotlin
class UserStore { ... }  // Line 1: Parsed as store class

typealias UserId = Int   // Line 2: Pass-through ✅

data class User(...)     // Line 3: Pass-through ✅

fun User.displayName()   // Line 4: FAILS ❌
```

### Execution Trace

**Phase 1: Parse loop (lines 61-141 in parser.rs)**

1. **Line 86-91**: `class UserStore`
   - Parses as store class
   - Sets `parsed_store_class = true`
   - ✅ Correct

2. **Line 94**: `typealias UserId`
   - `is_kotlin_syntax()` → TRUE
   - Captured as kotlin block
   - ✅ Correct

3. **Line 94**: `data class User`
   - `is_kotlin_syntax()` → TRUE
   - Captured as kotlin block
   - ✅ Correct

4. **Line 94**: `fun User.displayName()`
   - `is_kotlin_syntax()` checks:
     - `starts_with("inline fun ")` → FALSE
     - `starts_with("infix fun ")` → FALSE
     - `starts_with("operator fun ")` → FALSE
     - `starts_with("fun interface ")` → FALSE
     - **NO CHECK for plain `fun `**
   - Returns FALSE ❌

5. **Line 122-124**: `!parsed_store_class && self.consume_word("fun")`
   - `parsed_store_class` is TRUE
   - Condition evaluates to FALSE
   - **Doesn't parse or consume** ❌

6. **Line 138**: No conditions matched
   - **Breaks out of loop** ❌

7. **Lines 148-149**: `parse_markup()` called
   - Expects `<Component>`
   - Finds `fun User.displayName()`
   - **Errors: "Expected component"** ❌

### The Missing Logic

Comment at line 123 says:
```rust
// Parse functions only before store class, after that they pass through
```

**But there's no code to actually implement the "pass through" part!**

The logic gap:
- ❌ `is_kotlin_syntax()` doesn't recognize plain `fun `
- ❌ Line 122-124 has guard `!parsed_store_class` preventing execution
- ❌ No fallback to capture as kotlin block
- ❌ Falls through to `parse_markup()` which expects `<`

---

## Solution: Context-Aware `is_kotlin_syntax()`

### Design Principles

1. **Separation of Concerns:**
   - Before store class: parse component functions (transform to ViewModel)
   - After store class: pass through all functions unchanged

2. **Centralized Logic:**
   - Single function determines what is Kotlin syntax
   - Context (before/after store) affects the decision

3. **Backward Compatibility:**
   - Existing passing tests must continue to pass
   - No behavior change for component files without stores

### High-Level Approach

Make `is_kotlin_syntax()` context-aware by passing `parsed_store_class` flag:

**Before:**
```rust
fn is_kotlin_syntax(&self) -> bool
```

**After:**
```rust
fn is_kotlin_syntax(&self, after_store_class: bool) -> bool
```

### Behavior Matrix

| Syntax Type | Before Store | After Store | After Markup |
|-------------|--------------|-------------|--------------|
| `data class` | Pass-through | Pass-through | Pass-through |
| `sealed class` | Pass-through | Pass-through | Pass-through |
| `enum class` | Pass-through | Pass-through | Pass-through |
| `typealias` | Pass-through | Pass-through | Pass-through |
| `object` | Pass-through | Pass-through | Pass-through |
| `inline fun` | Pass-through | Pass-through | Pass-through |
| `infix fun` | Pass-through | Pass-through | Pass-through |
| `operator fun` | Pass-through | Pass-through | Pass-through |
| `fun interface` | Pass-through | Pass-through | Pass-through |
| `fun` (plain) | Parse as component fn | **Pass-through** | Pass-through |
| `suspend fun` | Parse as component fn | **Pass-through** | Pass-through |
| `val Type.prop` | Pass-through | Pass-through | Pass-through |

**Key Change:** Plain `fun` and `suspend fun` behavior depends on context.

---

## Implementation Plan

### Phase 1: Update Function Signature

**File:** `src/transpiler/parser.rs`

**Step 1.1: Modify `is_kotlin_syntax()` signature**

Location: Line 1803

```rust
// Before:
fn is_kotlin_syntax(&self) -> bool {
    let remaining = &self.input[self.pos..];
    // ...
}

// After:
fn is_kotlin_syntax(&self, after_store_class: bool) -> bool {
    let remaining = &self.input[self.pos..];
    // ...
}
```

### Phase 2: Update Function Logic

**Step 2.1: Add plain `fun` check with context**

Location: After line 1819 (before extension property check)

```rust
fn is_kotlin_syntax(&self, after_store_class: bool) -> bool {
    let remaining = &self.input[self.pos..];

    // Check for Kotlin keywords that we don't explicitly parse
    if remaining.starts_with("data class ") ||
       remaining.starts_with("sealed class ") ||
       remaining.starts_with("sealed interface ") ||
       remaining.starts_with("enum class ") ||
       remaining.starts_with("class ") ||
       remaining.starts_with("typealias ") ||
       remaining.starts_with("object ") ||
       // Specialized function modifiers (always pass through)
       remaining.starts_with("inline fun ") ||
       remaining.starts_with("infix fun ") ||
       remaining.starts_with("operator fun ") ||
       // Fun interfaces (SAM interfaces)
       remaining.starts_with("fun interface ") {
        return true;
    }

    // ✨ NEW: Plain functions - context-dependent behavior
    if after_store_class {
        // After store class, ALL functions pass through
        if remaining.starts_with("fun ") {
            return true;
        }
        // Also handle suspend functions after store class
        if remaining.starts_with("suspend fun ") {
            return true;
        }
    }

    // Extension properties: val Type.property
    if remaining.starts_with("val ") {
        let after_val = &remaining[4..];
        for (_i, ch) in after_val.char_indices() {
            if ch == '.' {
                return true;
            } else if ch == ':' || ch == '\n' || ch == '=' {
                break;
            }
        }
    }

    false
}
```

**Step 2.2: Document the context-dependent behavior**

Add comment block at function start:

```rust
/// Check if the current position is at Kotlin syntax that should pass through unchanged.
///
/// # Context-Aware Behavior
///
/// The `after_store_class` parameter affects how plain functions are handled:
/// - **Before store class:** `fun` declarations are parsed as component functions (transformed to ViewModel methods)
/// - **After store class:** `fun` declarations pass through unchanged (helpers, extensions, top-level functions)
/// - **After markup:** `fun` declarations always pass through unchanged
///
/// # Always Pass-Through (regardless of context):
/// - Data classes, sealed classes, enum classes
/// - Type aliases, objects, interfaces
/// - Specialized function modifiers (inline, infix, operator)
/// - Fun interfaces (SAM)
/// - Extension properties
fn is_kotlin_syntax(&self, after_store_class: bool) -> bool {
    // ... implementation ...
}
```

### Phase 3: Update Call Sites

**Call Site 1: Pre-markup loop**

Location: Line 94

```rust
// Before:
} else if self.is_kotlin_syntax() {

// After:
} else if self.is_kotlin_syntax(parsed_store_class) {
```

**Call Site 2: Post-markup loop**

Location: Line 171

```rust
// Before:
if self.is_kotlin_syntax() {

// After:
if self.is_kotlin_syntax(true) {  // After markup, always pass through
```

**Call Site 3: Add clarifying comment**

At line 171, add comment:

```rust
// After markup, all Kotlin syntax passes through (including plain functions)
// Context is always "after store" because we're past the component definition
if self.is_kotlin_syntax(true) {
```

### Phase 4: Remove Redundant Guards

**Step 4.1: Simplify `suspend fun` handling**

Location: Lines 114-121

**Before:**
```rust
} else if !parsed_store_class && self.peek_word() == Some("suspend") {
    // Handle suspend fun (only before store class, after that they pass through)
    self.consume_word("suspend");
    self.skip_whitespace();
    if !self.consume_word("fun") {
        return Err(self.error_at_pos("Expected 'fun' after 'suspend'"));
    }
    functions.push(self.parse_function_declaration(true)?);
```

**After:**
```rust
} else if !parsed_store_class && self.peek_word() == Some("suspend") {
    // Parse suspend functions as component functions (before store class only)
    // After store class, is_kotlin_syntax() will catch these and pass through
    self.consume_word("suspend");
    self.skip_whitespace();
    if !self.consume_word("fun") {
        return Err(self.error_at_pos("Expected 'fun' after 'suspend'"));
    }
    functions.push(self.parse_function_declaration(true)?);
```

**Analysis:** Keep this unchanged - the guard is correct. After store class, `suspend fun` will be caught by `is_kotlin_syntax(true)` at line 94.

**Step 4.2: Clarify `fun` handling**

Location: Lines 122-124

**Before:**
```rust
} else if !parsed_store_class && self.consume_word("fun") {
    // Parse functions only before store class, after that they pass through
    functions.push(self.parse_function_declaration(false)?);
```

**After:**
```rust
} else if !parsed_store_class && self.consume_word("fun") {
    // Parse plain functions as component functions (before store class only)
    // After store class, is_kotlin_syntax() will catch these and pass through
    functions.push(self.parse_function_declaration(false)?);
```

**Analysis:** Keep this unchanged - the guard is correct. After store class, `fun` will be caught by `is_kotlin_syntax(true)` at line 94.

---

## Testing Strategy

### Phase 1: Existing Tests Must Pass

Run transpiler tests to ensure no regression:

```bash
cargo test --test transpiler_examples_test -- --nocapture
```

**Expected:** All 40/40 tests PASS ✅

### Phase 2: Pass-Through Tests Should Now Pass

Run pass-through tests:

```bash
cargo test --test passthru_examples_test -- --nocapture
```

**Expected Before Fix:**
- ❌ 04-typealias-and-helpers.md
- ❌ 05-mixed-constructs.md
- ❌ 09-kotlin-edge-cases.md
- ❌ 10-real-world-patterns.md
- ✅ 6 other tests pass

**Expected After Fix:**
- ✅ All 10/10 tests PASS

### Phase 3: Run Full Test Suite

```bash
./scripts/test-examples.sh
```

**Expected:**
- ✅ Transpiler examples: PASSED (40/40)
- ✅ Pass-through examples: PASSED (10/10)
- ✅ Optimization examples: PASSED (2/2)

### Phase 4: Edge Case Testing

Create manual test cases:

**Test Case 1: Component file without store (no regression)**

File: `test-component-no-store.wh`
```whitehall
@prop val name: String

fun greet(): String {
    return "Hello, $name"
}

<Text>Hello</Text>
```

**Expected:** Function `greet()` is parsed as component function (transformed to ViewModel method).

---

**Test Case 2: Store file with functions after store**

File: `test-store-with-helpers.wh`
```whitehall
class UserStore {
    var users: List<User> = []
}

data class User(val id: Int, val name: String)

fun User.displayName(): String = name

fun createDefaultUser() = User(0, "Guest")
```

**Expected:**
- `data class User` passes through ✅
- `fun User.displayName()` passes through ✅
- `fun createDefaultUser()` passes through ✅

---

**Test Case 3: Multiple stores (edge case)**

File: `test-multiple-stores.wh`
```whitehall
class FirstStore {
    var count: Int = 0
}

class SecondStore {
    var name: String = ""
}

fun helper() = "test"
```

**Expected:** Second store and helper function pass through (only first class is parsed as store).

---

**Test Case 4: Suspend function after store**

File: `test-suspend-after-store.wh`
```whitehall
class ApiStore {
    var data: String = ""
}

suspend fun fetchData(): String {
    return "data"
}
```

**Expected:** `suspend fun fetchData()` passes through unchanged.

---

**Test Case 5: Mixed specialized and plain functions**

File: `test-mixed-functions.wh`
```whitehall
class Store {
    var items: List<Int> = []
}

inline fun <reified T> cast(value: Any): T? = value as? T

fun regularHelper() = 42

infix fun Int.times(other: Int) = this * other
```

**Expected:** All three functions pass through.

---

## Edge Cases & Considerations

### Edge Case 1: Generic Functions

```kotlin
fun <T> ApiResult<T>.isSuccess(): Boolean = this is ApiResult.Success
```

**Handling:** The check `remaining.starts_with("fun ")` will match.
**Status:** ✅ Will work correctly.

### Edge Case 2: Extension Functions

```kotlin
fun User.displayName(): String = "$name"
```

**Handling:** The check `remaining.starts_with("fun ")` will match.
**Status:** ✅ Will work correctly.

### Edge Case 3: Function with Receiver

```kotlin
fun String.Companion.empty() = ""
```

**Handling:** The check `remaining.starts_with("fun ")` will match.
**Status:** ✅ Will work correctly.

### Edge Case 4: Multi-line Function Signature

```kotlin
fun createUser(
    id: Int,
    name: String
): User = User(id, name)
```

**Handling:** `capture_kotlin_block()` handles balanced braces and multi-line declarations.
**Status:** ✅ Will work correctly.

### Edge Case 5: Function Inside String/Comment

```kotlin
val code = "fun fake() = 42"  // String literal
/* fun commented() = 0 */      // Comment
```

**Handling:** `capture_kotlin_block()` has string/comment tracking logic (lines 1911-2193).
**Status:** ✅ Already handled correctly.

### Edge Case 6: Anonymous Functions / Lambdas

```kotlin
val lambda = fun(x: Int) = x * 2
```

**Handling:** This is a variable declaration starting with `val`, not `fun`.
**Status:** ✅ Will be caught by state declaration parsing (line 111-113) before store class, or as extension property check after store class.

### Edge Case 7: Empty Store File

```kotlin
class EmptyStore {
}
```

**Handling:** Parses as store class, `parsed_store_class = true`, then EOF.
**Status:** ✅ Will work correctly.

### Edge Case 8: Only Functions (No Store)

```whitehall
fun helper1() = 1
fun helper2() = 2

<Text>Hello</Text>
```

**Handling:**
- `parsed_store_class` is FALSE
- Line 122-124 will parse `fun helper1()` as component function
- Line 122-124 will parse `fun helper2()` as component function
- Then parses `<Text>` as markup

**Status:** ✅ Correct behavior - component functions are transformed.

---

## Implementation Checklist

### Pre-Implementation

- [x] Document problem in FIX-PASSTHRU.md
- [x] Update test script to treat pass-through tests as required (not "expected to fail")
- [ ] Review plan with team/self
- [ ] Ensure all existing tests pass baseline

### Implementation

- [ ] **Step 1:** Update `is_kotlin_syntax()` signature
  - [ ] Add `after_store_class: bool` parameter
  - [ ] Add documentation comment

- [ ] **Step 2:** Update `is_kotlin_syntax()` logic
  - [ ] Add `after_store_class` condition for `fun `
  - [ ] Add `after_store_class` condition for `suspend fun `
  - [ ] Test the function in isolation (unit test if possible)

- [ ] **Step 3:** Update call site 1 (line 94)
  - [ ] Pass `parsed_store_class` flag
  - [ ] Add clarifying comment

- [ ] **Step 4:** Update call site 2 (line 171)
  - [ ] Pass `true` (always after store in post-markup)
  - [ ] Add clarifying comment

- [ ] **Step 5:** Update comments
  - [ ] Update comment at line 115
  - [ ] Update comment at line 123

### Testing

- [ ] **Phase 1:** Run transpiler tests
  - [ ] All 40/40 pass
  - [ ] No regressions

- [ ] **Phase 2:** Run pass-through tests
  - [ ] 04-typealias-and-helpers.md passes
  - [ ] 05-mixed-constructs.md passes
  - [ ] 09-kotlin-edge-cases.md passes
  - [ ] 10-real-world-patterns.md passes
  - [ ] All 10/10 pass

- [ ] **Phase 3:** Run full test suite
  - [ ] `./scripts/test-examples.sh` succeeds

- [ ] **Phase 4:** Manual edge case testing
  - [ ] Test component without store (no regression)
  - [ ] Test store with functions after
  - [ ] Test suspend functions after store
  - [ ] Test mixed function types

### Verification

- [ ] Review diff of changes
- [ ] Ensure comments are accurate
- [ ] No dead code introduced
- [ ] No unintended side effects

### Documentation

- [ ] Update PASSTHRU.md with "FIXED" status
- [ ] Update LANGUAGE-REFERENCE.md if needed
- [ ] Update REF-TRANSPILER.md if needed
- [ ] Close this FIX-PASSTHRU.md issue

### Commit

- [ ] Stage changes: `git add src/transpiler/parser.rs`
- [ ] Commit with message:
  ```
  Fix pass-through handling for functions after store class

  Make is_kotlin_syntax() context-aware to distinguish between:
  - Component functions (before store) - parsed and transformed
  - Helper functions (after store) - passed through unchanged

  Fixes pass-through tests:
  - 04-typealias-and-helpers.md
  - 05-mixed-constructs.md
  - 09-kotlin-edge-cases.md
  - 10-real-world-patterns.md

  All transpiler tests (40/40) and pass-through tests (10/10) now pass.
  ```

**Note:** Test script (scripts/test-examples.sh) has already been updated to treat pass-through tests as required (commit 0d504d4).

---

## Rollback Plan

If the fix causes issues:

### Step 1: Revert the commit

```bash
git revert HEAD
```

### Step 2: Investigate failure

- Identify which test failed
- Determine if it's a test issue or implementation issue
- Add specific test case for the failure

### Step 3: Re-implement with fix

- Address the specific failure
- Re-test
- Re-commit

---

## Success Criteria

✅ **All conditions must be met:**

1. All existing transpiler tests pass (40/40)
2. All pass-through tests pass (10/10)
3. All optimization tests pass (2/2)
4. Manual edge case tests pass
5. No behavioral regression for component files without stores
6. Code is well-commented and maintainable
7. Solution is architecturally sound (context-aware design)

---

## Post-Implementation

### Update Documentation

1. **PASSTHRU.md**
   - Update status from "🔴 Expected to fail" to "✅ FIXED"
   - Add fix date: 2025-11-08
   - Add reference to this document

2. **REF-TRANSPILER.md**
   - Add section on context-aware pass-through
   - Document the behavior matrix

3. **LANGUAGE-REFERENCE.md**
   - Clarify that functions before store are component functions
   - Clarify that functions after store pass through

### Code Cleanup

1. Check for any TODO/FIXME comments related to pass-through
2. Remove any dead code if applicable
3. Ensure consistent comment style

### Future Improvements

Ideas for future enhancements (NOT part of this fix):

1. **Better error messages**
   - When a function appears before store but has syntax that suggests pass-through intent
   - Suggest moving it after the store class

2. **LSP support**
   - Syntax highlighting for pass-through blocks
   - Hover hints showing "This will pass through unchanged"

3. **Validation**
   - Warn if pass-through block contains Whitehall syntax (likely user error)
   - Suggest using component syntax instead

---

## Timeline Estimate

- **Implementation:** 30-60 minutes
- **Testing:** 30 minutes
- **Documentation:** 15 minutes
- **Total:** 1.5-2 hours

---

## Risk Assessment

**Risk Level:** 🟢 LOW

**Risks:**
1. **Behavioral change for edge cases** - UNLIKELY
   - Mitigation: Comprehensive test suite
   - Impact: Would be caught in testing phase

2. **Performance regression** - VERY UNLIKELY
   - Additional parameter is negligible overhead
   - No algorithmic complexity change

3. **Breaking existing code** - UNLIKELY
   - Change is additive (adds context awareness)
   - Existing behavior preserved for component files
   - Mitigation: All existing tests must pass

**Confidence Level:** HIGH ✅

---

## Notes

- This fix addresses the root cause identified in parser flow analysis
- The solution is architecturally sound and follows the existing pattern
- Context-aware behavior is explicit and well-documented
- All edge cases are considered and handled
- Rollback plan is straightforward
- Success criteria are clear and measurable

---

## Questions & Clarifications

### Q: Why not just add `fun ` to `is_kotlin_syntax()` without context?

**A:** That would break component function parsing for files without stores. Component functions need to be transformed to ViewModel methods.

### Q: Why pass `true` at line 171 (post-markup)?

**A:** After markup, we're in the "trailing declarations" section. All functions here are helpers/extensions that should pass through unchanged, regardless of whether there was a store class or not.

### Q: What about `suspend` functions?

**A:** They follow the same pattern as `fun`:
- Before store: Parsed as component functions (can be async operations in ViewModel)
- After store: Pass through unchanged (helper coroutines)

### Q: Can we unit test `is_kotlin_syntax()`?

**A:** Yes, but it requires access to the Parser struct. Consider adding:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_kotlin_syntax_plain_fun() {
        let input = "fun helper() = 42";
        let parser = Parser::new(input);

        assert_eq!(parser.is_kotlin_syntax(false), false); // Before store
        assert_eq!(parser.is_kotlin_syntax(true), true);   // After store
    }
}
```

This could be added as a future enhancement but is not required for this fix.

---

**Document Version:** 1.0
**Last Updated:** 2025-11-08
**Status:** Ready for Implementation
