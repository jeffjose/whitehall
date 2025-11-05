# State Management in Whitehall

**Status:** 🚧 Design Document - Not Yet Implemented

---

## Quick Reference: Whitehall Support Status

| **Pattern** | **Status** | **Whitehall Syntax** |
|-------------|-----------|---------------------|
| Local state | ✅ Supported | `var count = 0` |
| Props | ✅ Supported | `@prop val name: String` |
| Two-way binding | ✅ Supported | `bind:value={email}` |
| Derived values | ✅ Supported | `val doubled = count * 2` |
| Hoisted state | ✅ Supported | Same as local state + props |
| StateFlow | ⚠️ Manual | Use Kotlin directly |
| ViewModel | ⚠️ Manual | Use Kotlin directly |
| Effects | ⚠️ Manual | Use `LaunchedEffect` directly |
| CompositionLocal | ⚠️ Manual | Use Kotlin directly |
| Lifecycle hooks | ❌ Not yet | Proposed: `onMount`, `onDispose` |
| Store files | ❌ Not yet | Proposed: `.store.wh` files |

**Legend:**
- ✅ **Supported** - Works today with clean syntax
- ⚠️ **Manual** - Works but requires Kotlin code
- ❌ **Not yet** - Future feature

---

## Table of Contents

1. [Background: Android/Kotlin State Management](#background-androidkotlin-state-management)
2. [Comparison: Svelte vs Android Patterns](#comparison-svelte-vs-android-patterns)
3. [Current State in Whitehall](#current-state-in-whitehall)
4. [Proposed Solutions](#proposed-solutions)
5. [Pattern Details & Examples](#pattern-details--examples)
6. [Shared State Strategies](#shared-state-strategies)
7. [Recommendations](#recommendations)

---

## Background: Android/Kotlin State Management

Android/Kotlin has evolved through several state management paradigms. Understanding these helps us design whitehall's approach.

### 1. Local Component State (Compose)

**What it is:** State scoped to a single composable, survives recomposition but not navigation.

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

**Whitehall syntax:** ✅ **Supported**

```whitehall
<!-- Counter.wh -->
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

**Key concepts:**
- `remember`: Preserves value across recompositions
- `mutableStateOf`: Creates observable state
- `by` delegate: Allows `count` to act like a regular variable
- When `count` changes, the composable automatically recomposes
- **Whitehall auto-wraps** `var` declarations in `remember { mutableStateOf() }`

**Lifecycle:**
- ✅ Survives recomposition
- ❌ Lost on configuration change (rotation)
- ❌ Lost on navigation away from screen
- ❌ Not shared between components

**Svelte equivalent:** Component-level variables
```svelte
<script>
  let count = 0; // Svelte 4
  // or
  let count = $state(0); // Svelte 5
</script>

<button on:click={() => count++}>
  Count: {count}
</button>
```

---

### 2. Hoisted State (State Lifting)

**What it is:** Moving state up to a parent component to share between siblings.

**Standard Kotlin/Compose:**
```kotlin
@Composable
fun Parent() {
    var text by remember { mutableStateOf("") }

    Column {
        InputField(
            value = text,
            onValueChange = { text = it }
        )
        DisplayText(text = text)
    }
}

@Composable
fun InputField(
    value: String,
    onValueChange: (String) -> Unit
) {
    TextField(
        value = value,
        onValueChange = onValueChange
    )
}

@Composable
fun DisplayText(text: String) {
    Text("You typed: $text")
}
```

**Whitehall syntax:** ✅ **Supported**

```whitehall
<!-- Parent.wh -->
<script>
  var text = ""
</script>

<Column>
  <InputField value={text} onValueChange={(newText) => text = newText} />
  <DisplayText text={text} />
</Column>
```

```whitehall
<!-- InputField.wh -->
<script>
  @prop val value: String
  @prop val onValueChange: (String) -> Unit
</script>

<TextField value={value} onValueChange={onValueChange} />
```

```whitehall
<!-- DisplayText.wh -->
<script>
  @prop val text: String
</script>

<Text>You typed: {text}</Text>
```

**Pattern:** Unidirectional data flow
- State flows down as props
- Events flow up as callbacks
- Parent owns the state

**Svelte equivalent:**
```svelte
<!-- Parent.svelte -->
<script>
  let text = '';
</script>

<InputField bind:value={text} />
<DisplayText {text} />

<!-- InputField.svelte -->
<script>
  export let value = '';
</script>
<input bind:value />

<!-- DisplayText.svelte -->
<script>
  export let text;
</script>
<p>You typed: {text}</p>
```

---

### 3. StateFlow (Reactive Streams)

**What it is:** Observable state container from Kotlin Coroutines, survives configuration changes and can be shared.

**Standard Kotlin:**
```kotlin
// Repository or ViewModel
class UserRepository {
    private val _user = MutableStateFlow<User?>(null)
    val user: StateFlow<User?> = _user.asStateFlow()

    fun updateUser(user: User) {
        _user.value = user
    }

    fun updateUserName(name: String) {
        _user.update { currentUser ->
            currentUser?.copy(name = name)
        }
    }
}

// In Composable
@Composable
fun ProfileScreen(repository: UserRepository) {
    val user by repository.user.collectAsState()

    Column {
        Text("Name: ${user?.name}")
        Button(onClick = {
            repository.updateUserName("New Name")
        }) {
            Text("Update")
        }
    }
}
```

**Whitehall syntax:** ⚠️ **Manual (Use Kotlin directly)**

```kotlin
// src/kotlin/repositories/UserRepository.kt (standard Kotlin file)
object UserRepository {
    private val _user = MutableStateFlow<User?>(null)
    val user: StateFlow<User?> = _user.asStateFlow()

    fun updateUser(user: User) {
        _user.value = user
    }

    fun updateUserName(name: String) {
        _user.update { it?.copy(name = name) }
    }
}
```

```whitehall
<!-- ProfileScreen.wh -->
<script>
  import com.app.repositories.UserRepository

  val user by UserRepository.user.collectAsState()

  fun handleUpdate() {
    UserRepository.updateUserName("New Name")
  }
</script>

<Column>
  <Text>Name: {user?.name}</Text>
  <Button onClick={handleUpdate}>Update</Button>
</Column>
```

**Note:** Repository/StateFlow code must be written in Kotlin files. The `.wh` file can consume it using standard `collectAsState()`.

**Key concepts:**
- `MutableStateFlow`: Hot observable that always has a value
- `asStateFlow()`: Read-only exposure
- `collectAsState()`: Converts Flow to Compose state
- Thread-safe, supports concurrent updates
- Fine-grained reactivity

**Lifecycle:**
- ✅ Survives recomposition
- ✅ Survives configuration changes (if in ViewModel/Repository)
- ✅ Can be shared across components
- ✅ Can be scoped to app lifecycle (singleton)

**Svelte equivalent:** Writable stores
```javascript
// stores.js
import { writable } from 'svelte/store';

export const user = writable(null);

// Usage
<script>
  import { user } from './stores';

  $: userName = $user?.name;

  function updateName() {
    user.update(u => ({ ...u, name: 'New Name' }));
  }
</script>

<p>Name: {userName}</p>
<button on:click={updateName}>Update</button>
```

**Svelte 5 equivalent:** Class with $state
```javascript
// stores.svelte.js
class UserStore {
  user = $state(null);

  updateName(name) {
    this.user = { ...this.user, name };
  }
}

export const userStore = new UserStore();
```

---

### 4. ViewModel Pattern

**What it is:** Screen-level state container that survives configuration changes, tied to navigation lifecycle.

**Standard Kotlin/Compose:**
```kotlin
// ViewModel
class ProfileViewModel : ViewModel() {
    data class UiState(
        val name: String = "",
        val email: String = "",
        val isLoading: Boolean = false,
        val error: String? = null
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    fun updateName(name: String) {
        _uiState.update { it.copy(name = name) }
    }

    fun saveProfile() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true) }
            try {
                repository.saveProfile(_uiState.value.name, _uiState.value.email)
                _uiState.update { it.copy(isLoading = false) }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, error = e.message) }
            }
        }
    }
}

// Screen
@Composable
fun ProfileScreen(
    viewModel: ProfileViewModel = viewModel()
) {
    val uiState by viewModel.uiState.collectAsState()

    Column {
        TextField(
            value = uiState.name,
            onValueChange = viewModel::updateName,
            label = { Text("Name") }
        )

        TextField(
            value = uiState.email,
            onValueChange = viewModel::updateEmail,
            label = { Text("Email") }
        )

        if (uiState.isLoading) {
            CircularProgressIndicator()
        }

        uiState.error?.let { error ->
            Text(error, color = Color.Red)
        }

        Button(
            onClick = viewModel::saveProfile,
            enabled = !uiState.isLoading
        ) {
            Text("Save")
        }
    }
}
```

**Whitehall syntax:** ⚠️ **Manual (Use Kotlin for ViewModel)**

```kotlin
// src/kotlin/viewmodels/ProfileViewModel.kt (standard Kotlin file)
class ProfileViewModel : ViewModel() {
    data class UiState(
        val name: String = "",
        val email: String = "",
        val isLoading: Boolean = false,
        val error: String? = null
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    fun updateName(name: String) {
        _uiState.update { it.copy(name = name) }
    }

    fun saveProfile() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true) }
            try {
                repository.saveProfile(_uiState.value.name, _uiState.value.email)
                _uiState.update { it.copy(isLoading = false) }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, error = e.message) }
            }
        }
    }
}
```

```whitehall
<!-- ProfileScreen.wh -->
<script>
  val viewModel: ProfileViewModel = viewModel()
  val uiState by viewModel.uiState.collectAsState()
