# Build Error Report: Example 5 - Navigation

## Build Status
✅ **Transpilation Successful**
❌ **Kotlin Compilation Failed**

## Commands Tested
```bash
cargo run -- compile main.wh  # ✅ Succeeds
cargo run -- build main.wh     # ❌ Fails at Kotlin compilation
```

## Errors Found

### Error 1: Icons.Default.ArrowBack not imported
**Lines:** 71, 94 in main.wh
```whitehall
<Icon imageVector={Icons.Default.ArrowBack} />
```

**Issue:** `Icons.Default.ArrowBack` used but import not detected
**Severity:** MEDIUM - Missing import

**Root Cause:** Icon detection already works for `Icons.Default.Delete` but ArrowBack is a different icon name. May need wildcard import or icon name detection.

---

### Error 2: Functions generated outside Composable scope
**Lines:** 24-43 in main.wh
```whitehall
fun navigateToHome() {
  currentRoute = "home"
  selectedUser = null
}

fun navigateToUsers() {
  currentRoute = "users"
}
```

**Generated:** Functions are placed AFTER the @Composable function closes

**Issue:** Functions reference state variables but are generated in wrong scope
**Severity:** CRITICAL - Functions can't access state variables

**Expected:** Functions should be inside the @Composable function body

---

### Error 3: Card onClick prop not supported
**Line:** 82 in main.wh
```whitehall
<Card p={12} onClick={() => navigateToUserDetail(user)}>
```

**Issue:** Card component doesn't have onClick prop, needs clickable modifier
**Severity:** MEDIUM - Prop transformation missing

**Expected Transform:**
```kotlin
Card(
    modifier = Modifier
        .padding(12.dp)
        .clickable { navigateToUserDetail(user) }
)
```

---

## Whitehall Syntax Tested

✅ Data classes before components
✅ Nullable types (User?)
✅ Multiple var declarations
✅ Array literals
✅ Multiple functions
✅ Complex nested @if/@else if logic
❌ Icons.Default.ArrowBack import
❌ Functions accessing state variables
❌ onClick on Card component

---

## Fixes Needed

### Fix #1: Icons import detection improvement
**Priority:** LOW
**Effort:** 30 minutes

Current implementation detects `Icons.Default.*` but may need to handle more icon names or use wildcard import.

### Fix #2: Function scope issue
**Priority:** CRITICAL
**Effort:** 2-3 hours

**Problem:** Functions are generated after the @Composable closes, so they can't access state variables.

**Solution:** Functions should be generated INSIDE the @Composable function body, not after it.

Current structure:
```kotlin
@Composable
fun Main() {
    var currentRoute by remember { ... }
}

fun navigateToHome() {  // ❌ Can't access currentRoute
    currentRoute = "home"
}
```

Expected structure:
```kotlin
@Composable
fun Main() {
    var currentRoute by remember { ... }

    fun navigateToHome() {  // ✅ Can access currentRoute
        currentRoute = "home"
    }
}
```

### Fix #3: onClick on Card component
**Priority:** MEDIUM
**Effort:** 1 hour

Add onClick prop support to Card component by transforming to:
```kotlin
modifier = Modifier.clickable { onClick() }
```

---

## Potential Workaround Code

Replace Card with onClick:
```whitehall
<Card p={12} modifier={Modifier.clickable { navigateToUserDetail(user) }}>
  ...
</Card>
```

But this won't work either because functions are out of scope!
