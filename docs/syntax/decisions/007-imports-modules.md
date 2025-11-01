# Decision 007: Imports & Modules

**Status:** ✅ Decided
**Date:** 2025-11-01
**Decider:** User preference

## Context

Components need to import and use:
1. **Other .wh components** (custom components)
2. **Kotlin/Java libraries** (Android SDK, third-party)
3. **Data classes/types** (models, utilities)

Key questions:
- How to import .wh components?
- Do we use Kotlin import syntax?
- How to distinguish .wh imports from Kotlin imports?
- Relative vs absolute imports?

---

## Part 1: Importing Custom .wh Components

### Option A: Kotlin-style Import (File Path)

```whitehall
// Import from relative path
import ../components/Button
import ./UserCard
import ../../shared/Avatar

// Import from absolute path
import @/components/Button
import @/shared/Avatar

<script>
  @prop val user: User
</script>

<Column>
  <Avatar url={user.avatarUrl} size={48} />
  <UserCard user={user} />
  <Button text="Edit" onClick={handleEdit} />
</Column>
```

**Pros:**
- Familiar syntax
- Clear file references
- Explicit imports

**Cons:**
- Path-based (can break on refactor)
- Verbose

---

### Option B: Auto-Import by Component Name

```whitehall
// No explicit import needed!
// Compiler scans src/components/ and src/shared/ automatically

<script>
  @prop val user: User
</script>

<Column>
  <Avatar url={user.avatarUrl} size={48} />
  <UserCard user={user} />
  <Button text="Edit" onClick={handleEdit} />
</Column>
```

**Pros:**
- Zero boilerplate
- Just use the component
- Magic but convenient

**Cons:**
- Magic - where does Avatar come from?
- Name collisions possible
- IDE can't easily resolve

---

### Option C: Kotlin-style Package Import (Recommended)

```whitehall
// Use Kotlin package syntax
import com.example.components.Button
import com.example.components.UserCard
import com.example.shared.Avatar

<script>
  @prop val user: User
</script>

<Column>
  <Avatar url={user.avatarUrl} size={48} />
  <UserCard user={user} />
  <Button text="Edit" onClick={handleEdit} />
</Column>
```

**How it works:**
- Each .wh file declares its package (inferred from directory or explicit)
- Import using Kotlin package names
- Maps to generated Kotlin @Composable functions

**Pros:**
- Standard Kotlin syntax
- IDE support
- No path dependencies
- Familiar to Android devs

**Cons:**
- Need to know package structure

---

### Option D: Hybrid (Kotlin + Relative Shorthand)

```whitehall
// Full Kotlin path
import com.example.components.Button

// Or relative shorthand
import ~/components/Avatar    // ~ = project root
import ./UserCard              // ./ = relative to current file
import ../shared/Card          // ../ = parent directory

<Column>
  <Avatar />
  <Button />
  <UserCard />
  <Card />
</Column>
```

**Pros:**
- Flexibility
- Can use either style

**Cons:**
- Two ways to do same thing
- Confusing which to use

---

## Part 2: Package Declaration

### Option A: Inferred from Directory

```
src/
├── components/
│   └── Button.wh          # Package: com.example.components
├── screens/
│   └── Home.wh            # Package: com.example.screens
└── shared/
    └── Avatar.wh          # Package: com.example.shared
```

**In whitehall.toml:**
```toml
[project]
name = "my-app"
package = "com.example"  # Base package
```

**Pros:**
- No manual package declarations
- Convention over configuration

**Cons:**
- Less flexible

---

### Option B: Explicit Declaration (Like Java/Kotlin)

**Button.wh:**
```whitehall
package com.example.components

<script>
  @prop val text: String
  @prop val onClick: () -> Unit
</script>

<TouchableRipple onClick={onClick}>
  <Text>{text}</Text>
</TouchableRipple>
```

**Pros:**
- Explicit
- Can organize differently from file structure

**Cons:**
- Boilerplate
- Another thing to maintain

