# Roadmap

## Phase 0: Foundation (✓ Completed)
**Goal: Prove the concept**

- [x] Basic CLI structure with `clap`
- [x] `whitehall init` - Project scaffolding
  - [x] Create project directory structure
  - [x] Generate `whitehall.toml` manifest
  - [x] Create sample `.wh` file
  - [x] Generate `.gitignore`
- [x] Define initial `whitehall.toml` schema
- [x] Documentation structure (VISION, ROADMAP, ARCHITECTURE)

**Success metric:** ✓ Can run `whitehall init my-app` and get a valid project structure

**Future enhancements (Phase 0.5 - Project Scaffolding):**

### `whitehall create` - Interactive Project Creation
Inspired by `npm create svelte` and `cargo init` workflows:

```bash
whitehall create my-app
# Interactive prompts:
# → What type of app? (Basic / With routing / Full-featured)
# → Package name? (com.example.myapp)
# → Initialize git? (Yes / No)
# → Template? (Counter / Todo / Blog / Blank)
```

**Features:**
- [ ] Interactive CLI with `inquire` or `dialoguer` crate
- [ ] Project type selection:
  - **Basic**: Single screen, no routing
  - **Routing**: Multi-screen with file-based routing
  - **Full-featured**: Routing + API client + state management
- [ ] Template selection from built-in and community templates
- [ ] Package name validation (Android rules)
- [ ] Git initialization option
- [ ] Prettier output with progress indicators

### Template Repository System
Separate `whitehall-templates` repository (like `cargo-generate`):

**Built-in templates:**
```
whitehall-templates/
├── counter/           # Minimal counter app
├── todo/              # Todo list with state
├── blog/              # Multi-screen blog reader
├── ecommerce/         # Full e-commerce example
└── social-media/      # Social app with routing + API
```

**Usage:**
```bash
# Use built-in template
whitehall init my-app --template counter

# Use GitHub template (without git history)
whitehall init my-app --template github:user/repo

# Use local template
whitehall init my-app --template ./path/to/template
```

**Implementation:**
- [ ] Template cloning (without `.git` directory)
- [ ] Variable substitution in templates (`{{package_name}}`, `{{app_name}}`)
- [ ] Community template registry (JSON manifest)
- [ ] Template validation and caching
- [ ] `whitehall template list` - Show available templates
- [ ] `whitehall template add <url>` - Add community template

**Benefits:**
- Templates separate from core Whitehall code
- Easy community contributions
- Real-world example projects
- Faster project setup with best practices

---

## Phase 1: Validation (✓ Completed - v0.1)
**Goal: Parse and validate .whitehall files**

- [x] `.whitehall` file format specification (syntax design)
- [x] Lexer-free recursive descent parser for `.wh` files
- [x] Parser for `.wh` files (handles all syntax features)
- [x] Meaningful error messages from transpiler
- [ ] `whitehall check` - Syntax validation CLI command (planned)
- [ ] Basic LSP support (syntax highlighting) (future)

**Success metric:** ✓ Can write `.wh` files and transpiler validates them (30/30 tests passing)

**Status**: Transpiler core is **100% complete** - all 30 test cases passing (28 transpiler + 2 optimization examples). See `docs/TRANSPILER.md` for details.

---

## Phase 2: Routing (✓ Completed - v0.2)
**Goal: File-based routing with type-safe navigation**

- [x] Scan `src/routes/` directory structure
- [x] Parse `+screen.wh` files and `[param]` syntax
- [x] Generate @Serializable route objects (Navigation 2.8+ compatible)
- [x] Generate sealed interface Routes hierarchy
- [x] Generate NavHost with `composable<T>` calls
- [x] Type-safe navigation API in components (`$routes.*` references)
- [x] Support dynamic parameters (`$screen.params.id`)
- [ ] Support nested routes (future enhancement)

**Success metric:** ✓ Can create routes by adding files, navigate with type safety

**Status**: **100% complete** - File-based routing fully implemented:
- File discovery recognizes `src/routes/**/+screen.wh`
- Derives screen names from paths (login/+screen.wh → LoginScreen)
- Extracts route params from [id] folders
- Generates Routes.kt sealed interface
- Generates MainActivity with complete NavHost setup
- All 7 microblog screens transpile correctly

---

## Phase 3: Compilation (✓ Completed - v0.3)
**Goal: Generate working Android code**

- [x] AST → Kotlin transpiler (100% complete, 30 tests passing)
- [x] Generate Activity code (MainActivity with NavHost)
- [x] Handle UI components (all Compose components supported)
- [x] `whitehall build` - CLI command to transpile project
- [x] Generate Gradle build files (complete scaffold generation)
- [x] Invoke Gradle to create APK (via `whitehall run`)

