# Whitehall Language Reference

**A Kotlin Superset for Android Development**

Whitehall is Kotlin with ergonomic enhancements for Jetpack Compose. Write idiomatic Kotlin alongside reactive UI syntax.

**Key Philosophy:**
- ✅ **Any valid Kotlin code is valid Whitehall code** - Use data classes, sealed classes, extension functions, coroutines, etc.
- ✅ **Additive syntax** - Whitehall adds convenient shortcuts on top of Kotlin (component syntax, state binding, reactive primitives)
- ✅ **Zero runtime overhead** - Transpiles to clean, idiomatic Kotlin/Compose code
- ✅ **Gradual adoption** - Mix Kotlin and Whitehall syntax in the same file

## Installation

```bash
cargo install whitehall
whitehall init myapp && cd myapp
whitehall run
```

---

## Kotlin Superset Features

**Pure Kotlin works as-is:**
```whitehall
// Use any Kotlin feature - data classes, sealed classes, extension functions, etc.
data class User(val id: String, val name: String)

sealed class LoadingState<out T> {
    object Idle : LoadingState<Nothing>()
    data class Success<T>(val data: T) : LoadingState<T>()
}

val User.displayName: String
    get() = "$name (#$id)"

fun List<User>.findById(id: String): User? = find { it.id == id }

typealias UserFilter = (User) -> Boolean

// Mix with Whitehall reactive primitives
class UserStore {
    var users: List<User> = []  // Auto-reactive via StateFlow

    suspend fun loadUsers() {   // Auto-wrapped in viewModelScope
        users = api.fetchUsers()
    }
}

// Component markup
<LazyColumn>
  @for (user in store.users) {
    <Text>{user.displayName}</Text>
  }
</LazyColumn>
```

**What Whitehall adds:**
- 📦 Component markup syntax (`<Text>`, `<Column>`, etc.)
- 🔄 Automatic state management (var → StateFlow)
- 🎯 Data binding shortcuts (`bind:value`, `bind:checked`)
- 🎨 UI conveniences (padding shortcuts, color helpers)
- ⚡ Lifecycle hooks (`$onMount`, `$onDispose`)
- 🚀 ViewModel auto-generation

**What stays pure Kotlin:**
- All language features (extension functions, sealed classes, etc.)
- Type system and inference
- Coroutines and suspend functions
- Standard library
- Third-party Kotlin libraries

---

## Syntax

### Components

```whitehall
<Text>Hello, World!</Text>
```

**Props:**
```whitehall
@prop val name: String
@prop val age: Int = 18

<Text>Name: {name}, Age: {age}</Text>
```

**Transpiles to:**
```kotlin
@Composable
fun UserCard(name: String, age: Int = 18) {
    Text("Name: $name, Age: $age")
}
```

---

### State

**Local (simple):**
```whitehall
var count = 0

<Button onClick={() => count++}>
  <Text>Count: {count}</Text>
</Button>
```
→ `remember { mutableStateOf() }`

**Local (complex) - auto ViewModel:**
```whitehall
var count = 0

suspend fun loadData() { }

$onMount { loadData() }
```
→ Generates `ViewModel` + wrapper (auto-detects: suspend fns, lifecycle hooks, or 3+ functions)

**Derived:**
```whitehall
var firstName = "John"
val fullName = "$firstName Doe"
```

**Arrays:**
```whitehall
val numbers = [1, 2, 3]        // → listOf(1, 2, 3)
var mutable = [10, 20]         // → mutableStateOf(mutableListOf(10, 20))
```

**Ranges:**
```whitehall
val simple = 1..10             // → (1..10).toList()
val stepped = 0..100:2         // → (0 rangeTo 100 step 2).toList()
val countdown = 10..1:-1       // → (10 downTo 1).toList()
```

---

### Data Binding

**TextField:**
```whitehall
var email = ""
<TextField bind:value={email} label="Email" />
```
→ `value={email}, onValueChange={email = it}`

**Password:**
```whitehall
<TextField bind:value={password} type="password" />
```
→ Adds `PasswordVisualTransformation()`

**Checkbox/Switch:**
```whitehall
var enabled = false
<Checkbox bind:checked={enabled} />
<Switch bind:checked={enabled} />
```
→ `checked={enabled}, onCheckedChange={enabled = it}`

---

### Control Flow

**@if:**
```whitehall
@if (isLoading) {
  <Text>Loading...</Text>
} else {
  <Text>Done</Text>
}
```

**@for:**
```whitehall
@for (item in items) {
  <Text>{item.name}</Text>
} empty {
  <Text>No items</Text>
}
```

