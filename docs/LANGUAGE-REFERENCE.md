# Whitehall Language Reference

Svelte-inspired Android framework transpiling to Kotlin + Jetpack Compose.

---

## Quick Start

```bash
# Install
cargo install whitehall

# Create project
whitehall init myapp
cd myapp

# Build (transpile to Kotlin)
whitehall build

# Run on device
whitehall run

# Watch mode
whitehall watch
```

---

## File Structure

```
myapp/
├── whitehall.toml          # Config
├── src/
│   ├── main.wh             # Entry point
│   ├── components/         # Reusable components
│   │   └── Button.wh
│   ├── screens/            # Screen components
│   │   └── HomeScreen.wh
│   └── stores/             # ViewModels
│       └── UserProfile.wh
└── build/                  # Generated Kotlin project
```

---

## Syntax

### Components

**Basic:**
```whitehall
<Text>Hello, World!</Text>
```

**With props:**
```whitehall
@prop val name: String
@prop val age: Int = 18
@prop val onClick: (() -> Unit)? = null

<Column>
  <Text>Name: {name}, Age: {age}</Text>
</Column>
```

**Transpiles to:**
```kotlin
@Composable
fun UserCard(
    name: String,
    age: Int = 18,
    onClick: (() -> Unit)? = null
) {
    Column {
        Text("Name: $name, Age: $age")
    }
}
```

---

### State

**Local (simple):**
```whitehall
var count = 0
var name = ""

<Button onClick={() => count++}>
  <Text>Count: {count}</Text>
</Button>
```

**Transpiles to:**
```kotlin
var count by remember { mutableStateOf(0) }
var name by remember { mutableStateOf("") }
```

**Local (complex) - auto ViewModel:**
```whitehall
var count = 0

suspend fun loadData() {
  // async work
}

onMount {
  loadData()
}

<Text>Count: {count}</Text>
```

**Generates:** `CounterViewModel.kt` + wrapper component
- Auto-detects: suspend fns, lifecycle hooks, or 3+ functions
- Survives config changes (rotation)

**Derived:**
```whitehall
var firstName = "John"
var lastName = "Doe"

val fullName = "$firstName $lastName"

<Text>{fullName}</Text>
```

**Array literals:**
```whitehall
val numbers = [1, 2, 3, 4, 5]
val strings = ["Apple", "Banana", "Cherry"]
var mutableNums = [10, 20, 30]

val nested = [[1, 2], [3, 4]]
```

**Transpiles to:**
```kotlin
val numbers = listOf(1, 2, 3, 4, 5)
val strings = listOf("Apple", "Banana", "Cherry")
var mutableNums by remember { mutableStateOf(mutableListOf(10, 20, 30)) }

val nested = listOf(listOf(1, 2), listOf(3, 4))
```

---

### Data Binding

```whitehall
var email = ""
var password = ""

<TextField bind:value={email} label="Email" />
<TextField bind:value={password} label="Password" type="password" />
```

**Transpiles to:**
```kotlin
TextField(
    value = email,
    onValueChange = { email = it },
    label = { Text("Email") }
)
```

---

### Control Flow

**@if / @else:**
```whitehall
@if (isLoading) {
  <Text>Loading...</Text>
} else {
  <Text>Done!</Text>
}
```

**@for:**
```whitehall
@for (item in items, key = { it.id }) {
  <Text>{item.name}</Text>
} empty {
  <Text>No items</Text>
}
```

**@when:**
```whitehall
@when (status) {
  is Status.Loading -> <Text>Loading...</Text>
  is Status.Success -> <Text>Success!</Text>
  is Status.Error -> <Text>Error: {status.message}</Text>
}
```

---

### Layouts & Shortcuts

**Column/Row:**
```whitehall
<Column spacing={16} padding={20}>
  <Text>Item 1</Text>
  <Text>Item 2</Text>
</Column>
```

**Transpiles to:**
```kotlin
Column(
    verticalArrangement = Arrangement.spacedBy(16.dp),
    modifier = Modifier.padding(20.dp)
) { ... }
```

**Text shortcuts:**
```whitehall
<Text fontSize={24} fontWeight="bold" color="primary">
  Title
</Text>
```

**Transpiles to:**
```kotlin
Text(
    text = "Title",
    fontSize = 24.sp,
    fontWeight = FontWeight.Bold,
    color = MaterialTheme.colorScheme.primary
)
```

**Button text:**
```whitehall
<Button text="Click Me" onClick={handleClick} />
```

**Transpiles to:**
```kotlin
Button(onClick = { handleClick() }) {
    Text("Click Me")
}
```

