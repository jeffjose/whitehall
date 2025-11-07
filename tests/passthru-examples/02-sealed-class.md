# Pass-Through: Sealed Class Hierarchy

Tests that sealed classes defined after the main class are passed through unchanged.

## Input

```whitehall
class TodoStore {
    var todos: List<Todo> = []
    var loadState: LoadState = LoadState.Idle

    suspend fun loadTodos() {
        loadState = LoadState.Loading
        // Load todos
        loadState = LoadState.Success(todos)
    }
}

data class Todo(
    val id: Int,
    val title: String,
    val completed: Boolean
)

sealed class LoadState {
    object Idle : LoadState()
    object Loading : LoadState()
    data class Success(val data: List<Todo>) : LoadState()
    data class Error(val message: String) : LoadState()
}
```

## Output

```kotlin
package com.example.app

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

class TodoStore : ViewModel() {
    data class UiState(
        val todos: List<Todo> = [],
        val loadState: LoadState = LoadState.Idle
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var todos: List<Todo>
        get() = _uiState.value.todos
        set(value) { _uiState.update { it.copy(todos = value) } }

    var loadState: LoadState
        get() = _uiState.value.loadState
        set(value) { _uiState.update { it.copy(loadState = value) } }

    fun loadTodos() {
        viewModelScope.launch {
            loadState = LoadState.Loading
            // Load todos
            loadState = LoadState.Success(todos)
        }
    }

}

data class Todo(
    val id: Int,
    val title: String,
    val completed: Boolean
)

sealed class LoadState {
    object Idle : LoadState()
    object Loading : LoadState()
    data class Success(val data: List<Todo>) : LoadState()
    data class Error(val message: String) : LoadState()
}
```

## Metadata

```
file: TodoStore.wh
package: com.example.app
```

## Status

🔴 **Currently:** Errors with "Expected component, found: data class"
🟢 **After Pass-Through:** Should pass through unchanged
