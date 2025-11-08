# Build Error Report: Example 1 - Button Counter

## Build Status
✅ **Transpilation Successful**
❌ **Kotlin Compilation Failed**

## Commands Tested
```bash
cargo run -- compile main.wh  # ✅ Succeeds with bugs
cargo run -- build main.wh     # ❌ Fails at Kotlin compilation
```

## Error Output
```
Finished transpilation for `Button Counter`
e: file:///...MainActivity.kt:38:9 Cannot find a parameter with this name: p

FAILURE: Build failed with an exception.
Execution failed for task ':app:compileDebugKotlin'.
Compilation error. See log for more details

BUILD FAILED in 3s
error: Gradle build failed
```

## Generated Code Issues

### Bug 1: Invalid `p` prop in Column
**Location:** Line 24 of generated code

**Generated:**
```kotlin
Column(
    p = 24,  // ❌ Invalid - 'p' is not a valid Column parameter
    verticalArrangement = Arrangement.spacedBy(16.dp)
)
```

**Expected:**
```kotlin
Column(
    modifier = Modifier.padding(24.dp),
    verticalArrangement = Arrangement.spacedBy(16.dp)
)
```

**Root Cause:** Padding shortcut `p={24}` not being transformed to `modifier = Modifier.padding(24.dp)` for Column component

**Severity:** HIGH - Generated code won't compile in Kotlin/Compose

---

### Bug 2: Invalid onClick lambda wrapping
**Location:** Lines 38-42, 45-48, 51-55 of generated code

**Generated:**
```kotlin
Button(
    onClick = { {
      count++
      showMessage = count >= 10
    }() }  // ❌ Wraps lambda and immediately invokes - returns Unit
)
```

**Expected:**
```kotlin
Button(
    onClick = {
      count++
      showMessage = count >= 10
    }
)
```

**Root Cause:** Lambda expression `onClick={() => {...}}` being incorrectly transformed to `onClick = { {...}() }` instead of `onClick = { ... }`

**Severity:** CRITICAL - Generated code won't work correctly (immediately invokes lambda at composition time)

---

## Whitehall Syntax Tested
✅ Simple state variables (`var count = 0`)
✅ Multiple state variables
✅ @if/@else if/@else control flow
❌ Padding shortcuts (`p={24}`)
✅ Text styling (`fontSize`, `fontWeight`, `color`)
✅ Hex colors (`#FF5722`)
✅ Theme colors (`color="primary"`)
✅ Spacer component
❌ Lambda onClick syntax (`onClick={() => ...}`)

## Potential Fixes Needed

1. **Transform padding shortcuts in prop transformer**
   - Detect `p`, `px`, `py`, `pt`, `pb`, `pl`, `pr` props
   - Convert to appropriate `Modifier.padding()` syntax
   - Add to component's `modifier` parameter

2. **Fix lambda expression transformation**
   - Arrow function syntax `() => expr` should map to `{ expr }`
   - Don't wrap in extra braces and immediate invocation
   - Handle multi-line lambda bodies correctly