**Success metric:** ✓ `whitehall build` produces a working Android project

**Status**: **100% complete** - Full compilation pipeline working:
- Transpiles all .wh files to idiomatic Kotlin
- Generates complete Gradle project structure
- Creates MainActivity with routing setup
- All CLI commands implemented (build, watch, run)

---

## Phase 4: Development Loop (✓ Completed - v0.4)
**Goal: Fast iteration**

- [x] `whitehall build` - Transpile project to Kotlin + generate Gradle scaffold
- [x] `whitehall watch` - File watching and auto-rebuild
- [x] `whitehall run` - Build + deploy to emulator/device
- [ ] Better error reporting with source maps
- [ ] `whitehall clean`

**Success metric:** Can develop a simple app using only Whitehall CLI

**Status**: ✓ All core CLI commands implemented and working:
- ✅ `whitehall init` - Creates project structure
- ✅ `whitehall build` - Transpiles .wh → .kt + generates Android scaffold
- ✅ `whitehall watch` - File watching with auto-rebuild (notify crate)
- ✅ `whitehall run` - Builds, runs Gradle, installs APK, launches app

**Next**: End-to-end testing with real apps to verify the complete workflow

---

## Phase 2.5: Single-File Mode ✅ COMPLETE (v0.25)
**Goal: Enable zero-config single-file apps** (Like `uv` for Python, `rust-script` for Rust)

- [x] Parse frontmatter configuration (`///` TOML comments)
- [x] Auto-generate package names from app name
- [x] `whitehall compile <file.wh>` - Transpile to Kotlin with `--package` and `--no-package` flags
- [x] `whitehall build <file.wh>` - Build APK from single file
- [x] `whitehall run <file.wh>` - Single-file execution
- [x] `whitehall watch <file.wh>` - Watch single file for changes
- [x] Temporary project generation in `~/.cache/whitehall/{hash}/`
- [x] Build caching for single-file apps (content-based SHA256 hashing)
- [x] 8 test cases for single-file mode
- [ ] Shebang support (`#!/usr/bin/env whitehall`) - Future
- [ ] Extract inline dependencies from frontmatter - Future

**Success metric:** ✅ Can write a complete app in one `.wh` file and run it instantly

**Status**: **COMPLETE** - Implemented Nov 4, 2025. See `docs/SINGLE-FILE-MODE.md` for details.

**Priority**: ✅ Done! Enables rapid prototyping and learning.

---

## Phase 5: Dependencies (v0.5)
**Goal: Reusable components**

- [ ] `whitehall install <dependency>` - Add libraries
- [ ] Support for Maven/Android dependencies
- [ ] Dependency resolution
- [ ] Lock file management

---

## Phase 6: Release & Polish (v0.6)
**Goal: Production-ready**

- [ ] `whitehall build --release` - Optimized builds
- [ ] ProGuard/R8 integration
- [ ] Code signing configuration
- [ ] `whitehall test` - Testing framework
- [ ] CI/CD examples
- [ ] `whitehall clean` - Clean build artifacts

---

## Phase 7: Distribution & Publishing (v0.7)
**Goal: Seamless app distribution**

### `whitehall publish` - Publish to Play Store
One-command publishing to Google Play Store:

```bash
# First time setup
whitehall publish --setup
# → Configure Play Console API credentials
# → Set up signing keys
# → Configure tracks (internal/alpha/beta/production)

# Publish to internal testing
whitehall publish --track internal

# Publish to production with rollout
whitehall publish --track production --rollout 10%

# Full release with release notes
whitehall publish --track production --notes "Bug fixes and improvements"
```

**Features:**
- [ ] App bundle (.aab) generation for optimal size
- [ ] Play Console API integration (Google Play Developer API)
- [ ] Automated screenshot upload from `screenshots/` directory
- [ ] Release notes from `CHANGELOG.md` or git commits
- [ ] Version bump automation (`whitehall.toml` version field)
- [ ] Multi-track support:
  - **Internal**: Quick testing with internal testers
  - **Alpha**: Early access testing
  - **Beta**: Public beta testing
  - **Production**: Full release
- [ ] Rollout percentage control (gradual rollouts)
- [ ] Previous version rollback support
- [ ] Status checks before publishing (tests, signing, etc.)

**Configuration (`whitehall.toml`):**
```toml
[publish]
play_console_credentials = ".secrets/play-console.json"
track = "internal"  # Default track
rollout_percentage = 100

[publish.release_notes]
from = "CHANGELOG.md"  # Or "git" for commit messages
language = "en-US"
```

