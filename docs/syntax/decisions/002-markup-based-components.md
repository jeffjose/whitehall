# Decision 002: Markup-Based Component Syntax

**Status:** Decided
**Date:** 2025-11-01
**Decider:** User preference

## Context

Components need a clear, ergonomic syntax that maps well to the mental model of UI development. The choice is between function-based composition (like Compose) vs markup-based templates (like HTML/XML).

## Options Considered

1. **Function-based (Compose-style)**
   ```whitehall
   component Button(text: String, onClick: () -> Unit) {
     render {
       Column(padding: 16) {
         Text(text)
       }
     }
   }
   ```

2. **Markup-based (Svelte/Vue-style)**
   ```whitehall
   component Button(text: String, onClick: () -> Unit)

   <script>
     // Logic here
   </script>

   <Column padding={16}>
     <Text>{text}</Text>
   </Column>
   ```

## Decision

**Use markup-based syntax (Option 2)** - Inspired by Svelte's single-file components.

## Rationale

- **Mental model:** Markup better represents UI hierarchy visually
- **Separation:** Clear distinction between logic (`<script>`) and presentation (markup)
- **Styling:** Can add `<style>` section later (like Svelte)
- **Familiarity:** Developers coming from web know HTML/JSX
- **Transpilation:** We're transpiling anyway - can convert markup → Compose functions
- **Ergonomics:** User strongly prefers this approach for developer experience
- **Complete file:** Like Svelte, everything (logic, style, markup) in one file

## Implementation Strategy

### Component structure:
```whitehall
component ComponentName(props...)

<script>
  // State, functions, lifecycle
</script>

<style>
  // Styling (future)
</style>

<!-- UI markup -->
<Column>
  <Text>Hello</Text>
</Column>
```

### Transpilation to Compose:
- Parse markup into AST
- Convert to Compose function calls
- Maintain source maps for error reporting
- Generate idiomatic Kotlin

### Example:
**Input:**
```whitehall
component LoginScreen(title: String)

<script>
  state {
    email = ""
    password = ""
  }

  fn login() {
    // auth logic
  }
</script>

<Column padding={16} spacing={8}>
  <Text fontSize={24}>{title}</Text>
  <Input bind:value={email} label="Email" />
  <Input bind:value={password} label="Password" type="password" />
  <Button onClick={login}>Login</Button>
</Column>
```

**Output (Kotlin):**
```kotlin
@Composable
fun LoginScreen(title: String) {
  var email by remember { mutableStateOf("") }
  var password by remember { mutableStateOf("") }

  fun login() {
    // auth logic
  }

  Column(
    modifier = Modifier.padding(16.dp),
    verticalArrangement = Arrangement.spacedBy(8.dp)
  ) {
    Text(text = title, fontSize = 24.sp)
    OutlinedTextField(
      value = email,
      onValueChange = { email = it },
      label = { Text("Email") }
    )
    OutlinedTextField(
      value = password,
      onValueChange = { password = it },
      label = { Text("Password") },
      visualTransformation = PasswordVisualTransformation()
    )
    Button(onClick = { login() }) {
      Text("Login")
    }
  }
}
```

## Open Questions (To Be Decided)

1. **Control flow syntax:**
   - Svelte-style: `{#if condition} ... {/if}`
   - Vue-style: `v-if={condition}`
   - JSX-style: `{condition ? <A /> : <B />}`

2. **List rendering:**
   - `{#each items as item} ... {/each}`
   - `{items.map(item => ...)}`
   - `<For each={items}> ... </For>`

3. **Two-way binding:**
   - `bind:value={email}` (Svelte)
   - `value={email} onChange={...}` (explicit)

4. **Styling approach:**
   - `<style>` section with CSS-like syntax
   - Inline props only
   - Separate style files

5. **Self-closing tags:**
   - `<Button />` vs `<Button></Button>`
   - Allow both or enforce one?

## Benefits

- Clear visual hierarchy
- Familiar to most developers
- Separates concerns naturally
- Easy to transpile to Compose
- Can optimize markup → Compose mapping
- Tooling can provide better autocomplete/validation

## Trade-offs

- Need to build markup parser
- Different from pure Compose
- Might feel unfamiliar to pure Android devs (but they can read generated Kotlin)
- Need to map all Compose concepts to markup

## Next Steps

1. Design control flow syntax (see 01-component-definition.md)
2. Design UI/View DSL details (see 02-ui-view-dsl.md)
3. Implement markup parser
4. Build Compose code generator
5. Ensure source maps for debugging
