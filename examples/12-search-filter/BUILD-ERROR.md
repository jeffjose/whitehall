# Build Error Report: Example 12 - Search & Filter

## Build Status
❌ **Transpilation Failed** - Parser error

## Commands Tested
```bash
cargo run -- compile main.wh  # ❌ Fails at parsing
cargo run -- build main.wh     # ❌ Fails at parsing
```

## Error Output
```
error: [Line 103:37] Expected '{', found ','
```

## Root Cause Analysis

### Error: Function call in text interpolation
**Line:** 102 in main.wh

**Whitehall Code:**
```whitehall
<Text fontSize={14} color="#666666">
  Showing {getFilteredItems().size} of {allItems.size} items
</Text>
```

**Issue:** Parser cannot handle function calls with parentheses `()` inside text interpolations `{...}`
**Severity:** CRITICAL - Parser error prevents transpilation
**Root Cause:** Text content parser expects simple expressions, not function calls

The parser is likely trying to parse `getFilteredItems()` and hitting the `.size` property access after the function call, which confuses it.

---

## Additional Issues (Not Yet Tested Due to Parser Error)

### Issue 1: val declaration at package level
**Lines:** 13-25 in main.wh

**Whitehall Code:**
```whitehall
val allItems = listOf(
  Item(1, "Laptop", "Electronics", 999.99),
  Item(2, "Mouse", "Electronics", 29.99),
  // ...
)
```

**Issue:** val declarations should be at package level, not inside @Composable
**Same as:** Example 9 issue

---

### Issue 2: List operations (filter, contains)
**Lines:** 35-48 in main.wh

**Whitehall Code:**
```whitehall
fun getFilteredItems(): List<Item> {
  var filtered = allItems

  if (!searchQuery.isEmpty()) {
    filtered = filtered.filter { it.name.lowercase().contains(searchQuery.lowercase()) }
  }

  if (selectedCategory != "All") {
    filtered = filtered.filter { it.category == selectedCategory }
  }

  filtered = filtered.filter { it.price >= minPrice && it.price <= maxPrice }

  return filtered
}
```

**Issue:** Collection operations (filter, lowercase, contains) - may need special handling
**Note:** These are standard Kotlin stdlib - should work if function is properly generated

---

### Issue 3: String.format in text interpolation
**Line:** 113 in main.wh

**Whitehall Code:**
```whitehall
<Text fontSize={18} fontWeight="bold" color="#4CAF50">
  ${String.format("%.2f", item.price)}
</Text>
```

**Issue:** String.format is a static method call - may not parse correctly
**Alternative:** Use Kotlin string formatting `"%.2f".format(item.price)` or `"${"%.2f".format(item.price)}"`

---

### Issue 4: FilterChip component
**Lines:** 63-68 in main.wh

**Whitehall Code:**
```whitehall
<FilterChip
  selected={selectedCategory == category}
  onClick={() => selectedCategory = category}
  label={<Text>{category}</Text>}
/>
```

**Issue:** FilterChip component may not be implemented
**Also:** label prop with component value needs special handling (same as Tab)

---

### Issue 5: TextField leadingIcon prop
**Line:** 54 in main.wh

**Whitehall Code:**
```whitehall
<TextField
  value={searchQuery}
  onValueChange={(value) => searchQuery = value}
  label="Search items..."
  leadingIcon={<Icon imageVector={Icons.Default.Search} />}
/>
```

**Issue:** leadingIcon prop with component value - needs lambda wrapper
**Similar to:** OutlinedTextField label issue in example 11

---

### Issue 6: Double type in data class
**Lines:** 8-12 in main.wh

**Whitehall Code:**
```whitehall
data class Item(
  val id: Int,
  val name: String,
  val category: String,
  val price: Double
)
```

**Issue:** Double type - should work but needs testing

---

### Issue 7: Column flex prop
**Lines:** 76-80 in main.wh

**Whitehall Code:**
```whitehall
<Column flex={1}>
  <TextField ... />
</Column>
```

**Issue:** flex prop may not be implemented on Column
**Alternative:** Use `modifier={Modifier.weight(1f)}` in parent Row

---

### Issue 8: Icon imageVector with Icons.Default.Search
**Line:** 54 in main.wh

**Issue:** Icons.Default.Search needs import detection
**Similar to:** Examples 5 and 8

---

### Issue 9: getFilteredItems() function call in multiple places
**Lines:** 102, 107, 117 in main.wh

**Issue:** Function calls in text, @for loops, and @if conditions
**Note:** Should work once parser handles function calls in expressions

---

## Whitehall Syntax Tested (Before Failure)

✅ Data class with multiple fields
✅ val declaration with listOf
✅ Double type
❌ Function call in text interpolation
❌ FilterChip component
❌ TextField leadingIcon prop
❌ Complex filtering logic
❌ String.format
❌ flex prop on Column

---

## Fixes Needed

### Fix #1: Parse function calls in text interpolations
**Priority:** CRITICAL
**Effort:** 2-3 hours

**Problem:** Text content parser cannot handle `{functionName().property}`

**Current:**
```whitehall
<Text>Items: {getFilteredItems().size}</Text>
```

Fails to parse.

**Solution:**
1. Enhance text content expression parser to handle:
   - Function calls with parentheses: `func()`
   - Property access on results: `.property`
   - Method chaining: `func().method().property`

2. Allow arbitrary Kotlin expressions in `{...}` interpolations

### Fix #2: val declarations at package level
**Priority:** HIGH
**Effort:** 3-4 hours
**Same as:** Example 9 issue

### Fix #3: FilterChip component
**Priority:** MEDIUM
**Effort:** 1-2 hours

Add FilterChip component with special handling for `label` prop (composable content).

### Fix #4: TextField/OutlinedTextField label and leadingIcon
**Priority:** HIGH
**Effort:** 1 hour

Both `label` and `leadingIcon` props accept composable lambdas:
- `label: (() -> Unit)?`
- `leadingIcon: (() -> Unit)?`

Need to wrap values appropriately.

### Fix #5: Column/Row flex prop
**Priority:** LOW
**Effort:** 1 hour

Add `flex` shortcut that transforms to `Modifier.weight(value.toFloat())` when inside Row/Column.

---

## Potential Workaround Code

Store filtered items in a var to avoid function call in text:
```whitehall
var filteredItems = getFilteredItems()

<Text>Showing {filteredItems.size} items</Text>
```

Use simple string formatting:
```whitehall
<Text>$${item.price}</Text>
```

Use string label for TextField:
```whitehall
<TextField
  label="Search items..."
  // Remove leadingIcon
/>
```

Inline FilterChip content:
```whitehall
@for (category in categories) {
  <Card
    onClick={() => selectedCategory = category}
    p={8}
    backgroundColor={selectedCategory == category ? "primary" : "surface"}
  >
    <Text>{category}</Text>
  </Card>
}
```

---

## Key Insights

1. **Function calls in text**: The text content parser is more limited than the prop expression parser. It needs to support the same expression complexity.

2. **Computed values**: Examples frequently need to call functions to get filtered/computed values for display. This is a common pattern that should be supported.

3. **Material3 composable props**: Many Material3 components have props that accept composable content (`() -> Unit`). Need a systematic way to detect and handle these.
