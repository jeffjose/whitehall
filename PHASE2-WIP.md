# Phase 2 Work In Progress

## Status: Dispatcher Syntax Implementation (95% Complete)

### ✅ Completed
1. **Dispatcher transformation function** - `transform_dispatchers()` added
   - Transforms `io { }` → `viewModelScope.launch(Dispatchers.IO) { }`
   - Transforms `cpu { }` → `viewModelScope.launch(Dispatchers.Default) { }`
   - Transforms `main { }` → `viewModelScope.launch(Dispatchers.Main) { }`

2. **Integration with prop transformation** - Called in `transform_prop()`
   - Properly ordered: route aliases → lambda arrow → dispatchers

3. **Dynamic import detection** - Adds `kotlinx.coroutines.Dispatchers` import when needed
   - Checks generated output for "Dispatchers." usage
   - Inserts import automatically

### ❌ Current Issue: Compilation Error

**Error:** `cannot borrow '*self' as mutable, as it is behind a '&' reference`

**Root Cause:** The function call chain needs to be made mutable:
- `generate()` calls `generate_markup()`
- `generate_markup()` calls `generate_markup_with_context()`
- `generate_markup_with_context()` calls `transform_prop()`
- All these need `&mut self` because we're tracking state

**Solution:** Make the entire chain take `&mut self`:
1. `fn generate(&mut self, ...)`
2. `fn generate_markup(&mut self, ...)`
3. `fn generate_markup_with_context(&mut self, ...)`
4. Already done: `fn transform_prop(&self, ...)` (doesn't actually mutate)

**Alternative (if above doesn't work):**
- Keep output-based detection but use a different approach
- Instead of checking in generate(), check in the calling code (transpile_with_registry)
- Post-process the output to add imports after generation

### 📝 Files Modified
- `src/transpiler/codegen/compose.rs`:
  - Added `transform_dispatchers()` function (line ~2190)
  - Modified `transform_prop()` to call transform_dispatchers (line ~1985)
  - Added output checking for Dispatchers import (line ~617)

### 🧪 Testing Needed Once Fixed
1. Create test component with `io { }` syntax
2. Verify transformation to `viewModelScope.launch(Dispatchers.IO) { }`
3. Verify Dispatchers import is added
4. Test all three dispatchers: io, cpu, main

### 📋 Remaining Phase 2 Tasks
1. Fix compilation error (above)
2. Test dispatcher syntax
3. Implement `$scope()` pattern detection
4. Implement `rememberCoroutineScope()` generation
5. Handle `myScope.launch { }` syntax
6. Comprehensive testing

### 💡 Notes
- Suspend function auto-wrap is working perfectly ✅
- Singleton suspend functions preserve `suspend` keyword ✅
- Foundation is solid, just needs the mutable reference fix
