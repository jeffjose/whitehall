# Store without Hilt (Regular ViewModel)

Tests @store without Hilt - generates regular ViewModel with viewModel<T>().

## Input

```whitehall
@store
class Counter(initialValue: Int) {
  var count = initialValue

  fun increment() {
    count = count + 1
  }

  fun reset() {
    count = 0
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

class Counter constructor(
    initialValue: Int
) : ViewModel() {
    data class UiState(
        val count: String = initialValue
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var count: String
        get() = _uiState.value.count
        set(value) { _uiState.update { it.copy(count = value) } }

    fun increment() {
        count = count + 1
    }

    fun reset() {
        count = 0
    }

}
```

## Metadata

```
file: Counter.wh
package: com.example.app
```