### `whitehall push` - Cloud Build Integration
Push code to remote build systems that build and distribute:

```bash
# Setup CI/CD integration
whitehall push --setup
# → Choose platform (GitHub Actions / GitLab CI / Bitrise / Custom)
# → Configure secrets (signing keys, API tokens)
# → Generate workflow files

# Push and trigger build
whitehall push

# Push to specific branch/environment
whitehall push --branch staging
whitehall push --env production

# Watch build status
whitehall push --watch
```

**Supported platforms:**
- [ ] **GitHub Actions**: `.github/workflows/whitehall.yml` generation
- [ ] **GitLab CI**: `.gitlab-ci.yml` generation
- [ ] **Bitrise**: `bitrise.yml` generation
- [ ] **Custom**: Webhook to custom build server

**Features:**
- [ ] Automatic workflow file generation
- [ ] Secure secrets management
- [ ] Build status notifications (Slack, Discord, Email)
- [ ] Artifact storage (APK/AAB upload to cloud storage)
- [ ] Automatic distribution after successful build
- [ ] Build caching for faster CI/CD
- [ ] Multi-environment support (dev, staging, prod)
- [ ] Parallel builds for multiple architectures
- [ ] Build logs streaming

**Example GitHub Actions workflow (auto-generated):**
```yaml
name: Whitehall Build
on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: whitehall-lang/setup@v1
      - run: whitehall build --release
      - run: whitehall publish --track internal
```

### Additional Distribution
- [ ] **Firebase App Distribution**: Beta testing distribution
  ```bash
  whitehall distribute firebase --groups "qa-team,beta-users"
  ```
- [ ] **App Center**: Microsoft App Center integration
- [ ] **Direct APK distribution**: Self-hosted download links
  ```bash
  whitehall build --release --output my-app.apk
  whitehall distribute s3 --bucket my-apps
  # Generates: https://my-apps.s3.amazonaws.com/my-app.apk
  ```
- [ ] **QR code generation**: Instant download QR codes
  ```bash
  whitehall distribute qr --output install-qr.png
  ```

**Success metric:** Can publish app to Play Store with one command

**Priority**: Low (after core features stable and tested)

**Inspiration**:
- Flutter's `flutter pub publish`
- Fastlane for iOS/Android
- Expo's `eas submit`

---

## Future Phases

### Phase 8: Developer Experience & Tooling (v0.8)
**Goal: Modern developer experience with visual tools and learning resources**

#### Component Playground
Interactive component testing and development environment:

```bash
whitehall playground Button.wh
# Opens interactive viewer with live props editor
# Adjust props in real-time, see component update instantly
```

**Features:**
- [ ] Component isolation viewer (test components without full app)
- [ ] Interactive props panel with type-aware inputs
  - String inputs for text props
  - Number sliders for numeric props
  - Toggle switches for boolean props
  - Color pickers for color props
- [ ] Live hot-reload preview
- [ ] Multiple viewport sizes (phone, tablet, foldable)
- [ ] Light/dark theme toggle
- [ ] Export component states as test fixtures
- [ ] Screenshot generation for documentation
- [ ] Component gallery mode (view all project components)
- [ ] Share playground sessions via URL

**Implementation:**
- Embedded web server with WebView
- Component introspection from AST
- Props UI auto-generated from type signatures
- Like Storybook for web or Flutter's DevTools

**Success metric:** Can test and document components in isolation without running full app

**Priority:** High - Essential for component-driven development

---

#### Interactive Tutorial Mode
Learn-by-doing tutorial system built into CLI:

```bash
whitehall tutorial start
# → Welcome to Whitehall! Let's build your first app.
# → Step 1/10: Create your first component
# → Creates Counter.wh with guided comments

whitehall tutorial check
# → ✓ Correct! You've created a stateful component.
# → Next: Add a button to increment the counter.

whitehall tutorial list
# → Available tutorials:
# →   1. Getting Started (10 steps)
# →   2. Routing & Navigation (8 steps)
# →   3. Forms & Validation (12 steps)
# →   4. API Integration (15 steps)
```

**Features:**
- [ ] Progressive tutorial system (beginner → advanced)
- [ ] Step-by-step guided exercises
- [ ] Code validation and hints
- [ ] Interactive feedback on mistakes
- [ ] Automatic file creation with scaffolding
- [ ] Expected vs actual output comparison
- [ ] Visual progress tracking
- [ ] Achievements/badges for completed tutorials
- [ ] Multiple learning paths:
  - **Basics**: Components, props, state
  - **Intermediate**: Routing, forms, lists
  - **Advanced**: State management, APIs, optimization
  - **Full-stack**: Backend integration, auth, deployment

