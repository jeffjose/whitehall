# Known Gaps and Limitations

Issues discovered during Pokemon app example development.

## Status Summary

**✅ Fixed (1):**
- Top-level Kotlin imports

**❌ Blocking Issues (3):**
- Private/public class-level fields with initialization
- Data classes outside main class
- Hex colors incorrectly transpiled

**⚠️ Needs Investigation (1):**
- Box width/height not properly handled

## Parser Limitations

### 1. Top-level Kotlin imports ~~not supported~~ ✅ FIXED

**Issue:** ~~Parser expects Whitehall component/class, can't parse raw Kotlin before class definition~~

**Example:**
```kotlin
import kotlinx.serialization.Serializable  // ✅ Now works!
import okhttp3.OkHttpClient                // ✅ Now works!

class MyStore {
    // ...
}
```

**Current State:** ✅ **FIXED** - Top-level imports now supported and pass through to generated Kotlin
**Fixed:** 2025-11-07
**Details:** Parser already supported import statements, but codegen was not including user imports in generated ViewModel/Store classes. Now all user imports are included and alphabetically sorted in the output.

~~**Location:** `src/stores/PokemonStore.wh:1-3`~~
~~**Error:** `Expected component, found: "import kotlinx.serialization"`~~
~~**Workaround:** None - must move imports inside Kotlin blocks~~

---

### 2. Private/public class-level fields with initialization

**Issue:** Parser can't handle `private val` or `private var` fields at class level with complex initialization

**Example:**
```kotlin
class MyStore {
    private val client = OkHttpClient()  // ❌ Parser error
    private val json = Json { }          // ❌ Parser error

    var data = []  // ✅ Works
}
```

**Current State:** ❌ Not supported
**Location:** `src/stores/PokemonStore.wh:7-8`
**Error:** `[Line 7:5] Unexpected content in class body`

**Workaround:** Initialize inside functions instead:
```kotlin
suspend fun loadData() {
    val client = OkHttpClient()  // ✅ Works in function
    val json = Json { }
}
```

---

### 3. Data classes outside main class

**Issue:** Parser can't parse `data class` definitions that appear after the main class

**Example:**
```kotlin
class MyStore {
    var items: List<Item> = []
}

@Serializable           // ❌ Parser error
data class Item(        // ❌ Parser error
    val id: Int,
    val name: String
)
```

**Current State:** ❌ Not supported
**Location:** `src/stores/PokemonStore.wh:58-107`
**Error:** `Expected component, found: "data class PokemonListResponse"`

**Workaround:** None - need separate Kotlin files for data classes

---

## Transpiler Bugs

### 1. Hex colors incorrectly transpiled

**Issue:** Hex colors in component props generate invalid Kotlin

**Example:**
```whitehall
<Box backgroundColor="#f0f0f0">  // ❌ Transpiles incorrectly
```

**Transpiled (incorrect):**
```kotlin
Box(modifier = Modifier.background(Color.#f0f0f0))  // Invalid: Color.#...
```

**Expected:**
```kotlin
Box(modifier = Modifier.background(Color(0xFFf0f0f0)))
```

**Current State:** ❌ Bug in transpiler
**Location:** `src/components/PokemonCard.wh:14,18`, `src/screens/PokemonDetailScreen.wh`
**Impact:** Build failure - invalid Kotlin syntax

**Code Location:** `src/transpiler/codegen/compose.rs` - hex color conversion in prop transformations

---

### 2. Box width/height not properly handled

**Issue:** Box component with `width`/`height` props doesn't generate correct modifier

**Example:**
```whitehall
<Box width={48} height={48}>  // May not transpile correctly
```

**Expected:**
```kotlin
Box(modifier = Modifier.size(48.dp, 48.dp))
```

**Current State:** ⚠️ Needs verification
**Location:** `src/components/PokemonCard.wh:11-14`

---

## Missing Features

### 1. Kotlin interop for complex types

**Needed for:** Data classes, sealed classes, annotations outside main class

**Use case:** JSON serialization models, API response types

**Impact:** Can't define complex domain models in same file as ViewModel

**Workaround:** Would need separate `.kt` files for models

**Note:** Top-level imports are now supported (fixed 2025-11-07), but top-level data class definitions after the main class are still not supported (see Parser Limitations #3).

---

### 2. ~~Import statements for external libraries~~ ✅ FIXED

~~**Needed for:** OkHttp, Kotlinx Serialization, other libraries~~

~~**Use case:** Network calls, JSON parsing~~

~~**Impact:** Can't use external libraries without workarounds~~

**Status:** ✅ **FIXED** (2025-11-07) - Import statements now fully supported and pass through to generated code.

---

## Priority

**High Priority (blocks real apps):**
1. Hex color transpilation bug
2. Top-level data classes
3. Private val fields with initialization

**Medium Priority:**
4. ~~Top-level imports (Kotlin interop)~~ ✅ FIXED
5. Box width/height transformation

**Low Priority:**
- Better error messages for unsupported syntax
- Suggestions for workarounds

---

## Test Case

See `examples/pokemon-app/` for real-world example hitting remaining issues.

**Current Status:**
- ✅ Import statements now work
- ❌ Still blocked by: private class fields, data classes outside main class, hex color bug

**To reproduce remaining issues:**
```bash
cd examples/pokemon-app
cargo run --manifest-path ../../Cargo.toml -- compile src/stores/PokemonStore.wh
# Error: [Line 12:5] Unexpected content in class body (private val field)
```

---

*Last Updated: 2025-11-07*
*Import support added: 2025-11-07*
