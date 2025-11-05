# Suspend Functions and Coroutine Scopes

**Status:** 🤔 Open Question - Decision Needed

---

## The Core Question

When users write `suspend fun` in their Whitehall code, should we:
- **Option A:** Keep them as `suspend` functions (explicit scope management)
- **Option B:** Auto-wrap them in coroutine scopes (hide complexity)

This decision affects:
- Call site ergonomics (how easy it is to call these functions)
- Control over coroutine context (threading, cancellation)
- Kotlin-native feel vs. Whitehall magic
- Different contexts (ViewModels, singletons, components)

---

## Option A: Keep Functions as `suspend` (Explicit Scopes)

### You Write:
```whitehall
<script>
  var isLoading = false

  suspend fun save() {
    isLoading = true
    api.save()
    isLoading = false
  }
</script>

<Button onClick={save}>Save</Button>
```

### Generated Code:
```kotlin
class MyScreenViewModel : ViewModel() {
    // ... state ...

    suspend fun save() {  // ← Still suspend
        isLoading = true
        api.save()
        isLoading = false
    }
}

@Composable
fun MyScreen() {
    val vm = viewModel<MyScreenViewModel>()
    val scope = rememberCoroutineScope()  // ← Need to auto-generate this

    Button(onClick = {
        scope.launch { vm.save() }  // ← Explicit launch needed
    })
}
```

### Pros:
- ✅ You control when/how coroutines launch
- ✅ More Kotlin-native (suspend functions are standard)
- ✅ Clear that async work is happening
- ✅ Can customize scope/context at call site: `scope.launch(Dispatchers.IO) { save() }`
- ✅ No magic/surprises

### Cons:
- ❌ More verbose at call site: `onClick={() => scope.launch { save() }}`
- ❌ Need to auto-generate `rememberCoroutineScope()` in components
- ❌ Easy to forget the scope and get compile errors
- ❌ Less beginner-friendly

---

## Option B: Auto-Wrap in Coroutine Scope (Hide Complexity)

### You Write:
```whitehall
<script>
  var isLoading = false

  suspend fun save() {  // ← You write suspend
    isLoading = true
    api.save()
    isLoading = false
  }
</script>

<Button onClick={save}>Save</Button>  // ← Direct call, simple!
```

### Generated Code:
```kotlin
class MyScreenViewModel : ViewModel() {
    // ... state ...

    fun save() {  // ← NOT suspend anymore
        viewModelScope.launch {  // ← Auto-wrapped
            isLoading = true
            api.save()
            isLoading = false
        }
    }
}

@Composable
fun MyScreen() {
    val vm = viewModel<MyScreenViewModel>()

    Button(onClick = { vm.save() })  // ← Simple call, no scope needed
}
```

### Pros:
- ✅ Clean call sites: `onClick={save}`
- ✅ Less boilerplate
- ✅ Beginner-friendly
- ✅ Matches what most Android developers do anyway
- ✅ Works seamlessly with event handlers

