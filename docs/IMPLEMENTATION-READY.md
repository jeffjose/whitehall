# Implementation Ready Summary

**Status:** 🚀 All Design Decisions Complete - Ready for Implementation

---

## Overview

This document summarizes all design decisions that have been finalized and are ready for implementation. All major design questions have been resolved.

---

## ✅ Completed Design Decisions

### 1. State Management (Auto-ViewModel)

**Document:** [STATE-MANAGEMENT.md](STATE-MANAGEMENT.md)

**Key Decisions:**
- **Auto-ViewModel based on `var` detection** - No annotation needed for screen-scoped state
- **`@store` redefined** - Now means global singleton only (not screen-scoped)
- **Inline `var` → ViewModel** - Components with local `var` automatically generate ViewModel
- **Imported classes with `var` → ViewModel** - Auto-detect and use `viewModel<T>()`
- **Plain classes (no `var`) → Plain** - Remain regular Kotlin classes

**Examples:**
```whitehall
<!-- Inline var (auto-ViewModel) -->
<script>
  var count = 0
  var name = ""

  suspend fun save() {
    api.save(name)
  }
</script>

<!-- Separate class (auto-ViewModel) -->
class UserProfile {
  var name = ""
  var email = ""

  suspend fun save() {
    repository.save(name, email)
  }
}

<!-- Singleton (global state) -->
@store
object AppSettings {
  var darkMode = false
  var language = "en"
}
```

---

### 2. Suspend Functions & Scopes

**Document:** [SUSPEND-FUNCTIONS.md](SUSPEND-FUNCTIONS.md)

**Key Decision:** Option C - Auto-infer scope with clean override syntax

#### Level 1: Auto-Infer (Default - 90% of cases)
```whitehall
<Button onClick={save}>Save</Button>
```
Auto-wraps in appropriate scope based on context.

#### Level 2: Thread Control (Dispatchers)
```whitehall
<Button onClick={() => io { loadData() }}>Load</Button>
<Button onClick={() => cpu { processData() }}>Process</Button>
<Button onClick={() => main { updateUI() }}>Update</Button>
```

**Dispatcher mapping:**
- `io { }` → `Dispatchers.IO` (network, disk, database)
- `cpu { }` → `Dispatchers.Default` (heavy computation)
- `main { }` → `Dispatchers.Main` (UI thread)

#### Level 3: Custom Scopes
```whitehall
val uploadScope = $scope()

<Button onClick={() => uploadScope.launch { uploadFile() }}>Upload</Button>
```

**Context-specific behavior:**
- Components with `var` → Use `viewModelScope.launch`
- Components without `var` → Use `rememberCoroutineScope()`
- Inside `onMount` → Already has `LaunchedEffect` scope
- Singletons (`@store object`) → Keep as `suspend`, caller provides scope

---

### 3. Special Syntax Patterns

**Two prefix patterns:**

| Pattern | Meaning | Examples |
|---------|---------|----------|
| `@` prefix | **Declare** special behavior | `@prop val name`, `@store object Settings`, `@inject constructor()` |
| `$` prefix | **Access** compiler-provided values | `$screen.params.id`, `$route.profile`, `$scope()` |

**Semantic rule:**
- `@` = "This IS something special" (declaration/annotation)
- `$` = "Get me something special" (reference/access)

**Consistency:**
```whitehall
// Declarations (using @)
@prop val onClick: () -> Unit
@store object AppSettings
@inject constructor(repository: Repo)

// References (using $)
val userId = $screen.params.id
navigate($route.profile(id = userId))
val uploadScope = $scope()
```

---

### 4. Routing (Already Implemented)

**Pattern:** SvelteKit-style file-based routing with `+screen.wh`

**File structure:**
```
screens/
  +screen.wh                      → / (home)
  profile/+screen.wh              → /profile
  profile/[userId]/+screen.wh     → /profile/:userId
  settings/
    +screen.wh                    → /settings
    account/+screen.wh            → /settings/account
```

**Route references:**
```whitehall
navigate($route.profile(id = userId))
```

**Route parameters:**
```whitehall
val userId = $screen.params.id
```

**Generated output:**
- `Routes.kt` - Sealed interface with type-safe routes
- Auto-integration with Jetpack Compose Navigation
- Type-safe navigation with parameters

---

## 📋 Implementation Tasks

### Phase 1: Auto-ViewModel (var-based detection)

**From STATE-MANAGEMENT.md:**

1. **Inline `var` in components:**
   - Scan `<script>` blocks for local `var` declarations (exclude `@prop var`)
   - Generate ViewModel for component if any `var` found
   - Generate UiState data class from all `var` declarations
   - Auto-wrap suspend functions in `viewModelScope.launch`
   - Generate `collectAsState()` and rewrite references

2. **Imported classes with `var`:**
   - During parsing, detect `var` properties in classes
   - Build registry of "reactive classes" (classes with `var`)
   - When `val x = ReactiveClass()` detected, generate `viewModel<T>(key = "x")`
   - Same UiState + StateFlow generation
   - Support `@Inject constructor()` for Hilt integration

