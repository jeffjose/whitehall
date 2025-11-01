# Syntax Examples

This directory contains complete, real-world examples of Whitehall components demonstrating the current syntax.

## Examples

### UserList.wh
Demonstrates:
- `@if` / `@when` for conditional rendering
- `@for` with `key` for list iteration
- `empty` block for empty states
- Short-circuit operators (`&&`)
- Nested control flow
- Loading/error states
- Props and state management

**Use case:** Fetching and displaying a filtered list of users

---

### ShoppingCart.wh
Demonstrates:
- Complex `@for` with item management
- Computed values (subtotal, tax, total)
- Conditional rendering with multiple states
- Short-circuit for conditional UI elements
- Event handlers and state updates
- Nested components with props
- Layout composition (Scaffold, TopAppBar, etc.)

**Use case:** E-commerce shopping cart with promo codes

---

## Syntax Reference

These examples showcase the **decided syntax** from:
- [Decision 003: @prop Annotation](../decisions/003-prop-annotation.md)
- [Decision 004: Control Flow](../decisions/004-control-flow.md)

### Control Flow Quick Reference

```whitehall
<!-- Conditionals -->
@if (condition) {
  <ComponentA />
} else if (other) {
  <ComponentB />
} else {
  <ComponentC />
}

<!-- When expressions -->
@when (value) {
  Value.A -> <ComponentA />
  Value.B -> <ComponentB />
  else -> <ComponentC />
}

<!-- Lists with keys -->
@for (item in items, key = { it.id }) {
  <ItemCard item={item} />
} empty {
  <EmptyState />
}

<!-- Short-circuit -->
{condition && <Component />}
{nullableValue?.property}
```

### Component Structure Quick Reference

```whitehall
import com.example.SomeClass

<script>
  @prop val propName: Type
  @prop val optionalProp: Type = defaultValue

  var stateVar = initialValue
  val computed = someExpression

  fun handler() {
    // logic
  }

  onMount {
    // lifecycle
  }
</script>

<!-- UI markup -->
<Column>
  <Text>{propName}</Text>
</Column>
```

---

## Running Examples

These are illustrative examples. Once the parser and transpiler are built:

```bash
# Single-file mode (future)
whitehall run UserList.wh

# In a project
# Add to src/components/UserList.wh
whitehall build
```
