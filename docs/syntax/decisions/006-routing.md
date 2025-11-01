# Decision 006: Routing & Navigation

**Status:** ✅ Decided
**Date:** 2025-11-01
**Decider:** User preference + Navigation 2.8+/3.0 compatibility

## Context

Routing defines how users navigate between screens in an Android app. Key requirements:

1. **Developer experience** - Easy to add new screens, minimal boilerplate
2. **Type safety** - Compile-time errors for invalid routes
3. **Compose compatibility** - Maps to Jetpack Compose Navigation 2.8+/3.0
4. **Deep linking** - Support for app:// and https:// URLs
5. **File organization** - Clear project structure

**Navigation 2.8+ (Type-Safe) uses:**
```kotlin
// Define routes with Kotlin Serialization
@Serializable object Home
@Serializable data class Profile(val userId: String)

NavHost(navController, startDestination = Home) {
  composable<Home> { HomeScreen() }
  composable<Profile> { backStackEntry ->
    val route = backStackEntry.toRoute<Profile>()
    ProfileScreen(userId = route.userId)
  }
}

// Navigate (type-safe!)
navController.navigate(Home)
navController.navigate(Profile(userId = "123"))
```

**Old string-based approach (deprecated):**
```kotlin
navController.navigate("profile/123")  // ❌ No type safety
```

---

## Option A: File-Based Routing (SvelteKit/Next.js style)

### Project Structure

```
src/routes/
├── +page.wh              # / (home)
├── login/
│   └── +page.wh          # /login
├── profile/
│   ├── +page.wh          # /profile
│   └── [id]/
│       └── +page.wh      # /profile/:id
└── settings/
    ├── +page.wh          # /settings
    ├── account/
    │   └── +page.wh      # /settings/account
    └── privacy/
        └── +page.wh      # /settings/privacy
```

### Route File

**`src/routes/profile/[id]/+page.wh`:**
```whitehall
<script>
  @prop val id: String  // Route parameter automatically injected

  var user: User? = null
  var isLoading = true

  onMount {
    user = UserRepository.getUser(id)
    isLoading = false
  }
</script>

@if (isLoading) {
  <LoadingSpinner />
} else if (user != null) {
  <Column padding={16}>
    <Text fontSize={24}>{user.name}</Text>
    <Text>{user.email}</Text>
  </Column>
}
```

### Navigation

```whitehall
// String-based
navigate("/profile/123")
navigate("/settings/account")

// Or with function
<Button text="View Profile" onClick={() => navigate("/profile/123")} />
```

**Pros:**
- Amazing DX - create file, it's a route
- Clear structure - file path = URL path
- Familiar from web frameworks
- No routing config needed

**Cons:**
- String-based navigation (no type safety)
- Magic - how does it work under the hood?
- Route params need conventions (`[id]` syntax)

---

## Option B: Explicit Routes File

### `src/routes.wh`

```whitehall
routes {
  "/" => HomePage
  "/login" => LoginPage
  "/profile/:id" => ProfilePage
  "/settings" => SettingsLayout {
    "/account" => SettingsAccount
    "/privacy" => SettingsPrivacy
  }
}
```

### Components live anywhere

```
src/
├── routes.wh
├── screens/
│   ├── Home.wh
│   ├── Login.wh
│   └── Profile.wh
```

**Pros:**
- Explicit - all routes visible at once
- Flexible file organization

**Cons:**
- Extra config file
- Still string-based paths
- Disconnected from components

---

## Option C: Type-Safe Routes (Code-based)

### `src/Navigation.wh`

```whitehall
sealed class Route {
  object Home
  object Login
  data class Profile(val id: String)

  object Settings {
    object Account
    object Privacy
  }
}

navigation {
  Route.Home => HomePage()
  Route.Login => LoginPage()
  Route.Profile => ProfilePage(id = it.id)
  Route.Settings.Account => SettingsAccountPage()
  Route.Settings.Privacy => SettingsPrivacyPage()
}
```

### Navigation

```whitehall
navigate(Route.Profile(id = "123"))  // Type-safe!
navigate(Route.Settings.Account)
```

**Pros:**
- Fully type-safe
- Compiler catches errors
- IDE autocomplete
- Similar to Compose type-safe navigation

**Cons:**
- More boilerplate
- Routes separate from screens
- Less file-based magic

---

## Option D: Hybrid (File-Based + Generated @Serializable Routes) ⭐

**The best of both worlds - compatible with Navigation 2.8+/3.0**

### File Structure (same as Option A)

