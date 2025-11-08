# Build Error Report: Example 9 - Theming

## Build Status
✅ **Transpilation Successful**
❌ **Kotlin Compilation Failed**

## Commands Tested
```bash
cargo run -- compile main.wh  # ✅ Succeeds with issues
cargo run -- build main.wh     # ❌ Fails at Kotlin compilation
```

## Errors Found

### Error 1: val declarations placed inside @Composable instead of top-level
**Lines:** 22-41 in main.wh

**Whitehall Code:**
```whitehall
val LightColors = AppColors(
  primary = Color(0xFF6200EE),
  secondary = Color(0xFF03DAC6),
  // ...
)

val DarkColors = AppColors(
  primary = Color(0xFFBB86FC),
  secondary = Color(0xFF03DAC6),
  // ...
)

var isDarkMode = false
var currentColors = LightColors
```

**Generated (incorrect):**
```kotlin
@Composable
fun Main() {
    var isDarkMode by remember { mutableStateOf(false) }
    var currentColors by remember { mutableStateOf(LightColors) }  // ❌ References LightColors before declaration

    val LightColors = AppColors(  // ❌ Should be top-level, not local
      primary = Color(0xFF6200EE),
      // ...
    )
    val DarkColors = AppColors(  // ❌ Should be top-level, not local
      primary = Color(0xFFBB86FC),
      // ...
    )
}
```

**Expected:**
```kotlin
val LightColors = AppColors(
  primary = Color(0xFF6200EE),
  // ...
)

val DarkColors = AppColors(
  primary = Color(0xFFBB86FC),
  // ...
)

@Composable
fun Main() {
    var isDarkMode by remember { mutableStateOf(false) }
    var currentColors by remember { mutableStateOf(LightColors) }
    // ...
}
```

**Issue:** `val` declarations are being placed inside the @Composable function instead of at package level
**Severity:** CRITICAL - Creates initialization order issues

**Root Cause:** Parser likely treating all top-level declarations (`val`, `var`, `fun`, `data class`) the same way, but `val` declarations should be hoisted outside the @Composable function.

---

### Error 2: Custom component function not transpiled
**Lines:** 144-157 in main.wh

**Whitehall Code:**
```whitehall
// Helper component for color boxes
fun ColorBox(color: Color, label: String) {
  <Column>
    <Box
      width={80}
      height={80}
      backgroundColor={color}
      modifier={Modifier.border(1.dp, Color.Gray)}
    />
    <Text fontSize={12}>{label}</Text>
  </Column>
}
```

**Generated:**
```kotlin
// NOT GENERATED AT ALL - function is missing
```

**Expected:**
```kotlin
@Composable
fun ColorBox(color: Color, label: String) {
    Column {
        Box(
            modifier = Modifier
                .size(80.dp)
                .background(color)
                .border(1.dp, Color.Gray)
        )
        Text(
            text = label,
            fontSize = 12.sp
        )
    }
}
```

**Issue:** Functions defined after the main component markup are not being transpiled
**Severity:** CRITICAL - Makes ColorBox calls fail

**Root Cause:** Parser/transpiler stops after processing the main component markup, doesn't continue to parse functions defined after.

---

### Error 3: backgroundColor with variable reference broken
**Lines:** 63, 73, 85 in main.wh

**Whitehall Code:**
```whitehall
<Card p={16} backgroundColor={currentColors.primary}>
```

**Generated (incorrect):**
```kotlin
Card(
    colors = CardDefaults.cardColors(
        containerColor = MaterialTheme.colorScheme.currentColors.primary
    )
)
```

**Expected:**
```kotlin
Card(
    colors = CardDefaults.cardColors(
        containerColor = currentColors.primary
    )
)
```

**Issue:** backgroundColor with variable reference gets incorrectly prefixed with `MaterialTheme.colorScheme.`
**Severity:** HIGH - Invalid Kotlin code

**Root Cause:** backgroundColor transformation assumes color scheme property names, doesn't detect variable references.

---

### Error 4: Box component not implemented
**Lines:** 147-152 in main.wh

**Whitehall Code:**
```whitehall
<Box
  width={80}
  height={80}
  backgroundColor={color}
  modifier={Modifier.border(1.dp, Color.Gray)}
/>
```

**Issue:** Box component may not be implemented with width/height props
**Severity:** HIGH - Common component not available

**Note:** Compose's Box uses `modifier = Modifier.size(width.dp, height.dp)` instead of separate width/height props.

