# Build Error Report: Example 8 - Animations

## Build Status
❌ **Transpilation Failed** - Parser error at line 22

## Commands Tested
```bash
cargo run -- compile main.wh  # ❌ Fails at parsing
cargo run -- build main.wh     # ❌ Fails at parsing
```

## Error Output
```
error: [Line 22:4] Expected identifier
```

**Fails at:** Line 22 in main.wh (but likely caused by line 21)

## Root Cause Analysis

### Error: Float literal suffix not supported
**Line:** 21 in main.wh
```whitehall
var rotationAngle = 0f
var boxSize = 100  // Line 22 - error reported here
```

**Issue:** Parser doesn't support `f` suffix for float literals (Kotlin syntax: `0f`, `90f`)
**Severity:** HIGH - Common Kotlin pattern not supported

**Expected Behavior:** Parser should accept float literals with `f` suffix (0f, 1.5f, etc.)

**Workaround:** Use explicit type annotation:
```whitehall
var rotationAngle: Float = 0.0
```

---

## Additional Issues (Not Yet Tested)

Once float literal parsing is fixed, these issues may appear:

### Issue 1: HTML-style comments
**Lines:** 27, 48, 68, 91

**Code:**
```whitehall
<!-- Animated Visibility -->
```

**Issue:** HTML-style comments `<!-- -->` may not be supported
**Workaround:** Use Kotlin comments `// Animated Visibility`

---

### Issue 2: Ternary operator in text prop
**Line:** 32

**Code:**
```whitehall
<Button text={isVisible ? "Hide" : "Show"} />
```

**Issue:** Ternary operator in props may need special handling
**Expected:** Should transform to `if (isVisible) "Hide" else "Show"`

---

### Issue 3: Function call expressions with operators in props
**Lines:** 38-39

**Code:**
```whitehall
<AnimatedVisibility
  enter={fadeIn() + slideInVertically()}
  exit={fadeOut() + slideOutVertically()}
>
```

**Issue:** Complex expressions with function calls and operators (`+`) in props
**May Need:** Special handling for animation composition

---

### Issue 4: Nested function calls in props
**Lines:** 81-84

**Code:**
```whitehall
modifier={Modifier.rotate(animateFloatAsState(
  targetValue = rotationAngle,
  animationSpec = tween(durationMillis = 500)
).value)}
```

**Issue:** Deeply nested function calls with named parameters
**May Need:** Complex prop expression parsing

---

### Issue 5: Named parameters in function calls
**Lines:** 82-83, 101-102

**Code:**
```whitehall
animateFloatAsState(
  targetValue = rotationAngle,
  animationSpec = tween(durationMillis = 500)
)
```

**Issue:** Named parameters with `=` syntax in prop expressions
**May Need:** Parser to distinguish named params from assignments

---

### Issue 6: `.dp` extension property
**Lines:** 101-102

**Code:**
```whitehall
width={animateDpAsState(targetValue = boxSize.dp).value}
```

**Issue:** Using `.dp` extension property in expressions
**Expected:** Should pass through as-is to Kotlin

---

### Issue 7: Box component not implemented
**Lines:** 77-87, 100-106

**Code:**
```whitehall
<Box width={100} height={100} backgroundColor="#FF5722">
```

**Issue:** Box component may not be implemented
**Alternative:** Use Compose's Box or replace with Card

---

### Issue 8: AnimatedVisibility component
**Lines:** 36-44

**Code:**
```whitehall
<AnimatedVisibility visible={isVisible}>
```

**Issue:** AnimatedVisibility component may need special handling
**Note:** This is a Compose component, should work if detection works

---

### Issue 9: Icons.Default.ExpandLess/ExpandMore
**Line:** 54

**Code:**
```whitehall
imageVector={isExpanded ? Icons.Default.ExpandLess : Icons.Default.ExpandMore}
```

**Issue:** Icon names may need import detection
**Similar to:** Example 5 Icons.Default.ArrowBack issue

---

### Issue 10: onClick on Row
**Line:** 51

**Code:**
```whitehall
<Row onClick={() => isExpanded = !isExpanded}>
```

**Issue:** Row doesn't have onClick prop, needs clickable modifier
**Similar to:** Example 5 onClick on Card issue

---

## Whitehall Syntax Tested (Before Failure)

✅ Import animation libraries
✅ Multiple var declarations
✅ Boolean variables
❌ Float literal with suffix (0f, 90f)
❌ HTML comments
❌ Ternary operators in props
❌ Complex expressions in props
❌ AnimatedVisibility
❌ Box component

---

## Fixes Needed

### Fix #1: Float literal parsing
**Priority:** HIGH
**Effort:** 30 minutes

Add support for float literals with `f` or `F` suffix in the lexer/parser:
- `0f` → `0f`
- `1.5f` → `1.5f`
- `90f` → `90f`

This is standard Kotlin syntax and should be supported.

### Fix #2: HTML comment handling
**Priority:** LOW
**Effort:** 1 hour

Either:
1. Parse and ignore HTML comments `<!-- -->`
2. Document that only Kotlin-style comments are supported

### Fix #3: Complex prop expressions
**Priority:** MEDIUM
**Effort:** 2-3 hours

Improve prop expression parsing to handle:
- Ternary operators
- Function calls with operators (`fadeIn() + slideInVertically()`)
- Nested function calls with named parameters
- Chained property access (`.value`, `.dp`)

---

## Potential Workaround Code

Replace float literals:
```whitehall
var rotationAngle: Float = 0.0  // Instead of 0f
```

Replace HTML comments:
```whitehall
// Animated Visibility  // Instead of <!-- Animated Visibility -->
```

Replace ternary in text:
```whitehall
<Button text={if (isVisible) "Hide" else "Show"} />
```

---

## Testing Strategy

After fixing float literal parsing:
1. Test if HTML comments work
2. Test ternary operators in props
3. Test complex animation expressions
4. Test Box component
5. Test AnimatedVisibility component
