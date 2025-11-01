# Component Definition Syntax

**Status:** ✅ DECIDED - See [Decision 003: @prop Annotation](./decisions/003-prop-annotation.md)

## Context

Components are the fundamental building block. The syntax needs to be:
- **Clear**: Easy to understand what's a prop vs state vs method
- **Concise**: Minimal boilerplate for common cases
- **Kotlin-native**: Use Kotlin keywords and conventions
- **Type-safe**: Map cleanly to Kotlin's type system

## DECIDED SYNTAX

**Filename = Component Name** - No redundant `component Button` declaration inside files.

**Component Structure:**
```whitehall
<!-- Optional: imports -->
import android.util.Log
import com.example.data.UserRepository

<!-- Script section: props, state, functions -->
<script>
  @prop val propName: Type
  @prop val optionalProp: Type = defaultValue

  var stateVariable = initialValue
  val computedValue = someExpression

  fun handleSomething() {
    // logic
  }
</script>

<!-- Optional: styles (future) -->
<style>
  // CSS-like styling
</style>

<!-- UI markup (everything outside script/style) -->
<Column>
  <Text>{propName}</Text>
</Column>
```

**Key Rules:**
- Props: `@prop val name: Type` (explicit type required, must use `val`)
- State: `var name = value` (Kotlin var/val)
- Functions: `fun name() { }` (Kotlin syntax)
- Filename determines component name (e.g., `Button.wh` → `Button` component)

---

## Option A: Current Proposal (Sections-Based)

```whitehall
component Button {
  props {
    text: String
    onClick: () -> Unit
    disabled: Boolean = false
  }

  state {
    isPressed: Boolean = false
  }

  view {
    // UI here
  }

  fn handlePress() {
    if (!disabled) {
      isPressed = true
      onClick()
    }
  }
}
```

**Pros:**
- Clear sections separate concerns
- Easy to find state vs props
- Explicit view section

**Cons:**
- Verbose for simple components
- Lots of nested blocks
- Props and state visually similar

**Maps to Kotlin:**
```kotlin
@Composable
fun Button(
  text: String,
  onClick: () -> Unit,
  disabled: Boolean = false
) {
  var isPressed by remember { mutableStateOf(false) }

  fun handlePress() {
    if (!disabled) {
      isPressed = true
      onClick()
    }
  }

  // Compose UI
}
```

---

## Option B: Function-Style (Compose-like)

```whitehall
component Button(
  text: String,
  onClick: () -> Unit,
  disabled: Boolean = false
) {
  let isPressed = state(false)

  fn handlePress() {
    if (!disabled) {
      isPressed.set(true)
      onClick()
    }
  }

  Column {
    // UI here
  }
}
```

**Pros:**
- Very close to Compose (easier transpilation)
- Concise
- Props are just function parameters

**Cons:**
- No explicit `view` section (could be confusing)
- State manipulation with `.set()` is verbose
- Less structured for complex components

**Maps to Kotlin:**
Nearly 1:1 with Compose

---

## Option C: Hybrid (Structured but Concise)

```whitehall
component Button(text: String, onClick: () -> Unit, disabled: Boolean = false) {
  state {
    isPressed = false
  }

  fn handlePress() {
    if (!disabled) {
      isPressed = true
      onClick()
    }
  }

  render {
    Column {
      // UI here
    }
  }
}
```

**Pros:**
- Props in signature (familiar, concise)
- State block for grouping
- Explicit render block
- Direct state assignment (reactive)

**Cons:**
- State types are inferred (could be unclear)
- Mixes styles

---

## Option D: Svelte-Inspired (Script + Template)

```whitehall
component Button(text: String, onClick: () -> Unit, disabled: Boolean = false)

<script>
  let isPressed: Boolean = false

  fn handlePress() {
    if (!disabled) {
      isPressed = true
      onClick()
    }
  }
</script>

<view>
  <Column>
    <!-- UI here -->
  </Column>
</view>
```

