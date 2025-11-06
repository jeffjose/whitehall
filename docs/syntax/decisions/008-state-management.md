# Decision 008: State Management & ViewModels

**Status:** Decided & Implemented
**Date:** 2025-11-06
**Implementation:** Phases 0-5 complete, Phase 1.1 complete

## Context

Whitehall needs comprehensive state management that:
- Supports simple component state (counters, form inputs)
- Supports complex state with async operations and lifecycle
- Survives screen rotation automatically
- Integrates with Android ViewModel and StateFlow patterns
- Supports dependency injection (Hilt)
- Requires minimal boilerplate

## Decision

**Multi-tier state management system** with automatic escalation based on complexity:

### Tier 1: Simple Inline State (remember/mutableStateOf)

For simple components without async operations or lifecycle hooks.

**Syntax:**
```whitehall
<script>
  var count = 0

  fun increment() {
    count++
  }
</script>

<Button onClick={() => increment()}>
  Count: {count}
</Button>
```

**Generates:**
```kotlin
var count by remember { mutableStateOf(0) }
```

**Use when:**
- Local UI state (form inputs, toggles, counters)
- No async operations
- No lifecycle hooks
- < 3 functions

---

### Tier 2: Auto-ViewModel (Phase 1.1)

For complex components with async operations or lifecycle hooks.

**Syntax:** (same as Tier 1, but triggers ViewModel generation)
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
```

**Generates:**
- `ComponentViewModel.kt` with StateFlow, UiState, and init block
- `Component.kt` wrapper with `viewModel<T>()` and `collectAsState()`

**Triggers:**
- ✅ Has suspend functions
- ✅ Has >= 3 functions
- ✅ Has lifecycle hooks (onMount/onDispose)

**Benefits:**
- Survives screen rotation
- Proper coroutine scoping
- Separation of concerns
- Testable state logic

---

### Tier 3: @store Classes (Screen-Level State)

For shared state across multiple components or screens.

**Syntax:**
```whitehall
<!-- stores/UserProfile.wh -->
@store class UserProfile {
  var name: String = ""
  var email: String = ""
  var age: Int = 0

  val isAdult: Boolean get() = age >= 18

  suspend fun save() {
    ApiClient.updateProfile(name, email, age)
  }
}
```

**Usage in components:**
```whitehall
<script>
  val profile = UserProfile()  // Auto-generates viewModel<UserProfile>()
</script>

<TextField bind:value={profile.name} />
<Button onClick={profile::save}>Save</Button>
```

**Generates:**
- `UserProfileViewModel.kt` with full StateFlow setup
- Auto-detection at usage sites → `viewModel<UserProfile>()`

**Use when:**
- State shared across multiple components
- Business logic separation
- Screen-scoped state management

---

## Advanced Features

### Hilt Integration

**Auto-detected from `@Inject` constructor:**
```whitehall
@store class UserProfile @Inject constructor(
  private val api: ApiClient
) {
  var name = ""

  suspend fun save() {
    api.updateProfile(name)
  }
}
```

**Or explicit `@hilt` annotation:**
```whitehall
@hilt
@store class UserProfile {
  var name = ""
}
```

**Generates:** `@HiltViewModel` annotation + `hiltViewModel<T>()` at usage sites

---

### Derived Properties

**Syntax:**
```whitehall
@store class Counter {
  var count = 0

  val doubled: Int get() = count * 2
  val isPositive: Boolean get() = count > 0
}
```

**Generates:** Computed properties with getters in ViewModel

---

### Suspend Functions

**Auto-wrapped in viewModelScope:**
```whitehall
@store class DataLoader {
  suspend fun load() {
    // Automatically wrapped in viewModelScope.launch
    data = ApiClient.fetchData()
  }
}
```

**Generates:**
```kotlin
fun load() {
    viewModelScope.launch {
        data = ApiClient.fetchData()
    }
}
```

---

## Implementation Status

| Feature | Status | Phase |
|---------|--------|-------|
| Inline state (remember/mutableStateOf) | ✅ Complete | 0 |
| Auto-ViewModel for complex components | ✅ Complete | 1.1 |
| @store class parsing | ✅ Complete | 1 |
| ViewModel generation | ✅ Complete | 1 |
| Auto-detection at usage sites | ✅ Complete | 2 |
| Derived properties | ✅ Complete | 3 |
| Suspend function auto-wrapping | ✅ Complete | 4 |
| Hilt integration | ✅ Complete | 5 |
| Lifecycle hooks | ✅ Complete | 1.1 |

**Test Coverage:** 38/38 tests passing (100%)

---

## Examples

### Simple Component (Tier 1)
```whitehall
<!-- Counter.wh -->
<script>
  var count = 0
</script>

<Button onClick={() => count++}>
  Count: {count}
</Button>
```

### Complex Component (Tier 2 - Auto-ViewModel)
```whitehall
<!-- FeedView.wh -->
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

### Screen Store (Tier 3)
```whitehall
<!-- stores/UserProfile.wh -->
@store class UserProfile {
  var name: String = ""
  var email: String = ""

  suspend fun save() {
    ApiClient.updateProfile(name, email)
  }
}
```

```whitehall
<!-- screens/ProfileScreen.wh -->
<script>
  val profile = UserProfile()
</script>

<Column>
  <TextField bind:value={profile.name} />
  <TextField bind:value={profile.email} />
  <Button onClick={profile::save}>Save</Button>
</Column>
```

---

## Rationale

**Why three tiers?**
1. **Simple stays simple** - Basic counters don't need ViewModel boilerplate
2. **Complex gets structure** - Async operations get proper architecture
3. **Shared state is explicit** - @store makes intent clear

**Why automatic escalation?**
- Developer doesn't choose - transpiler decides based on complexity
- No premature optimization
- Consistent patterns across codebase

**Why ViewModels by default for complex components?**
- Rotation survival should be default
- Modern Android best practices
- Proper coroutine lifecycle management

---

## Future Considerations

**Phase 1.2:** Imported classes with `var` - Auto-detect at usage sites
**Phase 1.3:** Global singletons - `@store object` pattern for app-wide state
**Phase 2.0:** Advanced features - Store composition, DevTools, persistence

---

## See Also

- [STORE.md](/docs/STORE.md) - Complete implementation documentation
- [Decision 009: Lifecycle Hooks](./009-lifecycle-hooks.md) - Lifecycle hook integration
- [NEXTSTEPS.md](/docs/NEXTSTEPS.md) - Current status and roadmap
