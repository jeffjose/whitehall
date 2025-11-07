# Private Class Fields

Tests support for private val/var fields at class level with initialization.

## Input

```whitehall
@store
class DataStore {
  var items: List<String> = []
  var isLoading = false

  private val apiKey = "secret-key-123"
  private var retryCount = 0

  suspend fun loadData() {
    isLoading = true
    retryCount++
    // Use apiKey here
  }
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

class DataStore : ViewModel() {
    data class UiState(
        val items: List<String> = [],
        val isLoading: Boolean = false
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    private val apiKey = "secret-key-123"
    private var retryCount = 0

    var items: List<String>
        get() = _uiState.value.items
        set(value) { _uiState.update { it.copy(items = value) } }

    var isLoading: Boolean
        get() = _uiState.value.isLoading
        set(value) { _uiState.update { it.copy(isLoading = value) } }

    fun loadData() {
        viewModelScope.launch {
            isLoading = true
            retryCount++
            // Use apiKey here
        }
    }

}
```

## Metadata

```
file: DataStore.wh
package: com.example.app
```
