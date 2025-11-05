# Counter Store - @store Example App

A complete, buildable Whitehall app demonstrating the `@store` annotation for state management.

## Features

This app demonstrates:
- ✅ **@store classes**: ViewModel-based state management
- ✅ **Reactive state**: Mutable properties with StateFlow
- ✅ **Derived properties**: Computed values (`val isPositive`)
- ✅ **State methods**: Functions that modify state
- ✅ **Nullable state**: `String?` properties

## Structure

```
counter-store/
├── whitehall.toml           # Project configuration
├── src/
│   ├── main.wh             # App entry point
│   ├── stores/
│   │   └── CounterStore.wh # @store class (generates ViewModel)
│   └── screens/
│       └── CounterScreen.wh # UI that uses the store
└── build/                   # Generated Android project
```

## CounterStore

The `@store` class automatically generates a ViewModel with StateFlow:

```whitehall
@store
class CounterStore {
  var count: Int = 0
  var lastIncrement: String? = null

  val isPositive: Boolean
    get() = count > 0

  fun increment() {
    count++
    lastIncrement = "Incremented"
  }
}
```

Transpiles to:
```kotlin
class CounterStore : ViewModel() {
    data class UiState(
        val count: Int = 0,
        val lastIncrement: String? = null
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var count: Int
        get() = _uiState.value.count
        set(value) { _uiState.update { it.copy(count = value) } }

    val isPositive: Boolean
        get() = count > 0

    fun increment() { /* ... */ }
}
```

## Build & Run

```bash
# From the whitehall root directory
cd examples/counter-store

# Build the app
whitehall build

# Run on connected device/emulator
whitehall run
```

## Current Status

- ✅ **Phase 1**: Store class generation (DONE)
  - CounterStore transpiles to ViewModel with StateFlow
- 🚧 **Phase 2**: Auto-detect store usage in screens (TODO)
  - Currently `val counter = CounterStore()` is treated as regular instantiation
  - Will auto-transform to `val counter = viewModel<CounterStore>()`

## What You'll See

A simple counter app with:
- Large counter display (changes color when positive)
- + and − buttons
- Reset button
- "Last increment" timestamp when available
