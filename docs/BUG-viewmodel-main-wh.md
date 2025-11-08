# BUG: ViewModel Not Generated for main.wh with Inline Variables

## Issue

When `main.wh` contains inline `var` declarations (mutable state), the transpiler correctly generates a ViewModel class (`AppViewModel`) but fails to write it to disk. The generated `MainActivity.kt` references `AppViewModel`, but the file doesn't exist, causing Kotlin compilation errors:

```
e: Unresolved reference: AppViewModel
```

## Why This is Complex

The bug involves multiple interconnected systems:

### 1. Dual File Generation System
When a component has inline mutable state, the transpiler should generate TWO files:
- **Primary file**: Wrapper component (e.g., `App.kt`)
- **Secondary file**: ViewModel class (e.g., `AppViewModel.kt`)

The `TranspileResult::Multiple` enum variant handles this, containing a vector of `(suffix, content)` tuples.

### 2. Special Handling for main.wh
Unlike regular components, `main.wh` requires special treatment:
- It's skipped in normal file processing (line 174 of build_pipeline.rs)
- It's transpiled separately in `generate_main_activity()` (line 264)
- The transpiled content is embedded inside the `MainActivity` class
- **The problem**: The special handling only extracted primary content, discarding ViewModel files

### 3. Component Name Mismatch
- Source file: `main.wh` (component_name = "main")
- Transpilation: uses component_name = "App"
- Expected ViewModel filename: `AppViewModel.kt` (not `mainViewModel.kt`)

The mismatch made it harder to identify why files weren't being written correctly.

### 4. onClick Handler Transformation
Button click handlers like `onClick={increment}` need to be transformed to `onClick = { viewModel.increment() }`. This transformation failed because:

1. The value `{increment}` was stripped to `increment`
2. `increment` was transformed (but function names weren't handled)
3. `()` was added to create `increment()`
4. The result `{ increment() }` had bare function calls, not `viewModel.` prefixed

The fix required adding `()` BEFORE transformation so the third pass in `transform_viewmodel_expression` could match the `increment(` pattern and apply the `viewModel.` prefix.

## Root Causes

1. **Missing File Writer**: `generate_main_activity()` called `result.primary_content()` (line 267), which extracts only the first file from `TranspileResult::Multiple`, discarding the ViewModel.

2. **Filename Generation Bug**: When writing secondary files, the code used `main_file.component_name` ("main") instead of the actual component name used during transpilation ("App").

3. **Transformation Ordering**: The onClick handler transformed expressions before adding `()`, preventing the function call pattern from matching in `transform_viewmodel_expression`.

## Resolution

### Fix 1: Write ViewModel Files for main.wh (build_pipeline.rs:267-290)

```rust
// Handle Multiple results (e.g., when main.wh has inline vars → generates ViewModel)
match &result {
    transpiler::TranspileResult::Multiple(files) => {
        // Write secondary files (e.g., AppViewModel.kt)
        // Note: main.wh is transpiled with component_name="App"
        let package_path = config.android.package.replace('.', "/");
        for (suffix, content) in files {
            if !suffix.is_empty() {
                let filename = format!("App{}.kt", suffix);
                let output_path = output_dir
                    .join("app/src/main/kotlin")
                    .join(&package_path)
                    .join(filename);
                fs::create_dir_all(output_path.parent().unwrap())?;
                fs::write(&output_path, content)?;
            }
        }
    }
    transpiler::TranspileResult::Single(_) => {
        // No extra files to write
    }
}
```

**Key points:**
- Check if result is `Multiple` variant
- Iterate through secondary files (suffix != "")
- Use hardcoded "App" prefix (matches transpilation component name)
- Write files to correct package directory

### Fix 2: Fix onClick Transformation (compose.rs:2268-2284)

```rust
("Button", "onClick") => {
    if !value.starts_with('{') {
        // increment → increment() → viewModel.increment()
        let with_parens = format!("{}()", value);
        let transformed = self.transform_viewmodel_expression(&with_parens);
        Ok(vec![format!("onClick = {{ {} }}", transformed)])
    } else {
        // {increment} → increment → increment() → viewModel.increment() → { viewModel.increment() }
        let inner = value.trim_start_matches('{').trim_end_matches('}').trim();
        let with_parens = format!("{}()", inner);
        let transformed = self.transform_viewmodel_expression(&with_parens);
        Ok(vec![format!("onClick = {{ {} }}", transformed)])
    }
}
```

**Key points:**
- Add `()` BEFORE calling `transform_viewmodel_expression`
- This creates the `increment(` pattern that the third pass can match
- The third pass then transforms `increment(` → `viewModel.increment(`
- Prevents double-prefixing issues

## Test Case

**Input** (`examples/counter/src/main.wh`):
```whitehall
var count = 0

fun increment() {
  count = count + 1
}

<Button onClick={increment} text="Increment" />
```

**Expected Output**:
1. `MainActivity.kt` with:
   ```kotlin
   @Composable
   fun App() {
       val viewModel = viewModel<AppViewModel>()
       Button(onClick = { viewModel.increment() }) {
           Text("Increment")
       }
   }
   ```

2. `AppViewModel.kt` with:
   ```kotlin
   class AppViewModel : ViewModel() {
       data class UiState(val count: Int = 0)
       fun increment() { count = count + 1 }
   }
   ```

## Related

- Phase 1.1: Component Inline Vars
- Test: `tests/transpiler-examples/30-component-inline-vars-basic.md`
- Transpiler tests are currently broken (won't compile)
