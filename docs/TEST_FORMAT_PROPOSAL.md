# Test Markdown Format Proposal

## Current Problems

1. **Component Name is Implicit**:
   - Test `01-basic-component.md` expects output `fun Avatar()` but filename would derive `BasicComponent`
   - Test `07-routing-simple.md` expects output `fun WelcomeScreen()` but filename would derive `RoutingSimple`

2. **Package Name Varies**:
   - Most tests use `com.example.app.components`
   - Routing tests use `com.example.app.screens`
   - No way to specify this per-test

3. **File Type Unknown**:
   - Is this a `.wh` component file?
   - Is this a `+screen.wh` screen file?
   - Affects how transpiler should process it

4. **Test Harness Logic**:
   - Currently derives component name from markdown filename
   - Hardcodes package as `com.example.app.components`
   - No way to override or customize

## Proposed Solution: Metadata Section

Add a `## Metadata` section at the top of each test file with explicit configuration.

### Format

```markdown
# Test Name

Description of what this test validates.

## Metadata

```yaml
file: Avatar.wh
package: com.example.app.components
```

## Input

```whitehall
// Whitehall code
```

## Output

```kotlin
// Expected Kotlin code
```
```

### Metadata Fields

| Field | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| `file` | string | Yes | Source filename (determines component name) | `Avatar.wh`, `+screen.wh`, `WelcomeScreen.wh` |
| `package` | string | Yes | Kotlin package for generated code | `com.example.app.components` |
| `type` | enum | No | File type hint (`component`, `screen`, `layout`) | `screen` |

### Component Name Derivation

Component name is derived from `file` field:
- `Avatar.wh` → `fun Avatar()`
- `MinimalText.wh` → `fun MinimalText()`
- `+screen.wh` → Special screen file (derive from parent dir or test title)
- `WelcomeScreen.wh` → `fun WelcomeScreen()`

### Benefits

1. **Explicit > Implicit**: No guessing, all metadata clearly stated
2. **Flexible**: Each test can specify different packages/names
3. **Maintainable**: Easy to update metadata without changing code
4. **Self-Documenting**: Reader immediately sees what file this represents
5. **Testable**: Test harness can validate metadata correctness

## Example Test Files

### Example 1: Simple Component

**File**: `tests/transpiler-examples/00-minimal-text.md`

```markdown
# Minimal Component - Just Text

The simplest possible component: just a single text element.

## Metadata

```yaml
file: MinimalText.wh
package: com.example.app.components
```

## Input

```whitehall
<Text>Hello, World!</Text>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable

@Composable
fun MinimalText() {
    Text(text = "Hello, World!")
}
```
```

### Example 2: Component with Props

**File**: `tests/transpiler-examples/01-basic-component.md`

```markdown
# Basic Component with Props

Tests a component with required and optional props.

## Metadata

```yaml
file: Avatar.wh
package: com.example.app.components
```

## Input

```whitehall
import $models.User

  @prop val url: String
  @prop val size: Int = 48
  @prop val onClick: (() -> Unit)? = null

<AsyncImage
  url={url}
  width={size}
  height={size}
  modifier={onClick?.let { Modifier.clickable { it() } } ?: Modifier}
/>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.foundation.clickable
import com.example.app.models.User
import coil.compose.AsyncImage

@Composable
fun Avatar(
    url: String,
    size: Int = 48,
    onClick: (() -> Unit)? = null
) {
    AsyncImage(
        url = url,
        width = size,
        height = size,
        modifier = onClick?.let { Modifier.clickable { it() } } ?: Modifier
    )
}
```
```

### Example 3: Screen/Route

**File**: `tests/transpiler-examples/07-routing-simple.md`

```markdown
# Routing: Simple Navigation

Tests basic $routes navigation without parameters.

## Metadata

```yaml
file: WelcomeScreen.wh
package: com.example.app.screens
type: screen
```

## Input

```whitehall
  fun handleLoginClick() {
    navigate($routes.login)
  }

  fun handleSignupClick() {
    navigate($routes.signup)
  }

<Column spacing={16}>
  <Text fontSize={24}>Welcome!</Text>

  <Button
    text="Login"
    onClick={handleLoginClick}
  />

  <Button
    text="Sign Up"
    onClick={handleSignupClick}
    variant="outlined"
  />
</Column>
```

## Output

```kotlin
package com.example.app.screens

import androidx.compose.runtime.Composable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController

