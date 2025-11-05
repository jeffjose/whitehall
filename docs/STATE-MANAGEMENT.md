# State Management in Whitehall

**Status:** 🚧 Design Document - Implementation Pending

---

## Quick Reference: Current Status

| **Pattern** | **Status** | **Whitehall Syntax** |
|-------------|-----------|---------------------|
| Local state | ✅ Supported | `var count = 0` |
| Props | ✅ Supported | `@prop val name: String` |
| Two-way binding | ✅ Supported | `bind:value={email}` |
| Derived values | ✅ Supported | `val doubled = count * 2` |
| Hoisted state | ✅ Supported | Local state + props |
| **Stores (screen-level)** | **✅ DECIDED - Not Implemented** | `@store class UserProfile {...}` + `val profile = UserProfile()` (auto-detected) |
| StateFlow (manual) | ⚠️ Works today | Use Kotlin files directly |
| Effects | ⚠️ Works today | Use `LaunchedEffect` directly |
| CompositionLocal | ⚠️ Works today | Use Kotlin directly |
| Lifecycle hooks | 🤔 Under consideration | `onMount` vs `LaunchedEffect` |
| Global stores | 🤔 Under consideration | Scope options TBD |
| Persistence | 🤔 Under consideration | Future feature |

**Legend:**
- ✅ **Supported** - Works today with clean syntax
- **✅ DECIDED** - Decision made, needs implementation
- ⚠️ **Works today** - No special syntax, use Kotlin/Compose directly
- 🤔 **Under consideration** - Options available, decision needed

---

## Table of Contents

