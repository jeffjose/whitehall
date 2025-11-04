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

## Phase 2.5: Single-File Mode (Planned - v0.25)
**Goal: Enable zero-config single-file apps** (Like `uv` for Python, `rust-script` for Rust)

- [ ] Parse frontmatter configuration (`///` TOML comments)
- [ ] Extract inline dependencies from frontmatter
- [ ] `whitehall run <file.wh>` - Single-file execution
- [ ] `whitehall build <file.wh>` - Build APK from single file
- [ ] Temporary project generation in `.whitehall/cache/{hash}/`
- [ ] Build caching for single-file apps (content-based hashing)
- [ ] Shebang support (`#!/usr/bin/env whitehall`)

**Success metric:** Can write a complete app in one `.wh` file and run it instantly

**Status**: Design complete (see `docs/SINGLE-FILE-MODE.md`). Implementation pending after end-to-end testing.

**Priority**: Medium (enables rapid prototyping and learning)

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
- Plugin system
- Hot reload
- Component marketplace
- Visual tooling
- Multi-platform support (iOS?)
- Desktop app generation (Compose Desktop)
