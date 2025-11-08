# Build Error Report: Example 10 - Tabs & Navigation

## Build Status
✅ **Transpilation Successful**
❌ **Kotlin Compilation Failed** - 80+ errors

## Commands Tested
```bash
cargo run -- compile main.wh  # ✅ Succeeds
cargo run -- build main.wh     # ❌ Fails at Kotlin compilation
```

## Errors Found

### Error 1: Tab text prop with component value not generated
**Lines:** 17-29 in main.wh

**Whitehall Code:**
```whitehall
<Tab
  selected={selectedTab == 0}
  onClick={() => selectTab(0)}
  text={<Text>Home</Text>}
/>
```

**Generated (incorrect):**
```kotlin
Tab(
    selected = selectedTab == 0,
    onClick = { selectTab(0) },
    text =
)
```

**Expected:**
```kotlin
Tab(
    selected = selectedTab == 0,
    onClick = { selectTab(0) },
    text = { Text("Home") }
)
```

**Issue:** Tab `text` prop with component value is not being generated at all
**Severity:** CRITICAL - Tab labels completely missing
**Root Cause:** Similar to AlertDialog issue - Tab text prop accepts composable content but PropValue::Markup is not being handled

---

### Error 2: Helper functions after main component not transpiled
**Lines:** 48-75 in main.wh

**Whitehall Code:**
```whitehall
fun HomeTab(counter: Int, onIncrement: () -> Unit) {
  <Column spacing={12}>
    <Text fontSize={24} fontWeight="bold">Home Tab</Text>
    <Text>Counter: {counter}</Text>
    <Button text="Increment" onClick={onIncrement} />
  </Column>
}
```

**Generated (incorrect):**
```kotlin
fun HomeTab(counter: Int, onIncrement: () -> Unit) {
  <Column spacing={12}>
    <Text fontSize={24} fontWeight="bold">Home Tab</Text>
    <Text>Counter: {counter}</Text>
    <Button text="Increment" onClick={onIncrement} />
  </Column>
}
```

**Expected:**
```kotlin
@Composable
fun HomeTab(counter: Int, onIncrement: () -> Unit) {
    Column(
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Text(
            text = "Home Tab",
            fontSize = 24.sp,
            fontWeight = FontWeight.Bold
        )
        Text(text = "Counter: ${counter}")
        Button(onClick = onIncrement) {
            Text("Increment")
        }
    }
}
```

**Issue:** Functions defined after the main component markup are copied as-is without transpilation
**Severity:** CRITICAL - Makes all helper components unusable
**Root Cause:** Parser/transpiler stops after first component, doesn't continue parsing functions

---

### Error 3: HorizontalDivider component not imported
**Line:** 7 in main.wh

**Whitehall Code:**
```whitehall
import androidx.compose.material3.HorizontalDivider
```

**Issue:** Import declared but component not recognized/registered
**Severity:** LOW - Can use Divider instead
**Note:** HorizontalDivider is Material3 component, may need import registration

---

### Error 4: Lambda parameter syntax in onTextChange
**Line:** 44 in main.wh

**Whitehall Code:**
```whitehall
<SettingsTab text={settingsText} onTextChange={(newText) => settingsText = newText} />
```

**Generated:**
```kotlin
onTextChange = { newText -> settingsText = newText }
```

**Issue:** Actually works! Lambda parameter transformation is working correctly ✅
**Severity:** NONE

---

### Error 5: Lambda with single parameter and increment
**Line:** 41 in main.wh

**Whitehall Code:**
```whitehall
<HomeTab counter={homeCounter} onIncrement={() => homeCounter++} />
```

**Generated:**
```kotlin
onIncrement = { homeCounter++ }
```

**Issue:** Actually works! Empty parameter list correctly generates parameterless lambda ✅
**Severity:** NONE

---

## Whitehall Syntax Tested

✅ TabRow and Tab components
✅ Tab selection state (selectedTab == 0)
✅ Lambda transformations (() => expr and (param) => expr)
✅ else if conditionals in markup
✅ Function calls in conditionals (selectTab(0))
❌ Tab text prop with component value
❌ Helper component functions after main markup
❌ HorizontalDivider component registration

---

## Fixes Needed

### Fix #1: Tab text prop with composable content
**Priority:** CRITICAL
**Effort:** 1 hour
**Related:** Similar to AlertDialog fix in example 7

**Problem:** Tab `text` prop accepts composable content but PropValue::Markup is not handled

**Solution:**
1. Add special handling for Tab component (like AlertDialog, Scaffold)
2. Detect `text` prop with PropValue::Markup
3. Wrap in lambda: `text = { Text("Home") }`

### Fix #2: Functions after main component
**Priority:** CRITICAL
**Effort:** 4-6 hours

**Problem:** Parser stops after first component, doesn't continue to parse subsequent functions

**Solution:**
1. After parsing main component, continue parsing for function declarations
2. Detect functions with markup content (have `<Component>` in body)
3. Mark these as @Composable and transpile the markup
4. Place after main App() function in generated file

This is the same issue as example 9 (ColorBox function).

### Fix #3: HorizontalDivider import
**Priority:** LOW
**Effort:** 5 minutes

Add HorizontalDivider to component import registry.

---

## Potential Workaround Code

Inline the helper components:
```whitehall
@if (selectedTab == 0) {
  <Column spacing={12}>
    <Text fontSize={24} fontWeight="bold">Home Tab</Text>
    <Text>Counter: {homeCounter}</Text>
    <Button text="Increment" onClick={() => homeCounter++} />
  </Column>
}
```

Use string for Tab text instead of component:
```whitehall
<Tab
  selected={selectedTab == 0}
  onClick={() => selectTab(0)}
  text="Home"
/>
```

However, Material3 Tab expects composable content for `text`, not a String.

---

## Key Insights

1. **Functions after markup**: This is a recurring architectural issue (also in example 9). The transpiler needs to support multiple top-level functions, not just the main component.

2. **Composable prop values**: More components need special handling for props that accept composable content (Tab.text, Scaffold.topBar, AlertDialog.title, etc.)

3. **Two-phase parsing**: May need to parse the entire file first to collect all functions, then generate code for all of them.
