# Dynamic List → No Optimization (Stay with Compose)

**Scenario:** User has a mutable list with event handlers that modify state

**Why NOT optimize:** List changes over time and has interactive state. RecyclerView would require manual state management and notifyDataSetChanged(). Compose handles this elegantly.

**Detection criteria (fails optimization):**
- ❌ Collection is `var` (mutable) OR has `mutableStateOf`
- ❌ Mutated in lifecycle hooks or functions
- ❌ Has event handlers that modify list or related state
- ❌ Interactive behavior (selections, toggles, etc.)

**Decision:** Keep Compose LazyColumn (correct choice!)

---

## Input

```whitehall
var todos by remember { mutableStateOf(emptyList<Todo>()) }
var selectedId by remember { mutableStateOf<String?>(null) }

onMount {
  launch {
    todos = ApiClient.getTodos()
  }
}

fun toggleTodo(id: String) {
  todos = todos.map {
    if (it.id == id) it.copy(done = !it.done) else it
  }
}

@for (todo in todos, key = { it.id }) {
  <Card
    padding={8}
    backgroundColor={todo.id == selectedId ? "primaryContainer" : "surface"}
    onClick={() => selectedId = todo.id}
  >
    <Row spacing={8}>
      <Checkbox
        bind:checked={todo.done}
        onChange={() => toggleTodo(todo.id)}
      />
      <Text
        fontSize={16}
        textDecoration={todo.done ? "line-through" : "none"}
      >
        {todo.text}
      </Text>
    </Row>
  </Card>
}
```

---

## Unoptimized Output (Current - Correct!)

**Backend:** Compose LazyColumn (no optimization applied)

```kotlin
package com.example.app.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch

@Composable
fun DynamicTodoList() {
    var todos by remember { mutableStateOf(emptyList<Todo>()) }
    var selectedId by remember { mutableStateOf<String?>(null) }

    val coroutineScope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        coroutineScope.launch {
            todos = ApiClient.getTodos()
        }
    }

    fun toggleTodo(id: String) {
        todos = todos.map {
            if (it.id == id) it.copy(done = !it.done) else it
        }
    }

    LazyColumn {
        items(todos, key = { it.id }) { todo ->
            Card(
                modifier = Modifier
                    .padding(8.dp)
                    .clickable { selectedId = todo.id },
                colors = CardDefaults.cardColors(
                    containerColor = if (todo.id == selectedId) {
                        MaterialTheme.colorScheme.primaryContainer
                    } else {
                        MaterialTheme.colorScheme.surface
                    }
                )
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    Checkbox(
                        checked = todo.done,
                        onCheckedChange = { toggleTodo(todo.id) }
                    )
                    Text(
                        text = todo.text,
                        fontSize = 16.sp,
                        textDecoration = if (todo.done) {
                            TextDecoration.LineThrough
                        } else {
                            TextDecoration.None
                        }
                    )
                }
            }
        }
    }
}
```

**Why this is correct:**
- Compose handles reactive state updates automatically
- `mutableStateOf` triggers recomposition when todos change
- Interactive selection state managed seamlessly
- Checkbox binding works out of the box
- Performance is fine for typical list sizes (<1000 items)

---

## Optimized Output (Phase 5 Target)

**Backend:** SAME - No optimization applied ✅

```kotlin
// Identical to unoptimized output above
// Whitehall analyzer detects this should NOT be optimized
```

**Why no optimization:**
- List is mutable (`var` with `mutableStateOf`)
- Modified in `onMount` lifecycle hook
- Modified in `toggleTodo` function
- Has `onClick` event handler that mutates `selectedId`
- Has `onChange` event handler that mutates `todos`
- Selection state depends on comparison with `selectedId`

**This is the RIGHT decision!** RecyclerView would be much harder to maintain here.

---

## Analyzer Decision Log (Future)

```
[ANALYZE] Checking for_loop over 'todos'
  ❌ Collection 'todos' is var with mutableStateOf: 0 confidence
  ❌ Mutated in onMount hook (todos = ApiClient.getTodos()): -50 confidence
  ❌ Mutated in toggleTodo function (todos = todos.map(...)): -50 confidence
  ❌ Has onClick handler: potential mutations
  ❌ Has onChange handler: potential mutations
  ❌ TOTAL CONFIDENCE: 0

[OPTIMIZE] Skipping optimization for 'todos'
  ❌ Confidence threshold not met (0 < 80)
  ✅ Keeping Compose LazyColumn (correct choice)

[CODEGEN] Generating standard Compose LazyColumn for 'todos'
  ✅ Using items() API with key
  ✅ Handling mutableStateOf reactivity
  ✅ Binding event handlers correctly
```

---

## Performance Comparison

### If We Wrongly Optimized to RecyclerView:

**Problems:**
```kotlin
// BAD - Would need manual state management
private var todos: List<Todo> = emptyList()
private var adapter: TodoAdapter? = null

fun updateTodos(newTodos: List<Todo>) {
    todos = newTodos
    adapter?.notifyDataSetChanged()  // Manual update!
}

fun toggleTodo(id: String) {
    val index = todos.indexOfFirst { it.id == id }
    if (index != -1) {
        todos = todos.toMutableList().apply {
            set(index, get(index).copy(done = !get(index).done))
        }
        adapter?.notifyItemChanged(index)  // Manual notification!
    }
}
```

**Issues:**
- Manual notifyDataSetChanged() calls
- Complex index tracking
- Selection state management messy
- Checkbox binding requires custom ViewHolder logic
- Much more code, harder to maintain
- Loses Compose's reactive benefits

### With Compose (Current):

**Benefits:**
- Automatic reactivity (mutableStateOf)
- Clean event handlers
- Simple state management
- Standard Compose patterns
- Easy to understand and modify

**Performance:** Good enough for typical use cases (<1000 items)

---

## Metadata

```
file: DynamicTodoList.wh
package: com.example.app.components
optimization: none
confidence: 0
reason: mutable_state_with_handlers
```
