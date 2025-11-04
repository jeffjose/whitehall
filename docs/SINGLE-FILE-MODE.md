# Single-File Android Apps

**Status**: 📋 Design Phase - Not Yet Implemented
**Priority**: Medium (after end-to-end testing)
**Inspiration**: Python's `uv`, Rust's `rust-script`

---

## Vision: Start Small, Scale Up

Whitehall should enable developers to:
1. **Learn** with a single file (0 boilerplate)
2. **Prototype** quickly without setup
3. **Scale up** to full projects seamlessly

```whitehall
single-file → multi-component → full project with routing
   (5 min)       (30 min)              (production)
```

---

## Example: Counter App

**counter.wh** (complete Android app in one file):

```whitehall
#!/usr/bin/env whitehall
/// [app]
/// name = "Counter"
/// package = "com.example.counter"
/// minSdk = 24
///
/// [dependencies]
/// # Optional - uses sensible defaults

var count = 0

<Column padding={16} spacing={8}>
  <Text fontSize={32}>{count}</Text>
  <Button onClick={() => count++}>
    <Text>Increment</Text>
  </Button>
</Column>
```

**Run it:**
```bash
chmod +x counter.wh
./counter.wh           # Direct execution
# OR
whitehall run counter.wh
```

---

## Command Interface

### `whitehall run <file.wh>`
**Single-file execution:**
```bash
whitehall run counter.wh
# 1. Parse frontmatter
# 2. Generate temp project in .whitehall/cache/{hash}/
# 3. Build APK
# 4. Install and launch
```

### `whitehall build <file.wh>`
**Build APK from single file:**
```bash
whitehall build counter.wh --output counter.apk
# Generates APK directly
```

---

## Frontmatter Format

### Required Fields
```whitehall
/// [app]
/// name = "MyApp"                    # Required
/// package = "com.example.myapp"    # Required
```

### Optional Fields (with defaults)
```whitehall
/// minSdk = 24          # Android 7.0 (default)
/// targetSdk = 34       # Latest stable (default)
/// compileSdk = 34      # Latest stable (default)
///
/// [dependencies]
/// # Optional - defaults to Material3 + Navigation
/// androidx.compose.material3 = "1.2.0"
/// androidx.navigation.compose = "2.8.0"
```

### Full Example
```whitehall
#!/usr/bin/env whitehall
/// [app]
/// name = "Todo List"
/// package = "com.example.todo"
/// minSdk = 26
///
/// [dependencies]
/// androidx.compose.material.icons = "1.6.0"

var todos = ["Buy milk", "Write code"]
var newTodo = ""

fun addTodo() {
  if (newTodo.isNotEmpty()) {
    todos = todos + newTodo
    newTodo = ""
  }
}

<Column padding={16} spacing={12}>
  <Text fontSize={24}>Todo List</Text>

  <Row>
    <TextField bind:value={newTodo} label="New Todo" />
    <Button onClick={addTodo}><Text>Add</Text></Button>
  </Row>

  @for (todo in todos) {
    <Card padding={8}>
      <Text>{todo}</Text>
    </Card>
  }
</Column>
```

---

## Implementation Architecture

### 1. Frontmatter Parser

```rust
// src/single_file.rs

pub struct SingleFileConfig {
    pub app_name: String,
    pub package: String,
    pub min_sdk: u32,
    pub target_sdk: u32,
    pub dependencies: HashMap<String, String>,
}

pub fn parse_frontmatter(content: &str) -> Result<(SingleFileConfig, String)> {
    // Extract lines starting with ///
    // Parse as TOML
    // Return config + remaining content
}
```

### 2. Temporary Project Generation

```
.whitehall/cache/
  └── {sha256-hash}/          # Hash of file content
      ├── whitehall.toml      # Generated from frontmatter
      ├── src/
      │   └── main.wh         # File content (without frontmatter)
      └── build/              # Standard build output
```

**Cache Benefits:**
- Same file → same hash → reuse existing build
- Edit file → new hash → clean rebuild
- Fast iteration for unchanged files

### 3. Command Routing

```rust
// src/main.rs

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Run { path } => {
            if is_single_file(&path) {
                commands::run_single_file(&path)?
            } else {
                commands::run::execute(&path)?
            }
        }
        // Similar for build, etc.
    }
}

fn is_single_file(path: &str) -> bool {
    path.ends_with(".wh") && !path.contains("whitehall.toml")
}
```

---

## Guardrails & Constraints

### File Size Limits
```
< 100 lines   ✅ Perfect for single file
100-500 lines ⚠️  Consider splitting
> 500 lines   🚫 Suggest: whitehall split
> 1000 lines  ❌ Error: File too large for single-file mode
```

### Feature Restrictions
**Single-file mode:**
- ✅ Single screen only
- ✅ State management
- ✅ Basic components (Text, Button, Column, etc.)
- ❌ No routing (suggest project mode)
- ❌ No multiple screens
- ❌ No custom components in separate files

**Upgrade prompts:**
```bash
# User tries routing in single file:
Error: Routing requires project mode
Suggestion: Create a project with `whitehall init` for routing support
```

