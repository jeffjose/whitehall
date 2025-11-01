# UI/View DSL Syntax

## Context

The UI DSL is where developers spend most of their time. Goals:
- **Readable**: Easy to visualize the UI tree
- **Concise**: Minimal noise for common patterns
- **Powerful**: Handle conditionals, lists, events cleanly
- **Compose-friendly**: Map naturally to Jetpack Compose

Key question: **How close should we mirror Compose vs create abstractions?**

---

## Option A: Direct Compose Mirroring (Current Proposal)

```whitehall
render {
  Column(
    modifier = Modifier
      .fillMaxWidth()
      .padding(16.dp),
    verticalArrangement = Arrangement.spacedBy(8.dp)
  ) {
    Text(
      text = "Hello, $name",
      style = MaterialTheme.typography.headlineMedium
    )

    Button(
      onClick = () => { handleClick() },
      enabled = !isLoading
    ) {
      Text("Submit")
    }

    if (showError) {
      Text(
        text = errorMessage,
        color = Color.Red
      )
    }
  }
}
```

**Pros:**
- 1:1 with Compose (easy transpilation)
- Developers can use Compose docs directly
- No "magic" - what you see is what you get

**Cons:**
- Verbose (Modifier chains, named parameters)
- Requires learning Compose concepts
- Less ergonomic than modern frameworks

**Transpiles to:** Literally the same Kotlin code

---

## Option B: Simplified DSL (Abstract Common Patterns)

```whitehall
render {
  Column(padding: 16, spacing: 8, fill: true) {
    Text("Hello, $name", style: "headline")

    Button(
      text: "Submit",
      onClick: handleClick,
      disabled: isLoading
    )

    if (showError) {
      Text(errorMessage, color: "error")
    }
  }
}
```

**Pros:**
- Much more concise
- Named parameters without `=`
- Shorthand for common patterns (fill, spacing, colors)
- Feels modern (like SwiftUI)

