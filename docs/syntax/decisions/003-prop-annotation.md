# Decision 003: @prop Annotation for Component Props

**Status:** Decided
**Date:** 2025-11-01
**Decider:** User preference

## Context

Component props need a clear, Kotlin-native syntax that:
- Distinguishes props from state
- Maintains type safety
- Feels natural to Kotlin/Android developers
- Works with syntax highlighting and tooling

## Options Considered

1. **Separate `props()` block**
   ```whitehall
   props(text: String, onClick: () -> Unit)
   <script>
     var pressed = false
   </script>
   ```

2. **`@prop` annotation on val declarations**
   ```whitehall
   <script>
     @prop val text: String
     @prop val onClick: () -> Unit
     var pressed = false
   </script>
   ```

3. **Svelte-style `export let`**
   ```whitehall
   <script>
     export let text: String
     export let onClick: () -> Unit
     var pressed = false
   </script>
   ```

## Decision

**Use `@prop` annotation (Option 2)** - Annotate props with `@prop` inside `<script>` section.

## Syntax Rules

1. **Props must be annotated with `@prop`**
   ```whitehall
   @prop val text: String
   ```

2. **Props must use `val` (immutable)**
   ```whitehall
   @prop val text: String     // ✅ Correct
   @prop var text: String     // ❌ Error: props must be val
   ```

3. **Explicit types are required**
   ```whitehall
   @prop val text: String     // ✅ Correct
   @prop val text             // ❌ Error: type required
   ```

4. **Default values are supported**
   ```whitehall
   @prop val disabled: Boolean = false
   @prop val color: String = "primary"
   ```

5. **Nullable types are supported**
   ```whitehall
   @prop val user: User? = null
   @prop val title: String?
   ```

6. **Order doesn't matter** - Props can be interspersed with state
   ```whitehall
   <script>
     var count = 0
     @prop val name: String
     var isLoading = false
     @prop val age: Int
   </script>
   ```
   Though recommended to group props together for readability.

## Rationale

- **Kotlin-native:** Uses `val`/`var` keywords, not `let` or `export`
- **Clear marker:** `@prop` makes it visually obvious what's a prop
- **Type-safe:** Explicit types required, caught at compile time
- **Flexible:** Props and state in same `<script>` block
- **Tooling-friendly:** Kotlin syntax highlighters will work
- **Future-proof:** Can add other annotations (`@state`, `@computed`) if needed
- **Immutable by design:** `val` enforces props can't be reassigned

## Examples

### Simple Component

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

<TouchableRipple onClick={handlePress} disabled={disabled}>
  <Text>{text}</Text>
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
</script>

<Card elevation={highlighted ? 8 : 2}>
  <Text>{user.name}</Text>
  <Text>{user.email}</Text>
</Card>
```

### Component with Only State (No Props)

**Counter.wh:**
```whitehall
<script>
  var count = 0

  fun increment() {
    count++
  }
</script>

<Column>
  <Text>{count}</Text>
  <Button text="+" onClick={increment} />
</Column>
```

## Transpilation

**Input:**
```whitehall
<script>
  @prop val text: String
  @prop val disabled: Boolean = false

  var pressed = false
</script>
```

**Output:**
```kotlin
@Composable
fun ComponentName(
  text: String,
  disabled: Boolean = false
) {
  var pressed by remember { mutableStateOf(false) }
  // ...
}
```

## Validation Rules

Compiler must enforce:
1. ❌ `@prop` on `var` → Error: "Props must use 'val', not 'var'"
2. ❌ `@prop` without type → Error: "Props must have explicit type annotation"
3. ✅ `@prop val name: String` → Valid
4. ✅ `@prop val count: Int = 0` → Valid with default
5. ✅ `@prop val user: User?` → Valid nullable

## Open Questions for Future

1. Should we support `@required` for props without defaults?
   ```whitehall
   @required @prop val userId: String  // Must be provided
   ```

2. Validation annotations?
   ```whitehall
   @prop @range(0, 100) val age: Int
   ```

3. Documentation annotations?
   ```whitehall
   /** The user's display name */
   @prop val name: String
   ```
