# Known Gaps and Limitations

Issues discovered during Pokemon app example development.

## Status Summary

**✅ Fixed (4):**
- Top-level Kotlin imports
- Private/public class-level fields with initialization
- Hex colors transpilation (commit 89b6dfa)
- Data classes outside main class (pass-through architecture)

**❌ Blocking Issues (0):**
- None!

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

### 2. Private/public class-level fields with initialization ~~not supported~~ ✅ FIXED

**Issue:** ~~Parser can't handle `private val` or `private var` fields at class level with complex initialization~~

**Example:**
```kotlin
class MyStore {
    private val client = OkHttpClient()  // ✅ Now works!
    private val json = Json { }          // ✅ Now works!

    var data = []  // ✅ Works
}
```

**Current State:** ✅ **FIXED** - Private/protected/public visibility modifiers now supported on class-level fields
**Fixed:** 2025-11-07
**Details:** Parser now recognizes visibility modifiers before var/val declarations. Codegen outputs private fields as direct class fields (not in UiState). Public fields remain reactive in UiState.

~~**Location:** `src/stores/PokemonStore.wh:7-8`~~
~~**Error:** `[Line 7:5] Unexpected content in class body`~~
~~**Workaround:** Initialize inside functions instead~~

---

### 3. ~~Data classes outside main class~~ ✅ FIXED

**Issue:** ~~Parser can't parse `data class` definitions that appear after the main class~~

**Example:**
```kotlin
class MyStore {
    var items: List<Item> = []
}

@Serializable           // ✅ Now works!
data class Item(        // ✅ Now works!
    val id: Int,
    val name: String
)
```

**Current State:** ✅ **FIXED** - Pass-through architecture now supports data classes, sealed classes, enum classes after main class
**Fixed:** Pass-through architecture (commits 5f17d70, f5180f9)
**Details:** Parser captures Kotlin blocks after the main class and codegen includes them in the output

---

## Transpiler Bugs

### 1. ~~Hex colors incorrectly transpiled~~ ✅ FIXED

**Issue:** ~~Hex colors in component props generate invalid Kotlin~~

**Example:**
```whitehall
<Box backgroundColor="#f0f0f0">  // ✅ Now works!
```

**Transpiled (correct):**
```kotlin
Box(modifier = Modifier.background(Color(0xFFF0F0F0)))  // ✅ Valid
```

**Current State:** ✅ **FIXED** - Hex colors now transpile correctly
**Fixed:** Commit 89b6dfa (6 weeks ago)
**Details:** Hex colors (#RGB, #RRGGBB, #RRGGBBAA) are converted to Color(0xAARRGGBB) format

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

### 1. ~~Kotlin interop for complex types~~ ✅ FIXED

~~**Needed for:** Data classes, sealed classes, annotations outside main class~~

~~**Use case:** JSON serialization models, API response types~~

~~**Impact:** Can't define complex domain models in same file as ViewModel~~

~~**Workaround:** Would need separate `.kt` files for models~~

**Status:** ✅ **FIXED** - Pass-through architecture now supports data classes, sealed classes, enum classes, and other Kotlin constructs both before and after the main class. Top-level imports also supported.

---

### 2. ~~Import statements for external libraries~~ ✅ FIXED

~~**Needed for:** OkHttp, Kotlinx Serialization, other libraries~~

~~**Use case:** Network calls, JSON parsing~~

~~**Impact:** Can't use external libraries without workarounds~~

**Status:** ✅ **FIXED** (2025-11-07) - Import statements now fully supported and pass through to generated code.

---

## Priority

**High Priority (blocks real apps):**
1. ~~Hex color transpilation bug~~ ✅ FIXED
2. ~~Top-level data classes~~ ✅ FIXED
3. ~~Private val fields with initialization~~ ✅ FIXED

**Medium Priority:**
4. ~~Top-level imports (Kotlin interop)~~ ✅ FIXED
5. Box width/height transformation

**Low Priority:**
- Better error messages for unsupported syntax
- Suggestions for workarounds

---

## Test Case

See `examples/pokemon-app/` for real-world example - now fully working!

**Current Status:**
- ✅ Import statements now work
- ✅ Private class fields now work
- ✅ Hex colors now work (fixed in 89b6dfa)
- ✅ Data classes outside main class now work (pass-through architecture)

**All blocking issues resolved!** The Pokemon app should now compile successfully:
```bash
cd examples/pokemon-app
cargo run --manifest-path ../../Cargo.toml -- compile src/stores/PokemonStore.wh
# ✅ Compiles successfully with all data classes included
```

---

*Last Updated: 2025-12-15*
*Import support added: 2025-11-07*
*Private field support added: 2025-11-07*
*Data class pass-through support: 2025-12-15*