---

### Error 5: ternary operator in conditionals
**Line:** 59 in main.wh

**Whitehall Code:**
```whitehall
<Button text={isDarkMode ? "Light Mode" : "Dark Mode"} />
```

**Generated:**
```kotlin
Text(text = if (isDarkMode) "Light Mode" else "Dark Mode")
```

**Issue:** Actually works! Ternary operators are correctly transformed to if/else expressions.
**Severity:** NONE - Working as expected ✅

---

### Error 6: backgroundColor on Column
**Line:** 49 in main.wh

**Whitehall Code:**
```whitehall
<Column p={20} spacing={16} backgroundColor={currentColors.background}>
```

**Generated:**
```kotlin
Column(
    modifier = Modifier
        .background(currentColors.background)
        .padding(20.dp),
    verticalArrangement = Arrangement.spacedBy(16.dp)
) {
```

**Issue:** Actually works! backgroundColor on Column correctly uses Modifier.background()
**Severity:** NONE - Working as expected ✅

---

## Whitehall Syntax Tested

✅ Data classes before components
✅ val declarations (but placed in wrong scope)
✅ Color(0xFF...) hex syntax
✅ Custom data types in state
✅ Ternary operators in props (transforms to if/else)
✅ MaterialTheme wrapper component
✅ backgroundColor on Column (transforms to Modifier.background)
❌ val declarations at package level
❌ Functions defined after main component
❌ backgroundColor with variable reference (not color name)
❌ Box component

---

## Fixes Needed

### Fix #1: val declarations at package level
**Priority:** CRITICAL
**Effort:** 2-3 hours

**Problem:** All top-level declarations (`var`, `val`, `fun`, `data class`) are being collected, but `val` declarations should be placed at package level, not inside the @Composable function.

**Current Behavior:**
- `var` → state variables inside @Composable ✅
- `val` → local vals inside @Composable ❌
- `fun` → local functions inside @Composable ✅ (from example 5 fix)
- `data class` → package level ✅

**Needed Behavior:**
- `var` → state variables inside @Composable ✅
- `val` → package level (outside @Composable) 🔧
- `fun` → local functions inside @Composable ✅
- `data class` → package level ✅

**Solution:**
1. Detect `val` declarations in parser
2. Place them at package level (same as data classes)
3. Keep `var` declarations as state inside @Composable

### Fix #2: Functions after main component markup
**Priority:** HIGH
**Effort:** 3-4 hours

**Problem:** Parser/transpiler stops after processing the first component markup, doesn't continue parsing functions defined after.

**Solution:**
1. After parsing main component, continue parsing for function declarations
2. Transform these functions into @Composable functions
3. Place them after the Main() @Composable function

### Fix #3: backgroundColor with variable references
**Priority:** HIGH
**Effort:** 1-2 hours

**Problem:** backgroundColor transformation assumes MaterialTheme color scheme properties, doesn't handle variable references correctly.

**Current:** `backgroundColor={currentColors.primary}` → `MaterialTheme.colorScheme.currentColors.primary`
**Needed:** `backgroundColor={currentColors.primary}` → `currentColors.primary`

**Solution:**
1. Detect if backgroundColor value is a variable reference (contains `.`)
2. If so, use it directly without MaterialTheme.colorScheme prefix
3. Only use MaterialTheme.colorScheme for simple property names (primary, secondary, etc.)

### Fix #4: Box component implementation
**Priority:** MEDIUM
**Effort:** 1-2 hours

**Current:** Box not implemented or doesn't support width/height props
**Needed:**
```kotlin
Box(
    modifier = Modifier
        .size(width.dp, height.dp)
        .background(color)
)
```

Transform width/height props to Modifier.size().

---

## Potential Workaround Code

Remove helper function, inline ColorBox:
```whitehall
<Row spacing={8}>
  <Column>
    <Card width={80} height={80} backgroundColor={currentColors.primary} />
    <Text fontSize={12}>Primary</Text>
  </Column>
</Row>
```

Use MaterialTheme colors directly:
```whitehall
<Card p={16} backgroundColor="primary">
```

---

## Key Insight

This example reveals an important architectural issue: the distinction between **immutable values** (`val`) and **reactive state** (`var`):

- `var` declarations should be reactive state inside @Composable
- `val` declarations should be package-level constants (or locally scoped if inside functions)

The current implementation treats both similarly, but they have fundamentally different scoping requirements.