</script>

<Column>
  <Input
    value={uiState.name}
    onValueChange={viewModel::updateName}
    label="Name"
  />

  <Input
    value={uiState.email}
    onValueChange={viewModel::updateEmail}
    label="Email"
  />

  @if (uiState.isLoading) {
    <CircularProgressIndicator />
  }

  @if (uiState.error != null) {
    <Text color={Color.Red}>{uiState.error}</Text>
  }

  <Button
    onClick={viewModel::saveProfile}
    disabled={uiState.isLoading}
  >
    Save
  </Button>
</Column>
```

**Note:** ViewModels are written in standard Kotlin files, then consumed in `.wh` files using `viewModel()` and `collectAsState()`.

**Lifecycle:**
- ✅ Survives configuration changes (rotation)
- ✅ Scoped to navigation destination
- ✅ Cleared when navigating away
- ✅ Handles coroutines/async work
- ✅ Survives process death with `SavedStateHandle`

**Svelte equivalent:** Closest is page-level stores with context
```svelte
<!-- routes/profile/+page.svelte -->
<script>
  import { writable } from 'svelte/store';

  let uiState = $state({
    name: '',
    email: '',
    isLoading: false,
    error: null
  });

  async function saveProfile() {
    uiState.isLoading = true;
    try {
      await repository.saveProfile(uiState.name, uiState.email);
      uiState.isLoading = false;
    } catch (e) {
      uiState.error = e.message;
      uiState.isLoading = false;
    }
  }
</script>

<input bind:value={uiState.name} placeholder="Name" />
<input bind:value={uiState.email} placeholder="Email" />