1. [Decisions Made](#decisions-made)
2. [Implementation Plan](#implementation-plan)
3. [Open Questions](#open-questions)
4. [Background: Android/Kotlin Patterns](#background-androidkotlin-patterns)
5. [Full Examples](#full-examples)

---

## Decisions Made

### Decision 1: Stores for Screen-Level State

**Problem:** ViewModels in Android require massive boilerplate (StateFlow, MutableStateFlow, update blocks, property accessors).

**Decision:** Use `@store` annotation to generate ViewModel boilerplate from simple class definitions.

**Syntax:**

**Define a store:**
```whitehall
<!-- stores/UserProfile.wh -->
@store
class UserProfile {
  var name = ""
  var email = ""
  var isLoading = false
  var error: String? = null

  suspend fun save() {
    isLoading = true
    try {
      repository.saveProfile(name, email)
      isLoading = false
    } catch (e: Exception) {
      error = e.message
      isLoading = false
    }
  }
}
```

**Use in a screen:**
```whitehall
<!-- screens/ProfileScreen.wh -->
<script>
  import com.app.stores.UserProfile

  val profile = UserProfile()  // Auto-detected as @store!

  // Or be explicit (optional):
  // @store val profile = UserProfile()
</script>

<Column>
  <Input bind:value={profile.name} label="Name" />
  <Button onClick={profile::save} disabled={profile.isLoading}>
    Save
  </Button>
</Column>
```

**Key principles:**
- ✅ Explicit import (regular Kotlin import)
- ✅ Auto-detection: `@store` classes automatically use `viewModel<T>()`
- ✅ Optional explicit annotation: `@store val profile = ...` for clarity
- ✅ Kotlin-native syntax (callable references with `::`)
- ✅ No magic auto-instantiation
- ✅ Regular class usage

**Rationale:**
- Web developers familiar with "stores" concept (Svelte, Redux, Zustand)
- Clear purpose: stores state
- Natural directory name: `stores/`
- Distinguishes from plain data models
- Short and memorable

---

### Decision 2: Callable References (`::`)

**Decision:** Use Kotlin's callable reference operator (`::`) for passing functions.

**Examples:**
```whitehall
<!-- Function reference -->
<Button onClick={profile::save}>Save</Button>

<!-- Lambda when you need parameters or additional logic -->
<Button onClick={() => profile.save("John")}>Save John</Button>
<Button onClick={() => {
  analytics.track("save_clicked")
  profile.save()
}}>
  Save
</Button>
```

**Rationale:**
- Standard Kotlin syntax
- Concise for simple cases
- Lambda available when needed

---

### Decision 3: Stores Directory Convention

**Decision:** `stores/` is the **recommended convention** for organizing store classes, but not required.

**Important:** The `@store` annotation is what matters - it works regardless of file location. The directory structure is purely for organization.

**Recommended structure:**
```
src/
  stores/              ← Reactive stores (@store) - RECOMMENDED location
    UserProfile.wh
    Settings.wh
    Cart.wh
  models/              ← Plain data classes (no reactivity)
    User.kt
    Post.kt
    Product.kt
  screens/
    ProfileScreen.wh
    SettingsScreen.wh
  components/
    Button.wh
```

**Alternative organizations (all work fine):**

**Option A: Co-located with screens**
```
src/
  screens/
    profile/
      ProfileScreen.wh
      ProfileStore.wh    ← Store next to screen
    settings/
      SettingsScreen.wh
      SettingsStore.wh
```

**Option B: Feature-based**
```
src/
  features/
    auth/
      LoginScreen.wh
      LoginStore.wh
    profile/
      ProfileScreen.wh
      ProfileStore.wh
```

**Option C: Android-style**
```
src/
  viewmodels/          ← If you prefer Android terminology
    ProfileViewModel.wh
    SettingsViewModel.wh
  ui/
    screens/
      ProfileScreen.wh
```

**Why we recommend `stores/`:**
- ✅ Clear separation from plain data models
- ✅ Familiar to web developers (Svelte, Redux, Zustand)
- ✅ Easy to find all reactive state in one place
- ✅ Natural grouping for shared state

**But use whatever structure fits your project!** The compiler only cares about the `@store` annotation.

---

## Implementation Plan

### Phase 0: Extend Semantic Analyzer (FOUNDATION)

**Goal:** Build a store registry during semantic analysis to enable auto-detection of `@store` classes.

**Architecture:** Whitehall already has a semantic analysis phase (`src/transpiler/analyzer.rs`) that runs after parsing:
```rust
// Current pipeline in src/transpiler/mod.rs
pub fn transpile(...) -> Result<String, String> {
    let ast = parser.parse()?;
    let semantic_info = Analyzer::analyze(&ast)?;  // ← Semantic analysis
    let optimized_ast = Optimizer::optimize(ast, semantic_info);
    codegen.generate(&optimized_ast)
}
```

**Implementation steps:**
1. Add `StoreRegistry` struct to `analyzer.rs`:
   ```rust
   pub struct StoreRegistry {
       stores: HashMap<String, StoreInfo>
   }

   pub struct StoreInfo {
       pub class_name: String,
       pub has_hilt: bool,        // Has @HiltViewModel annotation?
       pub has_inject: bool,      // Has @Inject constructor?
       pub package: String,        // Fully qualified package
   }
   ```

2. Add `store_registry: StoreRegistry` to `SemanticInfo` struct

3. During `Analyzer::analyze()`, scan AST for classes with `@store` annotation:
   - Extract class name
   - Check for `@HiltViewModel` annotation
   - Check for `@Inject constructor(...)`
   - Store in registry

4. Pass registry to codegen via `SemanticInfo`

**Why this phase matters:**
- Enables auto-detection of store classes at usage sites
- Distinguishes between `viewModel<T>()` and `hiltViewModel<T>()`
- Foundation for Phases 1-2

---

### Phase 1: Basic Store Generation

**Goal:** Generate ViewModel boilerplate from `@store` classes.

**Input:** `stores/UserProfile.wh`
```whitehall
@store
class UserProfile {
  var name = ""
  var email = ""
}
```

**Output:** `UserProfile.kt` (generated)
```kotlin
package com.app.stores

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

class UserProfile : ViewModel() {
    data class UiState(
        val name: String = "",
        val email: String = ""
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var name: String
        get() = _uiState.value.name
        set(value) { _uiState.update { it.copy(name = value) } }

    var email: String
        get() = _uiState.value.email
        set(value) { _uiState.update { it.copy(email = value) } }
}
```

**Implementation steps:**
1. Parse `@store` annotation on class
2. Extract all `var` declarations → generate UiState data class fields
3. Generate `_uiState` and `uiState` StateFlow pair
4. Generate property accessors (getter/setter with `.update { it.copy(...) }`)
5. Copy functions as-is, wrapping `suspend fun` in `viewModelScope.launch`

---

### Phase 2: Store Usage in Screens

**Input:** `screens/ProfileScreen.wh`
```whitehall
<script>
  import com.app.stores.UserProfile

  val profile = UserProfile()  // Auto-detected as @store class
</script>

<Input bind:value={profile.name} />
```

**Output:** `ProfileScreen.kt` (generated)
```kotlin
@Composable
fun ProfileScreen() {
    val profile = viewModel<UserProfile>()
    val uiState by profile.uiState.collectAsState()

    OutlinedTextField(
        value = uiState.name,
        onValueChange = { profile.name = it }
    )
}
```

**Implementation steps:**
1. **First pass:** Build registry of all `@store` annotated classes
2. **Component pass:** Detect `val profile = UserProfile()` pattern
3. **Check:** Is `UserProfile` in the `@store` registry?
   - Yes → Treat as store (continue to step 4)
   - No → Regular variable instantiation
4. Generate `val profile = viewModel<UserProfile>()`
5. Generate `val uiState by profile.uiState.collectAsState()`
6. Rewrite references:
   - `profile.name` (in expressions) → `uiState.name`
   - `profile.name = value` (in assignments) → `profile.name = value`
   - `profile::save` → `{ profile.save() }` or direct reference

**Optional explicit annotation:**
- User can write `@store val profile = UserProfile()` to be explicit
- Transpiler behavior is identical (checks registry first)
- Useful for code clarity or when the type inference is ambiguous

---

### Phase 3: Derived Properties

**Input:**
```whitehall
@store
class UserProfile {
  var firstName = ""
  var lastName = ""

  val fullName: String
    get() = "$firstName $lastName"
}
```

**Output:**
```kotlin
class UserProfile : ViewModel() {
    data class UiState(
        val firstName: String = "",
        val lastName: String = ""
    )

    // ... StateFlow setup ...

    val fullName: String
        get() = "${_uiState.value.firstName} ${_uiState.value.lastName}"
}
```

**Implementation:** Copy `val` properties with getters as-is, rewrite to access `_uiState.value.x`

---

### Phase 4: Suspend Functions

**Input:**
```whitehall
@store
class UserProfile {
  var isLoading = false

  suspend fun save() {
    isLoading = true
    repository.save()
    isLoading = false
  }
}
```

**Output:**
```kotlin
class UserProfile : ViewModel() {
    // ... state setup ...

    fun save() {
        viewModelScope.launch {
            isLoading = true
            repository.save()
            isLoading = false
        }
    }
}
```

**Implementation:** Wrap `suspend fun` body in `viewModelScope.launch { }`

---

### Phase 5: Dependency Injection (Hilt)

**Input:**
```whitehall
@store
@HiltViewModel
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

**Usage:**
```whitehall
<script>
  val profile = UserProfile()  // Auto-detects @HiltViewModel!
</script>
```

**Output (store):**
```kotlin
@HiltViewModel
class UserProfile @Inject constructor(
    private val repository: ProfileRepository,
    private val analytics: Analytics
) : ViewModel() {
    // ... generated StateFlow code ...
}
```

**Output (usage):**
```kotlin
val profile = hiltViewModel<UserProfile>()  // Auto-generated!
```

**Implementation:**
1. Preserve `@HiltViewModel` and `@Inject constructor()` on store class
2. In store registry, track both `@store` and presence of `@HiltViewModel`
3. When generating usage:
   - Check if class has `@HiltViewModel` in registry
   - Yes → Generate `hiltViewModel<T>()`
   - No → Generate `viewModel<T>()`
4. No annotation needed at usage site - fully automatic!

---

## Open Questions

### Question 1: Lifecycle Hooks ✅ DECIDED

**Decision:** Add lifecycle hooks for cleaner syntax.

**Syntax:**
```whitehall
<script>
  onMount {
    analytics.trackScreenView("profile")
  }

  onDestroy {
    subscription.cancel()
  }

  // Or for cleanup that needs setup:
  onMount {
    val subscription = eventBus.subscribe()
    onDestroy {
      subscription.cancel()
    }
  }
</script>
```

**Naming decision needed:** `onDestroy` vs `onDispose`
- **`onDestroy`** - Clearer intent (component is being destroyed)
- **`onDispose`** - Matches Compose's `DisposableEffect { onDispose { } }`

**Recommendation:** `onDispose` (more Kotlin/Compose-native)

**Transpiles to:**
```kotlin
LaunchedEffect(Unit) {
    analytics.trackScreenView("profile")
}

DisposableEffect(Unit) {
    val subscription = eventBus.subscribe()
    onDispose {
        subscription.cancel()
    }
}
```

**Why add this:** Cleaner, familiar to web developers, less verbose than Compose's `LaunchedEffect`/`DisposableEffect`.

---

### Question 2: Global Stores (App-Wide State)

**Question:** How to define stores that live for the entire app lifetime (not screen-scoped)?

**Context:** Stores defined with `@store val settings = AppSettings()` inside a component are screen-scoped. For global state (like app settings, current user), we need a different pattern.

**Option A: Global store files (Svelte-style)** ← RECOMMENDED

Define stores in standalone files that export singletons:

```whitehall
<!-- stores/AppSettings.wh -->
@store(singleton = true)
class AppSettings {
  var darkMode = false
  var language = "en"
}

// Auto-export singleton
export val appSettings = AppSettings()
```

```whitehall
<!-- screens/SettingsScreen.wh -->
<script>
  import { appSettings } from "../stores/AppSettings"

  // No @store annotation - just use the singleton
</script>

<Switch bind:checked={appSettings.darkMode} label="Dark Mode" />
```

**Pros:**
- ✅ Clear separation (global vs screen-scoped)
- ✅ Familiar to web developers
- ✅ Explicit singleton
- ✅ Reusable across all screens

**Option B: Repository pattern (Kotlin files)**

```kotlin
// repositories/AppSettings.kt
object AppSettings {
    private val _darkMode = MutableStateFlow(false)
    val darkMode: StateFlow<Boolean> = _darkMode.asStateFlow()

    fun toggleDarkMode() {
        _darkMode.value = !_darkMode.value
    }
}
```

```whitehall
<script>
  import com.app.repositories.AppSettings

  val darkMode by AppSettings.darkMode.collectAsState()
</script>
```

**Pros:**
- ✅ Standard Kotlin pattern
- ✅ No new whitehall syntax

**Cons:**
- ❌ Still verbose (manual StateFlow boilerplate)

**Decision needed:** Option A (global store files) or Option B (Kotlin repositories)?

---

### Question 3: Persistence

**Question:** How to persist store state across app restarts?

**Context:** Android has multiple persistence options:
- DataStore (modern, coroutine-based)
- SharedPreferences (legacy, synchronous)
- Room (database)
- File system

**Decision:** Keep it manual (no special syntax)

**Rationale:**
- DataStore is just one option, not the only one
- `@persisted` annotation would be too opinionated
- Manual approach is flexible and explicit

**Recommended pattern:**
```whitehall
@store
class AppSettings {
  var darkMode = false

  init {
    loadSettings()
  }

  private fun loadSettings() {
    viewModelScope.launch {
      dataStore.data.collect { prefs ->
        darkMode = prefs[DARK_MODE_KEY] ?: false
      }
    }
  }

  fun saveDarkMode(value: Boolean) {
    darkMode = value
    viewModelScope.launch {
      dataStore.edit { prefs ->
        prefs[DARK_MODE_KEY] = value
      }
    }
  }
}
```

**Future consideration:** Could add a helper library (not annotation):
```whitehall
@store
class AppSettings {
  var darkMode by persistedState("dark_mode", false)  // Helper, not compiler magic
}
```

---

### Question 4: Multiple Store Instances ✅ DECIDED

**Decision:** Support multiple instances with automatic key generation.

**Usage:**
```whitehall
<script>
  val adminProfile = UserProfile()  // Auto-detected
  val guestProfile = UserProfile()  // Auto-detected
</script>
```

**Problem:** Android's `viewModel<UserProfile>()` creates a singleton per type. Calling it twice returns the same instance.

**Solution:** Generate unique keys based on variable name.

**Transpiles to:**
```kotlin
val adminProfile = viewModel<UserProfile>(key = "adminProfile")
val guestProfile = viewModel<UserProfile>(key = "guestProfile")
```

**Implementation:**
- Detect `val <name> = UserProfile()` where `UserProfile` is in `@store` registry
- Extract variable name `<name>`
- Pass variable name as key to `viewModel(key = "...")`
- Each variable gets its own instance

**Why this works:** Kotlin's `viewModel()` supports keys for multiple instances of the same type.

**Manual key override (if needed):**
```whitehall
@store(key = "custom_key") val profile = UserProfile()
```

---

## Background: Android/Kotlin Patterns

### Local Component State

**Standard Kotlin/Compose:**
```kotlin
@Composable
fun Counter() {
    var count by remember { mutableStateOf(0) }
    Button(onClick = { count++ }) {
        Text("Count: $count")
    }
}
```

**Whitehall:** ✅ Supported
```whitehall
<script>
  var count = 0
</script>

<Button onClick={() => count++}>
  Count: {count}
</Button>
```

**Transpiles to:**
```kotlin
@Composable
fun Counter() {
    var count by remember { mutableStateOf(0) }
    Button(onClick = { count++ }) {
        Text("Count: $count")
    }
}
```

**Lifecycle:**
- ✅ Survives recomposition
- ❌ Lost on configuration change
- ❌ Lost on navigation
- ❌ Not shared between components

---

### Hoisted State (Props)

**Standard Kotlin/Compose:**
```kotlin
@Composable
fun Parent() {
    var text by remember { mutableStateOf("") }
    InputField(value = text, onValueChange = { text = it })
}

@Composable
fun InputField(value: String, onValueChange: (String) -> Unit) {
    TextField(value = value, onValueChange = onValueChange)
}
```

**Whitehall:** ✅ Supported
```whitehall
<!-- Parent.wh -->
<script>
  var text = ""
</script>
<InputField value={text} onValueChange={(newText) => text = newText} />

<!-- InputField.wh -->
<script>
  @prop val value: String
  @prop val onValueChange: (String) -> Unit
</script>
<TextField value={value} onValueChange={onValueChange} />
```

---

### Screen-Level State (ViewModels)

**Standard Kotlin/Compose:**
```kotlin
class ProfileViewModel : ViewModel() {
    data class UiState(
        val name: String = "",
        val isLoading: Boolean = false
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var name: String
        get() = _uiState.value.name
        set(value) { _uiState.update { it.copy(name = value) } }

    var isLoading: Boolean
        get() = _uiState.value.isLoading
        set(value) { _uiState.update { it.copy(isLoading = value) } }

    fun save() {
        viewModelScope.launch {
            isLoading = true
            repository.save(name)
            isLoading = false
        }
    }
}

@Composable
fun ProfileScreen() {
    val vm = viewModel<ProfileViewModel>()
    val uiState by vm.uiState.collectAsState()
    TextField(value = uiState.name, onValueChange = { vm.name = it })
}
```

**Whitehall:** ✅ DECIDED (not implemented)
```whitehall
<!-- stores/UserProfile.wh -->
@store
class UserProfile {
  var name = ""
  var isLoading = false

  suspend fun save() {
    isLoading = true
    repository.save(name)
    isLoading = false
  }
}

<!-- screens/ProfileScreen.wh -->
<script>
  import com.app.stores.UserProfile

  val profile = UserProfile()  // Auto-detected as @store
</script>

<Input bind:value={profile.name} />
<Button onClick={profile::save}>Save</Button>
```

**Lifecycle:**
- ✅ Survives recomposition
- ✅ Survives configuration changes
- ✅ Cleared on navigation away
- ✅ Handles async work (coroutines)

---

### App-Wide State (Repositories)

**Standard Kotlin:**
```kotlin
// kotlin/repositories/UserRepository.kt
object UserRepository {
    private val _currentUser = MutableStateFlow<User?>(null)
    val currentUser: StateFlow<User?> = _currentUser.asStateFlow()

    fun updateUser(user: User) {
        _currentUser.value = user
    }
}

@Composable
fun ProfileScreen() {
    val user by UserRepository.currentUser.collectAsState()
    Text("User: ${user?.name}")
}
```

**Whitehall:** ⚠️ Works today (use Kotlin directly)
```kotlin
// kotlin/repositories/UserRepository.kt (same as above)
```

```whitehall
<script>
  import com.app.repositories.UserRepository

  val user by UserRepository.currentUser.collectAsState()
</script>

<Text>User: {user?.name}</Text>
```

**Lifecycle:**
- ✅ App-wide (singleton)
- ✅ Survives everything
- ✅ Shared across all screens

---

### Dependency Injection (Hilt)

**What is Hilt?**

Hilt is Google's dependency injection library. It automatically provides dependencies to ViewModels/Stores without manual passing.

**Without Hilt (manual):**
```kotlin
@Composable
fun ProfileScreen(
    repository: UserRepository,  // ❌ Must pass manually
    analytics: Analytics         // ❌ Must pass manually
) {
    val vm = viewModel { ProfileViewModel(repository, analytics) }
}
```

**With Hilt:**
```kotlin
@HiltViewModel
class ProfileViewModel @Inject constructor(
    private val repository: UserRepository,
    private val analytics: Analytics
) : ViewModel()

@Composable
fun ProfileScreen() {
    val vm = hiltViewModel<ProfileViewModel>()  // ✅ Auto-injected
}
```

**Whitehall with Hilt:**
```whitehall
<!-- stores/UserProfile.wh -->
@store
@HiltViewModel
class UserProfile @Inject constructor(
  private val repository: ProfileRepository,
  private val analytics: Analytics
) {
  var name = ""
  suspend fun save() {
    repository.save(name)
    analytics.track("save")
  }
}

<!-- screens/ProfileScreen.wh -->
<script>
  val profile = UserProfile()  // Auto-detects @HiltViewModel!
</script>
```

**When to use Hilt:**
- ✅ Medium-to-large apps
- ✅ Many dependencies
- ✅ Need testability (swap implementations)
- ✅ Team development

**When to skip Hilt:**
- Simple apps
- Prototyping
- Learning
- Don't want build overhead

---

## Full Examples

### Example 1: Simple Counter (Local State)

```whitehall
<!-- Counter.wh -->
<script>
  var count = 0
</script>

<Column spacing={8}>
  <Text fontSize={24}>Count: {count}</Text>
  <Row spacing={8}>
    <Button onClick={() => count--}>-</Button>
    <Button onClick={() => count++}>+</Button>
    <Button onClick={() => count = 0}>Reset</Button>
  </Row>
</Column>
```

**Transpiles to:**
```kotlin
@Composable
fun Counter() {
    var count by remember { mutableStateOf(0) }

    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text("Count: $count", fontSize = 24.sp)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = { count-- }) { Text("-") }
            Button(onClick = { count++ }) { Text("+") }
            Button(onClick = { count = 0 }) { Text("Reset") }
        }
    }
}
```

---

### Example 2: Login Form (Hoisted State)

```whitehall
<!-- LoginForm.wh -->
<script>
  @prop val onLogin: (String, String) -> Unit

  var email = ""
  var password = ""
</script>

<Column spacing={16}>
  <Input bind:value={email} label="Email" keyboardType="email" />
  <Input bind:value={password} label="Password" type="password" />
  <Button onClick={() => onLogin(email, password)}>Login</Button>
</Column>
```

```whitehall
<!-- LoginScreen.wh -->
<script>
  import com.app.components.LoginForm

  fun handleLogin(email: String, password: String) {
    // Handle login
  }
</script>

<Scaffold>
  <LoginForm onLogin={handleLogin} />
</Scaffold>
```

---

### Example 3: Profile Screen (Store)

```whitehall
<!-- stores/UserProfile.wh -->
@store
class UserProfile {
  var name = ""
  var email = ""
  var isLoading = false
  var error: String? = null

  val canSave: Boolean
    get() = name.isNotEmpty() && !isLoading

  suspend fun save() {
    if (!canSave) return

    isLoading = true
    error = null
    try {
      repository.saveProfile(name, email)
      isLoading = false
    } catch (e: Exception) {
      error = e.message
      isLoading = false
    }
  }

  fun clearError() {
    error = null
  }
}
```

```whitehall
<!-- screens/ProfileScreen.wh -->
<script>
  import com.app.stores.UserProfile

  val profile = UserProfile()  // Auto-detected as @store
</script>

<Scaffold>
  @if (profile.isLoading) {
    <Box fillMaxSize contentAlignment={Alignment.Center}>
      <CircularProgressIndicator />
    </Box>
  } @else {
    <Column padding={16} spacing={12}>
      <Text fontSize={24}>Profile</Text>

      <Input
        bind:value={profile.name}
        label="Name"
        disabled={profile.isLoading}
      />

      <Input
        bind:value={profile.email}
        label="Email"
        disabled={profile.isLoading}
      />

      @if (profile.error) {
        <Card backgroundColor={Color.Red.copy(alpha = 0.1f)}>
          <Row padding={8} spacing={8}>
            <Icon name="error" tint={Color.Red} />
            <Text color={Color.Red} modifier={Modifier.weight(1f)}>
              {profile.error}
            </Text>
            <IconButton onClick={profile::clearError}>
              <Icon name="close" />
            </IconButton>
          </Row>
        </Card>
      }

      <Button
        onClick={profile::save}
        disabled={!profile.canSave}
      >
        @if (profile.isLoading) {
          <CircularProgressIndicator size={20} />
        } @else {
          <Text>Save</Text>
        }
      </Button>
    </Column>
  }
</Scaffold>
```

**Transpiles to:**

`UserProfile.kt`:
```kotlin
class UserProfile : ViewModel() {
    data class UiState(
        val name: String = "",
        val email: String = "",
        val isLoading: Boolean = false,
        val error: String? = null
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

    var error: String?
        get() = _uiState.value.error
        set(value) { _uiState.update { it.copy(error = value) } }

    val canSave: Boolean
        get() = _uiState.value.name.isNotEmpty() && !_uiState.value.isLoading

    fun save() {
        if (!canSave) return

        viewModelScope.launch {
            isLoading = true
            error = null
            try {
                repository.saveProfile(name, email)
                isLoading = false
            } catch (e: Exception) {
                error = e.message
                isLoading = false
            }
        }
    }

    fun clearError() {
        error = null
    }
}
```

`ProfileScreen.kt`:
```kotlin
@Composable
fun ProfileScreen() {
    val profile = viewModel<UserProfile>()
    val uiState by profile.uiState.collectAsState()

    Scaffold { padding ->
        if (uiState.isLoading) {
            Box(
                modifier = Modifier.fillMaxSize().padding(padding),
                contentAlignment = Alignment.Center
            ) {
                CircularProgressIndicator()
            }
        } else {
            Column(
                modifier = Modifier.padding(padding).padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Text("Profile", fontSize = 24.sp)

                OutlinedTextField(
                    value = uiState.name,
                    onValueChange = { profile.name = it },
                    label = { Text("Name") },
                    enabled = !uiState.isLoading
                )

                OutlinedTextField(
                    value = uiState.email,
                    onValueChange = { profile.email = it },
                    label = { Text("Email") },
                    enabled = !uiState.isLoading
                )

                if (uiState.error != null) {
                    Card(backgroundColor = Color.Red.copy(alpha = 0.1f)) {
                        Row(
                            modifier = Modifier.padding(8.dp),
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            Icon(
                                imageVector = Icons.Default.Error,
                                contentDescription = null,
                                tint = Color.Red
                            )
                            Text(
                                text = uiState.error!!,
                                color = Color.Red,
                                modifier = Modifier.weight(1f)
                            )
                            IconButton(onClick = { profile.clearError() }) {
                                Icon(Icons.Default.Close, contentDescription = null)
                            }
                        }
                    }
                }

                Button(
                    onClick = { profile.save() },
                    enabled = profile.canSave
                ) {
                    if (uiState.isLoading) {
                        CircularProgressIndicator(modifier = Modifier.size(20.dp))
                    } else {
                        Text("Save")
                    }
                }
            }
        }
    }
}
```

---

### Example 4: Profile with Hilt DI

```whitehall
<!-- stores/UserProfile.wh -->
@store
@HiltViewModel
class UserProfile @Inject constructor(
  private val repository: ProfileRepository,
  private val analytics: Analytics
) {
  var name = ""
  var isLoading = false

  init {
    loadProfile()
  }

  private fun loadProfile() {
    viewModelScope.launch {
      isLoading = true
      name = repository.getCurrentUser().name
      isLoading = false
    }
  }

  suspend fun save() {
    isLoading = true
    repository.updateUser(name)
    analytics.trackEvent("profile_saved")
    isLoading = false
  }
}
```

```whitehall
<!-- screens/ProfileScreen.wh -->
<script>
  import com.app.stores.UserProfile

  val profile = UserProfile()  // Auto-detects @HiltViewModel!
</script>

<Column>
  <Input bind:value={profile.name} />
  <Button onClick={profile::save}>Save</Button>
</Column>
```

**Hilt setup (Kotlin files):**

```kotlin
// di/AppModule.kt
@Module
@InstallIn(SingletonComponent::class)
object AppModule {
    @Provides
    @Singleton
    fun provideProfileRepository(): ProfileRepository = ProfileRepositoryImpl()

    @Provides
    @Singleton
    fun provideAnalytics(): Analytics = FirebaseAnalytics()
}

// MyApplication.kt
@HiltAndroidApp
class MyApplication : Application()
```

---

## Summary

### What Works Today
- ✅ Local state: `var count = 0`
- ✅ Props: `@prop val name: String`
- ✅ Two-way binding: `bind:value={email}`
- ✅ Derived values: `val doubled = count * 2`
- ✅ Manual StateFlow (Kotlin files)
- ✅ Manual ViewModels (Kotlin files)
- ✅ Effects: `LaunchedEffect`, `DisposableEffect`

### Decided - Needs Implementation
1. **@store annotation** for screen-level reactive state (class definition only)
2. **Auto-detection** - `val profile = UserProfile()` automatically uses `viewModel<T>()` when `UserProfile` has `@store`
3. **Optional explicit annotation** - `@store val profile = ...` for clarity (same behavior)
4. **Auto-detect Hilt** - Classes with `@HiltViewModel` automatically use `hiltViewModel<T>()`
5. **stores/** directory convention (recommended, not required)
6. **Callable references** (`profile::save`)
7. **Lifecycle hooks** (`onMount`, `onDispose`)
8. **Multiple store instances** (auto-keyed by variable name)
9. **All @annotations lowercase** (`@store`, `@prop`, not `@Store`)
10. **Persistence** - Manual (no special syntax)

### Open Questions Needing Decisions
1. **Lifecycle hook naming:** `onDestroy` vs `onDispose` (recommendation: `onDispose`)
2. **Global stores:** Svelte-style singleton exports vs Kotlin repositories

---

**Next Steps:**
1. Implement Phase 1 (basic @store generation) from Implementation Plan
2. Decide: `onDestroy` vs `onDispose`
3. Decide: Global store pattern (Option A or B)