**⚠️ Current Limitation:** The `key` parameter (`key = { it.id }`) is not yet supported by the parser. Use simple iteration without keys. This is planned for a future release.

**@when:**
```whitehall
@when (status) {
  is Loading -> {
    <Text>Loading...</Text>
  }
  is Success -> {
    <Text>Success</Text>
  }
  is Error -> {
    <Text>Error: {status.msg}</Text>
  }
}
```
**⚠️ Current Limitation:** Each arrow branch requires braces `{ }` around the component markup. Single-line branches like `is Loading -> <Text>Loading</Text>` are not yet supported but planned for a future release.

**Workaround:** Use `@if/@else if/@else` for pattern matching until single-line @when branches are supported:
```whitehall
@if (status is Loading) {
  <Text>Loading...</Text>
} else if (status is Success) {
  <Text>Success</Text>
}
```

---

### Shortcuts

**Padding (CSS-like):**
```whitehall
<Text p={16}>All sides</Text>
<Text px={20} py={8}>H & V</Text>
<Text pt={4} pb={12}>Top & Bottom</Text>
<Card pl={8} pr={16}>Left & Right</Card>
```
- `p` - all sides
- `px`, `py` - horizontal, vertical
- `pt`, `pb`, `pl`, `pr` - top, bottom, left, right
- `m*` variants same (no margin in Compose)

**Spacer:**
```whitehall
<Spacer h={16} />    // → Modifier.height(16.dp)
<Spacer w={24} />    // → Modifier.width(24.dp)
<Spacer />           // → 8.dp default
```

**Text:**
```whitehall
<Text fontSize={24} fontWeight="bold" color="primary">Title</Text>
```
→ `fontSize=24.sp, fontWeight=FontWeight.Bold, color=MaterialTheme.colorScheme.primary`

**Button:**
```whitehall
<Button text="Click Me" onClick={handleClick} />
```
→ `Button(...) { Text("Click Me") }`

**fillMaxWidth:**
```whitehall
<Text fillMaxWidth={true}>Full width</Text>
```

**Colors:**
```whitehall
<Text color="#FF5722">Hex</Text>          // → Color(0xFFFF5722)
<Text color="primary">Theme</Text>        // → MaterialTheme.colorScheme.primary
```

---

### Layouts

**Column/Row:**
```whitehall
<Column gap={16} padding={20}>
  <Text>Item 1</Text>
  <Text>Item 2</Text>
</Column>
```
→ `verticalArrangement = Arrangement.spacedBy(16.dp), modifier = Modifier.padding(20.dp)`

**LazyColumn:**
```whitehall
<LazyColumn>
  @for (item in items) {
    <ItemCard item={item} />
  }
</LazyColumn>
```
→ Auto-transforms to `items()` for lazy rendering

---

### ViewModels

**Class with var = ViewModel:**
```whitehall
// No @store annotation needed
class UserProfile {
  var name = ""
  var email = ""

  val isValid: Boolean get() = name.isNotEmpty()

  suspend fun save() {
    api.save(name, email)
  }
}
```

**Generates:**
- `ViewModel` with `StateFlow<UiState>`
- Property accessors for reactive updates
- `suspend` functions auto-wrapped in `viewModelScope.launch`

**Usage:**
```whitehall
import $.stores.UserProfile

val profile = UserProfile()  // Auto-detects, uses viewModel<UserProfile>()

<TextField bind:value={profile.name} />
<Button onClick={profile.save} enabled={profile.isValid}>Save</Button>
```

**With Hilt:**
```whitehall
class UserProfile @Inject constructor(
  private val repo: ProfileRepository
) {
  var name = ""
}
```
→ Adds `@HiltViewModel`, uses `hiltViewModel<T>()`

**Global singleton:**
```whitehall
@store object AppSettings {
  var darkMode = false
}
```
→ `object` with `StateFlow` (NOT ViewModel)

---

### Lifecycle

```whitehall
var data = []

$onMount {
  launch { data = api.fetch() }
}

$onDispose {
  cleanup()
}
```
→ `LaunchedEffect(Unit)` / `DisposableEffect(Unit)`

---

### Coroutines

**Dispatchers:**
```whitehall
<Button onClick={() => io { loadData() }}>Load</Button>       // IO thread
<Button onClick={() => cpu { process() }}>Process</Button>    // CPU thread
<Button onClick={() => main { update() }}>Update</Button>     // Main thread
```

**Custom scope:**
```whitehall
val uploadScope = $scope()

<Button onClick={() => uploadScope.launch { upload() }}>Upload</Button>
<Button onClick={() => uploadScope.cancel()}>Cancel</Button>
```

