# Pass-Through: Complex Nested Structures (Phase 6)

Tests complex nested Kotlin structures including sealed classes with nested data classes, companion objects, and deeply nested generics.

## Input

```whitehall
class MyStore {
    var state: LoadingState<User> = LoadingState.Idle
}

sealed class LoadingState<out T> {
    object Idle : LoadingState<Nothing>()
    object Loading : LoadingState<Nothing>()
    data class Success<T>(val data: T) : LoadingState<T>()
    data class Error(val error: Throwable) : LoadingState<Nothing>()

    companion object {
        fun <T> from(result: Result<T>): LoadingState<T> = when (result) {
            is Result.Success -> Success(result.data)
            is Result.Failure -> Error(result.error)
        }
    }
}

data class User(
    val id: String,
    val name: String,
    val metadata: Map<String, Any>
) {
    companion object {
        val EMPTY = User("", "", emptyMap())
    }
}

sealed class Result<out T> {
    data class Success<T>(val data: T) : Result<T>()
    data class Failure(val error: Throwable) : Result<Nothing>()
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

class MyStore : ViewModel() {
    data class UiState(
        val state: LoadingState<User> = LoadingState.Idle
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var state: LoadingState<User>
        get() = _uiState.value.state
        set(value) { _uiState.update { it.copy(state = value) } }

}

sealed class LoadingState<out T> {
    object Idle : LoadingState<Nothing>()
    object Loading : LoadingState<Nothing>()
    data class Success<T>(val data: T) : LoadingState<T>()
    data class Error(val error: Throwable) : LoadingState<Nothing>()

    companion object {
        fun <T> from(result: Result<T>): LoadingState<T> = when (result) {
            is Result.Success -> Success(result.data)
            is Result.Failure -> Error(result.error)
        }
    }
}

data class User(
    val id: String,
    val name: String,
    val metadata: Map<String, Any>
) {
    companion object {
        val EMPTY = User("", "", emptyMap())
    }
}

sealed class Result<out T> {
    data class Success<T>(val data: T) : Result<T>()
    data class Failure(val error: Throwable) : Result<Nothing>()
}
```

## Metadata

```
file: MyStore.wh
package: com.example.app
```

## Status

🟢 **Phase 6:** Complex nested structures test
✅ Sealed classes with nested data classes and objects
✅ Companion objects with functions
✅ Generic type parameters (covariant `out T`)
✅ Deeply nested generics: `Map<String, Any>`, `LoadingState<User>`