@Composable
fun WelcomeScreen(navController: NavController) {
    fun handleLoginClick() {
        navController.navigate(Routes.Login)
    }

    fun handleSignupClick() {
        navController.navigate(Routes.Signup)
    }

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = "Welcome!",
            fontSize = 24.sp
        )

        Button(
            onClick = { handleLoginClick() }
        ) {
            Text("Login")
        }

        Button(
            onClick = { handleSignupClick() },
            variant = "outlined"
        ) {
            Text("Sign Up")
        }
    }
}
```
```

## Test Harness Changes

Update `tests/transpiler_examples_test.rs` to:

1. Parse `## Metadata` section from markdown
2. Extract YAML fields (`file`, `package`, `type`)
3. Derive component name from `file` field
4. Pass metadata to transpiler function

### Parsing Logic

```rust
#[derive(Debug)]
struct TestMetadata {
    file: String,
    package: String,
    type_hint: Option<String>,
}

#[derive(Debug)]
struct TranspilerTest {
    name: String,
    metadata: TestMetadata,
    input: String,
    expected_output: String,
}

fn parse_test_file(content: &str, filename: &str) -> Result<TranspilerTest, String> {
    // Parse markdown sections
    // Extract ## Metadata section
    // Parse YAML from metadata code block
    // Extract ## Input and ## Output sections
    // ...
}
```

### Transpiler Call

```rust
// Derive component name from metadata.file
let component_name = metadata.file
    .trim_end_matches(".wh")
    .to_string();

// Use metadata.package for package name
let actual_output = transpile(
    &test.input,
    &metadata.package,
    &component_name
)?;
```

## Migration Plan

### Phase 1: Add Metadata to Existing Tests
Before resetting the transpiler, update all 14 test markdown files with metadata sections.

**Tests to Update**:
- `00-minimal-text.md` → Add: `file: MinimalText.wh, package: com.example.app.components`
- `00a-text-with-interpolation.md` → Add: `file: TextWithInterpolation.wh, package: com.example.app.components`
- `00b-single-prop.md` → Add: `file: SingleProp.wh, package: com.example.app.components`
- `01-basic-component.md` → Add: `file: Avatar.wh, package: com.example.app.components`
- `02-control-flow-if.md` → Add metadata
- `03-control-flow-for.md` → Add metadata
- `04-control-flow-when.md` → Add metadata
- `05-data-binding.md` → Add metadata
- `06-lifecycle-hooks.md` → Add metadata
- `07-routing-simple.md` → Add: `file: WelcomeScreen.wh, package: com.example.app.screens, type: screen`
- `08-routing-params.md` → Add: `file: ProfileScreen.wh, package: com.example.app.screens, type: screen`
- `09-imports.md` → Add metadata
- `10-nested-components.md` → Add metadata
- `11-complex-state-management.md` → Add metadata

### Phase 2: Update Test Harness
Update `tests/transpiler_examples_test.rs` to:
1. Parse metadata section (add YAML parsing dependency if needed, or use simple key:value parser)
2. Extract file and package fields
3. Derive component name from file field
4. Pass to transpiler

### Phase 3: Validate
Run `cargo test test_parse_markdown_files` to ensure all tests parse correctly with new metadata.

### Phase 4: Reset Transpiler
Once metadata is in place, reset to `e1ecf0a` and rebuild transpiler using proper metadata.

## Alternative: Simple Key-Value Format

If we want to avoid YAML dependency, use simple key-value format:

```markdown
## Metadata

```
file: Avatar.wh
package: com.example.app.components
type: component
```
```

Parse with:
```rust
for line in metadata_block.lines() {
    let parts: Vec<&str> = line.split(':').map(|s| s.trim()).collect();
    match parts[0] {
        "file" => metadata.file = parts[1].to_string(),
        "package" => metadata.package = parts[1].to_string(),
        "type" => metadata.type_hint = Some(parts[1].to_string()),
        _ => {}
    }
}
```

## Decision Needed

1. **Format**: YAML vs simple key-value?
2. **Required fields**: Just `file` and `package`, or more?
3. **Migration**: Update all tests before reset, or incrementally during rebuild?

## Recommendation

- Use **simple key-value format** (no new dependencies)
- Make `file` and `package` **required**
- Make `type` **optional** (for future use)
- **Update all tests before reset** (ensures clean foundation)
- Commit test updates separately from transpiler reset

This ensures we have a solid test foundation before beginning the transpiler rebuild.