**Spacer shortcuts:**
```whitehall
<Spacer size={16} />           // → Spacer(modifier = Modifier.size(16.dp))
<Spacer width={100} />         // → Spacer(modifier = Modifier.width(100.dp))
<Spacer height={50} />         // → Spacer(modifier = Modifier.height(50.dp))
```

---

### ViewModels / Stores

**Class with var = ViewModel:**
```whitehall
// src/stores/UserProfile.wh
class UserProfile {
  var name = ""
  var email = ""

  val isValid: Boolean get() = name.isNotEmpty()

  fun clear() {
    name = ""
  }

  suspend fun save() {
    api.save(name, email)
  }
}
```

**Generates ViewModel with StateFlow:**
```kotlin
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

    // suspend auto-wraps in viewModelScope.launch
    fun save() {
        viewModelScope.launch {
            api.save(name, email)
        }
    }
}
```

**Usage:**
```whitehall
import $.stores.UserProfile

val profile = UserProfile()  // Auto-detects and uses viewModel<UserProfile>()

<TextField bind:value={profile.name} label="Name" />
<Button onClick={profile.save} enabled={profile.isValid}>
  <Text>Save</Text>
</Button>
```

**With Hilt DI:**
```whitehall
class UserProfile @Inject constructor(
  private val repository: ProfileRepository
) {
  var name = ""

  suspend fun save() {
    repository.save(name)
  }
}
```

**Adds @HiltViewModel automatically**

**Global singleton:**
```whitehall
@store object AppSettings {
  var darkMode = false
  var language = "en"
}
```

**Generates object with StateFlow** (NOT ViewModel)

---

### Lifecycle Hooks

```whitehall
var data = []

onMount {
  launch {
    data = api.fetch()
  }
}

onDispose {
  cleanup()
}

<LazyColumn>
  @for (item in data, key = { it.id }) {
    <Text>{item.name}</Text>
  }
</LazyColumn>
```

**Transpiles to:**
```kotlin
LaunchedEffect(Unit) {
    // onMount code
}

DisposableEffect(Unit) {
    onDispose {
        cleanup()
    }
}
```

---

### Coroutines

**Dispatchers:**
```whitehall
<Button onClick={() => io { loadData() }}>Load</Button>      // Network/disk
<Button onClick={() => cpu { process() }}>Process</Button>    // Heavy compute
<Button onClick={() => main { updateUI() }}>Update</Button>   // UI thread
```

**Transpiles to:**
```kotlin
Button(onClick = {
    coroutineScope.launch(Dispatchers.IO) {
        loadData()
    }
})
```

**Custom scope:**
```whitehall
val uploadScope = $scope()

<Button onClick={() => uploadScope.launch { upload() }}>Upload</Button>
<Button onClick={() => uploadScope.cancel()}>Cancel</Button>
```

---

### Imports

```whitehall
import $models.User           // → com.example.app.models.User
import $routes                // → Routes
import $screen.params         // Route params (auto-extracts)
```

---

### Lists

**LazyColumn:**
```whitehall
<LazyColumn>
  @for (item in items, key = { it.id }) {
    <ItemCard item={item} />
  }
</LazyColumn>
```

**Transpiles to:**
```kotlin
LazyColumn {
    items(items, key = { it.id }) { item ->
        ItemCard(item = item)
    }
}
```

---

## Toolchain

### Commands

| Command | Purpose |
|---------|---------|
| `whitehall init <name>` | Create project |
| `whitehall build` | Transpile to Kotlin |
| `whitehall watch` | Auto-rebuild on save |
| `whitehall run` | Build + install + launch |
| `whitehall compile <file>` | Transpile single file |
| `whitehall toolchain list` | Show installed toolchains |
| `whitehall doctor` | Health check |
| `whitehall exec <cmd>` | Run with toolchain env |
| `whitehall shell` | Interactive shell |

### Config (whitehall.toml)

```toml
[project]
name = "myapp"
version = "0.1.0"

[android]
min_sdk = 24
target_sdk = 34
package = "com.example.myapp"

[toolchain]
java = "21"           # Auto-downloads
gradle = "8.4"
agp = "8.2.0"
kotlin = "2.0.0"

[build]
output_dir = "build"
```

### Toolchain Management

- **Zero-config:** Auto-downloads Java/Gradle/SDK on first build
- **Project-scoped:** Each project specifies versions in `whitehall.toml`
- **Cached:** `~/.whitehall/toolchains/` shared across projects
- **Version switching:** Different projects use different toolchains without conflicts

---

## Component Prop Transformations

Common patterns auto-transform:

