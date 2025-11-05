# Hilt Store with Auto-Detection

Tests @store with Hilt dependency injection. Hybrid auto-detection: Hilt is enabled when EITHER `@hilt` OR `@inject`/`@Inject` is present.

## Input

```whitehall
// Store with @Inject - auto-adds @HiltViewModel
@store
class UserProfile @Inject constructor(
  private val repository: ProfileRepository,
  private val analytics: Analytics
) {
  var name = ""
  var email = ""

  suspend fun save() {
    repository.save(name, email)
    analytics.track("profile_saved")
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
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject

@HiltViewModel
class UserProfile @Inject constructor(
    private val repository: ProfileRepository,
  private val analytics: Analytics
) : ViewModel() {
    data class UiState(
        val name: String = "",
        val email: String = ""
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var name: String
        get() = _uiState.value.name
        set(value) { _uiState.update { it.copy(name = value) } }

    var email: String
        get() = _uiState.value.email
        set(value) { _uiState.update { it.copy(email = value) } }

    fun save() {
        viewModelScope.launch {
            repository.save(name, email)
            analytics.track("profile_saved")
        }
    }

}
```

## Metadata

```
file: HiltStore.wh
package: com.example.app
```
