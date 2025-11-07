# Pass-Through: Real-World Patterns (Phase 6)

Tests real-world Kotlin patterns including extension properties, property delegates, DSL builders, and scope functions.

## Input

```whitehall
class UserStore {
    var users: List<User> = []
}

data class User(
    val id: String,
    val firstName: String,
    val lastName: String,
    val email: String
)

val User.fullName: String
    get() = "$firstName $lastName"

val User.displayEmail: String
    get() = email.lowercase()

fun User.toDisplayString(): String = "$fullName ($displayEmail)"

fun List<User>.findById(id: String): User? = find { it.id == id }

fun List<User>.sortedByName(): List<User> = sortedBy { it.fullName }

fun buildUser(block: UserBuilder.() -> Unit): User = UserBuilder().apply(block).build()

class UserBuilder {
    var id: String = ""
    var firstName: String = ""
    var lastName: String = ""
    var email: String = ""

    fun build(): User = User(id, firstName, lastName, email)
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

class UserStore : ViewModel() {
    data class UiState(
        val users: List<User> = []
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var users: List<User>
        get() = _uiState.value.users
        set(value) { _uiState.update { it.copy(users = value) } }

}

data class User(
    val id: String,
    val firstName: String,
    val lastName: String,
    val email: String
)

val User.fullName: String
    get() = "$firstName $lastName"

val User.displayEmail: String
    get() = email.lowercase()

fun User.toDisplayString(): String = "$fullName ($displayEmail)"

fun List<User>.findById(id: String): User? = find { it.id == id }

fun List<User>.sortedByName(): List<User> = sortedBy { it.fullName }

fun buildUser(block: UserBuilder.() -> Unit): User = UserBuilder().apply(block).build()

class UserBuilder {
    var id: String = ""
    var firstName: String = ""
    var lastName: String = ""
    var email: String = ""

    fun build(): User = User(id, firstName, lastName, email)
}
```

## Metadata

```
file: UserStore.wh
package: com.example.app
```

## Status

🟢 **Phase 6:** Real-world patterns test
✅ Extension properties with getters: `val User.fullName`
✅ Extension functions on types: `fun User.toDisplayString()`
✅ Extension functions on generics: `fun List<User>.findById()`
✅ DSL builder pattern: `fun buildUser(block: UserBuilder.() -> Unit)`
✅ Scope functions: `.apply(block).build()`
✅ Builder classes with mutable properties
