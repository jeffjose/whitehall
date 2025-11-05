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
| **Stores (screen-level)** | **✅ DECIDED - Not Implemented** | `@store val profile = UserProfile()` |
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

**Decision:** Use `@Store` annotation to generate ViewModel boilerplate from simple class definitions.

**Syntax:**

**Define a store:**
```whitehall
<!-- stores/UserProfile.wh -->
@Store
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

  @store val profile = UserProfile()
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
- ✅ Explicit instantiation with `@store` annotation
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

**Decision:** Store files live in `stores/` directory, plain data models in `models/` or `data/`.

**Project structure:**
```
src/
  stores/              ← Reactive stores (@Store)
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

---

## Implementation Plan

### Phase 1: Basic Store Generation

**Goal:** Generate ViewModel boilerplate from `@Store` classes.

**Input:** `stores/UserProfile.wh`
```whitehall
@Store
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
1. Parse `@Store` annotation on class
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

  @store val profile = UserProfile()
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
1. Detect `@store val profile = UserProfile()` pattern
2. Generate `val profile = viewModel<UserProfile>()`
3. Generate `val uiState by profile.uiState.collectAsState()`
4. Rewrite references:
   - `profile.name` (in expressions) → `uiState.name`
   - `profile.name = value` (in assignments) → `profile.name = value`
   - `profile::save` → `{ profile.save() }` or direct reference

---

### Phase 3: Derived Properties

**Input:**
```whitehall
@Store
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
@Store
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
@Store
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
  @hiltViewModel val profile = UserProfile()
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
val profile = hiltViewModel<UserProfile>()
```

**Implementation:**
1. Preserve `@HiltViewModel` and `@Inject constructor()`
2. Detect `@hiltViewModel` annotation on usage
3. Generate `hiltViewModel<T>()` instead of `viewModel<T>()`

---

## Open Questions

### Question 1: Lifecycle Hooks vs Direct Compose

**Option A: Use Compose directly (Current decision)**
```whitehall
<script>
  LaunchedEffect(Unit) {
    analytics.trackScreenView("profile")
  }

  DisposableEffect(Unit) {
    val subscription = eventBus.subscribe()
    onDispose { subscription.cancel() }
  }
</script>
```

**Option B: Add lifecycle hooks**
```whitehall
<script>
  onMount {
    analytics.trackScreenView("profile")
  }

  onDispose {
    subscription.cancel()
  }
</script>
```

**Trade-offs:**
- Option A: ✅ No new syntax, standard Compose | ❌ More verbose
- Option B: ✅ Cleaner, familiar to web devs | ❌ More syntax to learn

**Decision needed:** Stick with Option A or add Option B sugar?

---

### Question 2: Global vs Screen-Scoped Stores

**Current:** All `@store` instances are screen-scoped (cleared on navigation away)

**Question:** How to support app-wide stores?

**Option A: Scope parameter**
```whitehall
@store(scope = "global") val settings = AppSettings()
```

**Option B: Different annotation**
```whitehall
@globalStore val settings = AppSettings()
```

**Option C: Use Repository pattern (Kotlin files)**
```kotlin
// kotlin/repositories/SettingsRepository.kt
object SettingsRepository {
    private val _theme = MutableStateFlow(Theme.Light)
    val theme: StateFlow<Theme> = _theme.asStateFlow()
}
```

```whitehall
<script>
  import com.app.repositories.SettingsRepository

  val theme by SettingsRepository.theme.collectAsState()
</script>
```

**Decision needed:** Which approach for global state?

---

### Question 3: Persistence

**Question:** How to persist store state across app restarts?

**Option A: Manual (current)**
```whitehall
@Store
class Settings {
  var darkMode = false

  init {
    loadFromDataStore()
  }

  fun save() {
    saveToDataStore()
  }
}
```

**Option B: Annotation-based**
```whitehall
@Store
class Settings {
  @Persisted var darkMode = false  // Auto-saves to DataStore
}
```

**Option C: Helper function**
```whitehall
@Store
class Settings {
  var darkMode by persisted("dark_mode", false)
}
```

**Decision needed:** Phase 1 (manual), Phase 2+ (helper)?

---

### Question 4: Multiple Store Instances

**Question:** Can a screen have multiple instances of the same store?

**Current design:** No, each `@store val profile = UserProfile()` creates one instance.

**Potential need:**
```whitehall
<script>
  @store val adminProfile = UserProfile()
  @store val guestProfile = UserProfile()
</script>
```

**Problem:** Both would generate same `viewModel<UserProfile>()`, only one instance created.

**Options:**
- A: Don't support (force different store classes)
- B: Support with keys: `@store(key = "admin") val adminProfile = UserProfile()`
- C: Generate different instances automatically based on variable name

**Decision needed:** Support or not?

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
@Store
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

  @store val profile = UserProfile()
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
@Store
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
  @hiltViewModel val profile = UserProfile()
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
@Store
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

  @store val profile = UserProfile()
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
@Store
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

  @hiltViewModel val profile = UserProfile()
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

### Decided but Not Implemented
- **@Store annotation** for screen-level reactive state
- **@store val profile = UserProfile()** usage syntax
- **stores/** directory convention
- **Callable references** (`profile::save`)

### Open Questions Needing Decisions
1. Lifecycle hooks (`onMount`) vs direct Compose (`LaunchedEffect`)
2. Global store support (annotation vs repository pattern)
3. Persistence strategy (manual, annotation, helper)
4. Multiple store instances support

---

**Next Steps:** Implement Phase 1 (basic store generation) from the Implementation Plan above.
