# Decision 004: Control Flow in Markup

**Status:** ✅ Decided
**Date:** 2025-11-01
**Decider:** User preference

## Context

Need syntax for common control flow patterns in UI markup:
- Conditional rendering (if/else)
- List rendering (loops)
- Switch/when expressions

Requirements:
- Kotlin-native (not TypeScript-flavored)
- Minimal line noise
- Clear and readable
- Maps cleanly to Compose

---

## Part 1: Conditional Rendering

### Option A: Svelte-style Blocks

```whitehall
{#if isLoading}
  <Spinner />
{:else if hasError}
  <ErrorView message={errorMessage} />
{:else}
  <Content data={data} />
{/if}
```

**Pros:**
- Very clear block structure
- Explicit start/end markers
- Nested conditions easy to see
- Familiar to Svelte devs

**Cons:**
- More verbose
- `{#if}` and `{:else}` syntax is unusual
- Not Kotlin-like

---

### Option B: Kotlin-style When Expression

```whitehall
{when {
  isLoading -> <Spinner />
  hasError -> <ErrorView message={errorMessage} />
  else -> <Content data={data} />
}}
```

**Pros:**
- Direct Kotlin syntax
- Very Kotlin-native
- Powerful pattern matching

**Cons:**
- Nested braces look noisy: `{when {`
- Doesn't feel like markup

---

### Option C: Inline If Expression

```whitehall
{if (isLoading) <Spinner /> else <Content />}

<!-- Multi-line -->
{if (isLoading)
  <Spinner />
else if (hasError)
  <ErrorView message={errorMessage} />
else
  <Content data={data} />
}
```

**Pros:**
- Kotlin if syntax
- Concise for simple cases
- Familiar

**Cons:**
- Can get long and nested
- Multiple components need wrapping

---

### Option D: Attribute Directives (Vue-style)

```whitehall
<Spinner if={isLoading} />
<ErrorView if={hasError} message={errorMessage} />
<Content if={!isLoading && !hasError} data={data} />
```

**Pros:**
- Very concise
- No extra syntax
- Clean for simple cases

**Cons:**
- No else clause (need inverse conditions)
- Hidden logic in attributes
- Multiple elements need repeated logic

---

### Option E: Hybrid (Svelte blocks with Kotlin keywords)

```whitehall
@if (isLoading) {
  <Spinner />
} else if (hasError) {
  <ErrorView message={errorMessage} />
} else {
  <Content data={data} />
}
```

**Pros:**
- Uses Kotlin `if/else` keywords
- Block structure is clear
- `@` prefix distinguishes from regular tags
- Familiar to Kotlin devs

**Cons:**
- `@` prefix might feel odd

---

## Part 2: List Rendering

### Option A: Svelte-style Each

```whitehall
{#each users as user}
  <UserCard user={user} />
{/each}

<!-- With index -->
{#each users as user, index}
  <UserCard user={user} number={index + 1} />
{/each}

<!-- With key (for optimization) -->
{#each users as user (user.id)}
  <UserCard user={user} />
{/each}
```

**Pros:**
- Clear iteration syntax
- Built-in index support
- Key support for optimization

**Cons:**
- Not Kotlin-like
- Special syntax to learn

---

### Option B: Kotlin For Loop

```whitehall
{for (user in users) {
  <UserCard user={user} />
}}

<!-- With index -->
{users.forEachIndexed { index, user ->
  <UserCard user={user} number={index + 1} />
}}
```

**Pros:**
- Pure Kotlin syntax
- Familiar to Kotlin devs
- Standard library methods work

**Cons:**
- Nested braces: `{for (...) {`
- No built-in key support

---

### Option C: Attribute Directive

```whitehall
<UserCard for={user in users} user={user} />

<!-- With key -->
<UserCard for={user in users} key={user.id} user={user} />
```

**Pros:**
- Very concise
- One-liner for simple cases

**Cons:**
- Limited to single element
- Mixing iteration logic with props

---

### Option D: Hybrid (@for with Kotlin syntax)

```whitehall
@for (user in users) {
  <UserCard user={user} />
}

<!-- With index -->
@for ((index, user) in users.withIndex()) {
  <UserCard user={user} number={index + 1} />
}

<!-- With key hint -->
@for (user in users) key(user.id) {
  <UserCard user={user} />
}
```

**Pros:**
- Kotlin `for` syntax
- `@` prefix for control flow
- Key support via function call

**Cons:**
- `@` prefix again

---

## Part 3: When/Switch Expressions

### Option A: Kotlin When

```whitehall
{when (state) {
  State.Loading -> <Spinner />
  State.Error -> <ErrorView />
  State.Success -> <Content />
}}
```

**Pros:**
- Direct Kotlin syntax
- Type-safe pattern matching
- Exhaustive checking

**Cons:**
- Nested braces

---

### Option B: Attribute-based

```whitehall
<Spinner when={state == State.Loading} />
<ErrorView when={state == State.Error} />
<Content when={state == State.Success} />
```

