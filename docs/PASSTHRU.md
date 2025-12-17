# Pass-Through Architecture for Whitehall Parser

**Status:** 📋 Planned
**Created:** 2025-11-07
**Priority:** High - Architectural foundation for future flexibility

---

## Table of Contents
1. [Problem Statement](#problem-statement)
2. [Current Architecture](#current-architecture)
3. [Desired Architecture](#desired-architecture)
4. [Design Decisions](#design-decisions)
5. [Implementation Phases](#implementation-phases)
6. [Technical Challenges](#technical-challenges)
7. [Testing Strategy](#testing-strategy)
8. [Success Criteria](#success-criteria)

---

## Problem Statement

### Current Limitation

The Whitehall parser uses a **strict parsing** approach where it must explicitly understand every piece of syntax in a `.wh` file. When it encounters unknown syntax, it errors out:

```whitehall
class PokemonStore { ... }

data class Pokemon(...)  // ❌ ERROR: "Expected component, found: data class"
```

This creates several problems:

1. **Brittleness** - Every new Kotlin feature requires parser updates
2. **Maintenance burden** - Must implement parsers for Kotlin features we don't transform
3. **User frustration** - Can't use standard Kotlin patterns in `.wh` files
4. **Blocks adoption** - Real-world apps need data classes, sealed classes, enums, etc.

### Root Cause

The parser was designed for component-centric files:
```whitehall
// Whitehall-specific declarations
var count = 0
fun increment() { count++ }

// Component markup
<Button onClick={increment}>
  <Text>Count: {count}</Text>
</Button>
```

But Store files are Kotlin-centric:
```whitehall
// 90% regular Kotlin code
class PokemonStore { ... }
data class Pokemon(...)
sealed class Result { ... }
enum class Type { ... }
```

The parser tries to parse ALL of this, but it only needs to transform the `class PokemonStore` part.

---

## Current Architecture

### Parser Flow (Simplified)

```rust
// src/transpiler/parser.rs
pub fn parse(&mut self) -> Result<WhitehallFile, String> {
    loop {
        skip_whitespace();

        if peek == '@' -> parse_annotation()
        else if peek == "import" -> parse_import()
        else if peek == "class" -> parse_class()
        else if peek == "var"/"val" -> parse_state()
        else if peek == "fun" -> parse_function()
        else if peek == "$onMount" -> parse_lifecycle()
        else if peek == '<' -> break  // Start markup parsing
        else -> break  // Unknown, stop parsing ⚠️
    }

    // After loop, expect markup
    parse_markup()  // If not markup, ERROR!
}
```

**Problem:** After parsing the main class, the loop breaks. Then `parse_markup()` expects `<Component>` but finds `data class` → ERROR.

### AST Structure

```rust
// src/transpiler/ast.rs
pub struct WhitehallFile {
    pub imports: Vec<Import>,
    pub props: Vec<PropDeclaration>,
    pub state: Vec<StateDeclaration>,
    pub classes: Vec<ClassDeclaration>,
    pub functions: Vec<FunctionDeclaration>,
    pub lifecycle: Vec<LifecycleHook>,
    pub markup: Markup,
}
```

**Problem:** No place for "raw Kotlin blocks" that we don't understand.

---

## Desired Architecture

### Philosophy Shift

**From:** "Parse and understand everything"
**To:** "Parse only Whitehall-specific syntax, pass through everything else"

This aligns with:
- **JSX** - Only transforms `<Component>`, rest is JavaScript
- **TypeScript** - Only handles types, rest is JavaScript
- **Svelte** - Only transforms special syntax, rest is JavaScript

### What Should Parse vs Pass-Through

#### Parse (Transform These)

**Whitehall-specific features that get transformed:**

1. **Annotations:**
   - `@store` → Generate ViewModel wrapper
   - `@hilt` → Add @HiltViewModel
   - `@json` → Transform to @Serializable
   - `@prop` → Transform to Composable parameters

2. **Component Markup:**
   - `<Button>`, `<Text>`, `<Column>` → Compose function calls
   - `@if`, `@for`, `@when` → Control flow transformation
   - `bind:value={x}` → Two-way binding expansion

3. **Import Aliases:**
   - `import $models.User` → Resolve to full package path

4. **Lifecycle Hooks:**
   - `$onMount { }` → LaunchedEffect
   - `$onDispose { }` → DisposableEffect

5. **State Declarations (top-level):**
   - `var count = 0` → Detect if needs ViewModel

#### Pass-Through (Keep As-Is)

**Standard Kotlin that doesn't need transformation:**

1. **Data Structures:**
   - `data class Pokemon(...)`
   - `sealed class Result { ... }`
   - `enum class Type { ... }`
   - `object Constants { ... }` (without @store)

2. **Functions:**
   - Top-level functions
   - Extension functions
   - Inline functions

3. **Type System:**
   - `typealias UserId = Int`
   - `interface Repository { ... }`

4. **Kotlin Features:**
   - Companion objects
   - Inner classes
   - Nested classes
   - Property delegates
   - Custom getters/setters (not in reactive classes)

5. **Annotations (non-Whitehall):**
   - `@Serializable` (unless using @json alias)
   - `@Parcelize`
   - `@Inject` (in constructors, already supported)
   - Any other Kotlin/library annotation

### New AST Structure

```rust
// src/transpiler/ast.rs

pub struct WhitehallFile {
    // Existing parsed structures
    pub imports: Vec<Import>,
    pub props: Vec<PropDeclaration>,
    pub state: Vec<StateDeclaration>,
    pub classes: Vec<ClassDeclaration>,      // Only @store/Whitehall classes
    pub functions: Vec<FunctionDeclaration>,
    pub lifecycle: Vec<LifecycleHook>,
    pub markup: Markup,

    // NEW: Raw Kotlin blocks that pass through
    pub kotlin_blocks: Vec<KotlinBlock>,  // ✨ NEW
}

#[derive(Debug, Clone, PartialEq)]
pub struct KotlinBlock {
    pub content: String,           // Raw Kotlin source code
    pub position: usize,           // Position in original file (for ordering)
    pub block_type: KotlinBlockType,  // Hint for what this might be
}

#[derive(Debug, Clone, PartialEq)]
pub enum KotlinBlockType {
    Unknown,           // Don't know, don't care
    DataClass,         // Detected "data class" keyword
    SealedClass,       // Detected "sealed class"
    EnumClass,         // Detected "enum class"
    TopLevelFunction,  // Detected top-level fun
    TypeAlias,         // Detected "typealias"
    ObjectDeclaration, // Detected "object" (not @store)
}
```

**Note:** `block_type` is just a hint for debugging/tooling. We don't parse the content, just capture it.

---

## Design Decisions

### Decision 1: When to Parse vs Pass-Through?

**Principle:** Parse only if we need to **transform** the syntax.

**Examples:**

| Syntax | Parse? | Why |
|--------|--------|-----|
| `@store class Foo { ... }` | ✅ Yes | Transform to ViewModel |
| `class Foo { var x = 0 }` | ✅ Yes | Might need ViewModel (has var) |
| `data class Bar(...)` | ❌ No | No transformation needed |
| `<Button>` | ✅ Yes | Transform to Compose call |
| `$onMount { }` | ✅ Yes | Transform to LaunchedEffect |
| `fun helper() { }` | ❌ No | No transformation needed |

### Decision 2: How to Capture Pass-Through Blocks?

**Challenge:** Need to capture arbitrary Kotlin syntax without parsing it.

**Options Considered:**

**Option A: Balanced Brace Matching** ⭐ RECOMMENDED
```rust
fn capture_kotlin_block(&mut self) -> String {
    let start = self.pos;
    let mut depth = 0;

    // Capture until we see a top-level structure we recognize
    while self.peek_char().is_some() {
        let ch = self.peek_char().unwrap();

        // Track depth for braces
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    self.advance_char();
                    break;  // End of this block
                }
            }
            _ => {}
        }

        // Check if we hit a new top-level declaration at depth 0
        if depth == 0 && self.is_top_level_keyword() {
            break;
        }

        self.advance_char();
    }

    self.input[start..self.pos].to_string()
}
```

**Pros:**
- Simple to implement
- Handles nested braces correctly
- Works for most Kotlin syntax

**Cons:**
- Doesn't handle string literals with braces: `"foo { bar }"`
- Doesn't handle comments with braces: `// } comment`

**Option B: Kotlin Tokenizer** (Too Complex)
- Use a lightweight Kotlin tokenizer
- Track string literals, comments, etc.
- More accurate but much more complex

**Decision:** Use Option A (balanced brace matching) with refinements for string/comment handling.

### Decision 3: Output Order

**Challenge:** Maintain order of declarations in output.

**Solution:** Track position in `KotlinBlock` struct.

```rust
pub struct KotlinBlock {
    pub position: usize,  // Character position in original file
    // ...
}
```

During codegen:
1. Collect all declarations (imports, classes, kotlin_blocks)
2. Sort by position
3. Output in order

**Example:**

Input:
```whitehall
import foo     // position: 0

class Store    // position: 100
data class A   // position: 200 (kotlin_block)
data class B   // position: 300 (kotlin_block)
```

Output (sorted by position):
```kotlin
import foo
class Store
data class A
data class B
```

### Decision 4: When to Start Pass-Through?

**Challenge:** After parsing main structures, when do we start capturing pass-through blocks?

**Solution:** Continue parsing loop but add pass-through branch.

**Old:**
```rust
loop {
    if known_syntax() { parse_it() }
    else { break }  // ❌ Stop on unknown
}
```

**New:**
```rust
loop {
    if is_eof() { break }

    if known_whitehall_syntax() {
        parse_it()
    }
    else if is_kotlin_syntax() {
        kotlin_blocks.push(capture_kotlin_block())  // ✨ Pass through
    }
    else {
        break  // Truly unknown, error
    }
}
```

### Decision 5: String Literal Handling (HARD PART)

**Challenge:** Need to skip over string literals when looking for braces.

**Problem:**
```kotlin
data class Foo(val x: String = "{ brace in string }")
//                                  ^ This { should NOT increase depth
```

**Solution:** Track when inside string literals.

```rust
fn capture_kotlin_block(&mut self) -> String {
    let start = self.pos;
    let mut depth = 0;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    while self.peek_char().is_some() {
        let ch = self.peek_char().unwrap();

        // Handle escape sequences
        if escaped {
            escaped = false;
            self.advance_char();
            continue;
        }

        if ch == '\\' {
            escaped = true;
            self.advance_char();
            continue;
        }

        // Track string/char literals
        match ch {
            '"' if !in_char => in_string = !in_string,
            '\'' if !in_string => in_char = !in_char,
            _ => {}
        }

        // Only track braces outside strings
        if !in_string && !in_char {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        self.advance_char();
                        break;
                    }
                }
                _ => {}
            }

            // Check for top-level keyword at depth 0
            if depth == 0 && self.is_top_level_keyword() {
                break;
            }
        }

        self.advance_char();
    }

    self.input[start..self.pos].to_string()
}
```

**This is the HARD part.** Also need to handle:
- Multi-line strings: `""" ... """`
- Raw strings: `"""$foo"""`
- Comments: `// { }` and `/* { } */`

### Decision 6: Comment Handling (HARD PART)

**Challenge:** Comments can contain braces too.

```kotlin
data class Foo(
    val x: Int  // Don't increment this { value }
)
```

**Solution:** Track comment state similar to strings.

```rust
// Add to capture_kotlin_block:
let mut in_line_comment = false;
let mut in_block_comment = false;

while self.peek_char().is_some() {
    let ch = self.peek_char().unwrap();
    let next_ch = self.peek_ahead(1);

    // Check for comment start
    if ch == '/' && next_ch == Some('/') && !in_string && !in_block_comment {
        in_line_comment = true;
        self.advance_char();
        self.advance_char();
        continue;
    }

    if ch == '/' && next_ch == Some('*') && !in_string && !in_line_comment {
        in_block_comment = true;
        self.advance_char();
        self.advance_char();
        continue;
    }

    // Check for comment end
    if in_line_comment && ch == '\n' {
        in_line_comment = false;
    }

    if in_block_comment && ch == '*' && next_ch == Some('/') {
        in_block_comment = false;
        self.advance_char();
        self.advance_char();
        continue;
    }

    // Only track braces outside strings/comments
    if !in_string && !in_char && !in_line_comment && !in_block_comment {
        // ... brace tracking ...
    }
}
```

### Decision 7: How to Detect Top-Level Keywords?

**Challenge:** Know when we've hit the start of a new declaration.

**Solution:** Look for keywords at depth 0.

```rust
fn is_top_level_keyword(&self) -> bool {
    // Keywords that start a new top-level declaration
    let keywords = [
        "import",
        "class",
        "data class",
        "sealed class",
        "enum class",
        "object",
        "interface",
        "fun",
        "val",
        "var",
        "typealias",
        "@store",
        "@json",
    ];

    for keyword in &keywords {
        if self.input[self.pos..].starts_with(keyword) {
            // Make sure it's followed by whitespace or punctuation
            let after_pos = self.pos + keyword.len();
            if let Some(ch) = self.input.chars().nth(after_pos) {
                if ch.is_whitespace() || ch == '{' || ch == '(' {
                    return true;
                }
            }
        }
    }

    false
}
```

---

## Implementation Phases

### Phase 0: Preparation & Analysis ✅

**Goal:** Understand current codebase and create detailed plan.

**Tasks:**
1. ✅ Document current parser architecture
2. ✅ Identify all places that need changes
3. ✅ Create this design document
4. ✅ Get feedback on approach

**Deliverables:**
- This document (PASSTHRU.md)
- No code changes yet

---

### Phase 1: AST Changes (Foundation)

**Goal:** Update AST to support pass-through blocks.

**Estimated Complexity:** 🟢 Low
**Estimated Time:** 1-2 hours
**Risk:** Low - additive changes only

**Tasks:**

1. **Update AST** (`src/transpiler/ast.rs`)
   ```rust
   // Add new types
   #[derive(Debug, Clone, PartialEq)]
   pub struct KotlinBlock {
       pub content: String,
       pub position: usize,
       pub block_type: KotlinBlockType,
   }

   #[derive(Debug, Clone, PartialEq)]
   pub enum KotlinBlockType {
       Unknown,
       DataClass,
       SealedClass,
       EnumClass,
       TopLevelFunction,
       TypeAlias,
       ObjectDeclaration,
   }

   // Update WhitehallFile
   pub struct WhitehallFile {
       // ... existing fields ...
       pub kotlin_blocks: Vec<KotlinBlock>,  // ✨ ADD THIS
   }
   ```

2. **Update parser initialization**
   ```rust
   // In parser.rs - parse() function
   let mut kotlin_blocks = Vec::new();  // ✨ ADD THIS
   ```

3. **Run tests** - Should still pass (no behavior change yet)

**Success Criteria:**
- Code compiles
- All existing tests pass
- No behavior changes

---

### Phase 2: Basic Pass-Through (Core Logic)

**Goal:** Implement simple pass-through for balanced braces.

**Estimated Complexity:** 🟡 Medium
**Estimated Time:** 4-6 hours
**Risk:** Medium - core parser changes

**Tasks:**

1. **Add capture_kotlin_block method** (`src/transpiler/parser.rs`)
   ```rust
   fn capture_kotlin_block(&mut self) -> Result<KotlinBlock, String> {
       let start_pos = self.pos;
       let mut depth = 0;
       let content_start = self.pos;

       // Determine block type by looking at first few words
       let block_type = self.detect_block_type();

       // Simple brace matching (no string/comment handling yet)
       while self.peek_char().is_some() {
           let ch = self.peek_char().unwrap();

           match ch {
               '{' => depth += 1,
               '}' => {
                   depth -= 1;
                   if depth == 0 {
                       self.advance_char();
                       break;
                   }
               }
               _ => {}
           }

           // Check for next top-level keyword at depth 0
           if depth == 0 && self.is_top_level_keyword() {
               break;
           }

           self.advance_char();
       }

       let content = self.input[content_start..self.pos].trim().to_string();

       Ok(KotlinBlock {
           content,
           position: start_pos,
           block_type,
       })
   }

   fn detect_block_type(&self) -> KotlinBlockType {
       let remaining = &self.input[self.pos..];

       if remaining.starts_with("data class") {
           KotlinBlockType::DataClass
       } else if remaining.starts_with("sealed class") || remaining.starts_with("sealed interface") {
           KotlinBlockType::SealedClass
       } else if remaining.starts_with("enum class") {
           KotlinBlockType::EnumClass
       } else if remaining.starts_with("fun ") {
           KotlinBlockType::TopLevelFunction
       } else if remaining.starts_with("typealias ") {
           KotlinBlockType::TypeAlias
       } else if remaining.starts_with("object ") {
           KotlinBlockType::ObjectDeclaration
       } else {
           KotlinBlockType::Unknown
       }
   }

   fn is_top_level_keyword(&self) -> bool {
       let keywords = [
           "import", "class", "data class", "sealed class",
           "enum class", "object", "interface", "fun", "val",
           "var", "typealias",
       ];

       let remaining = &self.input[self.pos..];
       keywords.iter().any(|kw| {
           remaining.starts_with(kw) && {
               let after_idx = kw.len();
               remaining.chars().nth(after_idx)
                   .map(|ch| ch.is_whitespace() || ch == '{' || ch == '(')
                   .unwrap_or(false)
           }
       })
   }
   ```

2. **Update main parse loop** (`src/transpiler/parser.rs`)
   ```rust
   loop {
       self.skip_whitespace();

       // Check for EOF
       if self.peek_char().is_none() {
           break;
       }

       // ... existing parsing logic for known Whitehall syntax ...

       // NEW: After all known syntax checks, try pass-through
       else if self.is_kotlin_syntax() {
           kotlin_blocks.push(self.capture_kotlin_block()?);
       }
       else {
           break;
       }
   }

   fn is_kotlin_syntax(&self) -> bool {
       let remaining = &self.input[self.pos..];

       // Check for Kotlin keywords that we don't explicitly parse
       remaining.starts_with("data class") ||
       remaining.starts_with("sealed class") ||
       remaining.starts_with("sealed interface") ||
       remaining.starts_with("enum class") ||
       remaining.starts_with("typealias ") ||
       (remaining.starts_with("object ") && !self.peek_word() == Some("object")) // object without @store
   }
   ```

3. **Update parse() to return kotlin_blocks**
   ```rust
   Ok(WhitehallFile {
       imports,
       props,
       state,
       classes,
       functions,
       lifecycle_hooks,
       markup,
       kotlin_blocks,  // ✨ ADD THIS
   })
   ```

4. **Create test file** (`/tmp/test-passthrough.wh`)
   ```whitehall
   class MyStore {
       var items = []
   }

   data class Item(val id: Int)
   ```

5. **Test manually:**
   ```bash
   cargo run -- compile /tmp/test-passthrough.wh
   ```

**Success Criteria:**
- Simple data classes pass through without errors
- Generated Kotlin includes the data class
- No infinite loops or crashes

**Known Limitations at this phase:**
- String literals with braces will break: `"{ foo }"`
- Comments with braces will break: `// { comment }`
- Multi-line strings not handled

---

### Phase 3: String Literal Handling (Hard Part 1)

**Goal:** Handle string literals correctly when capturing blocks.

**Estimated Complexity:** 🔴 High
**Estimated Time:** 6-8 hours
**Risk:** High - tricky edge cases

**Tasks:**

1. **Add string tracking to capture_kotlin_block**
   ```rust
   fn capture_kotlin_block(&mut self) -> Result<KotlinBlock, String> {
       let start_pos = self.pos;
       let mut depth = 0;
       let content_start = self.pos;

       // String tracking
       let mut in_string = false;
       let mut in_char = false;
       let mut in_multiline_string = false;
       let mut escaped = false;

       let block_type = self.detect_block_type();

       while self.peek_char().is_some() {
           let ch = self.peek_char().unwrap();
           let next_ch = self.peek_ahead(1);
           let next_next_ch = self.peek_ahead(2);

           // Handle escape sequences
           if escaped {
               escaped = false;
               self.advance_char();
               continue;
           }

           if ch == '\\' && (in_string || in_char) && !in_multiline_string {
               escaped = true;
               self.advance_char();
               continue;
           }

           // Check for multi-line string (""")
           if ch == '"' && next_ch == Some('"') && next_next_ch == Some('"') {
               in_multiline_string = !in_multiline_string;
               self.advance_char();
               self.advance_char();
               self.advance_char();
               continue;
           }

           // Check for regular string/char
           if !in_multiline_string {
               match ch {
                   '"' if !in_char => in_string = !in_string,
                   '\'' if !in_string => in_char = !in_char,
                   _ => {}
               }
           }

           // Only track braces outside strings
           if !in_string && !in_char && !in_multiline_string {
               match ch {
                   '{' => depth += 1,
                   '}' => {
                       depth -= 1,
                       if depth == 0 {
                           self.advance_char();
                           break;
                       }
                   }
                   _ => {}
               }

               // Check for next top-level keyword
               if depth == 0 && self.is_top_level_keyword() {
                   break;
               }
           }

           self.advance_char();
       }

       let content = self.input[content_start..self.pos].trim().to_string();

       Ok(KotlinBlock {
           content,
           position: start_pos,
           block_type,
       })
   }
   ```

2. **Add peek_ahead helper**
   ```rust
   fn peek_ahead(&self, offset: usize) -> Option<char> {
       self.input[self.pos..].chars().nth(offset)
   }
   ```

3. **Create test cases** (`tests/transpiler-examples/34-passthrough-strings.md`)
   ```markdown
   # Pass-Through with String Literals

   ## Input

   ```whitehall
   class MyStore {
       var items = []
   }

   data class Item(
       val name: String = "{ default }",
       val template: String = """
           Hello { world }
       """
   )
   ```

   ## Output

   ```kotlin
   // ... ViewModel code for MyStore ...

   data class Item(
       val name: String = "{ default }",
       val template: String = """
           Hello { world }
       """
   )
   ```
   ```

4. **Run tests:**
   ```bash
   cargo test --test transpiler_examples_test
   ```

**Success Criteria:**
- String literals with braces handled correctly
- Multi-line strings (""") handled correctly
- Escaped quotes handled correctly
- All tests pass

**Edge Cases to Test:**
- `"{ brace in string }"`
- `'{ brace in char }'`
- `""" { brace in multiline } """`
- `"escaped \" quote"`
- `"""raw string with "quotes" """`

---

### Phase 4: Comment Handling (Hard Part 2)

**Goal:** Handle comments correctly when capturing blocks.

**Estimated Complexity:** 🔴 High
**Estimated Time:** 6-8 hours
**Risk:** High - many comment edge cases

**Tasks:**

1. **Add comment tracking to capture_kotlin_block**
   ```rust
   fn capture_kotlin_block(&mut self) -> Result<KotlinBlock, String> {
       // ... existing code ...

       // Comment tracking
       let mut in_line_comment = false;
       let mut in_block_comment = false;

       while self.peek_char().is_some() {
           let ch = self.peek_char().unwrap();
           let next_ch = self.peek_ahead(1);

           // ... existing escape/string handling ...

           // Check for comment start (only outside strings)
           if !in_string && !in_char && !in_multiline_string {
               // Line comment: //
               if ch == '/' && next_ch == Some('/') && !in_block_comment {
                   in_line_comment = true;
                   self.advance_char();
                   self.advance_char();
                   continue;
               }

               // Block comment: /*
               if ch == '/' && next_ch == Some('*') && !in_line_comment {
                   in_block_comment = true;
                   self.advance_char();
                   self.advance_char();
                   continue;
               }

               // End of line comment
               if in_line_comment && ch == '\n' {
                   in_line_comment = false;
                   self.advance_char();
                   continue;
               }

               // End of block comment: */
               if in_block_comment && ch == '*' && next_ch == Some('/') {
                   in_block_comment = false;
                   self.advance_char();
                   self.advance_char();
                   continue;
               }
           }

           // Only track braces outside strings AND comments
           if !in_string && !in_char && !in_multiline_string &&
              !in_line_comment && !in_block_comment {
               // ... brace tracking ...
           }

           self.advance_char();
       }

       // ... rest of function ...
   }
   ```

2. **Create test cases** (`tests/transpiler-examples/35-passthrough-comments.md`)
   ```markdown
   # Pass-Through with Comments

   ## Input

   ```whitehall
   class MyStore {
       var items = []
   }

   data class Item(
       val id: Int,  // Don't count this { brace }
       /* Also ignore this { brace } */
       val name: String
   )
   ```

   ## Output

   ```kotlin
   // ... ViewModel code ...

   data class Item(
       val id: Int,  // Don't count this { brace }
       /* Also ignore this { brace } */
       val name: String
   )
   ```
   ```

3. **Test edge cases:**
   - Nested block comments (if Kotlin supports them)
   - Comments in strings: `"// not a comment"`
   - URLs in comments: `// http://example.com`
   - Comment markers in strings

**Success Criteria:**
- Line comments (`//`) handled correctly
- Block comments (`/* */`) handled correctly
- Comments don't affect brace counting
- All tests pass

**Edge Cases to Test:**
- `// comment with { brace }`
- `/* block comment with { brace } */`
- `"string with // not comment"`
- `/* nested /* comments */ */` (if supported)

---

### Phase 5: Codegen Integration

**Goal:** Output kotlin_blocks in generated code.

**Estimated Complexity:** 🟡 Medium
**Estimated Time:** 4-6 hours
**Risk:** Medium - ordering is tricky

**Tasks:**

1. **Update generate_store_class** (`src/transpiler/codegen/compose.rs`)
   ```rust
   fn generate_view_model_store(&self, file: &WhitehallFile, class: &ClassDeclaration) -> Result<String, String> {
       let mut output = String::new();

       // Package
       output.push_str(&format!("package {}\n\n", self.package));

       // Imports
       // ... existing import generation ...

       // ViewModel class
       // ... existing ViewModel generation ...

       output.push_str("}\n\n");  // Close ViewModel

       // NEW: Append kotlin_blocks
       for block in &file.kotlin_blocks {
           output.push_str(&block.content);
           output.push_str("\n\n");
       }

       Ok(output)
   }
   ```

2. **Handle ordering for component files**

   For files that generate multiple outputs (ViewModel + Component), need to decide where kotlin_blocks go.

   **Option A:** Put at end of ViewModel file
   ```kotlin
   // ComponentViewModel.kt
   class MyComponentViewModel : ViewModel() { ... }

   data class Item(...)  // kotlin_blocks here
   ```

   **Option B:** Put in separate file (more complex)
   ```kotlin
   // ComponentViewModel.kt
   class MyComponentViewModel : ViewModel() { ... }

   // ComponentModels.kt
   data class Item(...)
   ```

   **Decision:** Use Option A for simplicity.

3. **Update generate_component** for non-store files
   ```rust
   fn generate_component(&mut self, file: &WhitehallFile) -> Result<String, String> {
       // ... existing component generation ...

       // NEW: Append kotlin_blocks after component
       for block in &file.kotlin_blocks {
           output.push_str(&block.content);
           output.push_str("\n\n");
       }

       Ok(output)
   }
   ```

4. **Test with Pokemon Store:**
   ```bash
   cd examples/pokemon-app
   cargo run --manifest-path ../../Cargo.toml -- compile src/stores/PokemonStore.wh
   ```

**Success Criteria:**
- Data classes appear in output
- Ordering is preserved
- Generated code compiles
- Pokemon Store compiles successfully

---

### Phase 6: Testing & Edge Cases

**Goal:** Comprehensive testing of pass-through functionality.

**Estimated Complexity:** 🟡 Medium
**Estimated Time:** 4-6 hours
**Risk:** Low - mostly test writing

**Tasks:**

1. **Create test suite** (`tests/transpiler-examples/36-passthrough-comprehensive.md`)

   Test combinations:
   - Data classes with string literals
   - Sealed classes with nested classes
   - Enum classes
   - Top-level functions
   - Type aliases
   - Objects
   - Interfaces
   - Mix of parsed and pass-through

2. **Test Kotlin syntax edge cases:**
   ```whitehall
   // Labeled expressions
   loop@ for (i in 1..10) { ... }

   // Lambda with label
   items.forEach lit@ { ... }

   // Backtick identifiers
   fun `test function`() { }

   // Operators
   operator fun plus(other: Int) = ...
   ```

3. **Test real-world patterns:**
   - Sealed class hierarchies
   - Companion objects
   - Extension functions
   - Inline functions
   - Property delegates

4. **Run full test suite:**
   ```bash
   cargo test --test transpiler_examples_test
   ```

5. **Test with Pokemon app:**
   ```bash
   cd examples/pokemon-app
   cargo run --manifest-path ../../Cargo.toml -- build
   ./gradlew assembleDebug
   ```

**Success Criteria:**
- All tests pass
- Pokemon app builds successfully
- No regressions in existing functionality

---

### Phase 7: Documentation & Polish

**Goal:** Document the new functionality and clean up.

**Estimated Complexity:** 🟢 Low
**Estimated Time:** 2-3 hours
**Risk:** Low

**Tasks:**

1. **Update LANGUAGE-REFERENCE.md**

   Add section:
   ```markdown
   ## Kotlin Pass-Through

   Whitehall only transforms Whitehall-specific syntax. Regular Kotlin code
   passes through unchanged:

   ```whitehall
   @store
   class MyStore { ... }

   // Regular Kotlin - passes through unchanged
   data class Item(val id: Int)
   sealed class Result { ... }
   enum class Type { FIRE, WATER }
   ```
   ```

2. **Update GAPS.md**

   Mark as fixed:
   ```markdown
   ### 3. Data classes outside main class ~~not supported~~ ✅ FIXED

   **Current State:** ✅ **FIXED** - All Kotlin syntax now passes through
   **Fixed:** 2025-11-07
   **Details:** Parser now uses pass-through architecture. Only Whitehall-specific
   syntax is parsed and transformed. Everything else passes through unchanged.
   ```

3. **Update REF-TRANSPILER.md**

   Add section on pass-through architecture.

4. **Remove dead code:**

   Check if any parser code is now unused after pass-through.

5. **Add inline comments** to complex parts:
   ```rust
   // IMPORTANT: We must track string literals because they can contain
   // braces that should not affect depth counting. For example:
   //   data class Foo(val x: String = "{ not a brace }")
   // The { inside the string should not increase depth.
   ```

6. **Run clippy:**
   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

**Success Criteria:**
- Documentation updated
- No clippy warnings
- Code is well-commented
- Ready for review

---

## Technical Challenges

### Challenge 1: Brace Tracking in Complex Contexts 🔴 HARD

**Problem:** Braces can appear in many contexts:
```kotlin
data class Foo(
    val template: String = """
        {
            "nested": { "json": true }
        }
    """,  // ← Braces in multi-line string
    val regex: Regex = """\{.*\}""".toRegex(),  // ← Escaped braces
    val lambda: () -> Unit = { println("{") }  // ← Brace in lambda, brace in string
)
```

**Solution:**
- Track all contexts: strings, chars, multi-line strings, comments
- Test extensively with real-world code
- Accept that some edge cases might not work (document them)

**Mitigation:**
- Start simple (Phase 2)
- Add complexity incrementally (Phases 3-4)
- Test each phase thoroughly before moving on

---

### Challenge 2: Maintaining Order 🟡 MODERATE

**Problem:** Output must maintain declaration order:
```whitehall
import foo
class A { }
data class B  // Must come after class A
class C { }
data class D  // Must come after class C
```

**Solution:**
- Track position for each declaration
- Sort by position before output
- Test with interleaved declarations

**Implementation:**
```rust
// Collect all declarations with positions
let mut all_decls: Vec<(usize, DeclType)> = vec![];

for import in &file.imports {
    all_decls.push((import.position, DeclType::Import(import)));
}

for class in &file.classes {
    all_decls.push((class.position, DeclType::Class(class)));
}

for block in &file.kotlin_blocks {
    all_decls.push((block.position, DeclType::KotlinBlock(block)));
}

// Sort by position
all_decls.sort_by_key(|(pos, _)| *pos);

// Output in order
for (_, decl) in all_decls {
    match decl {
        DeclType::Import(i) => output_import(i),
        DeclType::Class(c) => output_class(c),
        DeclType::KotlinBlock(b) => output.push_str(&b.content),
    }
}
```

---

### Challenge 3: Detecting Kotlin Syntax Start 🟡 MODERATE

**Problem:** Need to know when a pass-through block starts.

**Challenge:**
```whitehall
class Store { }

// Is this next thing a data class? Or something else?
data class Foo(...)
```

**Solution:**
- Check for known Kotlin keywords
- Use `is_kotlin_syntax()` helper
- Be conservative: if unsure, try to parse as Whitehall first

**Edge Cases:**
- Annotations before data class: `@Serializable data class`
- Visibility modifiers: `private data class`
- Inline modifier: `inline class`

---

### Challenge 4: Error Messages 🟢 EASY

**Problem:** When pass-through fails, error messages might be confusing.

**Before:**
```
Error: Expected component, found: "data class"
```

**After (better):**
```
Error: Failed to parse Kotlin block starting at line 10
Hint: Check for unmatched braces or string literals
```

**Solution:**
- Wrap capture_kotlin_block errors with context
- Show the problematic content
- Give hints about common issues

---

### Challenge 5: Performance 🟢 EASY

**Problem:** Capturing large blocks as strings might be slow.

**Analysis:**
- Most `.wh` files are small (< 1000 lines)
- String operations in Rust are efficient
- No significant performance concern

**If needed later:**
- Use `&str` slices instead of `String::to_string()`
- Only allocate when necessary

---

## Testing Strategy

### Unit Tests

**Location:** `src/transpiler/parser.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_simple_data_class() {
        let input = "data class Item(val id: Int)";
        let mut parser = Parser::new(input);
        let block = parser.capture_kotlin_block().unwrap();
        assert_eq!(block.content, "data class Item(val id: Int)");
        assert_eq!(block.block_type, KotlinBlockType::DataClass);
    }

    #[test]
    fn test_capture_with_string_literal() {
        let input = r#"data class Item(val msg: String = "{ brace }")"#;
        let mut parser = Parser::new(input);
        let block = parser.capture_kotlin_block().unwrap();
        assert!(block.content.contains("{ brace }"));
    }

    #[test]
    fn test_capture_with_comment() {
        let input = "data class Item(val id: Int)  // { comment }";
        let mut parser = Parser::new(input);
        let block = parser.capture_kotlin_block().unwrap();
        assert!(block.content.contains("// { comment }"));
    }

    #[test]
    fn test_nested_braces() {
        let input = r#"
            data class Item(
                val nested: Map<String, Map<String, Int>> = mapOf()
            )
        "#;
        let mut parser = Parser::new(input);
        let block = parser.capture_kotlin_block().unwrap();
        assert!(block.content.contains("Map<String, Map<String, Int>>"));
    }
}
```

### Integration Tests

**Location:** `tests/passthru-examples/`

**Status:** ✅ Test infrastructure created (2025-11-07)

**Test Files Created:**
1. **`01-data-class.md`** - Data classes after main class
   - Tests that `data class` definitions pass through unchanged
   - Currently errors with "Expected component, found: data class"

2. **`02-sealed-class.md`** - Sealed class hierarchies
   - Tests sealed classes for state management patterns
   - Currently errors with "Expected component, found: data class"

3. **`03-enum-class.md`** - Enum classes
   - Tests enum classes for type-safe constants
   - Currently errors with "Expected component, found: enum class"

4. **`04-typealias-and-helpers.md`** - Type aliases and top-level functions
   - Tests type aliases and extension functions
   - Currently errors with "Expected component, found: typealias"

5. **`05-mixed-constructs.md`** - Multiple Kotlin constructs
   - Tests mixed data classes, sealed classes, enums, objects, type aliases
   - Real-world pattern with API store
   - Currently errors with "Expected component, found: typealias"

**Running Tests:**
```bash
# Run just pass-through tests
cargo test --test passthru_examples_test

# Run with output
cargo test --test passthru_examples_test -- --nocapture

# Run all tests including pass-through
./scripts/test-examples.sh
```

**Expected Behavior:**
- 🔴 **Currently:** All 5 tests FAIL (expected)
- 🟢 **After implementation:** All 5 tests PASS

**Additional Tests to Add (Future):**
- String literals in pass-through blocks
- Comments in pass-through blocks
- Nested constructs with complex generics
- Interface definitions
- Inline classes

### Manual Testing

**Test with Pokemon App:**
```bash
cd examples/pokemon-app
cargo run --manifest-path ../../Cargo.toml -- compile src/stores/PokemonStore.wh
```

**Expected:** All data classes pass through, code compiles.

### Regression Testing

Run full test suite after each phase:
```bash
cargo test --test transpiler_examples_test
```

**Expected:** All existing tests still pass (no regressions).

---

## Success Criteria

### Must Have ✅

1. **Data classes pass through** without errors
2. **Pokemon app compiles** after changes
3. **All existing tests pass** (no regressions)
4. **String literals handled** correctly (braces in strings ignored)
5. **Comments handled** correctly (braces in comments ignored)
6. **Output order preserved** (declarations in correct order)
7. **Error messages** are clear when pass-through fails

### Should Have 🎯

1. **Sealed classes** pass through
2. **Enum classes** pass through
3. **Type aliases** pass through
4. **Top-level functions** pass through
5. **Objects** (without @store) pass through
6. **Comprehensive tests** for edge cases
7. **Documentation** updated

### Nice to Have 🌟

1. **Block type detection** works accurately
2. **Multi-line strings** handled perfectly
3. **Nested comments** handled (if Kotlin supports)
4. **Performance** is good (< 100ms for typical files)
5. **Clippy warnings** all resolved

---

## Risks & Mitigations

### Risk 1: Infinite Loops

**Risk:** Parser gets stuck in infinite loop if brace tracking is wrong.

**Mitigation:**
- Add loop guards in capture_kotlin_block
- Track previous position, error if no progress
- Add timeout in tests

**Example guard:**
```rust
let mut prev_pos = self.pos;
let mut no_progress_count = 0;

while self.peek_char().is_some() {
    // ... parsing logic ...

    if self.pos == prev_pos {
        no_progress_count += 1;
        if no_progress_count > 10 {
            return Err("Parser stuck, no progress made".to_string());
        }
    } else {
        no_progress_count = 0;
    }

    prev_pos = self.pos;
}
```

### Risk 2: String Literal Edge Cases

**Risk:** Weird string syntax breaks brace tracking.

**Examples:**
- `"""${"interpolation"}"""`
- `"escaped \" quote"`
- `"\u{1234}"` (unicode escapes)

**Mitigation:**
- Test extensively with real Kotlin code
- Document known limitations
- Provide clear error messages

### Risk 3: Performance Degradation

**Risk:** Large files take too long to parse.

**Mitigation:**
- Benchmark with large files (>5000 lines)
- Profile if slow
- Optimize hot paths if needed

### Risk 4: Ordering Issues

**Risk:** Declarations output in wrong order, causing compile errors.

**Mitigation:**
- Track positions carefully
- Test with interleaved declarations
- Sort before output

### Risk 5: Breaking Changes

**Risk:** Existing functionality breaks due to parser changes.

**Mitigation:**
- Run full test suite after each phase
- Test Pokemon app frequently
- Keep phases small and incremental
- Can revert easily if needed

---

## Future Enhancements (Out of Scope)

These are NOT part of this implementation but could be added later:

1. **Syntax highlighting hints** for pass-through blocks in IDEs
2. **Better error messages** with suggestions
3. **Auto-formatting** of pass-through blocks
4. **Incremental parsing** for large files
5. **LSP support** for pass-through blocks
6. **Kotlin syntax validation** in pass-through (call Kotlin compiler)

---

## Related Issues

- **GAPS.md #3** - Data classes outside main class
- **Future:** `@json` annotation alias (depends on this)
- **Future:** Other Kotlin-heavy features

---

## Questions for Reviewer

1. **Phase ordering:** Is the proposed phasing reasonable?
2. **Error handling:** Should we fail on ambiguous syntax or try to continue?
3. **Ordering:** Option A (sort by position) vs Option B (maintain parse order)?
4. **Edge cases:** What edge cases am I missing?
5. **Testing:** Is the test strategy sufficient?

---

## Appendix: Code Locations

**Parser:**
- `src/transpiler/parser.rs` - Main parser logic
- `src/transpiler/parser.rs::parse()` - Main parse loop (needs update)
- `src/transpiler/parser.rs::parse_class_declaration()` - Class parsing

**AST:**
- `src/transpiler/ast.rs::WhitehallFile` - Needs `kotlin_blocks` field
- `src/transpiler/ast.rs` - Add `KotlinBlock` struct

**Codegen:**
- `src/transpiler/codegen/compose.rs::generate_view_model_store()` - Needs to output kotlin_blocks
- `src/transpiler/codegen/compose.rs::generate_component()` - Needs to output kotlin_blocks

**Tests:**
- `tests/transpiler-examples/` - Add new test files
- `tests/transpiler_examples_test.rs` - Test runner

---

## Appendix: Example Before/After

### Before (Current - Errors)

**Input:** `examples/pokemon-app/src/stores/PokemonStore.wh`
```whitehall
import kotlinx.serialization.Serializable
import okhttp3.OkHttpClient

class PokemonStore {
    var pokemonList: List<PokemonListItem> = []

    private val client = OkHttpClient()

    suspend fun loadData() { ... }
}

data class PokemonListItem(
    val name: String,
    val url: String
)
```

**Error:**
```
Error: [Line 11:1] Expected component, found: "data class PokemonListItem"
```

### After (With Pass-Through)

**Input:** Same as above

**Output:** `PokemonStore.kt`
```kotlin
package com.example.app

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable
import okhttp3.OkHttpClient

class PokemonStore : ViewModel() {
    data class UiState(
        val pokemonList: List<PokemonListItem> = []
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    private val client = OkHttpClient()

    var pokemonList: List<PokemonListItem>
        get() = _uiState.value.pokemonList
        set(value) { _uiState.update { it.copy(pokemonList = value) } }

    fun loadData() {
        viewModelScope.launch {
            // ...
        }
    }
}

// ✨ Pass-through block
data class PokemonListItem(
    val name: String,
    val url: String
)
```

**Result:** ✅ Compiles successfully!

---

*Last Updated: 2025-11-07*
*Status: Ready for Implementation*
*Phases: 0-7 (Estimated 30-40 hours total)*
