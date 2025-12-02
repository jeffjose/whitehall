# Whitehall

> An experimental Kotlin superset for modern Android development

**Status:** 🧪 Experimental — 48/48 transpiler tests passing

## What is Whitehall?

Whitehall is **Kotlin with ergonomic enhancements** for Jetpack Compose - a true superset where any valid Kotlin code is valid Whitehall code.

```whitehall
// Pure Kotlin works unchanged
data class User(val id: String, val name: String)

sealed class LoadingState<out T> {
    object Idle : LoadingState<Nothing>()
    data class Success<T>(val data: T) : LoadingState<T>()
}

val User.displayName: String
    get() = "$name (#$id)"

// Mix with Whitehall's reactive primitives
class UserStore {
    var users: List<User> = []  // Auto-reactive via StateFlow

    suspend fun loadUsers() {   // Auto-wrapped in viewModelScope
        users = api.fetchUsers()
    }
}

// Component markup
<LazyColumn>
  @for (user in store.users, key = { it.id }) {
    <Text>{user.displayName}</Text>
  }
</LazyColumn>
```

### Core Philosophy

1. **Kotlin First** - Any valid Kotlin code works unchanged
   - data classes, sealed classes, extension functions, coroutines
   - Mix pure Kotlin and Whitehall syntax in the same file
   - Zero runtime overhead

2. **Additive Syntax** - Whitehall adds conveniences on top:
   - Component markup (`<Text>`, `<Column>`)
   - Automatic state management (var → StateFlow)
   - Data binding (`bind:value`, `bind:checked`)
   - Lifecycle hooks (`onMount`, `onDispose`)

3. **Toolchain Philosophy** - "cargo for Android"
   - Zero-config setup with automatic toolchain management
   - Opinionated defaults with project-level control

## Quick Start

```bash
# Install
cargo install whitehall

# Create new project
whitehall init my-app && cd my-app

# Run on device
whitehall run
```

## Features

**Transpiler** - 48/48 tests passing
- Hybrid parsing: transforms Whitehall syntax, passes through pure Kotlin
- Component markup with reactive state
- Automatic ViewModel generation
- Data binding shortcuts
- Lifecycle hooks

**Pass-Through Architecture** - 10/10 tests passing
- True Kotlin superset - any Kotlin syntax works
- Tested with: sealed classes, extension functions, companion objects, DSL builders
- Context-aware parsing (strings, comments, braces)

**Build System**
- `whitehall build` - One-shot transpilation
- `whitehall watch` - Continuous auto-rebuild
- `whitehall run` - Build + install + launch
- `whitehall compile <file>` - Single file testing

**Toolchain Management**
- Automatic download of Java, Gradle, Android SDK
- Project-scoped toolchains (like `rust-toolchain.toml`)
- `whitehall toolchain` commands for management

**State Management**
- Local state with `var` (auto-reactive)
- ViewModel auto-generation for complex components
- `@store` classes for screen-level state
- Hilt integration (hybrid auto-detection)
- Coroutine dispatchers: `io {}`, `cpu {}`, `main {}`

## Documentation

- [**LANGUAGE-REFERENCE.md**](docs/LANGUAGE-REFERENCE.md) - Complete syntax guide
- [**REF-OVERVIEW.md**](docs/REF-OVERVIEW.md) - Architecture overview
- [**REF-TRANSPILER.md**](docs/REF-TRANSPILER.md) - Transpiler details
- [**REF-STATE-MANAGEMENT.md**](docs/REF-STATE-MANAGEMENT.md) - State patterns
- [**REF-BUILD-SYSTEM.md**](docs/REF-BUILD-SYSTEM.md) - Build commands
- [**REF-TOOLCHAIN.md**](docs/REF-TOOLCHAIN.md) - Toolchain management
- [**PASSTHRU.md**](docs/PASSTHRU.md) - Pass-through architecture

## Example: Complete Form

```whitehall
class LoginStore {
    var email = ""
    var password = ""
    var isLoading = false

    val isValid: Boolean get() =
        email.isNotEmpty() && password.length >= 8

    suspend fun login() {
        isLoading = true
        try {
            val result = api.login(email, password)
            navigate(Routes.Home)
        } catch (e: Exception) {
            showError(e.message)
        } finally {
            isLoading = false
        }
    }
}

val store = LoginStore()

<Column spacing={16} p={24}>
  <TextField
    bind:value={store.email}
    label="Email"
  />
  <TextField
    bind:value={store.password}
    label="Password"
    type="password"
  />

  @if (store.isLoading) {
    <CircularProgressIndicator />
  } else {
    <Button
      text="Login"
      onClick={store.login}
      enabled={store.isValid}
    />
  }
</Column>
```

Transpiles to clean, idiomatic Kotlin/Compose with proper ViewModel, StateFlow, and viewModelScope handling.

## Development Status

| Component | Status |
|-----------|--------|
| Transpiler | 48/48 tests |
| Pass-Through | 10/10 tests |
| Build System | Implemented |
| State Management | Phase 1.1 |
| Toolchain | Phases 1-5 |
| Web Playground | Phase 1 |

⚠️ **Note:** Whitehall is experimental software. APIs and syntax may change. Not recommended for production apps yet.

## Why Whitehall?

### vs Pure Kotlin/Compose
- **Less boilerplate**: No manual StateFlow/ViewModel setup
- **Familiar syntax**: Svelte/Vue-like component markup
- **Same output**: Transpiles to idiomatic Kotlin/Compose
- **Gradual adoption**: Mix Kotlin and Whitehall freely

### vs Flutter
- **Native**: Uses Jetpack Compose (not a custom rendering engine)
- **Kotlin**: Better Android ecosystem integration
- **No runtime**: Transpiles to native code (no VM overhead)

### vs React Native
- **Performance**: True native compilation (not JavaScript bridge)
- **Type safety**: Kotlin's type system
- **Modern**: Jetpack Compose is state-of-the-art Android UI

## Contributing

Whitehall is an experimental project exploring new ways to write native Android apps. Feedback and contributions welcome! See [docs/REF-OVERVIEW.md](docs/REF-OVERVIEW.md) for architecture details.

## License

MIT
