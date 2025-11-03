# Dynamic List - No Optimization

Tests that mutable lists with event handlers correctly stay as Compose (no optimization).

**Behavior (all phases):** Always generates Compose LazyColumn (correct!)

**Why no optimization:**
- Collection is `var` with `mutableStateOf` (mutable)
- Modified in lifecycle hooks
- Has event handlers that mutate state
- Confidence: 0/100

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
      <Text fontSize={16}>{todo.text}</Text>
    </Row>
  </Card>
}
```

## Output (Unoptimized - Correct!)

```kotlin
package com.example.app.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
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

    Column {
        todos.forEach { todo ->
            key(todo.id) {
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
                            fontSize = 16.sp
                        )
                    }
                }
            }
        }
    }
}
```

**Why this is correct:** Compose handles reactive state elegantly. RecyclerView would require manual `notifyDataSetChanged()` and complex state management.

## Metadata

```
file: DynamicTodoList.wh
package: com.example.app.components
optimization: none
confidence: 0
reason: mutable_state_with_handlers
```
