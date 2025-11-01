# Routing & Navigation Syntax

## Context

Routing is critical for developer experience. The question: **File-based magic** (like Next.js/SvelteKit) vs **explicit configuration** (like React Router)?

**Constraints:**
- Must map to Jetpack Compose Navigation
- Need type-safe navigation
- Support deep linking
- Handle back stack properly

---

## Option A: File-Based Routing (SvelteKit-style)

### Project Structure:
```
src/
├── routes/
│   ├── +page.wh              # / (home screen)
│   ├── +layout.wh            # Root layout (optional)
│   ├── login/
│   │   └── +page.wh          # /login
│   ├── profile/
│   │   ├── +page.wh          # /profile
│   │   └── [id]/
│   │       └── +page.wh      # /profile/:id
│   └── settings/
│       ├── +page.wh          # /settings
│       ├── account/
│       │   └── +page.wh      # /settings/account
│       └── privacy/
│           └── +page.wh      # /settings/privacy
└── components/
    └── Header.wh
```

### File Contents:

**`src/routes/+page.wh`** (Home):
```whitehall
component HomePage() {
  render {
    Column {
      Text("Welcome!")
      Button("Go to Profile", onClick: () => {
        navigate("/profile")
      })
    }
  }
}
```

**`src/routes/profile/[id]/+page.wh`** (Dynamic route):
```whitehall
component ProfilePage(id: String) {  // Route param automatically injected
  state {
    user = null
  }

  onMount {
    user = fetchUser(id)
  }

  render {
    if (user) {
      Text("Profile: ${user.name}")
    } else {
      Loading()
    }
  }
}
```

**Navigation:**
```whitehall
navigate("/profile/123")           // String-based
navigate("/settings/account")      // Deep links work automatically
```

**Pros:**
- Amazing DX - just create a file, it's a route
- No routing config needed
- Clear project structure
- Familiar to web devs
- Deep linking "just works"

**Cons:**
- "Magic" - not obvious how it maps to Compose Navigation
- Type safety issues (string-based navigation)
- What about programmatic routes?
- Android back button behavior?
- How to handle navigation args (complex objects)?

**Transpilation Strategy:**
Generate a `Navigation.kt` file:
```kotlin
@Composable
fun AppNavigation() {
  val navController = rememberNavController()

  NavHost(navController, startDestination = "home") {
    composable("home") { HomePage() }
    composable("login") { LoginPage() }
    composable("profile/{id}") { backStackEntry =>
      ProfilePage(id = backStackEntry.arguments?.getString("id") ?: "")
    }
    composable("settings") { SettingsPage() }
    composable("settings/account") { SettingsAccountPage() }
    composable("settings/privacy") { SettingsPrivacyPage() }
  }
}
```

---

## Option B: Explicit Routes File (Next.js App Router-style)

### `src/routes.wh`:
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

### Components live anywhere:
```
src/
├── routes.wh
├── screens/
│   ├── Home.wh
│   ├── Login.wh
│   └── Profile.wh
└── components/
    └── Button.wh
```

**Pros:**
- Explicit - easy to see all routes at a glance
- Flexible file organization
- Clear nesting

**Cons:**
- Extra config file
- Routes and components separate (have to jump around)
- Still string-based paths

---

## Option C: Type-Safe Routes (Code-based)

### Using a DSL in code:

**`src/Navigation.wh`:**
```whitehall
sealed routes AppRoute {
  Home,
  Login,
  Profile(id: String),
  Settings {
    Account,
    Privacy
  }
}

navigation {
  AppRoute.Home => HomePage()
  AppRoute.Login => LoginPage()
  AppRoute.Profile => ProfilePage(id: it.id)
  AppRoute.Settings.Account => SettingsAccountPage()
  AppRoute.Settings.Privacy => SettingsPrivacyPage()
}
```

**Navigation:**
```whitehall
navigate(AppRoute.Profile(id: "123"))  // Type-safe!
navigate(AppRoute.Settings.Account)
```

**Pros:**
- Fully type-safe
- No string typos
- IDE autocomplete
- Compiler checks navigation
- Similar to Kotlin's type-safe navigation

**Cons:**
- More boilerplate
- Less "magical"
- Routing separate from components

**Transpiles to:**
```kotlin
sealed class AppRoute {
  object Home : AppRoute()
  object Login : AppRoute()
  data class Profile(val id: String) : AppRoute()
  // ...
}

@Composable
fun AppNavigation() {
  val navController = rememberNavController()

  NavHost(navController, startDestination = AppRoute.Home) {
    composable<AppRoute.Home> { HomePage() }
    composable<AppRoute.Login> { LoginPage() }
    composable<AppRoute.Profile> { ProfilePage(id = it.id) }
    // ...
  }
}
```

---

## Option D: Hybrid (File-based + Type Safety)

**Idea:** Use file structure BUT generate type-safe navigation APIs.

### File structure (same as Option A):
```
src/routes/
├── +page.wh              # Home
├── profile/
│   └── [id]/
│       └── +page.wh      # Profile with ID param
```