### Cons:
- ❌ Less control (can't customize scope or context)
- ❌ Less "Kotlin-native"
- ❌ Hides that async work is happening (could be surprising)
- ❌ Always runs on Main dispatcher (viewModelScope default)

---

## Different Contexts Matter

### Context 1: From `onClick` (Event Handlers)

**Option A (suspend):**
```whitehall
<script>
  // Need to auto-generate this in every component with suspend calls
  val scope = rememberCoroutineScope()

  suspend fun save() { ... }
</script>

<Button onClick={() => scope.launch { save() }}>Save</Button>
```
**Problem:** Where does `scope` come from? We'd need to:
- Detect that component calls suspend functions
- Auto-generate `val scope = rememberCoroutineScope()`
- Rewrite onClick to wrap in `scope.launch { }`

**Option B (auto-wrap):**
```whitehall
<script>
  suspend fun save() { ... }  // Auto-wrapped to non-suspend
</script>

<Button onClick={save}>Save</Button>
```
Clean and simple!

---

### Context 2: From `onMount` (Lifecycle Hooks)

**Option A (suspend):**
```whitehall
onMount {
  loadData()  // ✅ Works! LaunchedEffect provides scope
}
```

**Option B (auto-wrap):**
```whitehall
onMount {
  loadData()  // ✅ Also works! Function is non-suspend
}
```

Both work fine here.

---

### Context 3: Calling Other Suspend Functions

**Option A (suspend):**
```whitehall
suspend fun save() {
  repository.save(name)  // ✅ Can call suspend functions directly
  analytics.track()       // ✅ If this is suspend too
}
```

**Option B (auto-wrap):**
```whitehall
suspend fun save() {
  repository.save(name)  // ✅ Still works (inside viewModelScope.launch)
  analytics.track()       // ✅ Also works
}
```

Both work fine - the body is still in a suspend context.

---

### Context 4: Advanced Use Cases

**Scenario:** Need to run on IO thread for heavy computation

**Option A (suspend):**
```kotlin
// User can do this at call site:
scope.launch(Dispatchers.IO) { save() }
```

**Option B (auto-wrap):**
```kotlin
// Can't customize - always uses viewModelScope (Main dispatcher)
// User would need to wrap internals:
suspend fun save() {
  withContext(Dispatchers.IO) {
    // Heavy work here
  }
}
```

Option A is more flexible, Option B requires `withContext` internally.

---

## Singletons: A Special Case

```whitehall
@store
object AppSettings {
  var darkMode = false

  suspend fun loadFromDisk() {
    darkMode = dataStore.read()
  }
}
```

### Problem: Singletons Don't Have `viewModelScope`

**Option A (suspend):**
```kotlin
object AppSettings {
    // ... StateFlow ...

    suspend fun loadFromDisk() {  // ← Stays suspend
        darkMode = dataStore.read()
    }
}

// Call sites:
// From onMount:
onMount {
  AppSettings.loadFromDisk()  // ✅ Works (LaunchedEffect scope)
}

// From onClick:
<Button onClick={() => scope.launch { AppSettings.loadFromDisk() }}>
```

Caller provides the scope.

**Option B (auto-wrap):**
```kotlin
object AppSettings {
    // Need to create a global scope - risky!
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)

    fun loadFromDisk() {  // Non-suspend
        scope.launch {
            darkMode = dataStore.read()
        }
    }
}
```

**Problems with auto-wrap for singletons:**
- ❌ Global scope lives forever (potential memory leaks)
- ❌ No lifecycle - when to cancel?
- ❌ Who owns the scope?
- ❌ Hard to test (can't inject scope)

---

## Components Without ViewModel (No `var`)

```whitehall
<script>
  // No var, so no ViewModel generated

  suspend fun logAnalytics() {
    analytics.log("screen_viewed")
  }

  onMount {
    logAnalytics()
  }
</script>
```

**Option A (suspend):**
```kotlin
@Composable
fun MyComponent() {
    LaunchedEffect(Unit) {
        logAnalytics()  // ✅ Works
    }
}

suspend fun logAnalytics() { ... }
```

**Option B (auto-wrap):**
Problem! No ViewModel means no `viewModelScope`. We'd need `rememberCoroutineScope()`:
```kotlin
@Composable
fun MyComponent() {
    val scope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        logAnalytics()  // But logAnalytics() is non-suspend now...
    }
}

fun logAnalytics() {
    // What scope to use?
}
```

Doesn't work well without ViewModel.

---

## Hybrid Approach (Recommendation)

**Different contexts need different solutions:**

### For ViewModels (Components with `var`)
**Use Option B (auto-wrap):**
```whitehall
<script>
  var isLoading = false

  suspend fun save() { ... }  // → Auto-wrapped in viewModelScope.launch
</script>

<Button onClick={save}>Save</Button>  // Clean!
```

**Why:**
- Most common case (screen state)
- Clean call sites
- Matches Android conventions
- ViewModels have proper lifecycle

### For Singletons (`@store object`)
**Use Option A (keep suspend):**
```whitehall
@store
object AppSettings {
  var darkMode = false

  suspend fun loadFromDisk() { ... }  // → Stays suspend
}

// Caller provides scope:
onMount {
  AppSettings.loadFromDisk()
}
```

**Why:**
- Singletons shouldn't own scope lifecycle
- Safer (no global scope)
- Caller provides context
- Less common case, explicit is better

### For Components Without ViewModel (no `var`)
**Use Option A (keep suspend):**
```whitehall
<script>
  suspend fun logAnalytics() { ... }  // → Stays suspend

  onMount {
    logAnalytics()  // Works in LaunchedEffect
  }
</script>
```

**Why:**
- No ViewModel to provide scope
- Rare case
- Use LaunchedEffect naturally

---

## Summary Table

| Context | Option A (suspend) | Option B (auto-wrap) | Hybrid |
|---------|-------------------|---------------------|--------|
| **ViewModel + onClick** | Need `rememberCoroutineScope()` 😐 | Clean: `onClick={save}` ✅ | Auto-wrap ✅ |
| **ViewModel + onMount** | Clean ✅ | Clean ✅ | Auto-wrap ✅ |
| **Singleton + onClick** | Need scope at call site 😐 | Need global scope ❌ | Keep suspend ✅ |
| **Singleton + onMount** | Clean ✅ | Clean but risky scope ⚠️ | Keep suspend ✅ |
| **No ViewModel + onClick** | Need `rememberCoroutineScope()` 😐 | Doesn't work ❌ | Keep suspend ✅ |
| **No ViewModel + onMount** | Clean ✅ | Doesn't work ❌ | Keep suspend ✅ |
| **Advanced scoping** | Flexible ✅ | Limited ❌ | Keep suspend ✅ |
| **Learning curve** | Steeper 📚 | Easier 🎯 | Balanced |

---

## Open Questions

1. **Is the hybrid approach too complex?** Different rules for ViewModels vs singletons vs plain components.

2. **Should we support custom dispatchers?** If auto-wrapping, how does user specify `Dispatchers.IO`?

3. **What about error handling?** Wrapped functions swallow exceptions into viewModelScope. Suspend functions bubble up.

4. **Testing implications?** Auto-wrapped functions are harder to test (need to test coroutine behavior).

5. **Documentation burden?** Need to clearly explain when wrapping happens vs when it doesn't.

---

## Examples to Consider

### Example 1: Sequential API Calls
```whitehall
suspend fun loadProfile() {
  val user = api.getUser()      // First call
  val posts = api.getPosts(user.id)  // Second call (depends on first)
  // ...
}
```

**Option A:** Natural suspend flow
**Option B:** Still works (inside launch block)

### Example 2: Parallel API Calls
```whitehall
suspend fun loadDashboard() {
  val user = async { api.getUser() }
  val posts = async { api.getPosts() }
  // Wait for both
}
```

**Option A:** User controls with `async`/`await`
**Option B:** Still works, but wrapped in outer `launch`

### Example 3: Cancellation
```whitehall
suspend fun search(query: String) {
  delay(300)  // Debounce
  api.search(query)
}
```

**Option A:** Caller controls cancellation
**Option B:** ViewModelScope cancels on ViewModel clear (automatic)

### Example 4: Error Handling
```whitehall
suspend fun save() {
  try {
    api.save()
  } catch (e: Exception) {
    error = e.message
  }
}
```

**Option A:** Errors propagate to caller
**Option B:** Errors stay in ViewModel (contained)

---

## Decision Needed

**Questions to answer:**
1. Do we use Option A, B, or Hybrid?
2. If Hybrid, is the complexity worth it?
3. How do we document this clearly?
4. What's the migration path from current implementation?

**Next steps:** Review these tradeoffs and make a decision before implementing the new `var`-based auto-ViewModel system.
