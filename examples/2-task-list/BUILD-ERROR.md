# Build Error Report: Example 2 - Task List

## Build Status
❌ **Transpilation Failed** - Stops before Kotlin compilation

## Commands Tested
```bash
cargo run -- compile main.wh  # ❌ Fails at transpilation
cargo run -- build main.wh     # ❌ Fails at transpilation
```

## Error Output
```
error: [Line 4:1] Expected component, found: "data class Task(\n  val id: Int,\n  val title: Strin"
```

**Fails at:** Line 9 in main.wh

## Root Cause Analysis

### Bug: Parser rejects Kotlin data classes before component markup

**Location:** Line 4 in main.wh

**Code that fails:**
```whitehall
// Example 2: Task List - Intermediate Syntax
// Tests: data binding, lists with @for, arrays, data classes, derived state, form validation

data class Task(
  val id: Int,
  val title: String,
  val completed: Boolean = false
)

var tasks = [
  Task(1, "Learn Whitehall syntax", false),
  // ...
]
```

**Root Cause:** Parser expects component markup immediately and doesn't handle top-level Kotlin declarations (data classes, sealed classes, imports, etc.) before components.

**Why this is wrong:** According to LANGUAGE-REFERENCE.md:
- Line 8: "✅ **Any valid Kotlin code is valid Whitehall code**"
- Lines 26-38: Shows data classes being used freely alongside Whitehall syntax
- Lines 27-28: "Use data classes, sealed classes, extension functions, coroutines, etc."

**Severity:** CRITICAL - Core language promise broken

---

## Related Issues

This same bug would affect:
1. Sealed classes for state management
2. Type aliases
3. Extension functions
4. Top-level constants
5. Kotlin imports (if present)

---

## Whitehall Syntax That Would Be Tested (if transpilation worked)

This example tests:
- ✅ Data classes (pure Kotlin)
- ✅ Array literals `[...]`
- ✅ Data binding (`bind:value`, `bind:checked`)
- ✅ @for with key
- ✅ @for with empty block
- ✅ Derived state (`val incompleteTasks = ...`)
- ✅ Form validation patterns
- ✅ LazyColumn
- ✅ Modifier.weight(1f)
- ✅ Complex state mutations (map, filter)
- ✅ Function definitions
- ✅ Conditional expressions in JSX-like syntax

---

## Potential Fix

**Parser needs to:**
1. Detect and skip over pure Kotlin code before component markup
2. Pass through Kotlin declarations verbatim to output
3. Only start "Whitehall mode" when encountering:
   - Component tags (`<Column>`, `<Text>`, etc.)
   - Control flow directives (`@if`, `@for`, `@when`)
   - Lifecycle hooks (`onMount`, `onDispose`)

**Implementation approach:**
- Tokenize and identify Kotlin vs Whitehall syntax
- Allow arbitrary Kotlin before first Whitehall construct
- Maintain Kotlin compatibility throughout file