{#if uiState.isLoading}
  <Spinner />
{/if}

{#if uiState.error}
  <p class="error">{uiState.error}</p>
{/if}

<button on:click={saveProfile} disabled={uiState.isLoading}>
  Save
</button>
```

---

### 5. CompositionLocal (Dependency Injection)

**What it is:** Implicit context passed down the composition tree without prop drilling.

**Standard Kotlin/Compose:**
```kotlin
// Define the local
data class AppTheme(
    val primary: Color,
    val background: Color
)

val LocalTheme = compositionLocalOf { AppTheme(Color.Blue, Color.White) }

// Provide at root
@Composable
fun App() {
    val theme = AppTheme(Color.Red, Color.Black)

    CompositionLocalProvider(LocalTheme provides theme) {
        Screen1()
    }
}

// Consume anywhere in the tree
@Composable
fun DeepNestedComponent() {
    val theme = LocalTheme.current

    Box(
        modifier = Modifier.background(theme.background)
    ) {
        Text("Hello", color = theme.primary)
    }
}
```

**Whitehall syntax:** ⚠️ **Manual (Use Kotlin directly)**

```kotlin
// src/kotlin/AppTheme.kt
data class AppTheme(
    val primary: Color,
    val background: Color
)

val LocalTheme = compositionLocalOf { AppTheme(Color.Blue, Color.White) }
```

```whitehall
<!-- App.wh -->
<script>
  val theme = AppTheme(Color.Red, Color.Black)
</script>

<CompositionLocalProvider values={arrayOf(LocalTheme provides theme)}>
  <Screen1 />
</CompositionLocalProvider>
```

```whitehall
<!-- DeepNestedComponent.wh -->
<script>
  val theme = LocalTheme.current
</script>

<Box backgroundColor={theme.background}>
  <Text color={theme.primary}>Hello</Text>
</Box>
```

**Note:** CompositionLocal definitions are in Kotlin files, usage in `.wh` files is the same as Compose.

**Common use cases:**
- Theme/styling
- Locale/language
- Navigation
- Dependency injection (repositories, services)

**Svelte equivalent:** Context API
```svelte
<!-- App.svelte -->
<script>
  import { setContext } from 'svelte';

  setContext('theme', {
    primary: 'red',
    background: 'black'
  });
</script>

<Screen1 />

<!-- DeepNestedComponent.svelte -->
<script>
  import { getContext } from 'svelte';

  const theme = getContext('theme');
</script>

<div style="background: {theme.background}">
  <p style="color: {theme.primary}">Hello</p>
</div>
```

---

### 6. Derived State

**What it is:** Computed values that automatically update when dependencies change.

**Standard Kotlin/Compose:**
```kotlin
@Composable
fun ShoppingCart() {
    var items by remember { mutableStateOf(listOf<Item>()) }

    // Automatically recomputes when items changes
    val total = items.sumOf { it.price }
    val itemCount = items.size
    val hasItems = items.isNotEmpty()

    Column {
        Text("Items: $itemCount")
        Text("Total: $$total")

        if (hasItems) {
            Button(onClick = { /* checkout */ }) {
                Text("Checkout")
            }
        }
    }
}
```

**Whitehall syntax:** ✅ **Supported (automatic)**

```whitehall
<!-- ShoppingCart.wh -->
<script>
  var items: List<Item> = listOf()

  // Derived values - automatically recompute when items changes
  val total = items.sumOf { it.price }
  val itemCount = items.size
  val hasItems = items.isNotEmpty()
</script>

<Column>
  <Text>Items: {itemCount}</Text>
  <Text>Total: ${total}</Text>

  @if (hasItems) {
    <Button onClick={/* checkout */}>
      Checkout
    </Button>
  }
</Column>
```

**Transpiles to:**
```kotlin
@Composable
fun ShoppingCart() {
    var items by remember { mutableStateOf(listOf<Item>()) }

    val total = items.sumOf { it.price }
    val itemCount = items.size
    val hasItems = items.isNotEmpty()

    Column {
        Text("Items: $itemCount")
        Text("Total: $$total")

        if (hasItems) {
            Button(onClick = { /* checkout */ }) {
                Text("Checkout")
            }
        }
    }
}
```

**Note:** Compose automatically tracks dependencies and recomputes derived values. No special syntax needed!

**With StateFlow:**
```kotlin
class CartViewModel : ViewModel() {
    private val _items = MutableStateFlow<List<Item>>(emptyList())
    val items: StateFlow<List<Item>> = _items.asStateFlow()

    // Derived flows
    val total: StateFlow<Double> = items.map { items ->
        items.sumOf { it.price }
    }.stateIn(viewModelScope, SharingStarted.Lazily, 0.0)

    val itemCount: StateFlow<Int> = items.map { it.size }
        .stateIn(viewModelScope, SharingStarted.Lazily, 0)
}
```

**Svelte equivalent:**
```svelte
<!-- Svelte 4 -->
<script>
  let items = [];

  $: total = items.reduce((sum, item) => sum + item.price, 0);
  $: itemCount = items.length;
  $: hasItems = items.length > 0;
</script>

<!-- Svelte 5 -->
<script>
  let items = $state([]);

  let total = $derived(items.reduce((sum, item) => sum + item.price, 0));
  let itemCount = $derived(items.length);
  let hasItems = $derived(items.length > 0);
</script>
```

---

### 7. Side Effects

**What it is:** Code that runs in response to state changes (API calls, subscriptions, logging).

**Standard Kotlin/Compose:**
```kotlin
@Composable
fun UserProfile(userId: String) {
    var user by remember { mutableStateOf<User?>(null) }

    // Runs once when component mounts
    LaunchedEffect(Unit) {
        println("Component mounted")
    }

    // Runs when userId changes
    LaunchedEffect(userId) {
        user = repository.loadUser(userId)
    }

    // Cleanup when component unmounts
    DisposableEffect(Unit) {
        val subscription = eventBus.subscribe()
        onDispose {
            subscription.cancel()
        }
    }

    // Runs every recomposition (use sparingly)
    SideEffect {
        analytics.trackScreenView("profile")
    }
}
```

**Whitehall syntax:** ⚠️ **Manual (Use Compose directly)**

```whitehall
<!-- UserProfile.wh -->
<script>
  @prop val userId: String

  var user: User? = null

  // Runs once when component mounts
  LaunchedEffect(Unit) {
    println("Component mounted")
  }

  // Runs when userId changes
  LaunchedEffect(userId) {
    user = repository.loadUser(userId)
  }

  // Cleanup when component unmounts
  DisposableEffect(Unit) {
    val subscription = eventBus.subscribe()
    onDispose {
      subscription.cancel()
    }
  }

  // Runs every recomposition (use sparingly)
  SideEffect {
    analytics.trackScreenView("profile")
  }
</script>

<Column>
  @if (user != null) {
    <Text>{user.name}</Text>
    <Text>{user.email}</Text>
  }
</Column>
```

**Transpiles to:**
```kotlin
@Composable
fun UserProfile(userId: String) {
    var user by remember { mutableStateOf<User?>(null) }

    LaunchedEffect(Unit) {
        println("Component mounted")
    }

    LaunchedEffect(userId) {
        user = repository.loadUser(userId)
    }

    DisposableEffect(Unit) {
        val subscription = eventBus.subscribe()
        onDispose {
            subscription.cancel()
        }
    }

    SideEffect {
        analytics.trackScreenView("profile")
    }

    Column {
        if (user != null) {
            Text(user!!.name)
            Text(user!!.email)
        }
    }
}
```

**Note:** Currently use Compose effects directly. Future consideration: Add syntax sugar like `onMount`, `onDispose`, etc.

**Svelte equivalent:**
```svelte
<!-- Svelte 4 -->
<script>
  export let userId;
  let user;

  // Runs when userId changes
  $: {
    loadUser(userId).then(u => user = u);
  }

  // Runs on mount
  import { onMount } from 'svelte';
  onMount(() => {
    console.log('Component mounted');

    return () => {
      console.log('Cleanup');
    };
  });
</script>

<!-- Svelte 5 -->
<script>
  export let userId;
  let user = $state(null);

  $effect(() => {
    loadUser(userId).then(u => user = u);
  });

  $effect(() => {
    console.log('Component mounted');

    return () => {
      console.log('Cleanup');
    };
  });
</script>
```

---

## Comparison: Svelte vs Android Patterns

| **Pattern** | **Svelte 4** | **Svelte 5 (Runes)** | **Android/Kotlin** | **Lifecycle** |
|-------------|--------------|----------------------|--------------------|---------------|
| Local state | `let count = 0` | `let count = $state(0)` | `var count by mutableStateOf(0)` | Component-scoped |
| Derived values | `$: doubled = count * 2` | `let doubled = $derived(count * 2)` | `val doubled = count * 2` | Auto-recomputes |
| Side effects | `$: { log(count) }` | `$effect(() => log(count))` | `LaunchedEffect(count) { log(count) }` | On dependency change |
| Shared state | `writable(0)` | `class Store { x = $state(0) }` | `MutableStateFlow(0)` | App/screen-scoped |
| Context | `setContext/getContext` | Same | `CompositionLocal` | Tree-scoped |
| Props | `export let name` | `let { name } = $props()` | `@prop val name: String` | Passed from parent |
| Two-way binding | `bind:value={text}` | Same | Manual (value + onChange) | N/A |

---

## Current State in Whitehall

Based on existing documentation, whitehall currently supports:

### ✅ Implemented

1. **Local state variables**
   ```whitehall
   <script>
     var email = ""
     var password = ""
   </script>
   ```

   **Transpiles to:**
   ```kotlin
   var email by remember { mutableStateOf("") }
   var password by remember { mutableStateOf("") }
   ```

2. **Props**
   ```whitehall
   <script>
     @prop val title: String = "Login"
   </script>
   ```

   **Transpiles to:**
   ```kotlin
   @Composable
   fun Component(
       title: String = "Login"
   ) {
       // ...
   }
   ```

3. **Two-way binding** (syntactic sugar)
   ```whitehall
   <Input bind:value={email} />
   ```

   **Transpiles to:**
   ```kotlin
   OutlinedTextField(
       value = email,
       onValueChange = { email = it }
   )
   ```

4. **Derived values** (automatic)
   ```whitehall
   <script>
     var count = 0
     val doubled = count * 2  // Automatically recomputes
   </script>
   ```

   **Transpiles to:**
   ```kotlin
   var count by remember { mutableStateOf(0) }
   val doubled = count * 2  // Compose handles reactivity
   ```

### ❌ Missing / Not Designed

1. **Shared state across components**
   - No stores/StateFlow integration pattern
   - No global state management

2. **ViewModel integration**
   - No guidance on how to use ViewModels
   - No screen-level state patterns

3. **Effects system**
   - No `onMount`, `onDispose`, or effect hooks
   - No `LaunchedEffect` sugar

4. **Context/dependency injection**
   - No CompositionLocal patterns
   - No built-in DI

5. **Persistence**
   - No saved state handling
   - No DataStore integration

---

## Proposed Solutions

### Option 1: Minimal Sugar (Recommended)

**Philosophy:** Keep whitehall syntax minimal, allow Kotlin/Compose patterns directly.

**Pros:**
- Simple to implement
- No new concepts to learn
- Idiomatic Kotlin
- Easy to drop down to pure Compose

**Cons:**
- More verbose than Svelte
- Less "magic"

**Example:**

```whitehall
<!-- LoginScreen.wh -->
<script>
  // Local state - auto-wrapped
  var email = ""
  var password = ""
  var isLoading = false

  // ViewModel - use directly
  val viewModel: AuthViewModel = viewModel()
  val authState by viewModel.authState.collectAsState()

  // Effects - use Compose syntax
  LaunchedEffect(authState.isLoggedIn) {
    if (authState.isLoggedIn) {
      navigator.navigate("home")
    }
  }

  fun handleLogin() {
    isLoading = true
    viewModel.login(email, password)
  }
</script>

<Column padding={16}>
  <Input
    bind:value={email}
    label="Email"
    keyboardType="email"
  />

  <Input
    bind:value={password}
    label="Password"
    type="password"
  />

  @if (authState.error) {
    <Text color={Color.Red}>{authState.error}</Text>
  }

  <Button
    onClick={handleLogin}
    disabled={isLoading || authState.isLoading}
  >
    @if (isLoading || authState.isLoading) {
      <CircularProgressIndicator />
    } @else {
      <Text>Login</Text>
    }
  </Button>
</Column>
```

**Transpiles to:**

```kotlin
@Composable
fun LoginScreen(
    viewModel: AuthViewModel = viewModel()
) {
    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var isLoading by remember { mutableStateOf(false) }

    val authState by viewModel.authState.collectAsState()

    LaunchedEffect(authState.isLoggedIn) {
        if (authState.isLoggedIn) {
            navigator.navigate("home")
        }
    }

    fun handleLogin() {
        isLoading = true
        viewModel.login(email, password)
    }

    Column(modifier = Modifier.padding(16.dp)) {
        OutlinedTextField(
            value = email,
            onValueChange = { email = it },
            label = { Text("Email") },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email)
        )

        OutlinedTextField(
            value = password,
            onValueChange = { password = it },
            label = { Text("Password") },
            visualTransformation = PasswordVisualTransformation()
        )

        if (authState.error != null) {
            Text(authState.error!!, color = Color.Red)
        }

        Button(
            onClick = { handleLogin() },
            enabled = !isLoading && !authState.isLoading
        ) {
            if (isLoading || authState.isLoading) {
                CircularProgressIndicator()
            } else {
                Text("Login")
            }
        }
    }
}
```

---

### Option 2: Svelte-Style Runes

**Philosophy:** Add explicit markers for reactive primitives, closer to Svelte 5.

**Pros:**
- Familiar to web developers
- Clear intent (`$state`, `$derived`, `$effect`)
- Potentially more readable

**Cons:**
- New syntax to learn
- More work to implement
- Might confuse Kotlin developers
- Doesn't feel Kotlin-native

**Example:**

```whitehall
<!-- LoginScreen.wh -->
<script>
  $state var email = ""
  $state var password = ""
  $state var isLoading = false

  val viewModel: AuthViewModel = viewModel()
  $state val authState = viewModel.authState.collectAsState()

  $derived val canSubmit = email.isNotEmpty() && password.length >= 6

  $effect {
    if (authState.isLoggedIn) {
      navigator.navigate("home")
    }
  }

  fun handleLogin() {
    isLoading = true
    viewModel.login(email, password)
  }
</script>

<Column padding={16}>
  <Input bind:value={email} label="Email" />
  <Input bind:value={password} label="Password" type="password" />

  <Button onClick={handleLogin} disabled={!canSubmit}>
    Login
  </Button>
</Column>
```

**Transpiles to:**

```kotlin
@Composable
fun LoginScreen(viewModel: AuthViewModel = viewModel()) {
    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var isLoading by remember { mutableStateOf(false) }

    val authState by viewModel.authState.collectAsState()

    val canSubmit = email.isNotEmpty() && password.length >= 6

    LaunchedEffect(authState.isLoggedIn) {
        if (authState.isLoggedIn) {
            navigator.navigate("home")
        }
    }

    fun handleLogin() {
        isLoading = true
        viewModel.login(email, password)
    }

    // ... rest of UI
}
```

---

### Option 3: Hybrid Approach

**Philosophy:** Auto-wrap simple cases, explicit syntax for complex cases.

**Example:**

```whitehall
<script>
  // Auto-wrapped - simple cases
  var count = 0
  val doubled = count * 2

  // Explicit - complex cases
  val viewModel: CounterViewModel = viewModel()
  val state by viewModel.uiState.collectAsState()

  LaunchedEffect(count) {
    if (count > 10) {
      analytics.trackEvent("high_count")
    }
  }
</script>
```

---

## Pattern Details & Examples

### Pattern 1: Local Component State

#### Whitehall (Proposed)

```whitehall
<!-- Counter.wh -->
<script>
  var count = 0

  fun increment() {
    count++
  }

  fun decrement() {
    count--
  }

  fun reset() {
    count = 0
  }
</script>

<Column spacing={8}>
  <Text fontSize={24}>Count: {count}</Text>

  <Row spacing={8}>
    <Button onClick={decrement}>-</Button>
    <Button onClick={increment}>+</Button>
    <Button onClick={reset}>Reset</Button>
  </Row>
</Column>
```

#### Transpiles To

```kotlin
@Composable
fun Counter() {
    var count by remember { mutableStateOf(0) }

    fun increment() {
        count++
    }

    fun decrement() {
        count--
    }

    fun reset() {
        count = 0
    }

    Column(
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Text("Count: $count", fontSize = 24.sp)

        Row(
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Button(onClick = { decrement() }) {
                Text("-")
            }
            Button(onClick = { increment() }) {
                Text("+")
            }
            Button(onClick = { reset() }) {
                Text("Reset")
            }
        }
    }
}
```

---

### Pattern 2: Hoisted State (Form Example)

#### Whitehall (Proposed)

```whitehall
<!-- LoginForm.wh -->
<script>
  @prop val onLogin: (String, String) -> Unit

  var email = ""
  var password = ""
  var emailError: String? = null
  var passwordError: String? = null

  fun validateAndSubmit() {
    emailError = if (email.contains("@")) null else "Invalid email"
    passwordError = if (password.length >= 6) null else "Password too short"

    if (emailError == null && passwordError == null) {
      onLogin(email, password)
    }
  }
</script>

<Column spacing={16}>
  <Input
    bind:value={email}
    label="Email"
    error={emailError}
    keyboardType="email"
  />

  <Input
    bind:value={password}
    label="Password"
    error={passwordError}
    type="password"
  />

  <Button onClick={validateAndSubmit}>
    Login
  </Button>
</Column>
```

```whitehall
<!-- LoginScreen.wh -->
<script>
  val viewModel: AuthViewModel = viewModel()

  fun handleLogin(email: String, password: String) {
    viewModel.login(email, password)
  }
</script>

<Scaffold>
  <Column padding={16}>
    <Text fontSize={28}>Welcome Back</Text>

    <LoginForm onLogin={handleLogin} />
  </Column>
</Scaffold>
```

#### Transpiles To

```kotlin
// LoginForm.kt
@Composable
fun LoginForm(
    onLogin: (String, String) -> Unit
) {
    var email by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var emailError by remember { mutableStateOf<String?>(null) }
    var passwordError by remember { mutableStateOf<String?>(null) }

    fun validateAndSubmit() {
        emailError = if (email.contains("@")) null else "Invalid email"
        passwordError = if (password.length >= 6) null else "Password too short"

        if (emailError == null && passwordError == null) {
            onLogin(email, password)
        }
    }

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        OutlinedTextField(
            value = email,
            onValueChange = { email = it },
            label = { Text("Email") },
            isError = emailError != null,
            supportingText = emailError?.let { { Text(it) } },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email)
        )

        OutlinedTextField(
            value = password,
            onValueChange = { password = it },
            label = { Text("Password") },
            isError = passwordError != null,
            supportingText = passwordError?.let { { Text(it) } },
            visualTransformation = PasswordVisualTransformation()
        )

        Button(onClick = { validateAndSubmit() }) {
            Text("Login")
        }
    }
}

