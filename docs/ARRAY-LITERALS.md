# Array Literal Syntax

Whitehall supports convenient array literal syntax using square brackets `[...]`, which transpiles to Kotlin's `listOf()` or `mutableListOf()` based on whether the variable is declared with `val` or `var`.

## Syntax

```whitehall
val immutableList = [1, 2, 3]
var mutableList = ["A", "B", "C"]
```

## Transpilation Rules

### Immutable Lists (`val`)

Array literals in `val` declarations transpile to `listOf()`:

```whitehall
val numbers = [1, 2, 3]
```

Transpiles to:

```kotlin
val numbers = listOf(1, 2, 3)
```

### Mutable Lists (`var`)

Array literals in `var` declarations transpile to `mutableListOf()` wrapped in `remember { mutableStateOf(...) }`:

```whitehall
var items = [1, 2, 3]
```

Transpiles to:

```kotlin
var items by remember { mutableStateOf(mutableListOf(1, 2, 3)) }
```

## Supported Features

### Mixed Types

```whitehall
val mixed = [1, "two", 3.0, true]
```

Transpiles to:

```kotlin
val mixed = listOf(1, "two", 3.0, true)
```

### Nested Arrays

Nested arrays are recursively transformed (nested arrays are always immutable):

```whitehall
val matrix = [[1, 2], [3, 4], [5, 6]]
```

Transpiles to:

```kotlin
val matrix = listOf(listOf(1, 2), listOf(3, 4), listOf(5, 6))
```

### Multiline Arrays

Array literals support multiline formatting for readability:

```whitehall
val fruits = [
  "Apple",
  "Banana",
  "Cherry",
  "Date"
]
```

Transpiles to:

```kotlin
val fruits = listOf("Apple",
  "Banana",
  "Cherry",
  "Date")
```

### Empty Arrays

```whitehall
val empty = []
var mutable = []
```

Transpiles to:

```kotlin
val empty = listOf()
var mutable by remember { mutableStateOf(mutableListOf()) }
```

## Type Annotations

Array literals work with explicit type annotations:

```whitehall
val numbers: List<Int> = [1, 2, 3]
var items: List<String> = ["A", "B"]
```

Transpiles to:

```kotlin
val numbers: List<Int> = listOf(1, 2, 3)
var items by remember { mutableStateOf<List<String>>(mutableListOf("A", "B")) }
```

## Arrays vs Lists

**Important:** The `[...]` syntax creates **Lists**, not **Arrays** in Kotlin.

- `List<T>` - Interface-based, flexible, immutable by default
- `Array<T>` - Fixed-size, maps to JVM primitive arrays

If you need an actual `Array<T>`, use `arrayOf()` explicitly:

```whitehall
val array = arrayOf(1, 2, 3)  // Creates Array<Int>
```

## Why This Syntax?

The array literal syntax provides several benefits:

1. **Familiar** - Coming from JavaScript, Python, TypeScript? `[...]` feels natural
2. **Concise** - `[1, 2, 3]` vs `listOf(1, 2, 3)` saves typing
3. **Smart** - Automatically chooses between `listOf()` and `mutableListOf()` based on `val`/`var`
4. **Type-safe** - Kotlin compiler still validates types
5. **Semantically correct** - `val` → immutable, `var` → mutable

## Migration from `listOf()`

Both syntaxes are supported and can be mixed:

```whitehall
val old = listOf(1, 2, 3)  // Still works!
val new = [1, 2, 3]         // More concise
```

No need to migrate existing code - use whichever style you prefer!

## Examples

### Simple List

```whitehall
val colors = ["red", "green", "blue"]

<Column>
  @for (color in colors) {
    <Text text={color} />
  }
</Column>
```

### Mutable List with State

```whitehall
var todos = ["Buy milk", "Write code"]

fun addTodo(text: String) {
  todos = todos + text
}

<Column>
  @for (todo in todos) {
    <Text text={todo} />
  }
</Column>
```

### Nested Data Structures

```whitehall
val grid = [
  [1, 2, 3],
  [4, 5, 6],
  [7, 8, 9]
]

<Column>
  @for (row in grid) {
    <Row>
      @for (cell in row) {
        <Text text={cell.toString()} />
      }
    </Row>
  }
</Column>
```

## See Also

- [Variables](./CODE-SEMANTICS.md#variables)
- [State Management](./CODE-SEMANTICS.md#state-management)
- [Collections in Kotlin](https://kotlinlang.org/docs/collections-overview.html)