---

## Part 3: Using Imported Components

### In Markup (All Options)

```whitehall
import com.example.components.Button
import com.example.components.UserCard

<Column>
  <!-- Use as tags (component name must match) -->
  <Button text="Click" onClick={handleClick} />
  <UserCard user={currentUser} />
</Column>
```

### Component Name Resolution

**What if there's a name collision?**

```whitehall
import com.example.ui.Button
import com.example.material.Button  // ❌ Conflict!

// Option A: Alias
import com.example.ui.Button as UIButton
import com.example.material.Button as MaterialButton

<Column>
  <UIButton />
  <MaterialButton />
</Column>
```

---

## Part 4: Importing Kotlin/Java Libraries

### Standard Kotlin Imports

```whitehall
import android.util.Log
import kotlinx.coroutines.launch
import com.example.data.UserRepository
import com.example.models.User

<script>
  @prop val userId: String

  var user: User? = null

  onMount {
    launch {
      user = UserRepository.getUser(userId)
      Log.d("ProfileScreen", "Loaded user: $userId")
    }
  }
</script>

<Column>
  <Text>{user?.name}</Text>
</Column>
```

**This works exactly like Kotlin** - no difference for .kt imports.

---

## Part 5: Project Structure Patterns

### Pattern A: Flat Components

```
src/
├── components/
│   ├── Button.wh
│   ├── Card.wh
│   ├── Avatar.wh
│   └── Input.wh
├── screens/
│   ├── Home.wh
│   └── Profile.wh
└── routes/
    └── +screen.wh
```

**Imports:**
```whitehall
import com.example.components.Button
import com.example.components.Card
```

---

### Pattern B: Feature-Based

```
src/
├── features/
│   ├── auth/
│   │   ├── LoginScreen.wh
│   │   ├── SignupScreen.wh
│   │   └── components/
│   │       └── AuthButton.wh
│   └── profile/
│       ├── ProfileScreen.wh
│       └── components/
│           └── ProfileCard.wh
└── shared/
    └── components/
        ├── Button.wh
        └── Card.wh
```

**Imports:**
```whitehall
import com.example.shared.components.Button
import com.example.features.profile.components.ProfileCard
```

---

### Pattern C: Atomic Design

```
src/
├── components/
│   ├── atoms/
│   │   ├── Button.wh
│   │   ├── Text.wh
│   │   └── Icon.wh
│   ├── molecules/
│   │   ├── Card.wh
│   │   └── SearchBar.wh
│   ├── organisms/
│   │   ├── Header.wh
│   │   └── UserProfile.wh
│   └── templates/
│       └── MainLayout.wh
└── screens/
    └── Home.wh
```

**Imports:**
```whitehall
import com.example.components.atoms.Button
import com.example.components.molecules.Card
```

---

## Part 6: Re-exports (Barrel Files)

### Creating a Component Library

**components/index.wh:**
```whitehall
// Re-export pattern (like TypeScript)
export { Button } from "./atoms/Button"
export { Card } from "./molecules/Card"
export { Header } from "./organisms/Header"
```

**Usage:**
```whitehall
// Import multiple from one place
import { Button, Card, Header } from com.example.components
```

**Question:** Do we support this? Or just use Kotlin imports?

---

## Part 7: Third-Party Component Libraries

### Installing a Whitehall Component Library

```bash
whitehall install @whitehall/material
whitehall install @whitehall/charts
```

**In code:**
```whitehall
import com.whitehall.material.Button
import com.whitehall.material.Card
import com.whitehall.charts.LineChart

<Column>
  <Button text="Click" />
  <Card>
    <LineChart data={chartData} />
  </Card>
</Column>
```

**Implementation:**
- Component libraries are distributed as Whitehall packages
- Installed via package manager (future)
- Compiled to Kotlin like local components

---

## Part 8: Wildcard Imports