// LoginScreen.kt
@Composable
fun LoginScreen(
    viewModel: AuthViewModel = viewModel()
) {
    fun handleLogin(email: String, password: String) {
        viewModel.login(email, password)
    }

    Scaffold { padding ->
        Column(
            modifier = Modifier
                .padding(padding)
                .padding(16.dp)
        ) {
            Text("Welcome Back", fontSize = 28.sp)

            LoginForm(onLogin = ::handleLogin)
        }
    }
}
```

---

### Pattern 3: ViewModel Integration

#### Whitehall (Proposed)

```whitehall
<!-- ProfileScreen.wh -->
<script>
  val viewModel: ProfileViewModel = viewModel()
  val uiState by viewModel.uiState.collectAsState()

  fun handleSave() {
    viewModel.saveProfile()
  }
</script>

<Scaffold>
  @if (uiState.isLoading) {
    <Box fillMaxSize contentAlignment={Alignment.Center}>
      <CircularProgressIndicator />
    </Box>
  } @else {
    <Column padding={16}>
      <Input
        value={uiState.name}
        onValueChange={viewModel::updateName}
        label="Name"
      />

      <Input
        value={uiState.email}
        onValueChange={viewModel::updateEmail}
        label="Email"
      />

      @if (uiState.error) {
        <Text color={Color.Red}>{uiState.error}</Text>
      }

      <Button
        onClick={handleSave}
        disabled={uiState.isSaving}
      >
        @if (uiState.isSaving) {
          <CircularProgressIndicator size={20} />
        } @else {
          <Text>Save</Text>
        }
      </Button>
    </Column>
  }
