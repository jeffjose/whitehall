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
  val filteredUsers: List<User> = if (searchQuery.isBlank()) {
    users
  } else {
    users.filter { it.name.contains(searchQuery, ignoreCase = true) }
  }

  fun handleSearch(query: String) {
    searchQuery = query
  }

  fun handleUserSelect(userId: String) {
    selectedUserId = userId
  }

  $onMount {
    launch {
      isLoading = true
      val result = ApiClient.getUsers()
      users = result.getOrNull() ?: emptyList()
      isLoading = false
    }
  }

<Column gap={16}>
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
      <Column gap={8}>
        <Text fontSize={20}>Selected: {selectedUser.name}</Text>
        <Text color="secondary">{selectedUser.email}</Text>
      </Column>
    </Card>
  }
</Column>
```

## Output

**Primary File:** Test framework only validates primary output (wrapper component).

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.remember
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.app.lib.api.ApiClient
import com.example.app.models.User

@Composable
fun ComplexStateManagement() {
    val viewModel = viewModel<ComplexStateManagementViewModel>()
    val uiState by viewModel.uiState.collectAsState()
    val selectedUser by remember {
        uiState.users.firstOrNull { it.id == uiState.selectedUserId }
    }
    val filteredUsers by remember {
        if (uiState.searchQuery.isBlank()) {
    uiState.users
  } else {
    uiState.users.filter { it.name.contains(uiState.searchQuery, ignoreCase = true) }
  }
    }

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        TextField(
            label = { Text("Search users") },
            value = uiState.searchQuery,
            onValueChange = { viewModel.searchQuery = it },
            placeholder = { Text("Enter name...") }
        )
        if (uiState.isLoading) {
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
                            onClick = { viewModel.handleUserSelect(user.id) },
                            selected = user.id == uiState.selectedUserId
                        ) {
                            Text(text = "${user.name}")
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
                        text = "Selected: ${selectedUser.name}",
                        fontSize = 20.sp
                    )
                    Text(
                        text = "${selectedUser.email}",
                        color = MaterialTheme.colorScheme.secondary
                    )
                }
            }
        }
    }
}
```

## Metadata

```
file: ComplexStateManagement.wh
package: com.example.app.components
multi_file: true
```
