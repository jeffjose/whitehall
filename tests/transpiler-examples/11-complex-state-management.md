# Complex State Management

Tests multiple state variables, derived state, and complex interactions.

## Input

```whitehall
import $models.User
import $lib.api.ApiClient

  var users: List<User> = emptyList()
  var selectedUserId: String? = null
  var searchQuery = ""
  var isLoading = false

  val selectedUser: User? = users.firstOrNull { it.id == selectedUserId }
  val filteredUsers: List<User> = users.filter {
    it.name.contains(searchQuery, ignoreCase = true)
  }

  fun handleSearch(query: String) {
    searchQuery = query
  }

  fun handleUserSelect(userId: String) {
    selectedUserId = userId
  }

  onMount {
    launch {
      isLoading = true
      val result = ApiClient.getUsers()
      users = result.getOrNull() ?: emptyList()
      isLoading = false
    }
  }

<Column spacing={16}>
  <TextField
    label="Search users"
    bind:value={searchQuery}
    placeholder="Enter name..."
  />

  @if (isLoading) {
    <LoadingSpinner />
  } else {
    @for (user in filteredUsers, key = { it.id }) {
      <Card
        onClick={() => handleUserSelect(user.id)}
        selected={user.id == selectedUserId}
      >
        <Text>{user.name}</Text>
      </Card>
    } empty {
      <Text color="secondary">No users found</Text>
    }
  }

  @if (selectedUser != null) {
    <Card>
      <Column spacing={8}>
        <Text fontSize={20}>Selected: {selectedUser.name}</Text>
        <Text color="secondary">{selectedUser.email}</Text>
      </Column>
    </Card>
  }
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.runtime.*
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.material3.Card
import androidx.compose.material3.TextField
import androidx.compose.material3.Text
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.app.models.User
import com.example.app.lib.api.ApiClient
import kotlinx.coroutines.launch

@Composable
fun ComplexStateManagement() {
    var users by remember { mutableStateOf<List<User>>(emptyList()) }
    var selectedUserId by remember { mutableStateOf<String?>(null) }
    var searchQuery by remember { mutableStateOf("") }
    var isLoading by remember { mutableStateOf(false) }

    val selectedUser: User? = users.firstOrNull { it.id == selectedUserId }
    val filteredUsers: List<User> = users.filter {
        it.name.contains(searchQuery, ignoreCase = true)
    }

    fun handleSearch(query: String) {
        searchQuery = query
    }

    fun handleUserSelect(userId: String) {
        selectedUserId = userId
    }

    val coroutineScope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        coroutineScope.launch {
            isLoading = true
            val result = ApiClient.getUsers()
            users = result.getOrNull() ?: emptyList()
            isLoading = false
        }
    }

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        TextField(
            label = { Text("Search users") },
            value = searchQuery,
            onValueChange = { searchQuery = it },
            placeholder = { Text("Enter name...") }
        )

        if (isLoading) {
            LoadingSpinner()
        } else {
            if (filteredUsers.isEmpty()) {
                Text(
                    text = "No users found",
                    color = MaterialTheme.colorScheme.secondary
                )
            } else {
                filteredUsers.forEach { user ->
                    key(user.id) {
                        Card(
                            onClick = { handleUserSelect(user.id) },
                            selected = user.id == selectedUserId
                        ) {
                            Text(text = user.name)
                        }
                    }
                }
            }
        }

        if (selectedUser != null) {
            Card {
                Column(
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    Text(
                        text = "Selected: ${selectedUser!!.name}",
                        fontSize = 20.sp
                    )
                    Text(
                        text = selectedUser!!.email,
                        color = MaterialTheme.colorScheme.secondary
                    )
                }
            }
        }
    }
}
```
