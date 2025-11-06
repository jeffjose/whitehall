# Component Inline Vars - Suspend Functions (Phase 1.1)

Tests automatic ViewModel generation with suspend functions. Suspend functions in component inline vars should be auto-wrapped in `viewModelScope.launch`.

**Note:** This generates TWO files - primary output shown below is the wrapper component.

## Input

```whitehall
import $lib.api.ApiClient
import $models.User

var user: User? = null
var isLoading = false
var errorMessage: String? = null

suspend fun loadUser(userId: String) {
  isLoading = true
  errorMessage = null

  ApiClient.getUser(userId)
    .onSuccess {
      user = it
      isLoading = false
    }
    .onFailure { error ->
      errorMessage = error.message
      isLoading = false
    }
}

fun clearError() {
  errorMessage = null
}

<Column spacing={16}>
  @if (isLoading) {
    <LoadingSpinner />
  } else if (errorMessage != null) {
    <Column spacing={8}>
      <Text text="Error: {errorMessage}" color="#FF0000" />
      <Button onClick={() => clearError()} text="Dismiss" />
    </Column>
  } else if (user != null) {
    <Card>
      <Text text={user.name} fontSize={20} />
      <Text text={user.email} color="#666" />
    </Card>
  }
</Column>
```

## Output

**File 1 (Primary): UserProfile.kt (Wrapper Component)**

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.app.lib.api.ApiClient
import com.example.app.models.User

@Composable
fun UserProfile() {
    val viewModel = viewModel<UserProfileViewModel>()
    val uiState by viewModel.uiState.collectAsState()

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        if (uiState.isLoading) {
            LoadingSpinner()
        } else if (uiState.errorMessage != null) {
            Column(
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Text(
                    text = "Error: ${uiState.errorMessage}",
                    color = Color(0xFFFF0000)
                )
                Button(onClick = { viewModel.clearError() }) {
                    Text("Dismiss")
                }
            }
        } else if (uiState.user != null) {
            Card {
                Text(
                    text = "${uiState.user!!.name}",
                    fontSize = 20.sp
                )
                Text(
                    text = "${uiState.user!!.email}",
                    color = Color(0xFF666666)
                )
            }
        }
    }
}
```

**File 2 (Additional): UserProfileViewModel.kt**

```kotlin
package com.example.app.components

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import com.example.app.lib.api.ApiClient
import com.example.app.models.User

class UserProfileViewModel : ViewModel() {
    data class UiState(
        val user: User? = null,
        val isLoading: Boolean = false,
        val errorMessage: String? = null
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var user: User?
        get() = _uiState.value.user
        set(value) { _uiState.update { it.copy(user = value) } }

    var isLoading: Boolean
        get() = _uiState.value.isLoading
        set(value) { _uiState.update { it.copy(isLoading = value) } }

    var errorMessage: String?
        get() = _uiState.value.errorMessage
        set(value) { _uiState.update { it.copy(errorMessage = value) } }

    fun loadUser(userId: String) {
        viewModelScope.launch {
            isLoading = true
            errorMessage = null

            ApiClient.getUser(userId)
              .onSuccess {
                user = it
                isLoading = false
              }
              .onFailure { error ->
                errorMessage = error.message
                isLoading = false
              }
        }
    }

    fun clearError() {
        errorMessage = null
    }
}
```

## Metadata

```
file: UserProfile.wh
package: com.example.app.components
type_hint: component
multi_file: true
```