**Cons:**
- Repetitive
- No exhaustiveness checking

---

### Option C: Hybrid (@when)

```whitehall
@when (state) {
  State.Loading -> <Spinner />
  State.Error -> <ErrorView />
  State.Success -> <Content />
}
```

**Pros:**
- Kotlin when syntax
- Consistent with `@if`, `@for`

---

## Complete Comparison: Same Example in Each Style

### Svelte-style:
```whitehall
{#if isLoading}
  <Spinner />
{:else}
  {#if users.isEmpty()}
    <Text>No users found</Text>
  {:else}
    {#each users as user}
      <UserCard user={user} />
    {/each}
  {/if}
{/if}
```

### Kotlin-style (inline):
```whitehall
{if (isLoading)
  <Spinner />
else if (users.isEmpty())
  <Text>No users found</Text>
else
  for (user in users) {
    <UserCard user={user} />
  }
}
```

### Hybrid (@-prefix):
```whitehall
@if (isLoading) {
  <Spinner />
} else if (users.isEmpty()) {
  <Text>No users found</Text>
} else {
  @for (user in users) {
    <UserCard user={user} />
  }
}
```

### Directives:
```whitehall
<Spinner if={isLoading} />
<Text if={!isLoading && users.isEmpty()}>No users found</Text>
<UserCard for={user in users} if={!isLoading && users.isNotEmpty()} user={user} />
```

---

## Recommended Approach: Hybrid (@-prefix)

**Syntax:**
- `@if (condition) { } else { }`
- `@for (item in items) { }`
- `@when (value) { }`

**Why:**
1. **Kotlin-native**: Uses actual Kotlin keywords (if, for, when)
2. **Clear distinction**: `@` prefix marks control flow vs components
3. **Familiar**: Kotlin devs know this syntax
4. **Consistent**: Same pattern for all control flow
5. **Readable**: Block structure is clear

**Examples:**

```whitehall
<!-- Conditional -->
@if (user != null) {
  <Text>Welcome, {user.name}</Text>
} else {
  <LoginButton />
}

<!-- Iteration -->
@for (todo in todos) {
  <TodoItem todo={todo} />
}

<!-- When expression -->
@when (status) {
  Status.Loading -> <Spinner />
  Status.Error -> <ErrorView />
  Status.Success -> <SuccessView />
}
```

---

## Edge Cases

### Empty Lists
```whitehall
@if (todos.isEmpty()) {
  <Text>No todos yet</Text>
} else {
  @for (todo in todos) {
    <TodoItem todo={todo} />
  }
}
```

### Nested Conditions
```whitehall
@if (isLoggedIn) {
  @if (user.isAdmin) {
    <AdminPanel />
  } else {
    <UserPanel />
  }
} else {
  <LoginScreen />
}
```

### Complex Iteration
```whitehall
@for ((index, item) in items.withIndex()) {
  <ListItem
    item={item}
    number={index + 1}
    isLast={index == items.size - 1}
  />
}
```

### Optional Rendering (Nullable)
```whitehall
@if (errorMessage != null) {
  <ErrorBanner message={errorMessage} />
}

<!-- Or Kotlin's safe call -->
{errorMessage?.let { msg ->
  <ErrorBanner message={msg} />
}}
```

---

## FINAL DECISION

### Syntax Summary

**Conditional Rendering:**
```whitehall
@if (condition) {
  <ComponentA />
} else if (otherCondition) {
  <ComponentB />
} else {
  <ComponentC />
}

// Short-circuit (supported)
{isLoading && <Spinner />}
{errorMessage && <ErrorBanner message={errorMessage} />}
```

**When Expressions:**
```whitehall
@when (state) {
  State.Loading -> <Spinner />
  State.Error -> <ErrorView />
  State.Success -> <Content />
}

// Pattern matching (supported)
@when (result) {
  is Success -> <SuccessView data={result.data} />
  is Error -> <ErrorView error={result.error} />
}
```

**List Iteration:**
```whitehall
// Simple iteration
@for (user in users) {
  <UserCard user={user} />
}

// With key (lambda syntax - maps to Compose)
@for (user in users, key = { it.id }) {
  <UserCard user={user} />
}

// With empty state
@for (todo in todos, key = { it.id }) {
  <TodoItem todo={todo} />
} empty {
  <EmptyState message="No todos yet" />
}

// With index
@for ((index, item) in items.withIndex(), key = { it.id }) {
  <ListItem item={item} number={index + 1} />
}
```

### Key Design Decisions

1. **`@` prefix for control flow** - Clear visual distinction from components
2. **Kotlin keywords** - `if`, `for`, `when` (Kotlin-native)
3. **Short-circuit operators** - `{condition && <Component />}` (ergonomic)
4. **Lambda key syntax** - `key = { it.id }` (matches Compose exactly)
5. **Empty block** - `} empty {` (syntactic sugar for common pattern)

### Transpilation Examples