</Scaffold>
```

#### Corresponding ViewModel (Standard Kotlin)

```kotlin
// ProfileViewModel.kt
class ProfileViewModel(
    private val repository: ProfileRepository
) : ViewModel() {

    data class UiState(
        val name: String = "",
        val email: String = "",
        val isLoading: Boolean = true,
        val isSaving: Boolean = false,
        val error: String? = null
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    init {
        loadProfile()
    }

    private fun loadProfile() {
        viewModelScope.launch {
            try {
                val profile = repository.getProfile()
                _uiState.update { it.copy(
                    name = profile.name,
                    email = profile.email,
                    isLoading = false
                ) }
            } catch (e: Exception) {
                _uiState.update { it.copy(
                    isLoading = false,
                    error = e.message
                ) }
            }
        }
    }

    fun updateName(name: String) {
        _uiState.update { it.copy(name = name) }
    }

    fun updateEmail(email: String) {
        _uiState.update { it.copy(email = email) }
    }

    fun saveProfile() {
        viewModelScope.launch {
            _uiState.update { it.copy(isSaving = true, error = null) }
            try {
                repository.saveProfile(
                    name = _uiState.value.name,
                    email = _uiState.value.email
                )
                _uiState.update { it.copy(isSaving = false) }
            } catch (e: Exception) {
                _uiState.update { it.copy(
                    isSaving = false,
                    error = e.message
                ) }
            }
        }
    }
}
```

#### Transpiles To

```kotlin
@Composable
fun ProfileScreen(
    viewModel: ProfileViewModel = viewModel()
) {
    val uiState by viewModel.uiState.collectAsState()

    fun handleSave() {
        viewModel.saveProfile()
    }

    Scaffold { padding ->
        if (uiState.isLoading) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding),
                contentAlignment = Alignment.Center
            ) {
                CircularProgressIndicator()
            }
        } else {
            Column(
                modifier = Modifier
                    .padding(padding)
                    .padding(16.dp)
            ) {
                OutlinedTextField(
                    value = uiState.name,
                    onValueChange = viewModel::updateName,
                    label = { Text("Name") }
                )

                OutlinedTextField(
                    value = uiState.email,
                    onValueChange = viewModel::updateEmail,
                    label = { Text("Email") }
                )

                if (uiState.error != null) {
                    Text(uiState.error!!, color = Color.Red)
                }

                Button(
                    onClick = { handleSave() },
                    enabled = !uiState.isSaving
                ) {
                    if (uiState.isSaving) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(20.dp)
                        )
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

### Pattern 4: Effects (Side Effects)

#### Whitehall (Proposed - Option A: Direct Compose)

```whitehall
<!-- UserProfile.wh -->
<script>
  @prop val userId: String

  var user: User? = null
  var isLoading = true

  // Load user when userId changes
  LaunchedEffect(userId) {
    isLoading = true
    user = repository.loadUser(userId)
    isLoading = false
  }

  // Cleanup subscriptions
  DisposableEffect(userId) {
    val subscription = eventBus.subscribe(userId)
    onDispose {
      subscription.cancel()
    }
  }

  // Track screen view
  LaunchedEffect(Unit) {
    analytics.trackScreenView("user_profile", mapOf("userId" to userId))
  }
</script>

<Column>
  @if (isLoading) {
    <CircularProgressIndicator />
  } @else if (user != null) {
    <Text>{user.name}</Text>
    <Text>{user.email}</Text>
  }
</Column>
```

#### Whitehall (Proposed - Option B: Lifecycle Hooks)

```whitehall
<!-- UserProfile.wh -->
<script>
  @prop val userId: String

  var user: User? = null
  var isLoading = true

  // Syntax sugar for LaunchedEffect
  onMount {
    analytics.trackScreenView("user_profile")
  }

  // Re-run when userId changes
  onEffect(() => [userId]) {
    isLoading = true
    user = repository.loadUser(userId)
    isLoading = false
  }

  // Cleanup
  onDispose {
    subscription.cancel()
  }
</script>
```

#### Transpiles To

```kotlin
@Composable
fun UserProfile(userId: String) {
    var user by remember { mutableStateOf<User?>(null) }
    var isLoading by remember { mutableStateOf(true) }

    LaunchedEffect(userId) {
        isLoading = true
        user = repository.loadUser(userId)
        isLoading = false
    }

    DisposableEffect(userId) {
        val subscription = eventBus.subscribe(userId)
        onDispose {
            subscription.cancel()
        }
    }

    LaunchedEffect(Unit) {
        analytics.trackScreenView("user_profile", mapOf("userId" to userId))
    }

    Column {
        if (isLoading) {
            CircularProgressIndicator()
        } else if (user != null) {
            Text(user!!.name)
            Text(user!!.email)
        }
    }
}
```

---

## Shared State Strategies

### Strategy 1: StateFlow in Kotlin Files

**When to use:** App-wide state (user, theme, settings)

**Structure:**
```
src/
  kotlin/
    repositories/
      UserRepository.kt
      SettingsRepository.kt
  screens/
    ProfileScreen.wh
    SettingsScreen.wh
```

**Implementation:**

```kotlin
// src/kotlin/repositories/UserRepository.kt
object UserRepository {
    private val _currentUser = MutableStateFlow<User?>(null)
    val currentUser: StateFlow<User?> = _currentUser.asStateFlow()

    private val _isLoggedIn = MutableStateFlow(false)
    val isLoggedIn: StateFlow<Boolean> = _isLoggedIn.asStateFlow()

    suspend fun login(email: String, password: String) {
        val user = api.login(email, password)
        _currentUser.value = user
        _isLoggedIn.value = true
    }

    fun logout() {
        _currentUser.value = null
        _isLoggedIn.value = false
    }
}
```

**Usage in Whitehall:**

```whitehall
<!-- ProfileScreen.wh -->
<script>
  import com.app.repositories.UserRepository

  val user by UserRepository.currentUser.collectAsState()

  fun handleLogout() {
    UserRepository.logout()
  }
</script>

<Column>
  @if (user != null) {
    <Text>Hello, {user.name}!</Text>
    <Button onClick={handleLogout}>Logout</Button>
  }
</Column>
```

---

### Strategy 2: ViewModel with Dependency Injection (Hilt)

**When to use:** Screen-level state that needs access to repositories, APIs, or other services

---

#### The Problem: ViewModels Need Dependencies

ViewModels often need access to:
- Repositories (data layer)
- API clients
- Analytics services
- Database DAOs
- Other business logic

**Bad approach: Hardcoded dependencies**
```kotlin
class ProfileViewModel : ViewModel() {
    private val repository = UserRepository()  // ❌ Hardcoded - can't test or swap
    private val analytics = FirebaseAnalytics()  // ❌ Tightly coupled
}
```

**Problems:**
- Can't test (can't inject mocks)
- Can't swap implementations
- Violates dependency inversion principle
- Hard to maintain

---

#### Solution 1: Manual Dependency Injection

**Pass dependencies through constructor:**

```kotlin
// ProfileViewModel.kt
class ProfileViewModel(
    private val userRepository: UserRepository,
    private val analytics: Analytics
) : ViewModel() {

    val user: StateFlow<User?> = userRepository.currentUser
        .stateIn(viewModelScope, SharingStarted.Lazily, null)

    fun updateProfile(name: String) {
        viewModelScope.launch {
            userRepository.updateUser(name)
            analytics.trackEvent("profile_updated")
        }
    }
}
```

**Usage in Composable:**

```kotlin
@Composable
fun ProfileScreen(
    userRepository: UserRepository,
    analytics: Analytics
) {
    val viewModel = viewModel {
        ProfileViewModel(
            userRepository = userRepository,
            analytics = analytics
        )
    }

    val user by viewModel.user.collectAsState()

    // UI...
}
```

**Problems with manual DI:**
- **Prop drilling:** Every screen needs to pass dependencies down
- **Boilerplate:** Repetitive factory code
- **Error-prone:** Easy to forget a dependency
- **Hard to refactor:** Adding a new dependency requires updating every call site

---

#### Solution 2: Hilt (Automated Dependency Injection)

**What is Hilt?**

Hilt is Google's recommended dependency injection library for Android, built on top of Dagger. It automatically:
1. Creates instances of your dependencies
2. Manages their lifecycles
3. Injects them where needed
4. Handles scoping (singleton, per-activity, per-viewmodel, etc.)

**How it works:**

1. **Define what can be injected** (tell Hilt how to create instances)
2. **Mark where to inject** (ViewModels, Repositories, etc.)
3. **Hilt generates code** to wire everything together
4. **Access dependencies** without manual passing

---

#### Hilt Setup & Usage

**1. Add Hilt dependencies** (in `build.gradle`):

```gradle
plugins {
    id 'com.google.dagger.hilt.android'
    id 'kotlin-kapt'
}

dependencies {
    implementation 'com.google.dagger:hilt-android:2.48'
    kapt 'com.google.dagger:hilt-compiler:2.48'
    implementation 'androidx.hilt:hilt-navigation-compose:1.1.0'
}
```

**2. Mark your Application class:**

```kotlin
@HiltAndroidApp
class MyApplication : Application()
```

**3. Provide dependencies** (tell Hilt how to create them):

```kotlin
// di/AppModule.kt
@Module
@InstallIn(SingletonComponent::class)
object AppModule {

    @Provides
    @Singleton
    fun provideUserRepository(
        api: ApiClient,
        database: AppDatabase
    ): UserRepository {
        return UserRepositoryImpl(api, database)
    }

    @Provides
    @Singleton
    fun provideAnalytics(): Analytics {
        return FirebaseAnalytics()
    }

    @Provides
    @Singleton
    fun provideApiClient(): ApiClient {
        return Retrofit.Builder()
            .baseUrl("https://api.example.com")
            .build()
            .create(ApiClient::class.java)
    }
}
```

**4. Mark ViewModel for injection:**

```kotlin
// ProfileViewModel.kt
@HiltViewModel
class ProfileViewModel @Inject constructor(
    private val userRepository: UserRepository,
    private val analytics: Analytics
) : ViewModel() {

    val user: StateFlow<User?> = userRepository.currentUser
        .stateIn(viewModelScope, SharingStarted.Lazily, null)

    fun updateProfile(name: String) {
        viewModelScope.launch {
            userRepository.updateUser(name)
            analytics.trackEvent("profile_updated")
        }
    }
}
```

**Key annotations:**
- `@HiltViewModel` - Marks this as a Hilt-managed ViewModel
- `@Inject constructor()` - Tells Hilt to inject these dependencies

**5. Use in Composable:**

```kotlin
@Composable
fun ProfileScreen(
    viewModel: ProfileViewModel = hiltViewModel()
) {
    val user by viewModel.user.collectAsState()

    Column {
        Text("User: ${user?.name}")
        Button(onClick = { viewModel.updateProfile("New Name") }) {
            Text("Update")
        }
    }
}
```

**That's it!** No manual dependency passing. Hilt handles everything.

---

#### Comparison: Manual DI vs Hilt

**Manual DI:**
```kotlin
@Composable
fun ProfileScreen(
    userRepository: UserRepository,  // ❌ Must pass manually
    analytics: Analytics,            // ❌ Must pass manually
    database: AppDatabase,           // ❌ Needed for nested deps
    apiClient: ApiClient             // ❌ Needed for nested deps
) {
    val viewModel = viewModel {
        ProfileViewModel(
            userRepository = UserRepository(apiClient, database),
            analytics = analytics
        )
    }
}

// Caller must provide everything:
ProfileScreen(
    userRepository = UserRepository(apiClient, database),
    analytics = FirebaseAnalytics(),
    database = appDatabase,
    apiClient = retrofitClient
)
```

**With Hilt:**
```kotlin
@Composable
fun ProfileScreen(
    viewModel: ProfileViewModel = hiltViewModel()  // ✅ Auto-injected
) {
    // Just use it!
}

// Caller doesn't need to know about dependencies:
ProfileScreen()
```

---

#### Hilt Scopes

Hilt manages object lifecycles with scopes:

| **Scope** | **Lifetime** | **Use Case** |
|-----------|--------------|--------------|
| `@Singleton` | App lifetime | Repositories, API clients, Database |
| `@ActivityScoped` | Activity lifetime | Activity-specific services |
| `@ViewModelScoped` | ViewModel lifetime | ViewModel-specific helpers |

**Example:**

```kotlin
@Module
@InstallIn(SingletonComponent::class)
object AppModule {
    @Provides
    @Singleton  // ✅ One instance for entire app
    fun provideDatabase(): AppDatabase {
        return Room.databaseBuilder(...)
    }
}

@Module
@InstallIn(ViewModelComponent::class)
object ViewModelModule {
    @Provides
    @ViewModelScoped  // ✅ New instance per ViewModel
    fun provideUseCaseHelper(): UseCaseHelper {
        return UseCaseHelper()
    }
}
```

---

#### Whitehall Integration

**Standard Kotlin/Compose with Hilt:**

```kotlin
// ProfileViewModel.kt
@HiltViewModel
class ProfileViewModel @Inject constructor(
    private val userRepository: UserRepository,
    private val analytics: Analytics
) : ViewModel() {
    val user = userRepository.currentUser
        .stateIn(viewModelScope, SharingStarted.Lazily, null)

    fun updateProfile(name: String) {
        viewModelScope.launch {
            userRepository.updateUser(name)
            analytics.trackEvent("profile_updated")
        }
    }
}

// ProfileScreen.kt
@Composable
fun ProfileScreen(
    viewModel: ProfileViewModel = hiltViewModel()
) {
    val user by viewModel.user.collectAsState()

    Column {
        Text("Name: ${user?.name}")
        Button(onClick = { viewModel.updateProfile("New") }) {
            Text("Update")
        }
    }
}
```

**Whitehall syntax:**

```whitehall
<!-- ProfileScreen.wh -->
<script>
  val viewModel: ProfileViewModel = hiltViewModel()
  val user by viewModel.user.collectAsState()

  fun handleUpdate() {
    viewModel.updateProfile("New Name")
  }
</script>

<Column>
  <Text>Name: {user?.name}</Text>
  <Button onClick={handleUpdate}>Update</Button>
</Column>
```

**Note:** ViewModel definition still requires Kotlin file with `@HiltViewModel` annotation. Hilt modules also defined in Kotlin.

---

#### Pros & Cons

**Pros:**
- ✅ No prop drilling
- ✅ Easy to test (swap implementations)
- ✅ Type-safe
- ✅ Compile-time validation
- ✅ Industry standard (Google recommended)
- ✅ Automatic lifecycle management

**Cons:**
- ❌ Steeper learning curve
- ❌ More setup (annotations, modules)
- ❌ Longer build times (annotation processing)
- ❌ Cryptic error messages if misconfigured
- ❌ Overkill for simple apps

---

#### When to Use Hilt vs Manual DI

**Use Manual DI when:**
- Simple app with few dependencies
- Prototyping/learning
- Want full control over object creation
- Don't want build-time overhead

**Use Hilt when:**
- Medium-to-large app
- Many dependencies
- Multiple layers (data, domain, presentation)
- Team development
- Need testability

---

#### Full Example: Profile Screen with Hilt

**Hilt Setup:**

```kotlin
// di/AppModule.kt
@Module
@InstallIn(SingletonComponent::class)
object AppModule {

    @Provides
    @Singleton
    fun provideUserRepository(
        api: ApiClient,
        database: UserDao
    ): UserRepository = UserRepositoryImpl(api, database)

    @Provides
    @Singleton
    fun provideAnalytics(): Analytics = FirebaseAnalytics()

    @Provides
    @Singleton
    fun provideApiClient(): ApiClient {
        return Retrofit.Builder()
            .baseUrl("https://api.example.com")
            .addConverterFactory(GsonConverterFactory.create())
            .build()
            .create(ApiClient::class.java)
    }

    @Provides
    @Singleton
    fun provideDatabase(@ApplicationContext context: Context): AppDatabase {
        return Room.databaseBuilder(
            context,
            AppDatabase::class.java,
            "app-database"
        ).build()
    }

    @Provides
    fun provideUserDao(database: AppDatabase): UserDao {
        return database.userDao()
    }
}
```

**ViewModel:**

```kotlin
// ProfileViewModel.kt
@HiltViewModel
class ProfileViewModel @Inject constructor(
    private val userRepository: UserRepository,
    private val analytics: Analytics
) : ViewModel() {

    data class UiState(
        val user: User? = null,
        val isLoading: Boolean = true,
        val error: String? = null
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    init {
        loadUser()
    }

    private fun loadUser() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true) }
            try {
                userRepository.currentUser.collect { user ->
                    _uiState.update { it.copy(user = user, isLoading = false) }
                }
            } catch (e: Exception) {
                _uiState.update { it.copy(error = e.message, isLoading = false) }
            }
        }
    }

    fun updateProfile(name: String) {
        viewModelScope.launch {
            try {
                userRepository.updateUser(name)
                analytics.trackEvent("profile_updated")
            } catch (e: Exception) {
                _uiState.update { it.copy(error = e.message) }
            }
        }
    }
}
```

**Whitehall Screen:**

```whitehall
<!-- ProfileScreen.wh -->
<script>
  val viewModel: ProfileViewModel = hiltViewModel()
  val uiState by viewModel.uiState.collectAsState()
</script>

<Scaffold>
  @if (uiState.isLoading) {
    <Box fillMaxSize contentAlignment={Alignment.Center}>
      <CircularProgressIndicator />
    </Box>
  } @else if (uiState.error != null) {
    <Box fillMaxSize contentAlignment={Alignment.Center}>
      <Text color={Color.Red}>Error: {uiState.error}</Text>
    </Box>
  } @else {
    <Column padding={16}>
      <Text fontSize={24}>Profile</Text>

      <Input
        value={uiState.user?.name ?: ""}
        onValueChange={(name) => viewModel.updateProfile(name)}
        label="Name"
      />

      <Text>Email: {uiState.user?.email}</Text>
    </Column>
  }
</Scaffold>
```

**Application:**

```kotlin
// MyApplication.kt
@HiltAndroidApp
class MyApplication : Application()
```

**That's it!** Hilt handles creating and injecting:
- `UserRepository` (with `ApiClient` and `UserDao`)
- `Analytics`
- `AppDatabase`
- Everything needed

---

### Strategy 3: CompositionLocal for Tree-Scoped State

**When to use:** Theme, navigation, services needed deep in tree

**Implementation:**

```kotlin
// AppDependencies.kt
data class AppServices(
    val userRepository: UserRepository,
    val analytics: Analytics,
    val navigator: Navigator
)

val LocalAppServices = compositionLocalOf<AppServices> {
    error("No AppServices provided")
}
```

**Provide at root:**

```whitehall
<!-- App.wh -->
<script>
  val services = AppServices(
    userRepository = UserRepository,
    analytics = Analytics(),
    navigator = rememberNavigator()
  )
</script>

<CompositionLocalProvider values={arrayOf(LocalAppServices provides services)}>
  <MainScreen />
</CompositionLocalProvider>
```

**Consume anywhere:**

```whitehall
<!-- DeepNestedComponent.wh -->
<script>
  val services = LocalAppServices.current
  val user by services.userRepository.currentUser.collectAsState()

  fun handleClick() {
    services.analytics.trackEvent("button_click")
    services.navigator.navigate("details")
  }
</script>
```

---

### Strategy 4: Whitehall Store Files (Future Idea)

**Proposed:** Special `.store.wh` or `.state.wh` files

```whitehall
<!-- stores/user.store.wh -->
export class UserStore {
  private val _user = MutableStateFlow<User?>(null)
  val user: StateFlow<User?> = _user.asStateFlow()

  suspend fun login(email: String, password: String) {
    val user = api.login(email, password)
    _user.value = user
  }

  fun logout() {
    _user.value = null
  }
}

export val userStore = UserStore()
```

**Usage:**

```whitehall
<!-- screens/Profile.wh -->
<script>
  import { userStore } from "../stores/user"

  val user by userStore.user.collectAsState()
</script>

<Column>
  <Text>{user?.name}</Text>
  <Button onClick={() => userStore.logout()}>Logout</Button>
</Column>
```

---

## Recommendations

### Phase 1: Foundation (Immediate)

**Goal:** Enable basic state patterns with minimal new syntax

1. ✅ **Auto-wrap local state**
   ```whitehall
   var count = 0  // → var count by remember { mutableStateOf(0) }
   ```

2. ✅ **Support bind: directive**
   ```whitehall
   <Input bind:value={email} />  // → value + onValueChange
   ```

3. ✅ **Allow direct Compose/Kotlin in `<script>`**
   ```whitehall
   val viewModel: MyViewModel = viewModel()
   val state by viewModel.uiState.collectAsState()
   LaunchedEffect(Unit) { /* ... */ }
   ```

4. 📝 **Document patterns**
   - How to use ViewModels
   - How to integrate StateFlow
   - Hoisting state best practices
   - Create example projects

**Outcome:** Developers can use all Compose patterns, just with cleaner syntax for common cases.

---

### Phase 2: Ergonomics (3-6 months)

**Goal:** Add convenience helpers for common patterns

1. **Lifecycle hooks** (sugar for LaunchedEffect/DisposableEffect)
   ```whitehall
   onMount { /* runs once */ }
   onDispose { /* cleanup */ }
   onEffect(() => [dep1, dep2]) { /* runs when deps change */ }
   ```

2. **Effect syntax exploration**
   - Consider Svelte-style `$effect` vs explicit hooks
   - Gather feedback from Phase 1

3. **Store file convention**
   - `.store.wh` or similar for shared state
   - Automatic singleton creation
   - Export/import patterns

---

### Phase 3: Advanced (6-12 months)

**Goal:** Production-ready state management

1. **Persistence helpers**
   ```whitehall
   var theme = $persisted("theme", Theme.Light)  // Saves to DataStore
   ```

2. **Derived state helpers**
   ```whitehall
   $derived val total = items.sumOf { it.price }
   ```
   (May not be needed if Compose's auto-recompute is sufficient)

3. **Time-travel debugging**
   - State history tracking
   - Replay state changes
   - Integration with dev tools

4. **State migration**
   - Handle schema changes
   - Version management for persisted state

---

### Decision Matrix

| **Feature** | **Phase 1** | **Phase 2** | **Phase 3** | **Priority** |
|-------------|-------------|-------------|-------------|--------------|
| Auto-wrap `var` → `mutableStateOf` | ✅ | - | - | Critical |
| `bind:` directive | ✅ | - | - | Critical |
| Direct Compose in `<script>` | ✅ | - | - | Critical |
| ViewModel integration docs | ✅ | - | - | High |
| StateFlow patterns | ✅ | - | - | High |
| Lifecycle hooks (`onMount`, etc) | - | 🔄 | - | Medium |
| Store file convention | - | 🔄 | - | Medium |
| `$effect` syntax | - | 🔄 | - | Low |
| Persistence helpers | - | - | 🔜 | Low |
| Derived state helpers | - | - | 🔜 | Very Low |
| Time-travel debugging | - | - | 🔜 | Low |

**Legend:** ✅ Do now | 🔄 Evaluate | 🔜 Future | - Not planned

---

## Open Questions

1. **Runes vs Direct Compose?**
   - Should we add `$state`, `$derived`, `$effect`?
   - Or just keep Compose syntax directly?
   - **Recommendation:** Start without runes, see if developers ask for them

2. **Lifecycle hook syntax?**
   ```whitehall
   // Option A: Function-like
   onMount { }
   onDispose { }

   // Option B: Rune-like
   $effect(() => { })
   $effect.mount(() => { })

   // Option C: Just use Compose
   LaunchedEffect(Unit) { }
   DisposableEffect(Unit) { onDispose { } }
   ```
   - **Recommendation:** Option C for Phase 1, consider A or B in Phase 2 based on feedback

3. **How to handle complex state updates?**
   ```kotlin
   // Nested object updates
   user = user.copy(
     address = user.address.copy(
       street = "New Street"
     )
   )
   ```
   - Could `bind:value={user.address.street}` handle this?
   - **Yes** - Already designed in Decision 005 (data binding)

4. **Global state singleton vs DI?**
   - `object UserRepository` vs `@Singleton class UserRepository @Inject`
   - **Recommendation:** Support both, document DI patterns

5. **Store file extension?**
   - `.store.wh`
   - `.state.wh`
   - Just `.kt` files?
   - **Recommendation:** Use `.kt` files for Phase 1, consider special extensions in Phase 2

---

## Next Steps

1. **Update documentation**
   - Mark state management section in `PENDING.md` as in-progress
   - Create examples showing ViewModel integration
   - Document StateFlow patterns

2. **Create examples**
   - Counter (local state)
   - Login form (hoisted state + validation)
   - User profile (ViewModel + StateFlow)
   - Shopping cart (shared state across screens)

3. **Implement Phase 1**
   - Ensure `var` → `mutableStateOf` transpilation works
   - Test ViewModel integration
   - Validate StateFlow collectAsState patterns

4. **Gather feedback**
   - Build sample apps
   - Identify pain points
   - Decide on Phase 2 features

---

## Appendix: Full Example App

### Counter App with Multiple Patterns

```whitehall
<!-- CounterScreen.wh -->
<script>
  // Local state
  var localCount = 0

  // ViewModel state
  val viewModel: CounterViewModel = viewModel()
  val globalCount by viewModel.globalCount.collectAsState()

  // Repository (shared)
  import com.app.repositories.CounterRepository
  val totalCount by CounterRepository.totalCount.collectAsState()

  // Derived
  val sum = localCount + globalCount + totalCount

  // Effects
  LaunchedEffect(sum) {
    if (sum > 100) {
      analytics.trackEvent("high_count_reached")
    }
  }

  fun incrementLocal() {
    localCount++
  }

  fun incrementGlobal() {
    viewModel.increment()
  }

  fun incrementTotal() {
    CounterRepository.increment()
  }
</script>

<Scaffold>
  <Column padding={16} spacing={12}>
    <Text fontSize={24}>Counter Demo</Text>

    <Card>
      <Column padding={16}>
        <Text>Local Count: {localCount}</Text>
        <Button onClick={incrementLocal}>Increment Local</Button>
      </Column>
    </Card>

    <Card>
      <Column padding={16}>
        <Text>Global Count (ViewModel): {globalCount}</Text>
        <Button onClick={incrementGlobal}>Increment Global</Button>
      </Column>
    </Card>

    <Card>
      <Column padding={16}>
        <Text>Total Count (Repository): {totalCount}</Text>
        <Button onClick={incrementTotal}>Increment Total</Button>
      </Column>
    </Card>

    <Divider />

    <Text fontSize={20} fontWeight={FontWeight.Bold}>
      Sum: {sum}
    </Text>
  </Column>
</Scaffold>
```

**ViewModel:**

```kotlin
class CounterViewModel : ViewModel() {
    private val _globalCount = MutableStateFlow(0)
    val globalCount: StateFlow<Int> = _globalCount.asStateFlow()

    fun increment() {
        _globalCount.update { it + 1 }
    }
}
```

**Repository:**

```kotlin
object CounterRepository {
    private val _totalCount = MutableStateFlow(0)
    val totalCount: StateFlow<Int> = _totalCount.asStateFlow()

    fun increment() {
        _totalCount.update { it + 1 }
    }
}
```

---

**End of Document**