**Tutorial content examples:**
1. **Hello World** - First component (5 min)
2. **Counter App** - State management basics (10 min)
3. **Todo List** - Lists and data binding (15 min)
4. **Multi-screen App** - Routing and navigation (20 min)
5. **Form Validation** - Input handling and validation (25 min)
6. **API Client** - Fetching and displaying data (30 min)

**Implementation:**
- Tutorial definitions in TOML/YAML
- Code AST analysis for validation
- Diff-based hints (compare student code vs solution)
- Like rustlings (Rust) or exercism

**Success metric:** New users build working app in <30 minutes

**Priority:** High - Critical for onboarding and adoption

---

#### Global State Management
Built-in reactive state management without external libraries:

**Store definition syntax:**
```whitehall
// src/stores/auth.wh
export store AuthStore {
  var user: User? = null
  var isLoading = false
  var error: String? = null

  fun login(email: String, password: String) async {
    isLoading = true
    error = null
    try {
      user = await authApi.login(email, password)
    } catch (e: Exception) {
      error = e.message
    } finally {
      isLoading = false
    }
  }

  fun logout() {
    user = null
  }
}

// Auto-exports singleton: authStore
```

**Usage in components:**
```whitehall
import { authStore } from '@/stores/auth'

<script>
  // Reactive subscription - auto-updates when store changes
  var currentUser = authStore.user
  var loading = authStore.isLoading
</script>

<Column>
  @if (loading) {
    <LoadingSpinner />
  } @else if (currentUser != null) {
    <Text>Welcome, {currentUser.name}!</Text>
    <Button onClick={() => authStore.logout()}>Logout</Button>
  } @else {
    <LoginForm onSubmit={authStore.login} />
  }
</Column>
```

**Features:**
- [ ] `store` keyword for global state declarations
- [ ] Reactive subscriptions (components auto-update on store changes)
- [ ] Automatic singleton management
- [ ] Support for async actions
- [ ] Computed properties in stores
- [ ] Store composition (one store can use another)
- [ ] DevTools integration (inspect state, time-travel debugging)
- [ ] Persistence middleware (save/restore from preferences)
- [ ] TypeScript-style paths: `@/stores/auth` → `src/stores/auth.wh`

**Store types:**
```whitehall
// Simple value store
export store Counter {
  var count = 0
  fun increment() { count++ }
}

// Async data store with loading states
export store Posts {
  var items: List<Post> = []
  var loading = false
  var error: String? = null

  fun fetch() async {
    loading = true
    items = await api.posts.getAll()
    loading = false
  }
}

// Computed properties
export store Cart {
  var items: List<CartItem> = []

  val total: Double = items.sumOf { it.price * it.quantity }
  val itemCount: Int = items.sumOf { it.quantity }

  fun addItem(item: CartItem) { items = items + item }
  fun removeItem(id: String) { items = items.filter { it.id != id } }
}
```

**Transpiles to:**
```kotlin
// Kotlin StateFlow-based implementation
object AuthStore {
    private val _user = MutableStateFlow<User?>(null)
    val user: StateFlow<User?> = _user.asStateFlow()

    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    suspend fun login(email: String, password: String) {
        _isLoading.value = true
        // ...
    }
}

// Component usage
@Composable
fun MyScreen() {
    val currentUser by AuthStore.user.collectAsState()
    val loading by AuthStore.isLoading.collectAsState()
    // ...
}
```

**Implementation:**
- [ ] New AST node type: `StoreDeclaration`
- [ ] Parser support for `export store` syntax
- [ ] Codegen to Kotlin objects with StateFlow
- [ ] Import resolution for store paths
- [ ] Reactive subscription injection in components
- [ ] Store middleware system (logging, persistence)

**Success metric:** Can build multi-screen app with shared state without external libraries

**Priority:** Medium-High - Common need for real apps, but can use existing Kotlin patterns initially

---

### Phase 9: Advanced DevEx Features (v0.9)
- Plugin system (WASM or dynamic libraries)
- Hot reload / HMR (live code updates)
- LSP server (Language Server Protocol for editor support)
  - Syntax highlighting
  - Autocomplete
  - Go to definition
  - Error checking
  - Hover documentation
- Component marketplace (community components)
- Visual tooling (drag-and-drop UI builder)
- Design system tokens (theme management)
- Component snippets/templates generator

### Phase 10: Multi-Platform (v1.0+)
- Multi-platform support (iOS via Compose Multiplatform?)
- Desktop app generation (Compose Desktop)
- Web target (Compose for Web)
- Unified codebase across all platforms
