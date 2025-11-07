# Pass-Through: Mixed Kotlin Constructs

Tests that multiple Kotlin constructs (data classes, sealed classes, enums, type aliases, objects) are all passed through unchanged.

## Input

```whitehall
import kotlinx.serialization.Serializable

class ApiStore {
    var result: ApiResult<User> = ApiResult.Loading
    var config: ApiConfig = ApiConfig

    suspend fun fetchUser(id: UserId) {
        result = ApiResult.Loading
        // Fetch logic here
    }
}

typealias UserId = String

@Serializable
data class User(
    val id: UserId,
    val name: String,
    val role: UserRole
)

enum class UserRole {
    ADMIN,
    USER,
    GUEST
}

sealed class ApiResult<T> {
    object Loading : ApiResult<Nothing>()
    data class Success<T>(val data: T) : ApiResult<T>()
    data class Error(val message: String) : ApiResult<Nothing>()
}

object ApiConfig {
    const val BASE_URL = "https://api.example.com"
    const val TIMEOUT_MS = 5000L
}

fun <T> ApiResult<T>.isSuccess(): Boolean = this is ApiResult.Success

fun User.isAdmin(): Boolean = role == UserRole.ADMIN
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
import kotlinx.serialization.Serializable

class ApiStore : ViewModel() {
    data class UiState(
        val result: ApiResult<User> = ApiResult.Loading,
        val config: ApiConfig = ApiConfig
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var result: ApiResult<User>
        get() = _uiState.value.result
        set(value) { _uiState.update { it.copy(result = value) } }

    var config: ApiConfig
        get() = _uiState.value.config
        set(value) { _uiState.update { it.copy(config = value) } }

    fun fetchUser(id: UserId) {
        viewModelScope.launch {
            result = ApiResult.Loading
            // Fetch logic here
        }
    }

}

typealias UserId = String

@Serializable
data class User(
    val id: UserId,
    val name: String,
    val role: UserRole
)

enum class UserRole {
    ADMIN,
    USER,
    GUEST
}

sealed class ApiResult<T> {
    object Loading : ApiResult<Nothing>()
    data class Success<T>(val data: T) : ApiResult<T>()
    data class Error(val message: String) : ApiResult<Nothing>()
}

object ApiConfig {
    const val BASE_URL = "https://api.example.com"
    const val TIMEOUT_MS = 5000L
}

fun <T> ApiResult<T>.isSuccess(): Boolean = this is ApiResult.Success

fun User.isAdmin(): Boolean = role == UserRole.ADMIN
```

## Metadata

```
file: ApiStore.wh
package: com.example.app
```

## Status

🔴 **Currently:** Errors with "Expected component, found: typealias"
🟢 **After Pass-Through:** Should pass through unchanged