```whitehall
// Import everything from a package
import com.example.components.*

<Column>
  <Button />
  <Card />
  <Avatar />
</Column>
```

**Pros:**
- Less verbose for component-heavy files

**Cons:**
- Unclear where components come from
- Name collision risk

---

## FINAL DECISION

**Use Kotlin-style imports with configurable `$` aliases (Option 1 + Option 5 Hybrid)**

### Core Principles

1. **Standard Kotlin import syntax** - No new syntax
2. **`$` prefix aliases** - Configurable shortcuts (Svelte-inspired)
3. **Package inferred from directory** - Convention over configuration
4. **Multiple aliases supported** - `$app`, `$shared`, `$lib`, etc.
5. **Full paths always work** - Can use complete package names

### Syntax

```whitehall
// Using $ aliases (configured in whitehall.toml)
import $app.components.Button
import $shared.Avatar
import $lib.utils.Helper

// Using full package path (always works)
import com.example.myapp.components.Button
import com.example.myapp.shared.Avatar

// Using Kotlin imports
import androidx.compose.material3.Card
import android.util.Log
import com.thirdparty.Library

// Wildcards
import $app.components.*

// Aliases for name conflicts
import $app.ui.Button as UIButton
import com.material.Button as MaterialButton
```

### Configuration (whitehall.toml)

```toml
[project]
name = "my-app"
package = "com.example.myapp"

[imports.aliases]
# Define $ prefix aliases
app = "com.example.myapp"              # $app.components.Button
shared = "com.example.myapp.shared"    # $shared.Avatar
lib = "com.example.myapp.lib"          # $lib.utils.Helper
models = "com.example.myapp.models"    # $models.User
# Add as many as you want!
```

**Defaults (if not configured):**
- `$app` always maps to base package
- Can rename `$app` to `$core`, `$src`, whatever you prefer

### Package Inference

**Automatic from directory structure:**
```
src/components/Button.wh     → {package}.components.Button
src/shared/Avatar.wh         → {package}.shared.Avatar
src/lib/utils/Helper.wh      → {package}.lib.utils.Helper
src/screens/Home.wh          → {package}.screens.Home
```

Where `{package}` = base package from whitehall.toml

### Benefits

✅ **Svelte-inspired** - Familiar `$lib` pattern
✅ **Customizable** - Define your own aliases
✅ **Short** - `$app` instead of `com.example.myapp`
✅ **Standard Kotlin** - Full paths always work
✅ **IDE support** - Transpiles to standard Kotlin imports
✅ **Flexible** - Use shortcuts or full paths

---

## Examples

### Example 1: Simple Component Import

**File:** `src/screens/Profile.wh`

```whitehall
import com.example.myapp.components.Avatar
import com.example.myapp.components.Button
import com.example.myapp.models.User
import com.example.myapp.data.UserRepository

<script>
  @prop val userId: String

  var user: User? = null
  var isLoading = true

  onMount {
    user = UserRepository.getUser(userId).getOrNull()
    isLoading = false
  }
</script>

@if (isLoading) {
  <LoadingSpinner />
} else if (user != null) {
  <Column padding={16} spacing={12}>
    <Avatar url={user.avatarUrl} size={80} />
    <Text fontSize={24}>{user.name}</Text>
    <Text color="secondary">{user.email}</Text>
    <Button text="Edit Profile" onClick={handleEdit} />
  </Column>
}
```

### Example 2: Wildcard Import

**File:** `src/screens/Dashboard.wh`

```whitehall
import com.example.myapp.components.*
import com.example.myapp.models.DashboardData

<script>
  var data: DashboardData? = null
</script>

<Column>
  <Header title="Dashboard" />
  <Card>
    <StatWidget value={data?.userCount} label="Users" />
  </Card>
  <Chart data={data?.chartData} />
  <Button text="Refresh" onClick={refresh} />
</Column>
```

### Example 3: Aliased Import

**File:** `src/screens/Home.wh`