### Shebang Support
```whitehall
#!/usr/bin/env whitehall
/// [app]
/// name = "Counter"
/// ...

var count = 0
...
```

```bash
chmod +x counter.wh
./counter.wh  # Direct execution!
```

---

## Comparison to Other Tools

| Feature | uv (Python) | rust-script | **Whitehall** |
|---------|-------------|-------------|---------------|
| **Inline deps** | ✅ `# dependencies = [...]` | ✅ Cargo block | ✅ TOML frontmatter |
| **Shebang** | ✅ `#!/usr/bin/env uv` | ✅ `#!/usr/bin/env rust-script` | ✅ `#!/usr/bin/env whitehall` |
| **Caching** | ✅ `.uv/cache/` | ✅ `~/.rust-script/` | ✅ `.whitehall/cache/{hash}/` |
| **Type safety** | ❌ Runtime | ✅ Compile-time | ✅ Compile-time |

---

## Upgrade Path Examples

### Stage 1: Learning (Single File)
```whitehall
// counter.wh
var count = 0
<Button onClick={() => count++}>{count}</Button>
```

```bash
whitehall run counter.wh
```

### Stage 2: Growing (Still Single File)
```whitehall
// todo.wh - More complex, still manageable
var todos = [...]
var filter = "all"

fun addTodo() { ... }
fun toggleTodo() { ... }

<Column>
  <Header />
  <TodoList />
  <Footer />
</Column>
```

### Stage 3: Production (Full Project)
```bash
whitehall split todo.wh --output todo-app/
cd todo-app

# Now have:
# - src/components/ (Header, TodoList, Footer)
# - src/routes/ (routing)
# - Proper structure
```

---

## Implementation Phases

### Phase 1: Basic Single-File (2-3 hours)
**Tasks:**
1. Frontmatter parser (`///` comments)
2. Temp project generator (`.whitehall/cache/`)
3. Single-file detection in commands
4. `whitehall run <file.wh>`

**Success:** Can run counter.wh end-to-end

### Phase 2: Build & Watch (1-2 hours)
**Tasks:**
1. `whitehall build <file.wh>`
2. `whitehall watch <file.wh>` command
3. Better error messages

**Success:** Can build APK from single file, can watch for changes

### Phase 3: Polish (1 hour)
**Tasks:**
1. Shebang support
2. Size warnings
3. Feature detection (routing → suggest project mode)

**Success:** Production-ready single-file mode

---

## Example Use Cases

### 1. Learning Tutorial
```bash
# Tutorial step 1: Hello World
cat > hello.wh << 'EOF'
/// [app]
/// name = "Hello"
/// package = "com.tutorial.hello"

<Text>Hello, World!</Text>
EOF

whitehall run hello.wh
```

### 2. Quick Prototype
```bash
# Test idea in 5 minutes
vim prototype.wh
whitehall run prototype.wh
# Iterate quickly
```

### 3. Code Sharing
```bash
# Share complete app as single file
curl https://gist.github.com/.../counter.wh | whitehall run -
# Or
whitehall run https://gist.github.com/.../counter.wh
```

### 4. Gradual Complexity
```whitehall
// Week 1: Simple counter
var count = 0
<Button onClick={() => count++}>{count}</Button>

// Week 2: Add state
var count = 0
var history = []
fun increment() { count++; history += count }

// Week 3: Add components
fun Counter() { ... }
fun History() { ... }

// Week 4: Too complex → upgrade to full project
$ whitehall init app
# Migrate code to app/ directory
✅ Now a full project with proper structure
```

---

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_parse_frontmatter() {
    let input = r#"
/// [app]
/// name = "Test"
/// package = "com.test"
///
var x = 5
"#;
    let (config, content) = parse_frontmatter(input).unwrap();
    assert_eq!(config.app_name, "Test");
    assert!(content.contains("var x = 5"));
}
```

### Integration Tests
```bash
# Test full workflow
echo "..." > test.wh
whitehall run test.wh
# Verify APK generated and installed
```

---

## Open Questions

1. **Inline components?**
   - Allow `fun Counter() { ... }` in single file?
   - Or require `whitehall split` for multi-component?

2. **Dependencies from Maven?**
   - Allow custom dependencies in frontmatter?
   - Or restrict to standard Material3?

3. **Maximum file size?**
   - Hard limit at 1000 lines?
   - Or just warnings?

4. **Shebang location?**
   - Support `#!` as first line (standard)
   - Or require frontmatter first?

---

## Future Enhancements

- **Hot reload for single files** (watch + auto-install)
- **Web playground** (run single files in browser)
- **Template library** (`whitehall create counter` → downloads template)
- **Snippet sharing** (publish to gist/pastebin automatically)

---

## Success Metrics

Single-file mode is successful when:
- ✅ Can go from idea to running app in <5 minutes
- ✅ Zero boilerplate for simple apps
- ✅ Clear upgrade path to full projects
- ✅ Matches or beats `flutter create` + `flutter run` UX

---

**Status**: Design complete, awaiting implementation
**Dependencies**: None (can implement independently)
**Priority**: Medium (after e2e testing of project mode)