### Generated API:
```whitehall
// Automatically available in any component
Routes.home()                  // Navigate to /
Routes.profile(id: "123")      // Navigate to /profile/123
Routes.settings.account()      // Navigate to /settings/account

// Usage:
Button("View Profile", onClick: () => {
  navigate(Routes.profile(id: user.id))
})
```

### Type safety from file structure:
- `[id]` in filename = required String parameter
- `[id?]` = optional parameter
- `[...slug]` = catch-all parameter

**Pros:**
- File-based DX
- Type-safe navigation
- Best of both worlds
- Generated code is predictable

**Cons:**
- Most complex transpilation
- Magic generation might be confusing
- What about query params?

---

## Deep Linking

### Option A (Automatic from routes):
File `src/routes/product/[id]/+page.wh` automatically handles:
- `myapp://product/123`
- `https://myapp.com/product/123`

### Option B (Explicit declaration):
```whitehall
component ProductPage(id: String) {
  deeplink {
    scheme: "myapp"
    host: "product"
    path: "/{id}"
  }

  deeplink {
    scheme: "https"
    host: "myapp.com"
    path: "/product/{id}"
  }

  render { /* ... */ }
}
```

---

## Back Stack Handling

How to control back button behavior?

### Option A (Automatic):
```whitehall
navigate("/profile", replace: true)      // Replace current screen
navigate("/login", clearStack: true)     // Clear back stack
```

### Option B (Declarative):
```whitehall
routes {
  "/login" => LoginPage {
    backstack: "clear"  // Clear stack when navigating here
  }

  "/home" => HomePage {
    backstack: "root"   // Can't go back from here
  }
}
```

---

## Nested Navigation (Tabs)

Common pattern: Bottom navigation with separate stacks per tab.

### Option A (Nested routes):
```
src/routes/
├── (tabs)/            # Route group (doesn't affect URL)
│   ├── +layout.wh     # Tabs layout
│   ├── home/
│   │   └── +page.wh
│   ├── search/
│   │   └── +page.wh
│   └── profile/
│       └── +page.wh
```

**`(tabs)/+layout.wh`:**
```whitehall
component TabsLayout() {
  render {
    Scaffold(
      bottomBar: {
        BottomNavigation {
          Tab(icon: "home", route: "/home")
          Tab(icon: "search", route: "/search")
          Tab(icon: "profile", route: "/profile")
        }
      }
    ) {
      Outlet()  // Child route renders here
    }
  }
}
```

---

## Real-World Example

**App with:**
- Welcome screen (first launch)
- Login/Signup flow
- Main app with tabs (Home, Search, Profile)
- Settings with sub-pages
- Product detail screens

### Option A (File-based):
```
src/routes/
├── +page.wh                    # Welcome/splash
├── (auth)/
│   ├── login/+page.wh
│   └── signup/+page.wh
├── (app)/
│   ├── +layout.wh              # Bottom tabs
│   ├── home/
│   │   ├── +page.wh
│   │   └── product/[id]/+page.wh
│   ├── search/+page.wh
│   └── profile/
│       ├── +page.wh
│       └── settings/
│           ├── +page.wh
│           ├── account/+page.wh
│           └── privacy/+page.wh
```

### Option D (Hybrid):
Same structure, but navigation looks like:
```whitehall
// Type-safe, generated from files
navigate(Routes.auth.login())
navigate(Routes.app.home.product(id: "123"))
navigate(Routes.app.profile.settings.account())
```

---

## Recommendation

**Start with Option D (Hybrid: File-based + Generated Type-Safe API):**

1. **File structure defines routes** (amazing DX)
2. **Generate type-safe navigation helpers** (safety + autocomplete)
3. **Map to Compose Navigation** (proven system)
4. **Support deep linking automatically** (free feature)

**Implementation phases:**
1. Phase 1: Basic file-based routing (string-based navigation)
2. Phase 2: Generate type-safe APIs
3. Phase 3: Layouts and nested routing
4. Phase 4: Deep linking configuration

**Why this approach:**
- File-based routing is incredibly ergonomic (proven by web frameworks)
- Type safety prevents runtime errors
- Can show type errors at compile time
- Still maps cleanly to Compose Navigation
- Familiar to developers from other ecosystems

---

## Open Questions

1. **Route grouping:** Should `(auth)` folders affect URLs or just organization?
   - Next.js: Just organization
   - Recommendation: Just organization (more flexible)

2. **Query parameters:**
   ```whitehall
   navigate("/search?q=android&sort=date")
   // vs
   navigate(Routes.search(query: "android", sort: "date"))
   ```

3. **Programmatic guards:** (like route middleware)
   ```whitehall
   routes {
     "/admin/*" => {
       guard: requireAuth,
       guard: requireAdmin
     }
   }
   ```

4. **Navigation transitions:**
   ```whitehall
   navigate("/profile", transition: "slide")
   navigate("/login", transition: "fade")
   ```

5. **Modal/Dialog routes:** Should modals be routes?
   ```
   src/routes/
   ├── product/[id]/
   │   ├── +page.wh
   │   └── @modal/        # Modal over product page?
   │       └── share/+page.wh
   ```