```whitehall
import com.example.myapp.components.Button as PrimaryButton
import com.thirdparty.material.Button as MaterialButton

<Column>
  <PrimaryButton text="Click Me" />
  <MaterialButton text="Material Design" />
</Column>
```

---

## Transpilation

**Input (Button.wh):**
```whitehall
package com.example.myapp.components

<script>
  @prop val text: String
  @prop val onClick: () -> Unit
</script>

<TouchableRipple onClick={onClick}>
  <Text>{text}</Text>
</TouchableRipple>
```

**Output (Button.kt):**
```kotlin
package com.example.myapp.components

import androidx.compose.runtime.*
import androidx.compose.material3.*

@Composable
fun Button(
  text: String,
  onClick: () -> Unit
) {
  Surface(onClick = onClick) {
    Text(text)
  }
}
```

**Usage (Profile.wh):**
```whitehall
import com.example.myapp.components.Button

<Button text="Click" onClick={handleClick} />
```

**Transpiles to:**
```kotlin
import com.example.myapp.components.Button

Button(text = "Click", onClick = { handleClick() })
```

---

## Implementation Notes

### Compiler Tasks:

1. **Parse imports** - Extract import statements from .wh files
2. **Resolve packages** - Map directory → package name
3. **Build dependency graph** - Determine compilation order
4. **Generate Kotlin imports** - Convert .wh imports to .kt imports
5. **Component registry** - Track all available components
6. **Validate imports** - Error on missing/cyclic dependencies

### Error Messages:

```
error: Cannot resolve component
  ┌─ src/screens/Profile.wh:1:8
  │
1 │ import com.example.components.UserCard
  │        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ component not found
  │
  = help: did you mean `com.example.myapp.components.UserCard`?
  = note: available components in com.example.myapp.components:
          - Avatar
          - Button
          - Card
```

---

## Implementation Notes

### Transpilation

**Input (.wh file):**
```whitehall
import $app.components.Button
import $shared.Avatar
import androidx.compose.material3.Card
```

**Output (.kt file):**
```kotlin
import com.example.myapp.components.Button
import com.example.myapp.shared.Avatar
import androidx.compose.material3.Card
```

The compiler:
1. Reads `whitehall.toml` for alias mappings
2. Expands `$` aliases to full package paths
3. Generates standard Kotlin imports

### Error Messages

```
error: Cannot resolve import
  ┌─ src/screens/Profile.wh:1:8
  │
1 │ import $app.components.UserCard
  │        ^^^^^^^^^^^^^^^^^^^^^^^^ component not found
  │
  = help: did you mean one of these?
          - $app.components.Avatar
          - $app.components.Button
          - $app.components.Card
  = note: searching in package: com.example.myapp.components
```

---

## Open Questions (Pending Decisions)

### 1. Auto-import Compose Primitives?

**Question:** Should `Column`, `Row`, `Text`, `Button` (Compose widgets) be auto-imported?

```whitehall
// Option A: Auto-imported (no import needed)
<Column>
  <Text>Hello</Text>
</Column>

// Option B: Must import explicitly
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text

<Column>
  <Text>Hello</Text>
</Column>
```

**Status:** ⏸️ Pending decision

---

### 2. Component Name Conventions

**Questions:**
- PascalCase enforced? `Button.wh` (recommended)
- Must filename match component name?
- File `Button.wh` exports `Button` component (one-to-one)

**Status:** ⏸️ Pending decision

---

### 3. Multiple Components Per File?

```whitehall
// Option A: One component per file (recommended)
// Button.wh contains only Button component

// Option B: Multiple components allowed
// Buttons.wh
component Button(...) { }
component IconButton(...) { }
component TextButton(...) { }
```

**Status:** ⏸️ Pending decision (recommend: one per file)

---

### 4. Private Components?

```whitehall
// Internal helper component (not exported)
private component InternalHelper { }

// Public component (exported)
component PublicButton {
  <InternalHelper />
}
```

**Status:** ⏸️ Pending decision
