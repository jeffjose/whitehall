# Build Error Report: Example 6 - Async Data Loading

## Build Status
❌ **Transpilation Failed** - Doesn't reach Kotlin compilation

## Commands Tested
```bash
cargo run -- compile main.wh  # ❌ Fails at transpilation
cargo run -- build main.wh     # ❌ Fails at transpilation
```

## Error Output
```
error: [Line 87:54] Expected identifier
```

**Fails at:** Line 87 in main.wh

## Root Cause Analysis

### Error: `Spacer` component used without braces
**Line:** 87 in main.wh
```whitehall
    <Spacer />
    <IconButton onClick={() => launch { refreshPosts() }}>
```

**Issue:** Parser error - "Expected identifier" at position after Spacer

**Possible Root Causes:**
1. Self-closing component `<Spacer />` followed by another component causing parse issue
2. `launch` keyword in onClick lambda might be triggering parser confusion
3. Nested braces in `launch { refreshPosts() }` within onClick lambda

---

## Whitehall Syntax Tested (Before Failure)

✅ Import kotlinx.coroutines.delay
✅ Data classes before components
✅ Sealed classes with generics
✅ Nullable types in suspend functions
✅ Array literals
✅ suspend functions
✅ onMount lifecycle hook
❌ Launch inside onClick lambda
❌ Self-closing Spacer followed by IconButton

---

## Fixes Needed

### Fix #1: Investigate parser error at line 87
**Priority:** HIGH
**Effort:** 1-2 hours

Need to determine exact cause:
1. Test if `<Spacer />` alone works
2. Test if `launch { }` inside onClick works
3. Test if combination causes issue

### Fix #2: launch inside onClick
**Priority:** HIGH
**Effort:** Unknown (depends on if supported)

The code attempts:
```whitehall
<IconButton onClick={() => launch { refreshPosts() }}>
```

This should transform to:
```kotlin
IconButton(
    onClick = {
        coroutineScope.launch {
            refreshPosts()
        }
    }
)
```

But `launch` needs a coroutine scope. May need special handling.

### Fix #3: suspend function calls in onClick
**Priority:** HIGH
**Effort:** 2-3 hours

The pattern `onClick={() => launch { suspendFun() }}` is common for async operations. Need to:
1. Detect `launch { }` blocks in onClick
2. Ensure coroutineScope is available
3. Transform correctly

---

## Potential Workarounds

### Workaround 1: Remove problematic line
Simplify the UI to test other features:
```whitehall
<Row spacing={8}>
  <Text fontSize={28} fontWeight="bold">
    Blog Posts
  </Text>
</Row>
```

### Workaround 2: Avoid launch in onClick
Use non-suspend logic for now:
```whitehall
<IconButton onClick={() => postsState = DataState.Loading}>
  <Icon imageVector={Icons.Default.Refresh} />
</IconButton>
```

---

## Additional Issues (Not Yet Tested)

Once transpilation succeeds, these issues may appear:

1. **Icons.Default.Refresh import** - Same as example 5
2. **CircularProgressIndicator import** - May need to be added
3. **Array literal type inference** - `[Post(...)]` may have type issues
4. **Suspend function scope** - `loadPosts()` and `refreshPosts()` need proper coroutine scope
5. **onMount with launch** - Lifecycle hook with async code may need special handling

These will be documented once transpilation succeeds.
