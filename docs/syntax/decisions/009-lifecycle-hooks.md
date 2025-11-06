# Decision 009: Lifecycle Hooks

**Status:** Decided & Implemented
**Date:** 2025-11-06
**Implementation:** Fully implemented with Phase 1.1 fixes

## Context

Components need to run code at specific lifecycle points:
- **onMount** - Run code when component first appears (data loading, subscriptions)
- **onDispose** - Cleanup when component is removed (unsubscribe, cancel operations)

Compose provides:
- `LaunchedEffect(Unit)` - runs on composition
- `DisposableEffect(Unit)` - runs on composition with cleanup

Whitehall needs simpler, more intuitive syntax that integrates with the automatic ViewModel generation system.

## Decision

**Use `onMount` and `onDispose` hooks** with smart transpilation based on component complexity.

## Syntax

```whitehall
<script>
  var data = emptyList()

  onMount {
    // Runs when component first appears
    data = loadData()
  }

  onDispose {
    // Runs when component is removed
    cleanup()
  }
</script>
```

## Smart Transpilation

### Simple Components → LaunchedEffect/DisposableEffect

For components **without** ViewModel generation (< 3 functions, no suspend, no complexity):

```kotlin
@Composable
fun SimpleComponent() {
    var data by remember { mutableStateOf(emptyList()) }

    LaunchedEffect(Unit) {
        data = loadData()
    }

    DisposableEffect(Unit) {
        onDispose {
            cleanup()
        }
    }
}
```

---

### Complex Components → ViewModel init Block

For components **with** ViewModel generation (>= 3 functions, suspend, or lifecycle):

**Wrapper Component:**
```kotlin
@Composable
fun ComplexComponent() {
    val viewModel = viewModel<ComplexComponentViewModel>()
    val uiState by viewModel.uiState.collectAsState()

    // NO lifecycle hooks here - they're in the ViewModel!
}
```

**ViewModel:**
```kotlin
class ComplexComponentViewModel : ViewModel() {
    data class UiState(
        val data: List<Item> = emptyList()
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var data: List<Item>
        get() = _uiState.value.data
        set(value) { _uiState.update { it.copy(data = value) } }

    init {
        // onMount code goes here
        viewModelScope.launch {
            data = loadData()
        }
    }

    override fun onCleared() {
        super.onCleared()
        // onDispose code could go here (future)
        cleanup()
    }
}
```

---

## Lifecycle Hook Behavior

### onMount

**When it runs:**
- Simple components: on composition via `LaunchedEffect(Unit)`
- Complex components: in ViewModel's `init {}` block

**Use cases:**
- Data loading
- Starting subscriptions
- Initializing resources
- Setting up listeners

**Example:**
```whitehall
<script>
  var posts: List<Post> = emptyList()

  onMount {
    launch {  // Automatically scoped
      posts = ApiClient.getPosts()
    }
  }
</script>
```

**Note:** In ViewModel, `launch` is automatically wrapped in `viewModelScope.launch`

---

### onDispose

**When it runs:**
- Simple components: on disposal via `DisposableEffect(Unit) { onDispose {} }`
- Complex components: **NOT SUPPORTED** (ViewModels use `onCleared()` instead)

**Use cases:**
- Canceling subscriptions
- Closing connections
- Releasing resources
- Removing listeners

**Example:**
```whitehall
<script>
  var subscription: Subscription? = null

  onMount {
    subscription = EventBus.subscribe()
  }

  onDispose {
    subscription?.cancel()
  }
</script>
```

**ViewModel limitation:** `onDispose` in complex components doesn't map well to ViewModel lifecycle. Use ViewModel's `onCleared()` override directly in Kotlin if needed.

---

## Implementation Details

### Triggers for ViewModel Generation

When any of these conditions are true, lifecycle hooks go in ViewModel `init` block:

1. Component has **suspend functions**
2. Component has **>= 3 functions**
3. Component has **lifecycle hooks** (onMount/onDispose)

This ensures lifecycle code has proper coroutine scope and survives rotation.

---

### Variable References in Lifecycle Hooks

