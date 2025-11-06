# Whitehall State Management Reference

**Comprehensive guide to state management patterns and implementation**

---

## Status

✅ **Phases 0-5 Complete** | ✅ **Phase 1.1 Complete**

Core state management features are production-ready. Advanced ViewModel generation for inline component vars is underway.

---

## Quick Summary

Whitehall provides multiple state management patterns inspired by Svelte and modern React:

| Pattern | Syntax | Status | When to Use |
|---------|--------|--------|-------------|
| Local state (simple) | `var count = 0` | ✅ Complete | Simple components, forms |
| Local state (complex) | `var count = 0` (auto-ViewModel) | ✅ Phase 1.1 | Components with suspend/lifecycle/3+ functions |
| Props | `@prop val name: String` | ✅ Complete | Parent-owned state |
| Two-way binding | `bind:value={email}` | ✅ Complete | Form inputs |
| Derived values | `val doubled = count * 2` | ✅ Complete | Computed properties |
| ViewModels (class with var) | `class UserProfile { var ... }` | ✅ Complete | Screen-scoped state with rotation survival |
| Suspend functions | `suspend fun save()` | ✅ Complete | Async operations |
| Coroutine dispatchers | `io { }`, `cpu { }`, `main { }` | ✅ Complete | Thread control |
| Lifecycle hooks | `onMount`, `onDispose` | ✅ Complete | Side effects and cleanup |
| Hilt integration | `@Inject` or `@hilt` | ✅ Complete | Dependency injection |

---

## Table of Contents