3. **Redefine `@store` for singletons:**
   - Detect `@store object` pattern
   - Generate StateFlow-based singleton (NOT ViewModel)
   - No `viewModel()` call, direct property access
   - Lives for app lifetime

4. **Update registry system:**
   - Change from "has @store" to "has var" detection
   - Track class type (screen-scoped vs singleton)
   - Support Hilt detection (preserve existing Phase 5 work)

---

### Phase 2: Suspend Functions & Scopes

**From SUSPEND-FUNCTIONS.md:**

1. **Auto-wrap for ViewModels:**
   - Detect `suspend fun` in ViewModel contexts
   - Transform to non-suspend, wrap body in `viewModelScope.launch { }`
   - Default dispatcher: `Dispatchers.Main`

2. **Dispatcher syntax:**
   - Parse `io { }`, `cpu { }`, `main { }` blocks
   - Transform to `viewModelScope.launch(Dispatchers.X) { }`
   - Map: `io` → `IO`, `cpu` → `Default`, `main` → `Main`

3. **Custom scope syntax:**
   - Parse `val myScope = $scope()` pattern
   - Detect `$scope()` function call
   - Transform to `val myScope = rememberCoroutineScope()`
   - Parse `myScope.launch { }` blocks
   - Transform to Kotlin `myScope.launch { }`

4. **Context detection:**
   - Components with `var`: Use `viewModelScope`
   - Components without `var`: Use `rememberCoroutineScope()` (if needed)
   - Inside `onMount`: Already in scope
   - Singletons: Keep as `suspend`

5. **Error messages:**
   - Warn if using `io` for pure computation (suggest `cpu`)
   - Warn if using `cpu` for network calls (suggest `io`)
   - Clear errors for scope misuse

---

### Phase 3: Testing & Examples

1. **Update existing examples:**
   - Migrate `examples/counter-store/` to new syntax (remove `@store` annotation)
   - Create examples showing all three levels (auto, dispatcher, custom scope)
   - Create singleton example (`@store object`)

2. **Create new examples:**
   - Simple counter (inline `var`)
   - Profile form (separate class with `var`)
   - Settings screen (singleton `@store object`)
   - File upload (custom scope with cancellation)
   - Data processing (dispatcher control)

3. **Test cases:**
   - Inline `var` generates ViewModel
   - Imported class with `var` uses `viewModel<T>()`
   - Plain class (no `var`) remains plain
   - `@store object` generates singleton StateFlow
   - `@prop var` does NOT generate ViewModel
   - Multiple instances get unique keys
   - Hilt integration still works
   - Dispatcher syntax generates correct Dispatchers
   - `$scope()` generates `rememberCoroutineScope()`

---

## 🎯 Implementation Order

**Recommended order:**

1. **Phase 1.1:** Inline `var` detection and ViewModel generation
2. **Phase 1.2:** Imported class `var` detection
3. **Phase 1.3:** `@store object` singleton generation
4. **Phase 2.1:** Auto-wrap suspend functions (already mostly done in Phase 4)
5. **Phase 2.2:** Dispatcher syntax (`io`/`cpu`/`main`)
6. **Phase 2.3:** Custom scope syntax (`$scope()`)
7. **Phase 3:** Update examples and tests

**Why this order:**
- Phase 1.1-1.3 builds the core auto-ViewModel system
- Phase 2.1 is mostly done, just needs refinement
- Phase 2.2-2.3 adds advanced features on top
- Phase 3 validates everything works

---

## 🔄 Migration Path

### From Current Implementation

**Current state (Phase 0-5):**
- `@store` annotation for screen-scoped state ✅ (Phase 1-4)
- Auto-detection at usage sites ✅ (Phase 2)
- Hilt integration ✅ (Phase 5)
- Auto-wrap suspend functions ✅ (Phase 4)

**Changes needed:**
1. **Remove `@store` requirement** for screen-scoped state
2. **Detect `var` instead** for auto-ViewModel
3. **Redefine `@store`** to mean singleton only
4. **Add dispatcher syntax** (new feature)
5. **Add custom scope syntax** (new feature)

**Breaking changes:**
- `@store class` for screen state → Just `class` (remove annotation)
- No breaking changes for users (annotation still works, just optional now)
- `@store object` is new syntax (previously unsupported)

---

## 📖 Related Documents

- [STATE-MANAGEMENT.md](STATE-MANAGEMENT.md) - Complete state management design
- [SUSPEND-FUNCTIONS.md](SUSPEND-FUNCTIONS.md) - Suspend functions and scopes design

---

## ✅ Sign-off

All design decisions are finalized:
- ✅ Auto-ViewModel based on `var` detection
- ✅ `@store` redefined for singletons
- ✅ Suspend function auto-wrap with override
- ✅ Dispatcher syntax (`io`/`cpu`/`main`)
- ✅ Custom scope syntax (`$scope()`)
- ✅ Special syntax patterns (`@` vs `$`)

**Ready for implementation!** 🚀
