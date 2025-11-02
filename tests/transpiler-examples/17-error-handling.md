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

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.app.lib.api.ApiClient
import com.example.app.models.User
import kotlinx.coroutines.launch

@Composable
fun UserProfile() {
    var user by remember { mutableStateOf<User?>(null) }
    var isLoading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }

    val coroutineScope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        coroutineScope.launch {
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

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        if (isLoading) {
            LoadingSpinner()
        } else if (error != null) {
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
                        text = error!!,
                        color = MaterialTheme.colorScheme.error
                    )
                }
            }
        } else if (user != null) {
            Card {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = user!!.name,
                        fontSize = 20.sp
                    )
                    Text(
                        text = user!!.email,
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
```
