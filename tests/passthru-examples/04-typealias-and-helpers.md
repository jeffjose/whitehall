# Pass-Through: Type Alias and Top-Level Functions

Tests that type aliases and top-level helper functions are passed through unchanged.

## Input

```whitehall
class UserStore {
    var users: UserList = emptyList()
    var currentUserId: UserId? = null

    fun selectUser(id: UserId) {
        currentUserId = id
    }

    fun findUser(id: UserId): User? {
        return users.find { it.id == id }
    }
}

typealias UserId = Int
typealias UserList = List<User>

data class User(
    val id: UserId,
    val name: String,
    val email: String
)

fun User.displayName(): String = "$name <$email>"

fun createDefaultUser(): User = User(
    id = 0,
    name = "Guest",
    email = "guest@example.com"
)
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

class UserStore : ViewModel() {
    data class UiState(
        val users: UserList = emptyList(),
        val currentUserId: UserId? = null
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var users: UserList
        get() = _uiState.value.users
        set(value) { _uiState.update { it.copy(users = value) } }

    var currentUserId: UserId?
        get() = _uiState.value.currentUserId
        set(value) { _uiState.update { it.copy(currentUserId = value) } }

    fun selectUser(id: UserId) {
        currentUserId = id
    }

    fun findUser(id: UserId): User? {
        return users.find { it.id == id }
    }

}

typealias UserId = Int
typealias UserList = List<User>

data class User(
    val id: UserId,
    val name: String,
    val email: String
)

fun User.displayName(): String = "$name <$email>"

fun createDefaultUser(): User = User(
    id = 0,
    name = "Guest",
    email = "guest@example.com"
)
```

## Metadata

```
file: UserStore.wh
package: com.example.app
```

## Status

🔴 **Currently:** Errors with "Expected component, found: typealias"
🟢 **After Pass-Through:** Should pass through unchanged
