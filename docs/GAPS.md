# Known Gaps and Limitations

Issues discovered during Pokemon app example development.

## Parser Limitations

### 1. Top-level Kotlin code not supported

**Issue:** Parser expects Whitehall component/class, can't parse raw Kotlin before class definition

**Example:**
```kotlin
import kotlinx.serialization.Serializable  // ❌ Parser error
import okhttp3.OkHttpClient                // ❌ Parser error

class MyStore {
    // ...
}
```

**Current State:** ❌ Not supported
**Location:** `src/stores/PokemonStore.wh:1-3`
**Error:** `Expected component, found: "import kotlinx.serialization"`

**Workaround:** None - must move imports inside Kotlin blocks

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

---

### 2. Import statements for external libraries

**Needed for:** OkHttp, Kotlinx Serialization, other libraries

**Use case:** Network calls, JSON parsing

**Impact:** Can't use external libraries without workarounds

---

## Priority

**High Priority (blocks real apps):**
1. Hex color transpilation bug
2. Top-level data classes
3. Private val fields with initialization

**Medium Priority:**
4. Top-level imports (Kotlin interop)
5. Box width/height transformation

**Low Priority:**
- Better error messages for unsupported syntax
- Suggestions for workarounds

---

## Test Case

See `examples/pokemon-app/` for real-world example hitting all these issues.

**To reproduce:**
```bash
cd examples/pokemon-app
cargo run --manifest-path ../../Cargo.toml -- build
```

---

*Last Updated: 2025-11-07*