1. [Local State (Simple)](#local-state-simple)
2. [Local State (Complex) - Phase 1.1](#local-state-complex---phase-11)
3. [Props](#props)
4. [Two-Way Binding](#two-way-binding)
5. [Derived Values](#derived-values)

6. [ViewModels (Auto-Inferred)](#viewmodels-auto-inferred)
7. [Global Singletons (@store object)](#global-singletons-store-object)
8. [Suspend Functions & Coroutines](#suspend-functions--coroutines)
9. [Lifecycle Hooks](#lifecycle-hooks)
10. [Hilt Integration](#hilt-integration)
11. [Implementation Details](#implementation-details)
12. [Known Gaps & Next Steps](#known-gaps--next-steps)









## Local State (Simple)

### Status: ✅ Complete

**Use case:** Simple components without complex logic

**Syntax:**

```whitehall
var count = 0
var name = ""
var isLoading = false

<Column>
  <Text>Count: {count}</Text>
  <Button onClick={() => count++}>Increment</Button>
</Column>
```

**Generated Kotlin:**

```kotlin
@Composable
fun Counter() {
    var count by remember { mutableStateOf(0) }
    var name by remember { mutableStateOf("") }
    var isLoading by remember { mutableStateOf(false) }

    Column {
        Text("Count: $count")
        Button(onClick = { count++ }) {
            Text("Increment")
        }
    }
}
```

**Characteristics:**
- State survives recomposition
- Does NOT survive configuration changes (rotation)
- Uses `remember { mutableStateOf() }`
- Simple and lightweight

---

## Local State (Complex) - Phase 1.1

### Status: 🔄 In Progress

**Use case:** Complex components with suspend functions, lifecycle hooks, or many functions

**Syntax (same as simple):**

```whitehall
var count = 0
var isLoading = false

suspend fun loadData() {
  isLoading = true
  // ... async work
  isLoading = false
}

onMount {
  loadData()
}

<Column>
  <Text>Count: {count}</Text>
  @if (isLoading) {
    <Text>Loading...</Text>
  }
</Column>
```

**Smart Detection Heuristic:**

Component generates ViewModel if it has:
- Suspend functions (needs viewModelScope)
- >= 3 functions (complex state logic)
- Lifecycle hooks (onMount/onDispose)

**Generated Kotlin (Multi-File Output):**

**File 1: CounterViewModel.kt**
```kotlin
class CounterViewModel : ViewModel() {
    data class UiState(
        val count: Int = 0,
        val isLoading: Boolean = false,
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var count: Int
        get() = _uiState.value.count
        set(value) { _uiState.update { it.copy(count = value) } }

    var isLoading: Boolean
        get() = _uiState.value.isLoading
        set(value) { _uiState.update { it.copy(isLoading = value) } }

    init {
        viewModelScope.launch {
            loadData()
        }
    }

    suspend fun loadData() {
        isLoading = true
        // ... async work
        isLoading = false
    }
}
```

**File 2: Counter.kt (Wrapper)**
```kotlin
@Composable
fun Counter() {
    val viewModel = viewModel<CounterViewModel>()
    val uiState by viewModel.uiState.collectAsState()

    Column {
        Text("Count: ${uiState.count}")
        if (uiState.isLoading) {
            Text("Loading...")
        }
    }
}
```

**Characteristics:**
- State survives configuration changes (rotation)
- Automatic ViewModel generation
- Multi-file output (ViewModel + wrapper component)
- Markup references transformed: `count` → `uiState.count`, `loadData()` → `viewModel.loadData()`

**Implementation Status:**
- ✅ Infrastructure complete
- ✅ Markup transformation working
- ✅ All 38 tests passing (as of 2025-11-06)

**Next Steps:**
- Continue refining edge cases
- Add more test coverage for complex scenarios

---

## Props

### Status: ✅ Complete

**Use case:** Parent-owned state passed to child components

**Syntax:**

```whitehall
@prop val name: String
@prop val age: Int = 18
@prop val onClick: (() -> Unit)? = null

<Column>
  <Text>Name: {name}</Text>
  <Text>Age: {age}</Text>
  @if (onClick != null) {
    <Button onClick={onClick}>Click me</Button>
  }
</Column>
```

**Generated Kotlin:**

```kotlin
@Composable
fun UserCard(
    name: String,
    age: Int = 18,
    onClick: (() -> Unit)? = null
) {
    Column {
        Text("Name: $name")
        Text("Age: $age")
        if (onClick != null) {
            Button(onClick = onClick) {
                Text("Click me")
            }
        }
    }
}
```

**Key Rules:**
- Props must use `val` (immutable)
- Props can have default values
- Props can be nullable with `?`
- Props become function parameters

---

## Two-Way Binding

### Status: ✅ Complete

**Use case:** Form inputs with automatic value synchronization

**Syntax:**

```whitehall
var email = ""
var password = ""

<Column>
  <TextField bind:value={email} label="Email" />
  <TextField bind:value={password} label="Password" type="password" />
  <Button onClick={submit}>
    <Text>Submit</Text>
  </Button>
</Column>
```

**Generated Kotlin:**

```kotlin
@Composable
fun LoginForm() {
    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }

    Column {
        TextField(
            value = email,
            onValueChange = { email = it },
            label = { Text("Email") }
        )
        TextField(
            value = password,
            onValueChange = { password = it },
            label = { Text("Password") },
            visualTransformation = PasswordVisualTransformation()
        )
        Button(onClick = { submit() }) {
            Text("Submit")
        }
    }
}
```

**Transformation:**
- `bind:value={var}` → `value = var, onValueChange = { var = it }`
- `bind:checked={bool}` → `checked = bool, onCheckedChange = { bool = it }`

---

## Derived Values

### Status: ✅ Complete

**Use case:** Computed properties based on other state

**Syntax:**

```whitehall
var firstName = "John"
var lastName = "Doe"

val fullName = "$firstName $lastName"
val isValid = firstName.isNotEmpty() && lastName.isNotEmpty()

<Column>
  <Text>Full Name: {fullName}</Text>
  <Button enabled={isValid}>
    <Text>Submit</Text>
  </Button>
</Column>
```

**Generated Kotlin:**

```kotlin
@Composable
fun NameForm() {
    var firstName by remember { mutableStateOf("John") }
    var lastName by remember { mutableStateOf("Doe") }

    val fullName = "$firstName $lastName"
    val isValid = firstName.isNotEmpty() && lastName.isNotEmpty()

    Column {
        Text("Full Name: $fullName")
        Button(enabled = isValid) {
            Text("Submit")
        }
    }
}
```

**Characteristics:**
- Recomputes automatically when dependencies change
- No special syntax needed (just `val`)
- For expensive computations, use `derivedStateOf` (see test 20)

---

## ViewModels (Auto-Inferred)

### Status: ✅ Complete (Phases 0-5)

**Use case:** Screen-scoped state that survives rotation, with complex logic

**Key Insight:** ViewModels are **automatically inferred** from classes with `var` properties. You do NOT need `@store` annotation for ViewModels!

**Architecture Overview:**

```
class with var properties (.wh file)
    ↓
Analyzer detects var → adds to StoreRegistry as StoreSource::Class
    ↓
Transpiler generates ViewModel with StateFlow
    ↓
Usage site auto-detects and generates viewModel<T>()
```

### Defining a ViewModel

**File:** `src/stores/UserProfile.wh`

```whitehall
// NO @store annotation needed! var properties trigger ViewModel generation
class UserProfile {
  var name: String = ""
  var email: String = ""
  var isLoading: Boolean = false

  // Derived property (NOT in UiState)
  val isValid: Boolean get() = name.isNotEmpty() && email.isNotEmpty()

  // Regular function
  fun clear() {
    name = ""
    email = ""
  }

  // Suspend function (auto-wrapped in viewModelScope.launch)
  suspend fun save() {
    isLoading = true
    api.saveProfile(name, email)
    isLoading = false
  }
}
```

### Generated ViewModel

**File:** `build/.../stores/UserProfile.kt`

```kotlin
class UserProfile : ViewModel() {
    data class UiState(
        val name: String = "",
        val email: String = "",
        val isLoading: Boolean = false,
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var name: String
        get() = _uiState.value.name
        set(value) { _uiState.update { it.copy(name = value) } }

    var email: String
        get() = _uiState.value.email
        set(value) { _uiState.update { it.copy(email = value) } }

    var isLoading: Boolean
        get() = _uiState.value.isLoading
        set(value) { _uiState.update { it.copy(isLoading = value) } }

    val isValid: Boolean
        get() = name.isNotEmpty() && email.isNotEmpty()

    fun clear() {
        name = ""
        email = ""
    }

    fun save() {
        viewModelScope.launch {
            isLoading = true
            api.saveProfile(name, email)
            isLoading = false
        }
    }
}
```

### Using a ViewModel

**File:** `src/screens/ProfileScreen.wh`

```whitehall
import $.stores.UserProfile

val profile = UserProfile()

<Column padding={16}>
  <TextField bind:value={profile.name} label="Name" />
  <TextField bind:value={profile.email} label="Email" />
  <Button onClick={profile.save} enabled={profile.isValid}>
    <Text>Save</Text>
  </Button>
</Column>
```

### Generated Usage Site

**File:** `build/.../screens/ProfileScreen.kt`

```kotlin
@Composable
fun ProfileScreen() {
    val profile = viewModel<UserProfile>(key = "profile")
    val uiState by profile.uiState.collectAsState()

    Column(modifier = Modifier.padding(16.dp)) {
        TextField(
            value = uiState.name,
            onValueChange = { profile.name = it },
            label = { Text("Name") }
        )
        TextField(
            value = uiState.email,
            onValueChange = { profile.email = it },
            label = { Text("Email") }
        )
        Button(
            onClick = { profile.save() },
            enabled = profile.isValid
        ) {
            Text("Save")
        }
    }
}
```

**Key Features:**
- ✅ Auto-inferred from `var` properties (NO `@store` annotation needed!)
- ✅ Auto-detection at usage sites (no manual viewModel<T>() needed)
- ✅ StateFlow reactivity with UiState data class
- ✅ Property accessors for reactive updates
- ✅ Derived properties excluded from UiState
- ✅ Suspend functions auto-wrapped in viewModelScope.launch
- ✅ Multiple instances supported with automatic key generation

---

## Global Singletons (@store object)

### Status: ✅ Complete

**Use case:** App-wide state that lives for entire app lifetime

**Key Difference:** `@store object` generates a **singleton with StateFlow** (NOT a ViewModel!)

**Syntax:**

```whitehall
@store object AppSettings {
  var darkMode = false
  var language = "en"
  
  val isDarkModeEnabled: Boolean get() = darkMode
}
```

**Generated Code:**

```kotlin
object AppSettings {  // object, NOT class
    data class State(
        val darkMode: Boolean = false,
        val language: String = "en"
    )

    private val _state = MutableStateFlow(State())
    val state: StateFlow<State> = _state.asStateFlow()

    var darkMode: Boolean
        get() = _state.value.darkMode
        set(value) { _state.update { it.copy(darkMode = value) } }

    var language: String
        get() = _state.value.language
        set(value) { _state.update { it.copy(language = value) } }
    
    val isDarkModeEnabled: Boolean
        get() = darkMode
}
```

**Key Differences from ViewModel:**
- ✅ `object` keyword (not `class`)
- ✅ StateFlow only (NO ViewModel, NO viewModelScope)
- ✅ Suspend functions keep `suspend` keyword (caller provides scope)
- ✅ Lives for entire app lifetime (not screen-scoped)
- ✅ Accessed directly: `AppSettings.darkMode = true`

---

## Suspend Functions & Coroutines

### Status: ✅ Complete (Phase 2)

Whitehall provides three levels of control for async operations:

### Level 1: Auto-Infer (Future)

**Status:** ⏳ Pending (requires Phase 1.1 complete)

**Design:** Auto-infer the appropriate scope from context

```whitehall
var isLoading = false

suspend fun save() {
  isLoading = true
  api.save()
  isLoading = false
}
```

**Auto-inference rules:**
- Component with `var` → Uses `viewModelScope.launch`
- Inside `onMount` → Already in `LaunchedEffect` scope
- Component without `var` → Uses `rememberCoroutineScope()`
- Singleton (`@store object`) → Keep as `suspend`, caller provides scope

### Level 2: Thread Control (Dispatchers) ✅ Complete

**Status:** ✅ Complete

**Use case:** Explicit thread control for I/O, CPU-bound work, or UI updates

**Syntax:**

```whitehall
var data = []

suspend fun loadData() {
  data = api.fetch()
}

suspend fun processData() {
  data = heavyComputation(data)
}

<Column>
  <!-- Explicit IO thread (network/disk) -->
  <Button onClick={() => io { loadData() }}>Load Data</Button>

  <!-- Explicit CPU thread (heavy computation) -->
  <Button onClick={() => cpu { processData() }}>Process</Button>

  <!-- Force main thread (rare, usually automatic) -->
  <Button onClick={() => main { updateUI() }}>Update UI</Button>
</Column>
```

**Generated Kotlin:**

```kotlin
@Composable
fun DataScreen() {
    var data by remember { mutableStateOf(emptyList()) }
    val dispatcherScope = rememberCoroutineScope()  // Auto-generated

    suspend fun loadData() { data = api.fetch() }
    suspend fun processData() { data = heavyComputation(data) }

    Column {
        Button(onClick = {
            dispatcherScope.launch(Dispatchers.IO) {
                loadData()
            }
        })

        Button(onClick = {
            dispatcherScope.launch(Dispatchers.Default) {
                processData()
            }
        })
    }
}
```

**Dispatchers:**

| Dispatcher | Syntax | When to Use | Kotlin Equivalent |
|------------|--------|-------------|-------------------|
| **Main** | `main { }` | UI updates (auto default) | `Dispatchers.Main` |
| **IO** | `io { }` | Network, disk, database | `Dispatchers.IO` |
| **CPU** | `cpu { }` | Heavy computation | `Dispatchers.Default` |

### Level 3: Custom Scopes ✅ Complete

**Status:** ✅ Complete

**Use case:** Independent lifecycle management (e.g., cancellable operations)

**Syntax:**

```whitehall
var isUploading = false

val uploadScope = $scope()  // Special compiler-provided scope

suspend fun uploadLargeFile() {
  isUploading = true
  api.upload(largeFile)
  isUploading = false
}

fun cancelUpload() {
  uploadScope.cancel()
}

<Column>
  <Button onClick={() => uploadScope.launch { uploadLargeFile() }}>
    Upload File
  </Button>
  <Button onClick={cancelUpload} disabled={!isUploading}>
    Cancel Upload
  </Button>
</Column>
```

**Generated Kotlin:**

```kotlin
@Composable
fun UploadScreen() {
    var isUploading by remember { mutableStateOf(false) }
    val uploadScope = rememberCoroutineScope()  // Generated by $scope()

    suspend fun uploadLargeFile() {
        isUploading = true
        api.upload(largeFile)
        isUploading = false
    }

    fun cancelUpload() {
        uploadScope.cancel()
    }

    Column {
        Button(onClick = {
            uploadScope.launch {
                uploadLargeFile()
            }
        })

        Button(
            onClick = { cancelUpload() },
            enabled = !isUploading
        )
    }
}
```

---

## Lifecycle Hooks

### Status: ✅ Complete

**Use case:** Side effects on component mount/unmount

### onMount

**Syntax:**

```whitehall
var posts = []

onMount {
  launch {
    posts = api.fetchPosts()
  }
}

<Column>
  @for (post in posts) {
    <PostCard post={post} />
  }
</Column>
```

**Generated Kotlin:**

```kotlin
@Composable
fun FeedScreen() {
    var posts by remember { mutableStateOf(emptyList()) }
    val coroutineScope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        coroutineScope.launch {
            posts = api.fetchPosts()
        }
    }

    Column {
        posts.forEach { post ->
            PostCard(post = post)
        }
    }
}
```

### onDispose

**Syntax:**

```whitehall
var messages = []

onMount {
  websocket.connect()
}

onDispose {
  websocket.disconnect()
}

<Column>
  @for (msg in messages) {
    <Text>{msg}</Text>
  }
</Column>
```

**Generated Kotlin:**

```kotlin
@Composable
fun ChatScreen() {
    var messages by remember { mutableStateOf(emptyList()) }

    DisposableEffect(Unit) {
        websocket.connect()
        onDispose {
            websocket.disconnect()
        }
    }

    Column {
        messages.forEach { msg ->
            Text(msg)
        }
    }
}
```

**Smart Hook Combination:**
- **onMount only** → `LaunchedEffect`
- **onMount + onDispose** → `DisposableEffect`
- **Auto-generates coroutineScope** when `launch` calls detected

---

## Hilt Integration

### Status: ✅ Complete (Phase 5)

**Use case:** Dependency injection for stores

### Hybrid Auto-Detection

Hilt is enabled when EITHER:
1. `@hilt` annotation on class
2. `@Inject` annotation on constructor

### With @Inject Constructor (Recommended)

**Store Definition:**

```whitehall
@store
class UserProfile @Inject constructor(
  private val repository: ProfileRepository,
  private val analytics: Analytics
) {
  var name = ""

  suspend fun save() {
    repository.save(name)
    analytics.track("profile_saved")
  }
}
```

**Generated ViewModel:**

```kotlin
@HiltViewModel
class UserProfile @Inject constructor(
    private val repository: ProfileRepository,
    private val analytics: Analytics
) : ViewModel() {
    // ... UiState and reactive properties ...
}
```

**Usage Site:**

```whitehall
val profile = UserProfile()  // Auto-detects @Inject!
```

**Generated Usage:**

```kotlin
val profile = hiltViewModel<UserProfile>()  // Uses hiltViewModel, not viewModel
```

### With @hilt Annotation

**Store Definition:**

```whitehall
@store
@hilt
class UserProfile {
  var name = ""
}
```

**Generated ViewModel:**

```kotlin
@HiltViewModel
class UserProfile : ViewModel() {
    // ...
}
```

**Key Features:**
- ✅ Hybrid detection: `@hilt` OR `@Inject` enables Hilt
- ✅ Automatic `hiltViewModel<T>()` generation at usage sites
- ✅ Proper imports: `dagger.hilt.android.lifecycle.HiltViewModel`, `javax.inject.Inject`
- ✅ Works with dependency injection (constructor parameters)

**User Responsibilities:**
- Set up Hilt application class: `@HiltAndroidApp`
- Define Hilt modules with `@Provides` for dependencies
- Add Hilt Gradle plugin to project

---

## Implementation Details

### Store Registry Architecture

**Location:** `src/transpiler/analyzer.rs`

```rust
pub struct StoreRegistry {
    stores: HashMap<String, StoreInfo>,
}

pub struct StoreInfo {
    pub class_name: String,
    pub has_hilt: bool,          // @hilt annotation detected?
    pub has_inject: bool,        // @Inject constructor detected?
    pub package: String,
    pub source: StoreSource,     // Class, ComponentInline, or Singleton
}

pub enum StoreSource {
    Class,           // Class with var properties → ViewModel (NO @store needed)
    ComponentInline, // Inline vars in component → ViewModel (Phase 1.1)
    Singleton,       // @store object → StateFlow singleton (global state)
}
```

**How it works:**
1. Analyzer scans all AST nodes for `@store` classes
2. Builds registry: class name → StoreInfo
3. Detects Hilt annotations (`@hilt`, `@Inject`)
4. Registry passed to code generator
5. Code generator checks registry when generating components

### Multi-File Output (Phase 1.1)

**Challenge:** ComponentInline requires generating TWO files from ONE input

**Solution:** `TranspileResult` enum

```rust
pub enum TranspileResult {
    /// Single output file (standard case)
    Single(String),

    /// Multiple output files (ComponentInline case)
    /// Each tuple is (filename_suffix, content)
    Multiple(Vec<(String, String)>),
}
```

**API:**
- `primary_content()` - Get main content (backward compatible)
- `is_multiple()` - Check if multi-file result
- `files()` - Get all files as Vec<(suffix, content)>

### Markup Transformation (Phase 1.1)

**Context Tracking:**

```rust
pub struct ComposeBackend {
    // ... other fields ...
    in_viewmodel_wrapper: bool,
    mutable_vars: HashSet<String>,
    derived_props: HashSet<String>,
    function_names: HashSet<String>,
}
```

**Transformation Rules:**
- Mutable vars: `count` → `uiState.count`
- Derived properties: `displayName` → `viewModel.displayName`
- Function calls: `increment()` → `viewModel.increment()`
- Bind directives: `bind:value={count}` → `value=uiState.count, onValueChange={viewModel.count = it}`

**Implementation:** `src/transpiler/codegen/compose.rs` lines 3299-3304, 764, 790, 821

---

## Known Gaps

### Phase 1.1: Component Inline Vars → ViewModel ✅ COMPLETE

**Status:** Infrastructure complete, markup transformation working, 38/38 tests passing

**What's Working:**
- ✅ StoreSource enum with ComponentInline variant
- ✅ TranspileResult multi-file output
- ✅ ViewModel generation for complex components
- ✅ Wrapper component generation
- ✅ Smart detection heuristic (suspend/lifecycle/3+ functions)
- ✅ Markup transformation (count → uiState.count)
- ✅ Lifecycle hooks in ViewModel init block
- ✅ All tests passing

**Next Steps:**
- Continue refining edge cases
- Add more test coverage for complex scenarios
- Documentation updates

### Phase 1.2: Imported Classes with `var` ⏳ FUTURE

**Goal:** Auto-detect classes with `var` at usage sites, generate viewModel<T>() automatically

**Design:**

```whitehall
import $.models.User  // User has var properties

val user = User()  // Should auto-detect and use viewModel<User>()
```

**Challenge:** Requires cross-file type analysis


---

## Next Steps

### Completed (Phase 1.1) ✅
- ✅ Infrastructure complete
- ✅ Markup transformation working
- ✅ All tests passing
- Continue refining based on real-world usage

### Short-term (Phase 1.2)
- Imported classes with `var` auto-detection
- Cross-file type analysis
- Auto-generate viewModel<T>() at usage sites

### Medium-term (Phase 1.3)
- Global singletons (`@store object`)
- StateFlow-based (not ViewModel)
- App-wide state management

### Long-term (Phase 2.0)
- Store composition patterns
- DevTools integration
- Persistence middleware
- Time-travel debugging

---

## Key Files Reference

| File | Lines | Purpose |
|------|-------|---------|
| `src/transpiler/analyzer.rs` | ~300 | Store registry, semantic analysis |
| `src/transpiler/codegen/compose.rs` | ~3500 | Code generation, ViewModel output |
| `src/transpiler/mod.rs` | ~100 | TranspileResult enum, public API |
| `src/build_pipeline.rs` | ~400 | Component inline var detection |
| `tests/transpiler-examples/27-29.md` | 3 files | ViewModel test cases (classes with var) |
| `tests/transpiler-examples/30-32.md` | 3 files | Phase 1.1 test cases |

---

## Related Documentation

- [REF-OVERVIEW.md](./REF-OVERVIEW.md) - Architecture overview
- [REF-TRANSPILER.md](./REF-TRANSPILER.md) - Transpiler details
- [REF-BUILD-SYSTEM.md](./REF-BUILD-SYSTEM.md) - Build commands

---

*Last Updated: 2025-01-06*
*Version: 1.0*
*Status: Phases 0-5 Complete | Phase 1.1 Complete*
*Last Updated: 2025-11-06*
