# Routing: Route Parameters

Tests route parameter access via $screen.params and parameterized navigation.

## Input

```whitehall
import $lib.api.ApiClient
import $models.User

  var user: User? = null
  var isLoading = true

  onMount {
    launch {
      val result = ApiClient.getUser($screen.params.id)
      user = result.getOrNull()
      isLoading = false
    }
  }

  fun handlePostClick(postId: String) {
    navigate($routes.post.detail(id = postId))
  }

<Scaffold>
  @if (isLoading) {
    <LoadingSpinner />
  } else if (user != null) {
    <Column spacing={16}>
      <Text fontSize={24}>{user.name}</Text>
      <Text color="secondary">{user.email}</Text>
    </Column>
  }
</Scaffold>
```

## Output

**Primary File:** Test framework only validates primary output (wrapper component).

```kotlin
package com.example.app.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import com.example.app.lib.api.ApiClient
import com.example.app.models.User
import androidx.compose.material3.Scaffold
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.lifecycle.viewmodel.compose.viewModel

@Composable
fun ProfileScreen() {
    val viewModel = viewModel<ProfileScreenViewModel>()
    val uiState by viewModel.uiState.collectAsState()

    Scaffold {
        if (uiState.isLoading) {
            LoadingSpinner()
        } else if (uiState.user != null) {
            Column(
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                Text(
                    text = "${user.name}",
                    fontSize = 24.sp
                )
                Text(
                    text = "${user.email}",
                    color = MaterialTheme.colorScheme.secondary
                )
            }
        }
    }
}
```

## Metadata

```
file: ProfileScreen.wh
package: com.example.app.screens
type: screen
multi_file: true
```