---

### i18n

**String resources:**
```whitehall
<Text>{R.string.welcome}</Text>
<Text>{R.string.greeting(userName)}</Text>
<Button text={R.string.continue} />
```
→ `stringResource(R.string.welcome)`, `stringResource(R.string.greeting, userName)`

---

### Advanced

**derivedStateOf:**
```whitehall
var items = []
val sorted = derivedStateOf { items.sortedBy { it.name } }
```
→ `by remember { derivedStateOf { ... } }`

**Escape braces:**
```whitehall
<Text>Literal \{braces\}, interpolate {count}</Text>
```
→ `"Literal {braces}, interpolate $count"`

**Multi-line strings:**
```whitehall
var json = """{"name": "Alice", "age": 30}"""
var markdown = """
# Title
Content here
"""
```
→ Kotlin raw string literal `"""..."""` (preserves newlines, no escaping)

**Imports:**
```whitehall
import $models.User         // → com.example.app.models.User
import $routes              // → Routes
```

**Lambda syntax:**
```whitehall
onClick={() => doSomething()}     // Arrow
onClick={doSomething}              // Direct ref
```
Both → `onClick = { doSomething() }`

**Kotlin operators (supported):**
```whitehall
// Safe navigation and elvis operators work as-is
<Text>{user?.name ?: "Unknown"}</Text>
<Text>{profile?.email ?: "No email"}</Text>

// Ternary operator transforms to if/else
<Text>{count > 0 ? "Items" : "No items"}</Text>  // → if (count > 0) "Items" else "No items"
```

**Helper composable functions:**
```whitehall
// Functions with markup bodies are auto-transpiled as @Composable
fun UserCard(user: User, onClick: () -> Unit) {
  <Card p={16}>
    <Column gap={8}>
      <Text fontSize={18} fontWeight="bold">{user.name}</Text>
      <Text fontSize={14} color="#666666">{user.email}</Text>
      <Button text="View Profile" onClick={onClick} />
    </Column>
  </Card>
}

// Use in main component
@for (user in users) {
  <UserCard user={user} onClick={() => navigateTo(user)} />
}
```
→ Helper functions are placed after the main @Composable wrapper and correctly transpiled

---

### FFI (Foreign Function Interface)

Call native Rust and C++ code with zero JNI boilerplate. Auto-generates bindings and JNI bridges.

**Rust FFI:**
```whitehall
// src/ffi/rust/lib.rs
use whitehall::ffi;

#[ffi]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

```whitehall
// src/main.wh
import $ffi.rust.Math

<Text>{Math.add(5, 3)}</Text>      // Calls Rust, returns 8
```
→ Auto-generates: Kotlin object, JNI bridge, builds for all ABIs

**C++ FFI:**
```whitehall
// src/ffi/cpp/string_utils.cpp
#include <string>

// @ffi
std::string to_uppercase(const std::string& str) {
    // ... implementation
}
```

```whitehall
// src/main.wh
import $ffi.cpp.StringUtils

<Text>{StringUtils.toUppercase("hello")}</Text>   // Calls C++, returns "HELLO"
```

**Naming:**
- Snake_case functions → camelCase in Kotlin
- `is_prime()` → `isPrime()`, `to_uppercase()` → `toUppercase()`
- Object name from file: `string_utils.cpp` → `StringUtils`

**Supported types:**
- Primitives: `int`, `long`, `float`, `double`, `bool`
- Strings: `std::string` / `String`
- Arrays: `std::vector<T>` / `Vec<T>`

**Package structure:**
- Rust: `package.ffi.rust.ObjectName`
- C++: `package.ffi.cpp.ObjectName`

---

### Component Limitations & Experimental APIs

**Experimental Material3 APIs (require @OptIn):**

Some Material3 components are marked as experimental and require special handling:

```whitehall
// ⚠️ These are experimental - avoid or use workarounds

// Card onClick (experimental)
<Card onClick={() => action()}>  // ❌ Requires @OptIn
  <Text>Content</Text>
</Card>

// Workaround: Use clickable modifier
<Card p={16}>
  <Column modifier={Modifier.clickable { action() }}>
    <Text>Content</Text>
  </Column>
</Card>

// FilterChip (experimental)
<FilterChip                        // ❌ Requires @OptIn
  selected={isSelected}
  onClick={() => toggle()}
  label="Filter"
/>

// Workaround: Use Button
<Button
  text="Filter"
  onClick={() => toggle()}
/>

// ModalBottomSheet (experimental)
<ModalBottomSheet>                 // ❌ Requires @OptIn
  <Content />
</ModalBottomSheet>