| Whitehall | Kotlin/Compose |
|-----------|----------------|
| `<Column spacing={16}>` | `verticalArrangement = Arrangement.spacedBy(16.dp)` |
| `<Row spacing={8}>` | `horizontalArrangement = Arrangement.spacedBy(8.dp)` |
| `<Text fontSize={20}>` | `fontSize = 20.sp` |
| `<Text fontWeight="bold">` | `fontWeight = FontWeight.Bold` |
| `<Text color="primary">` | `color = MaterialTheme.colorScheme.primary` |
| `<Column padding={16}>` | `modifier = Modifier.padding(16.dp)` |
| `<Button text="Click">` | `Button(...) { Text("Click") }` |
| `<TextField label="Name">` | `label = { Text("Name") }` |

---

## Key Concepts

### File Types (auto-detected)

- `src/components/Foo.wh` → `com.example.app.components.Foo`
- `src/screens/Bar.wh` → `com.example.app.screens.Bar`
- `src/stores/Baz.wh` → `com.example.app.stores.Baz`
- `src/main.wh` → `MainActivity.kt`

### State Management Patterns

| Pattern | Survives Rotation? | Use Case |
|---------|-------------------|----------|
| `var count = 0` (simple) | ❌ | Simple forms, toggles |
| `var count = 0` (complex) | ✅ | Components with suspend/lifecycle/3+ fns |
| `class UserProfile { var ... }` | ✅ | Screen state, API calls |
| `@store object AppSettings { var ... }` | ✅ | App-wide settings |

### Lambda Syntax

```whitehall
onClick={() => doSomething()}     // Arrow syntax
onClick={doSomething}              // Direct reference
```

**Both transpile to:** `onClick = { doSomething() }`

### Escape Braces

```whitehall
<Text>Use \{braces\} literally</Text>
```

**Transpiles to:** `Text("Use {braces} literally")`

---

## Development Workflow

```bash
# Terminal 1: Watch mode
whitehall watch

# Terminal 2: Edit files
vim src/components/Button.wh

# Test on device
whitehall run
```

**Or quick iteration:**
```bash
whitehall run        # Build + install + launch in one command
```

---

## Testing

```bash
# Run transpiler tests
cargo test --test transpiler_examples_test examples

# Single test
cargo test --test transpiler_examples_test -- 00-minimal-text
```

38 test cases in `tests/transpiler-examples/`

---

## Advanced Features

### String Resources

```whitehall
<Text text={@string/app_name} />
```

### Checkbox/Switch

```whitehall
var checked = false
<Checkbox bind:checked={checked} label="Agree" />
```

### derivedStateOf (expensive computations)

```whitehall
var items = []
val sortedItems = derivedStateOf { items.sortedBy { it.name } }
```

### AsyncImage

```whitehall
<AsyncImage url={user.avatarUrl} width={48} height={48} />
```

### Card with backgroundColor

```whitehall
<Card backgroundColor="surface">
  <Text>Content</Text>
</Card>
```

### Inline Lambdas

```whitehall
<Button onClick={vm.save}>Save</Button>         // Direct ref
<Button onClick={() => vm.save()}>Save</Button> // Inline lambda
```

---

## Common Patterns

### Form with validation

```whitehall
var email = ""
var password = ""

val isValid = email.isNotEmpty() && password.length >= 8

<Column spacing={16}>
  <TextField bind:value={email} label="Email" />
  <TextField bind:value={password} label="Password" type="password" />
  <Button onClick={submit} enabled={isValid} text="Login" />
</Column>
```

### List with loading state

```whitehall
var posts = []
var isLoading = true

onMount {
  launch {
    posts = api.fetchPosts()
    isLoading = false
  }
}

<Column>
  @if (isLoading) {
    <CircularProgressIndicator />
  } else {
    @for (post in posts, key = { it.id }) {
      <PostCard post={post} />
    }
  }
</Column>
```

### Navigation

```whitehall
<Button onClick={() => navigate($routes.post.detail(id = post.id))}>
  <Text>View Post</Text>
</Button>
```

---

## Notes

- **No @store needed for ViewModels** - Any class with `var` auto-generates ViewModel
- **@store object** - For global singletons (StateFlow, NOT ViewModel)
- **Suspend functions** - Auto-wrapped in viewModelScope.launch in ViewModels
- **Hilt** - Auto-detects `@Inject` constructor OR `@hilt` annotation
- **Multi-file output** - Complex components generate ViewModel + wrapper

---

## Resources

- Examples: `tests/transpiler-examples/`
- Architecture: `docs/REF-OVERVIEW.md`
- Transpiler: `docs/REF-TRANSPILER.md`
- State: `docs/REF-STATE-MANAGEMENT.md`
- Build: `docs/REF-BUILD-SYSTEM.md`
- Toolchain: `docs/REF-TOOLCHAIN.md`
