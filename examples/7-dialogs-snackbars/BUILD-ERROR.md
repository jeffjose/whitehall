# Build Error Report: Example 7 - Dialogs & Snackbars

## Build Status
✅ **Transpilation Successful**
❌ **Gradle Build Failed** - Manifest merge error (likely due to "&" in app name)

## Commands Tested
```bash
cargo run -- compile main.wh  # ✅ Succeeds with issues
cargo run -- build main.wh     # ❌ Fails at manifest merge
```

## Errors Found

### Error 1: Manifest merge failure
**Output:**
```
Error parsing /home/jeffjose/.cache/whitehall/.../build/app/src/main/AndroidManifest.xml
```

**Issue:** App name "Dialogs & Snackbars" contains "&" which needs escaping in AndroidManifest.xml
**Severity:** LOW - Workaround is simple
**Fix:** Either escape as `&amp;` in manifest or avoid special characters in app name

---

### Error 2: Components passed as props not transformed
**Lines:** 50, 51, 66, 67, 68, 76, 77 in main.wh

**Whitehall Code:**
```whitehall
<AlertDialog
  onDismissRequest={() => showDialog = false}
  title={<Text>Information</Text>}
  text={<Text>{dialogMessage}</Text>}
  confirmButton={
    <Button
      text="OK"
      onClick={() => showDialog = false}
    />
  }
/>
```

**Generated (incorrect):**
```kotlin
AlertDialog(
    onDismissRequest = { showDialog = false },
    title = ,
    text = ,
    confirmButton =
)
```

**Expected:**
```kotlin
AlertDialog(
    onDismissRequest = { showDialog = false },
    title = { Text("Information") },
    text = { Text(dialogMessage) },
    confirmButton = {
        Button(onClick = { showDialog = false }) {
            Text("OK")
        }
    }
)
```

**Issue:** Components nested inside prop values are not being parsed/transformed
**Severity:** CRITICAL - AlertDialog completely broken without this

---

### Error 3: Hex color with MaterialTheme.colorScheme
**Line:** 41 in main.wh

**Whitehall Code:**
```whitehall
<Card p={16} backgroundColor="#E3F2FD">
```

**Generated (incorrect):**
```kotlin
Card(
    colors = CardDefaults.cardColors(
        containerColor = MaterialTheme.colorScheme.#E3F2FD
    )
)
```

**Expected:**
```kotlin
Card(
    colors = CardDefaults.cardColors(
        containerColor = Color(0xFFE3F2FD)
    )
)
```

**Issue:** Hex color string "#E3F2FD" incorrectly combined with MaterialTheme.colorScheme
**Severity:** HIGH - Invalid Kotlin syntax

---

### Error 4: Bare launch without coroutine scope
**Lines:** 14-18 in main.wh

**Whitehall Code:**
```whitehall
fun showSnackbar(message: String) {
  launch {
    snackbarHostState.showSnackbar(message)
  }
}
```

**Generated (incorrect):**
```kotlin
fun showSnackbar(message: String) {
    launch {
        snackbarHostState.showSnackbar(message)
      }
}
```

**Expected:**
```kotlin
val coroutineScope = rememberCoroutineScope()

fun showSnackbar(message: String) {
    coroutineScope.launch {
        snackbarHostState.showSnackbar(message)
    }
}
```

**Issue:** `launch` used without a CoroutineScope - needs rememberCoroutineScope()
**Severity:** CRITICAL - Won't compile

---

## Whitehall Syntax Tested

✅ AlertDialog component structure
✅ Multiple @if blocks for conditional dialogs
✅ SnackbarHost component
❌ Components as prop values (title={<Text>...</Text>})
❌ Hex colors in backgroundColor
❌ launch in functions (needs coroutine scope)

---

## Fixes Needed

### Fix #1: Manifest escaping for app names
**Priority:** LOW
**Effort:** 15 minutes

Escape special characters in app name when generating AndroidManifest.xml:
- `&` → `&amp;`
- `<` → `&lt;`
- `>` → `&gt;`

### Fix #2: Component prop values
**Priority:** CRITICAL
**Effort:** 4-6 hours

**Problem:** Props that accept composable content (like AlertDialog's title, text, confirmButton) need to transform nested components into lambdas.

**Current:** `title={<Text>Info</Text>}` generates `title = ,`
**Needed:** `title={<Text>Info</Text>}` generates `title = { Text("Info") }`

**Solution:**
1. Detect when prop value contains component markup
2. Wrap transformed component in a lambda `{ }`
3. Handle both single components and multiple children

### Fix #3: Hex color parsing
**Priority:** HIGH
**Effort:** 1 hour

**Problem:** backgroundColor="#E3F2FD" generates invalid `MaterialTheme.colorScheme.#E3F2FD`

**Solution:**
1. Detect hex color strings (starts with #)
2. Transform to `Color(0xFFE3F2FD)` format
3. Add proper Color import

### Fix #4: Coroutine scope for launch
**Priority:** HIGH
**Effort:** 2-3 hours

**Problem:** Functions using `launch` need access to a CoroutineScope

**Solution:**
1. Detect `launch` usage in functions
2. Auto-inject `val coroutineScope = rememberCoroutineScope()` in @Composable
3. Transform `launch` to `coroutineScope.launch`

Alternatively, require explicit `coroutineScope.launch` syntax in Whitehall.

---

## Potential Workaround Code

Simplify to avoid nested components in props:
```whitehall
@if (showDialog) {
  <Column p={16}>
    <Text fontSize={18} fontWeight="bold">Information</Text>
    <Text>{dialogMessage}</Text>
    <Button text="OK" onClick={() => showDialog = false} />
  </Column>
}
```

But this loses the Dialog behavior - not a good workaround.