**Cons:**
- Abstracts Compose (learning two systems)
- Need to map shorthands to Compose (e.g., `color: "error"`)
- Less control (where's Modifier?)
- Escape hatch needed for advanced cases

**Transpiles to:**
```kotlin
Column(
  modifier = Modifier.fillMaxWidth().padding(16.dp),
  verticalArrangement = Arrangement.spacedBy(8.dp)
) {
  Text("Hello, $name", style = MaterialTheme.typography.headlineMedium)
  Button(
    onClick = { handleClick() },
    enabled = !isLoading
  ) {
    Text("Submit")
  }
  if (showError) {
    Text(errorMessage, color = MaterialTheme.colorScheme.error)
  }
}
```

---

## Option C: Hybrid (Shortcuts + Escape Hatch)

```whitehall
render {
  // Shorthand syntax for common cases
  Column(padding: 16, spacing: 8) {
    Text("Hello, $name", style: "headline")

    Button("Submit", onClick: handleClick, disabled: isLoading)

    // But allow raw Compose when needed
    @compose {
      LazyColumn(
        modifier = Modifier.weight(1f),
        state = rememberLazyListState()
      ) {
        items(messages) { msg =>
          MessageCard(msg)
        }
      }
    }
  }
}
```

**Pros:**
- Best of both worlds
- Shortcuts for 90% case
- Power user escape hatch
- Gradual learning curve

**Cons:**
- Two syntaxes to learn
- When to use which?
- `@compose` blocks might feel jarring

---

## Option D: CSS-Style Modifiers

```whitehall
render {
  Column {
    Text("Hello, $name") {
      fontSize: 24.sp
      fontWeight: bold
      color: primary
    }

    Button("Submit", onClick: handleClick) {
      padding: 12.dp
      backgroundColor: primary
      cornerRadius: 8.dp
      disabled: isLoading
    }
  }

  style {
    padding: 16.dp
    spacing: 8.dp
  }
}
```

**Pros:**
- CSS-like (familiar to web devs)
- Clean separation of structure and style
- Reads like a component hierarchy

**Cons:**
- Very different from Compose
- Style blocks might be confusing (when do they apply?)
- Mixing properties with styles

---

## Conditional Rendering Comparison

### Option A (Compose-like):
```whitehall
if (isLoggedIn) {
  ProfileScreen()
} else {
  LoginScreen()
}
```

### Option B (Ternary):
```whitehall
isLoggedIn ? ProfileScreen() : LoginScreen()
```

### Option C (When expression):
```whitehall
when {
  isLoggedIn => ProfileScreen()
  hasError => ErrorScreen()
  else => LoginScreen()
}
```

---

## List Rendering Comparison

### Option A (Compose-like):
```whitehall
LazyColumn {
  items(todos) { todo =>
    TodoItem(todo)
  }
}
```

### Option B (For loop):
```whitehall
Column {
  for todo in todos {
    TodoItem(todo)
  }
}
```
*Issue: Not lazy!*

### Option C (Special directive):
```whitehall
Column {
  @each(todo in todos) {
    TodoItem(todo)
  }
}
```

### Option D (Built-in List component):
```whitehall
List(items: todos, renderItem: (todo) => {
  TodoItem(todo)
})
```

---

## Event Handlers

### Option A (Lambda):
```whitehall
Button(onClick: () => { handleClick() })
```

### Option B (Direct reference):
```whitehall
Button(onClick: handleClick)
```

### Option C (String reference - like Vue):
```whitehall
Button(@click: "handleClick")
```
*Too magical?*

---

## Real-World Example: Login Form

**Option A (Direct Compose):**
```whitehall
render {
  Column(
    modifier = Modifier.fillMaxSize().padding(16.dp),
    verticalArrangement = Arrangement.Center,
    horizontalAlignment = Alignment.CenterHorizontally
  ) {
    TextField(
      value = email,
      onValueChange = (value) => { email = value },
      label = { Text("Email") },
      modifier = Modifier.fillMaxWidth()
    )

    Spacer(modifier = Modifier.height(8.dp))

    TextField(
      value = password,
      onValueChange = (value) => { password = value },
      label = { Text("Password") },
      visualTransformation = PasswordVisualTransformation(),
      modifier = Modifier.fillMaxWidth()
    )

    Spacer(modifier = Modifier.height(16.dp))

    Button(
      onClick = () => { login() },
      modifier = Modifier.fillMaxWidth()
    ) {
      Text("Login")
    }
  }
}
```

**Option B (Simplified):**
```whitehall
render {
  Column(padding: 16, center: true, spacing: 8) {
    Input(
      value: email,
      onChange: (value) => { email = value },
      label: "Email",
      fill: true
    )

    Input(
      value: password,
      onChange: (value) => { password = value },
      label: "Password",
      type: "password",
      fill: true
    )

    Spacer(height: 16)

    Button("Login", onClick: login, fill: true)
  }
}
```

**Which feels better?**

---

## Recommendation

**Start with Option B (Simplified DSL) with escape hatch plan:**

1. Create ergonomic shortcuts for common patterns:
   - `padding: 16` instead of `modifier = Modifier.padding(16.dp)`
   - `fill: true` instead of `modifier = Modifier.fillMaxWidth()`
   - Named colors: `color: "primary"` instead of `MaterialTheme.colorScheme.primary`

2. Map directly to Compose in transpiler

3. Plan for `@compose { }` blocks later for advanced cases

4. Use Compose naming where possible (Column, Row, Button, etc.)

**Why:**
- Developer ergonomics is the priority
- Can always expose Compose later
- Most apps don't need full Compose power
- Easier to optimize transpilation when we control the DSL

---

## Open Questions

1. **Modifiers:** Do we even expose `Modifier` or hide it completely?
   - Option: All styling via named parameters that map to Modifier internally

2. **Units:** `16.dp` vs `16` (assume dp) vs `16dp` (string)?
   ```whitehall
   padding: 16        // Assume dp?
   padding: 16.dp     // Explicit
   padding: "16dp"    // String (easier parsing?)
   ```

3. **Slots/Children:** How to handle components that accept children?
   ```whitehall
   Card {
     Text("I'm inside the card")  // Block syntax
   }

   // vs

   Card(content: { Text("Inside") })  // Explicit content param
   ```

4. **Lists:** Should we have a separate `List` component or just use `LazyColumn`?
   - Most apps need lazy lists
   - But `for` loops are more intuitive

5. **Animations:** How to expose Compose's animation APIs cleanly?
