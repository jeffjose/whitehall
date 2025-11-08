# Build Error Report: Example 13 - Bottom Sheet & Modals

## Build Status
❌ **Transpilation Failed** - Parser error

## Commands Tested
```bash
cargo run -- compile main.wh  # ❌ Fails at parsing
cargo run -- build main.wh     # ❌ Fails at parsing
```

## Error Output
```
error: [Line 9:7] Expected component, found: "(ExperimentalMaterial3Api::class)\n\nvar showBottomS"
```

## Root Cause Analysis

### Error: @OptIn annotation not supported
**Line:** 13 in main.wh

**Whitehall Code:**
```whitehall
import androidx.compose.material3.ExperimentalMaterial3Api

@OptIn(ExperimentalMaterial3Api::class)

var showBottomSheet = false
```

**Issue:** Parser doesn't support @OptIn annotation (or any standalone annotations)
**Severity:** CRITICAL - Parser error prevents transpilation
**Root Cause:** Parser expects component after `@`, treating @OptIn as a directive like @if/@for

**Context:** Material3 ModalBottomSheet is marked as @ExperimentalMaterial3Api, requiring opt-in annotation.

---

## Additional Issues (Not Yet Tested Due to Parser Error)

### Issue 1: val declaration with listOf
**Line:** 20 in main.wh

**Whitehall Code:**
```whitehall
val options = listOf("Option 1", "Option 2", "Option 3", "Option 4")
```

**Issue:** val at package level (same as examples 9 and 12)
**Severity:** HIGH

---

### Issue 2: ModalBottomSheet component
**Lines:** 47-54 in main.wh

**Whitehall Code:**
```whitehall
<ModalBottomSheet
  onDismissRequest={() => closeBottomSheet()}
>
  @if (sheetContent == "info") {
    <BottomSheetInfoContent />
  } else if (sheetContent == "form") {
    <BottomSheetFormContent />
  }
</ModalBottomSheet>
```

**Issue:** ModalBottomSheet component may not be implemented
**Also:** Contains @if/@else conditionals as direct children
**Severity:** HIGH

---

### Issue 3: rememberModalBottomSheetState function
**Line:** 10 in main.wh

**Whitehall Code:**
```whitehall
import androidx.compose.material3.rememberModalBottomSheetState
```

**Issue:** Not used in the code (was planning to use for state management)
**Note:** Can be removed

---

### Issue 4: Helper functions after main component
**Lines:** 68-112 in main.wh

**Whitehall Code:**
```whitehall
fun BottomSheetInfoContent() {
  <Column p={24} spacing={16}>
    <Text fontSize={24} fontWeight="bold">Information</Text>
    // ...
  </Column>
}

fun BottomSheetFormContent() {
  var name = ""
  var email = ""
  // ...
}

fun BottomSheetActionContent() {
  // ...
}
```

**Issue:** Same as examples 9, 10 - functions after main component not transpiled
**Severity:** CRITICAL

---

### Issue 5: Local state in helper functions
**Lines:** 86-87 in BottomSheetFormContent

**Whitehall Code:**
```whitehall
fun BottomSheetFormContent() {
  var name = ""
  var email = ""
  // ...
}
```

**Issue:** Local state in helper function - needs to be lifted or use remember
**Severity:** MEDIUM - State won't be reactive
**Note:** Should use `var name by remember { mutableStateOf("") }` or lift state

---

### Issue 6: Button with flex prop
**Lines:** 102-108 in main.wh

**Whitehall Code:**
```whitehall
<Button
  text="Cancel"
  onClick={() => closeBottomSheet()}
  flex={1}
/>
```

**Issue:** flex prop on Button (same as Column flex issue in example 12)
**Severity:** LOW

---

## Whitehall Syntax Tested (Before Failure)

✅ Import experimental API
✅ val declaration with listOf
❌ @OptIn annotation
❌ ModalBottomSheet component
❌ Conditional content in bottom sheet
❌ Helper functions after main component
❌ Local state in helper functions

---

## Fixes Needed

### Fix #1: Support @OptIn annotation
**Priority:** HIGH
**Effort:** 2-3 hours

**Problem:** Parser treats `@` as start of directive (@if/@for) or component (@Composable functions)

**Options:**

**Option 1 - Parse and pass through:**
```kotlin
// Detect @OptIn at file level
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun App() {
    // ...
}
```

**Option 2 - Auto-inject for experimental APIs:**
Detect usage of experimental components (ModalBottomSheet) and auto-add @OptIn

**Option 3 - Remove requirement:**
Document that experimental APIs should be stable or provide workaround

### Fix #2: ModalBottomSheet component
**Priority:** HIGH
**Effort:** 2-3 hours

Add ModalBottomSheet component with:
- `onDismissRequest` prop handling
- Content lambda support
- Proper Material3 imports

### Fix #3: Helper functions after main
**Priority:** CRITICAL
**Effort:** 4-6 hours
**Same as:** Examples 9, 10

This is a recurring issue. Need architectural solution to parse and transpile multiple @Composable functions.

### Fix #4: Local state in functions
**Priority:** MEDIUM
**Effort:** 1-2 hours

**Problem:** Local vars in helper functions aren't reactive

**Solution Options:**
1. Auto-transform local vars to `remember { mutableStateOf() }`
2. Require explicit remember syntax
3. Document best practices for state management

---

## Potential Workaround Code

Remove @OptIn and use stable APIs only:
```whitehall
// Don't use ModalBottomSheet - use Dialog instead
<Dialog
  onDismissRequest={() => closeBottomSheet()}
>
  // Content
</Dialog>
```

Inline helper components to avoid function-after-component issue:
```whitehall
@if (showBottomSheet) {
  <ModalBottomSheet onDismissRequest={() => closeBottomSheet()}>
    @if (sheetContent == "info") {
      <Column p={24} spacing={16}>
        <Text fontSize={24} fontWeight="bold">Information</Text>
        // ... inline content
      </Column>
    }
  </ModalBottomSheet>
}
```

Use lifted state:
```whitehall
var formName = ""
var formEmail = ""

// In render:
<TextField value={formName} onValueChange={(v) => formName = v} />
```

---

## Key Insights

1. **Experimental APIs**: Material3 has many @ExperimentalMaterial3Api components. Need strategy for handling opt-in requirements.

2. **Annotations**: Whitehall needs to support Kotlin annotations beyond @Composable. Common ones:
   - @OptIn
   - @Preview
   - @Stable
   - @Immutable

3. **Component architecture**: Bottom sheets, dialogs, and modals are common UI patterns that need proper support.

4. **Function organization**: The "helper functions after main component" issue keeps appearing. This needs a systematic fix.

5. **State scoping**: Clear guidelines needed for where state should live:
   - Global (file level vars)
   - Component level (App function vars)
   - Helper function level (local remember)