**Input:**
```whitehall
@for (todo in todos, key = { it.id }) {
  <TodoItem todo={todo} />
} empty {
  <Text>No todos yet</Text>
}
```

**Output:**
```kotlin
if (todos.isEmpty()) {
  Text("No todos yet")
} else {
  LazyColumn {
    items(todos, key = { it.id }) { todo ->
      TodoItem(todo)
    }
  }
}
```

**Input:**
```whitehall
{isLoading && <Spinner />}

@when (status) {
  is Success -> <Content data={status.data} />
  is Error -> <ErrorView error={status.error} />
}
```

**Output:**
```kotlin
if (isLoading) {
  Spinner()
}

when (status) {
  is Success -> Content(data = status.data)
  is Error -> ErrorView(error = status.error)
}
```

### Validation Rules

Compiler must enforce:
1. ✅ `@for` with `key` must use lambda syntax: `key = { it.property }`
2. ✅ `empty` block only valid after `@for`
3. ✅ Short-circuit `&&` only works with boolean left side
4. ✅ `@when` must be exhaustive (or have `else`)
5. ✅ Pattern matching in `@when` works like Kotlin (is, equality, etc.)

### Full Real-World Example

**TodoList.wh:**
```whitehall
import com.example.models.Todo

<script>
  @prop val todos: List<Todo>
  @prop val onToggle: (Todo) -> Unit
  @prop val onDelete: (Todo) -> Unit

  var filter = FilterType.ALL
  var searchQuery = ""

  val filteredTodos = todos.filter { todo ->
    val matchesSearch = todo.title.contains(searchQuery, ignoreCase = true)
    val matchesFilter = when (filter) {
      FilterType.ACTIVE -> !todo.completed
      FilterType.COMPLETED -> todo.completed
      FilterType.ALL -> true
    }
    matchesSearch && matchesFilter
  }
</script>

<Column padding={16} spacing={16}>
  <Text fontSize={24} fontWeight="bold">My Todos</Text>

  <!-- Search -->
  <Input
    bind:value={searchQuery}
    placeholder="Search todos..."
    leadingIcon="search"
  />

  <!-- Filter buttons -->
  <Row spacing={8}>
    <FilterButton
      text="All"
      active={filter == FilterType.ALL}
      onClick={() => filter = FilterType.ALL}
    />
    <FilterButton
      text="Active"
      active={filter == FilterType.ACTIVE}
      onClick={() => filter = FilterType.ACTIVE}
    />
    <FilterButton
      text="Done"
      active={filter == FilterType.COMPLETED}
      onClick={() => filter = FilterType.COMPLETED}
    />
  </Row>

  <!-- Loading state -->
  {isLoading && <LoadingSpinner />}

  <!-- Todo list -->
  @if (!isLoading) {
    @for (todo in filteredTodos, key = { it.id }) {
      <TodoItem
        todo={todo}
        onToggle={() => onToggle(todo)}
        onDelete={() => onDelete(todo)}
      />
    } empty {
      <Card padding={24}>
        <Column center spacing={8}>
          <Icon name="check_circle" size={48} color="secondary" />
          <Text color="secondary" textAlign="center">
            @when (filter) {
              FilterType.ACTIVE -> "No active todos"
              FilterType.COMPLETED -> "No completed todos"
              FilterType.ALL -> "No todos yet. Create one to get started!"
            }
          </Text>
        </Column>
      </Card>
    }
  }

  <!-- Summary -->
  @if (todos.isNotEmpty()) {
    <Divider />
    <Text color="secondary" fontSize={14}>
      {todos.count { !it.completed }} active, {todos.count { it.completed }} completed
    </Text>
  }
</Column>
```

---

## Rationale

**Why `@` prefix:**
- Clear visual marker (control flow vs components)
- Kotlin annotations use `@`, so familiar pattern
- Doesn't conflict with HTML/XML tags
- Easy to parse

**Why lambda key syntax:**
- Exact match to Jetpack Compose's `items(list, key = { it.id })`
- Developers can copy examples from Compose docs
- No new syntax to learn
- Type-safe

**Why empty block:**
- Extremely common pattern (empty state)
- Reads naturally: "for each todo... or if empty..."
- Prevents repetitive if/else boilerplate
- Inspired by Svelte's `{:else}` in each blocks

**Why short-circuit:**
- Common in React/JS ecosystems
- Very concise for "show spinner if loading"
- Kotlin supports `&&` operator
- Easy to transpile to simple `if`

---

## Open Questions for Future

1. **Should we support `@else` as standalone?**
   ```whitehall
   @if (condition) {
     <A />
   }
   @else {  <!-- standalone, not chained -->
     <B />
   }
   ```

2. **Ternary in expressions?**
   ```whitehall
   <Text color={isError ? "red" : "green"}>
   ```
   Or require `if` expression?

3. **@for with destructuring?**
   ```whitehall
   @for ((key, value) in map.entries, key = { it.key }) {
     <MapItem key={key} value={value} />
   }
   ```