// Workaround: Use AlertDialog
<AlertDialog
  onDismissRequest={() => close()}
  title={<Text>Title</Text>}
  text={<Content />}
  confirmButton={<Button text="Close" onClick={() => close()} />}
/>
```

**Supported stable components:**
- ✅ Text, Button, TextField, OutlinedTextField
- ✅ Column, Row, Box, LazyColumn, LazyRow
- ✅ Card, Checkbox, Switch, Icon
- ✅ AlertDialog, Spacer
- ✅ CircularProgressIndicator
- ✅ Tab, TabRow, Divider

**Auto-generated parameters:**
```whitehall
// Icon automatically gets contentDescription if not provided
<Icon imageVector={Icons.Default.Home} />
// → Icon(imageVector = Icons.Default.Home, contentDescription = null)
```

---

## Prop Transformations

| Component | Prop | Example | Result |
|-----------|------|---------|--------|
| Column/Row | `gap` | `gap={16}` | `Arrangement.spacedBy(16.dp)` |
| Text | `fontSize` | `fontSize={20}` | `fontSize = 20.sp` |
| Text | `fontWeight` | `fontWeight="bold"` | `fontWeight = FontWeight.Bold` |
| Text | `color` | `color="#FF5722"` | `color = Color(0xFFFF5722)` |
| Button | `text` | `text="Click"` | `Button { Text("Click") }` |
| TextField | `label` | `label="Name"` | `label = { Text("Name") }` |
| TextField | `type` | `type="password"` | `PasswordVisualTransformation()` |
| OutlinedTextField | `label` | `label="Email"` | `label = { Text("Email") }` |
| Tab | `text` | `text={<Text>Home</Text>}` | `text = { Text("Home") }` |
| AlertDialog | `title` | `title={<Text>Alert</Text>}` | `title = { Text("Alert") }` |
| AlertDialog | `text` | `text={<Content />}` | `text = { Content() }` |
| Icon | (auto) | `<Icon imageVector={...} />` | Adds `contentDescription = null` |
| Spacer | `h/w` | `h={16}` | `Modifier.height(16.dp)` |
| Any | `p` | `p={16}` | `Modifier.padding(16.dp)` |
| Any | `px/py` | `px={20} py={8}` | `padding(horizontal=20.dp, vertical=8.dp)` |
| Any | `fillMaxWidth` | `fillMaxWidth={true}` | `Modifier.fillMaxWidth()` |

---

## Toolchain

### Commands

```bash
whitehall init <name>      # Create project
whitehall build            # Transpile to Kotlin
whitehall watch            # Auto-rebuild
whitehall run              # Build + install + launch
whitehall compile <file>   # Single file transpile
whitehall doctor           # Health check
```

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
```

**Zero-config:** Auto-downloads Java/Gradle/SDK on first build

---

## File Structure

```
myapp/
├── whitehall.toml          # Config
├── src/
│   ├── main.wh             # Entry
│   ├── components/         # Reusable
│   ├── screens/            # Screens
│   └── stores/             # ViewModels
└── build/                  # Generated Kotlin
```

**Auto-detected packages:**
- `src/components/Foo.wh` → `com.example.app.components.Foo`
- `src/screens/Bar.wh` → `com.example.app.screens.Bar`
- `src/stores/Baz.wh` → `com.example.app.stores.Baz`

---

## State Management

| Pattern | Survives Rotation? | Use Case |
|---------|-------------------|----------|
| `var count = 0` (simple) | ❌ | Simple forms |
| `var count = 0` (complex) | ✅ | Suspend/lifecycle/3+ fns |
| `class Profile { var ... }` | ✅ | Screen state |
| `@store object Settings { var ... }` | ✅ | App-wide |

---

## Common Patterns

**Form validation:**
```whitehall
var email = ""
var password = ""
val isValid = email.isNotEmpty() && password.length >= 8

<Column gap={16}>
  <TextField bind:value={email} label="Email" />
  <TextField bind:value={password} label="Password" type="password" />
  <Button onClick={submit} enabled={isValid} text="Login" />
</Column>
```

**List with loading:**
```whitehall
var items = []
var isLoading = true

$onMount {
  launch {
    items = api.fetch()
    isLoading = false
  }
}

@if (isLoading) {
  <CircularProgressIndicator />
} else {
  <LazyColumn>
    @for (item in items) {
      <ItemCard item={item} />
    }
  </LazyColumn>
}
```

---

## Learn More

- Kotlin: https://kotlinlang.org/docs/compose-compiler.html
- Jetpack Compose: https://developer.android.com/jetpack/compose
- Material 3: https://m3.material.io/