**Problem:** Lifecycle hooks reference component state that gets split into ViewModel

**Solution:** Variable references are automatically transformed:

```whitehall
<script>
  var posts: List<Post> = emptyList()
  var isLoading = true

  onMount {
    launch {
      isLoading = true
      posts = ApiClient.getPosts()
      isLoading = false
    }
  }
</script>
```

**Generates ViewModel with:**
```kotlin
init {
    viewModelScope.launch {
        isLoading = true  // Uses property setter
        posts = ApiClient.getPosts()  // Uses property setter
        isLoading = false
    }
}
```

Property setters automatically update StateFlow via `.update { it.copy(...) }`

---

## Examples

### Simple Data Loading
```whitehall
<script>
  var users: List<User> = emptyList()

  onMount {
    users = ApiClient.getUsers()
  }
</script>

<Column>
  @for (user in users) {
    <UserCard user={user} />
  }
</Column>
```

---

### Async Data Loading with Suspend
```whitehall
<script>
  var posts: List<Post> = emptyList()
  var isLoading = true

  suspend fun loadPosts() {
    isLoading = true
    posts = ApiClient.getPosts()
    isLoading = false
  }

  onMount {
    loadPosts()
  }
</script>

<Column>
  @if (isLoading) {
    <LoadingSpinner />
  } else {
    @for (post in posts) {
      <PostCard post={post} />
    }
  }
</Column>
```

**Generates ViewModel** because it has `suspend` function + `onMount`

---

### Subscription with Cleanup
```whitehall
<script>
  var messages: List<String> = emptyList()

  onMount {
    WebSocket.connect()
    WebSocket.onMessage { msg ->
      messages = messages + msg
    }
  }

  onDispose {
    WebSocket.disconnect()
  }
</script>

<Column>
  @for (message in messages) {
    <Text>{message}</Text>
  }
</Column>
```

**Generates simple component** with `LaunchedEffect` + `DisposableEffect`

---

## Testing & Verification

**Test Coverage:**
- Test 06: `onMount` with API loading → ViewModel generation
- Test 08: `onMount` with routing params → ViewModel generation
- Test 11: `onMount` with complex state → ViewModel generation
- Test 16: `onMount` + `onDispose` together → ViewModel generation
- Test 17: `onMount` with error handling → ViewModel generation

**Status:** ✅ All 38/38 tests passing

---

## Rationale

**Why `onMount`/`onDispose` instead of `LaunchedEffect`/`DisposableEffect`?**
1. **Simpler mental model** - Lifecycle is about "when" not "how"
2. **Familiar to web developers** - Similar to React's useEffect, Vue's mounted/unmounted
3. **Less verbose** - No need to specify keys like `LaunchedEffect(Unit)`
4. **Abstracts implementation** - Transpiler chooses best strategy

**Why move to ViewModel init for complex components?**
1. **Survives rotation** - ViewModel lifecycle independent of UI
2. **Proper scope** - `viewModelScope` handles coroutine lifecycle
3. **Separation of concerns** - Initialization logic in state layer, not UI
4. **Testable** - Can test ViewModel init independently

**Why not support `onDispose` in ViewModels?**
- ViewModels have `onCleared()` which runs when ViewModel is destroyed
- Different timing than Compose disposal
- Can override `onCleared()` directly in Kotlin if needed
- Most cleanup is handled by viewModelScope cancellation

---

## Known Limitations

1. **onDispose in ViewModels** - Not supported. Use `onCleared()` override in Kotlin.
2. **Multiple onMount blocks** - Only one `onMount` per component supported.
3. **Keys/Dependencies** - No support for `LaunchedEffect(key)` style re-runs.

Future enhancement: `onUpdate(() => [dep1, dep2]) { }` for dependency-based effects

---

## See Also

- [Decision 008: State Management](./008-state-management.md) - ViewModel generation rules
- [STORE.md](/docs/STORE.md) - Complete implementation details
- [NEXTSTEPS.md](/docs/NEXTSTEPS.md) - Lifecycle fix implementation (Nov 6, 2025)