```
src/routes/
├── +page.wh              # / → Routes.Home
├── login/
│   └── +page.wh          # /login → Routes.Login
├── profile/
│   └── [id]/
│       └── +page.wh      # /profile/:id → Routes.Profile(id)
└── settings/
    ├── +page.wh          # /settings → Routes.Settings.Index
    ├── account/
    │   └── +page.wh      # /settings/account → Routes.Settings.Account
    └── privacy/
        └── +page.wh      # /settings/privacy → Routes.Settings.Privacy
```

### Generated Type-Safe API (Navigation 2.8+ Compatible)

Compiler auto-generates **@Serializable objects/data classes:**

```kotlin
// Auto-generated Routes.kt
import kotlinx.serialization.Serializable

sealed interface Routes {
  @Serializable object Home : Routes
  @Serializable object Login : Routes
  @Serializable data class Profile(val id: String) : Routes

  sealed interface Settings : Routes {
    @Serializable object Index : Settings
    @Serializable object Account : Settings
    @Serializable object Privacy : Settings
  }
}
```

### Generated NavHost

```kotlin
NavHost(navController, startDestination = Routes.Home) {
  composable<Routes.Home> {
    HomePage()
  }

  composable<Routes.Login> {
    LoginPage()
  }

  composable<Routes.Profile> { backStackEntry ->
    val route = backStackEntry.toRoute<Routes.Profile>()
    ProfilePage(id = route.id)
  }

  composable<Routes.Settings.Index> {
    SettingsPage()
  }

  composable<Routes.Settings.Account> {
    SettingsAccountPage()
  }

  composable<Routes.Settings.Privacy> {
    SettingsPrivacyPage()
  }
}
```

### Usage in Whitehall Components

```whitehall
<script>
  fun goToProfile(userId: String) {
    navigate(Routes.Profile(id = userId))  // Type-safe!
  }
</script>

<Column>
  <Button text="Home" onClick={() => navigate(Routes.Home)} />
  <Button text="Login" onClick={() => navigate(Routes.Login)} />
  <Button text="Profile" onClick={() => goToProfile("123")} />
  <Button text="Settings" onClick={() => navigate(Routes.Settings.Account)} />
</Column>
```

**Pros:**
- File-based DX (create file = route exists)
- Type-safe navigation (Navigation 2.8+ compatible)
- Autocomplete in IDE
- Clear file structure
- Uses @Serializable (standard Kotlin pattern)
- Best of both worlds

**Cons:**
- Most complex transpilation
- Need to generate @Serializable routes
- Magic (but predictable and standard)

---

## Route Parameters

### Dynamic Segments

**File:** `src/routes/post/[id]/+page.wh`

```whitehall
<script>
  @prop val id: String  // Automatically injected from route

  var post: Post? = null

  onMount {
    post = PostRepository.getPost(id)
  }
</script>
```

**Generated:**
```whitehall
Routes.post(id: String)  // Type-safe function
```

**Navigation:**
```whitehall
navigate(Routes.post("123"))
```

---

### Multiple Parameters

**File:** `src/routes/users/[userId]/posts/[postId]/+page.wh`

```whitehall
<script>
  @prop val userId: String
  @prop val postId: String
</script>
```

**Generated:**
```whitehall
Routes.users.posts(userId: String, postId: String)
```

**Navigation:**
```whitehall
navigate(Routes.users.posts(userId = "42", postId = "123"))
```

---

### Optional Parameters

**File:** `src/routes/search/[[query]]/+page.wh`

Double brackets = optional

```whitehall
<script>
  @prop val query: String? = null  // Optional

  val results = query?.let { searchPosts(it) } ?: emptyList()
</script>
```

**Generated:**
```whitehall
Routes.search(query: String? = null)
```

**Navigation:**
```whitehall
navigate(Routes.search())           // /search
navigate(Routes.search("android"))  // /search/android
```

---

### Catch-All Routes

**File:** `src/routes/docs/[...path]/+page.wh`

```whitehall
<script>
  @prop val path: List<String>  // ["api", "users", "create"]

  val doc = loadDoc(path.joinToString("/"))
</script>
```

**Generated:**
```whitehall
Routes.docs(path: List<String>)
```

**Navigation:**
```whitehall
navigate(Routes.docs(listOf("api", "users", "create")))
// /docs/api/users/create
```

---

## Query Parameters

### Syntax

```whitehall
// Option A: Map
navigate(Routes.search(), query = mapOf("q" to "android", "sort" to "date"))

// Option B: Type-safe with data class
data class SearchQuery(val q: String, val sort: String = "relevance")
navigate(Routes.search(), query = SearchQuery(q = "android"))

// Option C: Builder
navigate(Routes.search()
  .withQuery("q", "android")
  .withQuery("sort", "date")
)
```

