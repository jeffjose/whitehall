# Suspend Functions and Coroutine Scopes

**Status:** ✅ Decided - Option C (Auto-Infer + Override)

---

## Decision: Auto-Infer Scope with Clean Override Syntax

**Chosen Approach:** Auto-infer the appropriate scope from context (90% case), but allow explicit override with clean syntax (10% advanced cases).

**Rationale:**
- Scope is usually obvious from context (component, ViewModel, effect)
- Clean syntax for common case (like Svelte's `async`)
- Power users can override for thread control or custom scopes
- Whitehall compiler can provide better names than Kotlin's verbose APIs

---

## Implementation Plan: Three Levels of Control

### Level 1: Auto-Infer (Default - 90% of cases)

**No scope thinking required - just works!**

```whitehall
<script>
  var isLoading = false

  suspend fun save() {
    isLoading = true
    api.save()
    isLoading = false
  }
</script>

<Button onClick={save}>Save</Button>
```

**Transpiles to:**
```kotlin
class MyScreenViewModel : ViewModel() {
    fun save() {
        viewModelScope.launch {  // ← Auto-inferred
            isLoading = true
            api.save()
            isLoading = false
        }
    }
}

@Composable
fun MyScreen() {
    val vm = viewModel<MyScreenViewModel>()
    Button(onClick = { vm.save() })  // ← Simple call
}
```

**Auto-inference rules:**
- Component with `var` → Uses `viewModelScope.launch`
- Inside `onMount` → Already in `LaunchedEffect` scope
- Component without `var` → Uses `rememberCoroutineScope()`
- Singleton (`@store object`) → Keep as `suspend`, caller provides scope

---

### Level 2: Thread Control (Dispatchers)

**For when you need to control which thread the work runs on.**

#### The Three Dispatchers

| Dispatcher | Whitehall Syntax | When to Use | Kotlin Equivalent |
|------------|------------------|-------------|-------------------|
| **Main** | `main { }` | UI updates (auto default) | `Dispatchers.Main` |
| **IO** | `io { }` | Network, disk, database | `Dispatchers.IO` |
| **CPU** | `cpu { }` | Heavy computation | `Dispatchers.Default` |

#### Examples:

```whitehall
<script>
  var data = []
  var processedData = []

  suspend fun loadData() {
    data = api.fetch()  // Network call
  }

  suspend fun processData() {
    processedData = heavyComputation(data)  // CPU-intensive
  }
</script>

<!-- Auto (main thread via viewModelScope) -->
<Button onClick={save}>Save</Button>

<!-- Explicit IO thread (network/disk operations) -->
<Button onClick={() => io { loadData() }}>Load Data</Button>

<!-- Explicit CPU thread (heavy computation) -->
<Button onClick={() => cpu { processData() }}>Process</Button>

<!-- Force main thread (rare, usually automatic) -->
<Button onClick={() => main { updateUI() }}>Update UI</Button>
```

**Transpiles to:**
```kotlin
// io { }
Button(onClick = {
    viewModelScope.launch(Dispatchers.IO) {
        vm.loadData()
    }
})

// cpu { }
Button(onClick = {
    viewModelScope.launch(Dispatchers.Default) {
        vm.processData()
    }
})

// main { }
Button(onClick = {
    viewModelScope.launch(Dispatchers.Main) {
        vm.updateUI()
    }
})
```

---

### Level 3: Custom Scopes (Advanced - Rare)

**For independent lifecycle management (e.g., cancellable operations).**

```whitehall
<script>
  var isUploading = false

  val uploadScope = scope()  // Custom scope

  suspend fun uploadLargeFile() {
    isUploading = true
    api.upload(largeFile)
    isUploading = false
  }

  fun cancelUpload() {
    uploadScope.cancel()
  }
</script>

<!-- Launch in custom scope -->
<Button onClick={() => uploadScope.launch { uploadLargeFile() }}>
  Upload File
</Button>

<Button onClick={cancelUpload} disabled={!isUploading}>
  Cancel Upload
</Button>
```

**Transpiles to:**
```kotlin
@Composable
fun MyScreen() {
    val vm = viewModel<MyScreenViewModel>()
    val uploadScope = rememberCoroutineScope()  // ← Hidden verbose name

    Button(onClick = {
        uploadScope.launch {
            vm.uploadLargeFile()
        }
    })

    Button(
        onClick = { vm.cancelUpload() },
        enabled = !vm.isUploading
    )
}
```

**Note:** `scope()` in Whitehall hides Kotlin's verbose `rememberCoroutineScope()` name.

---

## Complete Example: All Three Levels

```whitehall
<script>
  var data = []
  var isUploading = false

  val uploadScope = scope()

  // Level 1: Auto (uses viewModelScope, main thread)
  suspend fun saveSimple() {
    api.save(data)
  }

  // Level 1: Auto (but we know it's IO work)
  suspend fun loadData() {
    data = api.fetch()
  }

  // Level 1: Auto (but we know it's CPU work)
  suspend fun processData() {
    data = processLargeDataset(data)
  }

  // Level 3: Custom scope for cancellation
  suspend fun uploadFile() {
    isUploading = true
    api.upload(largeFile)
    isUploading = false
  }

  fun cancelUpload() {
    uploadScope.cancel()
  }
</script>

<!-- Level 1: Auto (simple) -->
<Button onClick={saveSimple}>Save</Button>

<!-- Level 2: Explicit IO thread -->
<Button onClick={() => io { loadData() }}>Load Data</Button>

<!-- Level 2: Explicit CPU thread -->
<Button onClick={() => cpu { processData() }}>Process Data</Button>

<!-- Level 3: Custom scope -->
<Button onClick={() => uploadScope.launch { uploadFile() }}>Upload</Button>
<Button onClick={cancelUpload}>Cancel</Button>
```

---

## Dispatcher Details

### Main Dispatcher (Default)
**Purpose:** UI thread
**Thread pool:** Single thread
**Default for:** ViewModels, components with `var`
**When to use explicitly:** Rarely (it's the default)

```whitehall
main { updateUI() }  // Force main thread (rarely needed)
```

### IO Dispatcher
**Purpose:** I/O operations (waiting for external resources)
**Thread pool:** Large pool (64+ threads) optimized for waiting
**When to use:**
- Network calls (API requests)
- File system operations
- Database queries/writes
- Anything that **waits** for external I/O

```whitehall
io {
  data = api.fetch()        // Network
  file.write(data)          // Disk
  db.insert(record)         // Database
}
```

### CPU Dispatcher
**Purpose:** CPU-intensive computation
**Thread pool:** Sized to CPU cores (e.g., 8 threads on 8-core device)
**When to use:**
- Image/video processing
- Data processing/parsing
- Complex algorithms
- Anything that **uses a lot of CPU**

```whitehall
cpu {
  processImage()            // Image processing
  parseHugeJSON()          // Heavy parsing
  sortMassiveArray()       // Computation
}
```

---

## Context-Specific Behavior

### Components with `var` (ViewModel Generated)

```whitehall
<script>
  var count = 0

  suspend fun increment() {
    count++
  }
</script>

<Button onClick={increment}>  <!-- Auto: viewModelScope.launch -->
```

**Auto-inference:** Uses `viewModelScope.launch`

---

### Components without `var` (No ViewModel)

```whitehall
<script>
  suspend fun logAnalytics() {
    analytics.log("screen_viewed")
  }

  onMount {
    logAnalytics()  // Works in LaunchedEffect scope
  }
</script>
```

**Auto-inference:**
- In `onMount`: Already has `LaunchedEffect` scope
- In `onClick`: Auto-generate `rememberCoroutineScope()`

---

### Singletons (`@store object`)

```whitehall
@store
object AppSettings {
  var darkMode = false

  suspend fun loadFromDisk() {  // ← Stays suspend
    darkMode = dataStore.read()
  }
}

// Usage:
onMount {
  AppSettings.loadFromDisk()  // Caller provides scope
}

<Button onClick={() => io { AppSettings.loadFromDisk() }}>Load</Button>
```

**Decision:** Singletons keep functions as `suspend` - no auto-wrap
**Reason:** No lifecycle to tie scope to, caller provides context

---

## Implementation Tasks

### 1. Auto-Wrap for ViewModels
- Detect `suspend fun` in ViewModel-generating contexts (components with `var`)
- Transform to non-suspend, wrap body in `viewModelScope.launch { }`
- Default dispatcher: `Dispatchers.Main`

### 2. Dispatcher Syntax
- Parse `io { }`, `cpu { }`, `main { }` blocks
- Transform to `viewModelScope.launch(Dispatchers.X) { }`
- Map: `io` → `IO`, `cpu` → `Default`, `main` → `Main`

### 3. Custom Scope Syntax
- Parse `val myScope = scope()` declaration
- Transform to `val myScope = rememberCoroutineScope()`
- Parse `myScope.launch { }` blocks
- Transform to Kotlin `myScope.launch { }`

### 4. Context Detection
- Components with `var`: Use `viewModelScope`
- Components without `var`: Use `rememberCoroutineScope()` (if needed)
- Inside `onMount`: Already in scope
- Singletons: Keep as `suspend`

### 5. Error Messages
- Warn if using `io` for pure computation (suggest `cpu`)
- Warn if using `cpu` for network calls (suggest `io`)
- Clear errors for scope misuse

---

## Alternative Syntax Considered

### For Dispatchers:
- ✅ `io { }` - Chosen (clean, standard term)
- ❌ `background { }` - Less specific (IO or CPU?)
- ❌ `launch(on: IO) { }` - More verbose

### For Custom Scopes:
- ✅ `scope()` - Chosen (clean, minimal)
- ❌ `Scope()` - Constructor style (less clear)
- ❌ `@scope val myScope` - Annotation style (overkill)

### For Thread Names:
- ✅ `io`, `cpu`, `main` - Chosen (clear purpose)
- ❌ `io`, `default`, `main` - "default" is unclear
- ❌ `network`, `compute`, `ui` - More words, not standard

---

## Comparison to Original Options

This approach combines the best of both:

| Aspect | Option A (suspend) | Option B (auto-wrap) | **Option C (chosen)** |
|--------|-------------------|---------------------|---------------------|
| Common case | Verbose | Clean ✅ | Clean ✅ |
| Thread control | Flexible ✅ | Limited | Flexible ✅ |
| Custom scopes | Supported ✅ | No | Supported ✅ |
| Learning curve | Steeper | Easy ✅ | Balanced ✅ |
| Kotlin-native | Yes ✅ | No | Hybrid ✅ |

---

## Migration from Current Implementation

Current implementation (Phase 4) auto-wraps all `suspend fun` in ViewModels.

**Changes needed:**
1. Keep auto-wrap behavior (already correct)
2. Add dispatcher syntax (`io { }`, `cpu { }`, `main { }`)
3. Add custom scope syntax (`val myScope = scope()`)
4. Add scope inference for components without ViewModels
5. Update singleton handling (keep as suspend)

**Breaking changes:** None - current auto-wrap behavior is preserved as default.

---

## Next Steps

1. ✅ Decision made: Option C (auto-infer + override)
2. 📋 Implement dispatcher syntax (`io`, `cpu`, `main`)
3. 📋 Implement custom scope syntax (`scope()`)
4. 📋 Update context detection rules
5. 📋 Add helpful error messages
6. 📋 Document in user-facing guides
