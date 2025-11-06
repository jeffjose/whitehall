# Error Handling in Async Operations

Tests try/catch error handling patterns with loading, success, and error states.

## Input

```whitehall
import $lib.api.ApiClient
import $models.User

  var user: User? = null
  var isLoading = true
  var error: String? = null

  onMount {
    launch {
      try {
        isLoading = true
        error = null
        val result = ApiClient.getUser("123")
        user = result.getOrThrow()
      } catch (e: Exception) {
        error = e.message ?: "Unknown error"
      } finally {
        isLoading = false
      }
    }
  }

<Column spacing={16}>
  @if (isLoading) {
    <LoadingSpinner />
  } else if (error != null) {
    <Card backgroundColor="errorContainer">
      <Column padding={16}>
        <Text color="error" fontWeight="bold">Error</Text>
        <Text color="error">{error}</Text>
      </Column>
    </Card>
  } else if (user != null) {
    <Card>
      <Column padding={16}>
        <Text fontSize={20}>{user.name}</Text>
        <Text color="secondary">{user.email}</Text>
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
import androidx.compose.ui.unit.dp
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.Modifier
import androidx.compose.foundation.layout.padding
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import com.example.app.lib.api.ApiClient
import com.example.app.models.User
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Card
import androidx.compose.material3.Text
import androidx.lifecycle.viewmodel.compose.viewModel

@Composable
fun UserProfile() {
    val viewModel = viewModel<UserProfileViewModel>()
    val uiState by viewModel.uiState.collectAsState()

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        if (uiState.isLoading) {
            LoadingSpinner()
        } else if (uiState.error != null) {
            Card(
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.errorContainer
                )
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = "Error",
                        color = MaterialTheme.colorScheme.error,
                        fontWeight = FontWeight.Bold
                    )
                    Text(
                        text = "${uiState.error}",
                        color = MaterialTheme.colorScheme.error
                    )
                }
            }
        } else if (uiState.user != null) {
            Card {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = "${uiState.user.name}",
                        fontSize = 20.sp
                    )
                    Text(
                        text = "${uiState.user.email}",
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
file: UserProfile.wh
package: com.example.app.components
multi_file: true
```