### In Component

```whitehall
<script>
  @prop val query: SearchQuery?  // Parsed from URL query params

  val searchTerm = query?.q ?: ""
  val sortBy = query?.sort ?: "relevance"
</script>
```

---

## Layouts

### Shared Layout for Multiple Routes

**File:** `src/routes/(app)/+layout.wh`

Applies to all routes under `(app)` group.

```
src/routes/
├── (app)/
│   ├── +layout.wh        # Layout for app routes
│   ├── home/
│   │   └── +page.wh
│   ├── profile/
│   │   └── +page.wh
│   └── settings/
│       └── +page.wh
└── (auth)/
    ├── +layout.wh        # Different layout for auth
    ├── login/
    │   └── +page.wh
    └── signup/
        └── +page.wh
```

**`(app)/+layout.wh`:**
```whitehall
<script>
  @prop val children: @Composable () -> Unit  // Child route
</script>

<Scaffold
  bottomBar={
    <BottomNavigation>
      <NavItem icon="home" route={Routes.home()} />
      <NavItem icon="person" route={Routes.profile()} />
      <NavItem icon="settings" route={Routes.settings()} />
    </BottomNavigation>
  }
>
  {children()}  <!-- Child route renders here -->
</Scaffold>
```

**Note:** `(app)` and `(auth)` are **route groups** - parentheses mean they don't affect the URL path.

---

## Navigation Guards

### Protected Routes

```whitehall
// Option A: Per-route guard
<script>
  @guard requireAuth

  // This route requires authentication
</script>
```

```whitehall
// Option B: Layout guard
// (auth-required)/+layout.wh
<script>
  onMount {
    if (!AuthRepository.isLoggedIn()) {
      navigate(Routes.login())
    }
  }
</script>
```

```whitehall
// Option C: Global guards in routes config (future)
guards {
  "/admin/*" => requireAdmin
  "/app/*" => requireAuth
}
```

---

## Deep Linking

### Automatic Deep Link Support

Every route automatically supports deep linking:

**File:** `src/routes/product/[id]/+page.wh`

Automatically handles:
- `myapp://product/123`
- `https://myapp.com/product/123`

### Custom Deep Link Configuration

```whitehall
// whitehall.toml
[android]
deepLinks = [
  { scheme = "myapp", host = "product" },
  { scheme = "https", host = "myapp.com" }
]
```

Compiler generates Android manifest entries.

---

## Navigation API

### Core Functions

```whitehall
// Navigate to route
navigate(Routes.home())
navigate(Routes.profile("123"))

// Navigate with options
navigate(Routes.login(), options = {
  popUpTo = Routes.home()  // Clear back stack to home
  singleTop = true         // Don't create duplicate
})

// Replace current route
replace(Routes.home())

// Go back
goBack()

// Pop to route
popTo(Routes.home())
```

### In Components

```whitehall
<script>
  // Access navigation
  val navController = useNavigation()

  fun handleLogout() {
    AuthRepository.logout()
    navController.navigate(Routes.login()) {
      popUpTo(Routes.home()) { inclusive = true }
    }
  }
</script>
```

---

## Real-World Example

### E-commerce App Structure

```
src/routes/
├── +page.wh                          # Home / splash
├── (auth)/
│   ├── login/+page.wh
│   └── signup/+page.wh
├── (shop)/
│   ├── +layout.wh                    # Bottom nav layout
│   ├── home/+page.wh
│   ├── search/
│   │   └── [[query]]/+page.wh
│   ├── categories/
│   │   ├── +page.wh
│   │   └── [categoryId]/+page.wh
│   ├── products/
│   │   └── [productId]/
│   │       ├── +page.wh
│   │       └── reviews/+page.wh
│   └── cart/+page.wh
└── (account)/
    ├── +layout.wh                    # Account layout
    ├── profile/+page.wh
    ├── orders/
    │   ├── +page.wh
    │   └── [orderId]/+page.wh
    └── settings/
        ├── +page.wh
        ├── account/+page.wh
        └── privacy/+page.wh
```

### Generated Routes API

```whitehall
object Routes {
  fun home() = "/"

  object auth {
    fun login() = "/login"
    fun signup() = "/signup"
  }

  object shop {
    fun home() = "/home"
    fun search(query: String? = null) = "/search" + (query?.let { "/$it" } ?: "")

    object categories {
      fun index() = "/categories"
      fun details(categoryId: String) = "/categories/$categoryId"
    }

    object products {
      fun details(productId: String) = "/products/$productId"
      fun reviews(productId: String) = "/products/$productId/reviews"
    }

    fun cart() = "/cart"
  }

  object account {
    fun profile() = "/profile"

    object orders {
      fun index() = "/orders"
      fun details(orderId: String) = "/orders/$orderId"
    }

    object settings {
      fun index() = "/settings"
      fun account() = "/settings/account"
      fun privacy() = "/settings/privacy"
    }
  }
}
```