**Pros:**
- Very Svelte-like (proven DX)
- Clear separation of logic and template
- Could use XML-style UI tags

**Cons:**
- XML-style might feel dated for Compose
- Mixed syntax styles (declaration, script tag, view tag)
- Harder to parse

---

## Edge Cases to Consider

### Simple Component (no state, no logic)

**Option A:**
```whitehall
component Label {
  props {
    text: String
  }

  view {
    Text(text)
  }
}
```

**Option C:**
```whitehall
component Label(text: String) {
  render {
    Text(text)
  }
}
```

### Component with Generic Type

**Option C:**
```whitehall
component List<T>(items: List<T>, renderItem: (T) -> Component) {
  render {
    Column {
      for item in items {
        renderItem(item)
      }
    }
  }
}
```

### Private vs Public Components

Should we support:
```whitehall
component Button(...) { }           // Public (exported)
private component InternalHelper { } // Private to file
```

Or make everything public by default and use file organization?

---

## Complete Examples with DECIDED Syntax

### Simple Button Component

**Button.wh:**
```whitehall
<script>
  @prop val text: String
  @prop val onClick: () -> Unit
  @prop val disabled: Boolean = false

  var isPressed = false

  fun handlePress() {
    if (!disabled) {
      isPressed = true
      onClick()
    }
  }
</script>

<TouchableRipple
  onClick={handlePress}
  disabled={disabled}
>
  <Text color={disabled ? "gray" : "primary"}>
    {text}
  </Text>
</TouchableRipple>
```

### Component with Complex Props

**UserCard.wh:**
```whitehall
import com.example.models.User

<script>
  @prop val user: User
  @prop val onEdit: (User) -> Unit
  @prop val highlighted: Boolean = false

  var showMenu = false

  fun toggleMenu() {
    showMenu = !showMenu
  }
</script>

<Card elevation={highlighted ? 8 : 2}>
  <Row padding={16} spacing={12}>
    <Avatar url={user.avatarUrl} size={48} />
    <Column fill>
      <Text fontSize={18} fontWeight="bold">{user.name}</Text>
      <Text color="secondary">{user.email}</Text>
    </Column>
    {#if showMenu}
      <IconButton icon="edit" onClick={() => onEdit(user)} />
    {/if}
  </Row>
</Card>
```

### Component with No Props

**Counter.wh:**
```whitehall
<script>
  var count = 0

  fun increment() {
    count++
  }

  fun reset() {
    count = 0
  }
</script>

<Column center spacing={16}>
  <Text fontSize={48} fontWeight="bold">{count}</Text>
  <Row spacing={8}>
    <Button text="+" onClick={increment} />
    <Button text="Reset" onClick={reset} />
  </Row>
</Column>
```

---

## Historical Options Considered (For Reference)

### Option C (Previous Recommendation)

**Start with Option C** for these reasons:
1. **Props in signature**: Familiar to most developers (JS/TS/Rust)
2. **State block**: Groups stateful declarations, clear from props
3. **Render block**: Explicit UI section
4. **Direct assignment**: `isPressed = true` feels reactive, transpiles to `isPressed = true` in Compose (which works with `by remember { mutableStateOf() }`)

We can always add shorthand later (e.g., allow omitting `render` for single-expression components).

---

## Open Questions

1. Should we require type annotations for state, or allow inference?
   ```whitehall
   state {
     count = 0         // Infer Int?
     count: Int = 0    // Explicit
   }
   ```

2. How to handle multiple return values from `render`?
   ```whitehall
   render {
     Text("Hello")  // Implicit return?
     Text("World")  // Multiple children in what?
   }
   ```

3. Do we need a concept of "slots" (like Vue/Svelte)?
   ```whitehall
   component Card(title: String, @slot content: Component) {
     render {
       Column {
         Text(title)
         content  // How to render passed component?
       }
     }
   }
   ```
