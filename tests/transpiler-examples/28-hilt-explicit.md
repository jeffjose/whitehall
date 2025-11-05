# Hilt Store with Explicit @hilt Annotation

Tests @store with explicit `@hilt` annotation (alternative to @Inject).

## Input

```whitehall
@store
@hilt
class AppSettings @Inject constructor(
  private val prefs: Preferences
) {
  var theme = "light"
  var notifications = true
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
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject

@HiltViewModel
class AppSettings @Inject constructor(
    private val prefs: Preferences
) : ViewModel() {
    data class UiState(
        val theme: String = "light",
        val notifications: Boolean = true
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var theme: String
        get() = _uiState.value.theme
        set(value) { _uiState.update { it.copy(theme = value) } }

    var notifications: Boolean
        get() = _uiState.value.notifications
        set(value) { _uiState.update { it.copy(notifications = value) } }

}
```

## Metadata

```
file: AppSettings.wh
package: com.example.app
```