### Usage

```whitehall
// ProductCard.wh
<script>
  @prop val product: Product

  fun viewProduct() {
    navigate(Routes.shop.products.details(product.id))
  }
</script>

<Card onClick={viewProduct}>
  <Text>{product.name}</Text>
  <Text>${product.price}</Text>
</Card>
```

---

## Recommendation: Option D (Hybrid)

**File-based routing with auto-generated @Serializable type-safe navigation.**

### Navigation 2.8+/3.0 Compatibility

Our approach **fully compatible** with Navigation 2.8+/3.0:

| Whitehall | Navigation 2.8+ |
|-----------|----------------|
| File-based routes | @Serializable route objects |
| `[id]` parameters | `data class Route(val id: String)` |
| Generated Routes | Kotlin sealed interfaces |
| `navigate(Routes.Profile(id))` | `navController.navigate(Profile(id))` |
| Auto-generated NavHost | `composable<RouteType>` |

**Dependencies auto-added:**
```kotlin
// Generated in build.gradle.kts
implementation("androidx.navigation:navigation-compose:2.8.0+")
implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")

plugins {
  id("org.jetbrains.kotlin.plugin.serialization")
}
```

### Implementation Strategy

1. **File scanning** - Compiler scans `src/routes/` directory
2. **@Serializable route generation** - Generate sealed interfaces with @Serializable
3. **Type extraction** - Parse `[param]` syntax → `data class` properties
4. **NavHost generation** - Generate with `composable<T>` (Navigation 2.8+ API)
5. **Deep linking** - Auto-generate manifest entries

### Benefits

✅ **Amazing DX** - Just create a file, it's a route
✅ **Type-safe** - Navigation 2.8+ @Serializable routes
✅ **Autocomplete** - IDE knows all routes
✅ **Maintainable** - File structure is self-documenting
✅ **Standard** - Uses official Navigation 3.0 patterns
✅ **Future-proof** - Based on latest Navigation APIs
✅ **Deep links** - Automatic support

### Phase 1 Features

- File-based routes with `+page.wh`
- Dynamic parameters `[id]`
- Nested routes
- Generated Routes object
- Basic navigation functions

### Phase 2 Features (Future)

- Layouts with `+layout.wh`
- Route groups `(name)/`
- Optional parameters `[[id]]`
- Catch-all routes `[...path]`
- Navigation guards
- Query parameters
- Transitions/animations

---

## FINAL DECISION

**Use Option D: Hybrid file-based routing with generated @Serializable routes**

### What This Means

1. **File structure defines routes** - Create `src/routes/profile/[id]/+page.wh` → Route exists
2. **Compiler generates @Serializable objects** - Following Navigation 2.8+ patterns
3. **Type-safe navigation** - `navigate(Routes.Profile(id = "123"))`
4. **Standard Compose Navigation** - Uses official `composable<T>` API
5. **Auto-generated NavHost** - No manual route configuration

### Example Generated Code

**From file:** `src/routes/profile/[id]/+page.wh`

**Generates:**
```kotlin
@Serializable
data class Profile(val id: String) : Routes

composable<Routes.Profile> { backStackEntry ->
  val route = backStackEntry.toRoute<Routes.Profile>()
  ProfilePage(id = route.id)
}
```

### Phase 1 Implementation

- File-based routes with `+page.wh`
- Dynamic parameters `[id]`
- Nested routes
- Generated @Serializable Routes sealed interface
- NavHost generation with `composable<T>`
- Type-safe navigation API

### Phase 2 (Future)

- Layouts with `+layout.wh`
- Route groups `(name)/`
- Optional parameters `[[id]]`
- Catch-all routes `[...path]`
- Navigation guards
- Query parameters
- Transitions/animations

---

## Open Questions for Future

1. **Route naming convention?**
   - Decision: `+page.wh` (SvelteKit-style) for Phase 1
   - Consider `index.wh` as alternative

2. **Special files?**
   - `+layout.wh` for layouts (Phase 2)
   - `+error.wh` for error boundaries (Phase 2)
   - `+loading.wh` for loading states (Phase 2)

3. **Modal routes?**
   Phase 2 consideration:
   ```
   products/[id]/
   ├── +page.wh
   └── @modal/
       └── share/+page.wh  # Modal overlay
   ```

4. **Tab navigation?**
   Tabs should be component state, not routes (following Android patterns)
