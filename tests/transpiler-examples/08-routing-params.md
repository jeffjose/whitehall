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

```kotlin
package com.example.app.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import com.example.app.lib.api.ApiClient
import com.example.app.models.User
import kotlinx.coroutines.launch

@Composable
fun ProfileScreen(
    navController: NavController,
    id: String
) {
    var user by remember { mutableStateOf<User?>(null) }
    var isLoading by remember { mutableStateOf(true) }

    val coroutineScope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        coroutineScope.launch {
              val result = ApiClient.getUser(id)
              user = result.getOrNull()
              isLoading = false
            }
    }

    fun handlePostClick(postId: String) {
        navController.navigate(Routes.Post.Detail(id = postId))
    }

    Scaffold {
        if (isLoading) {
            LoadingSpinner()
        } else if (user != null) {
            Column(
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                Text(
                    text = "${user!!.name}",
                    fontSize = 24.sp
                )
                Text(
                    text = "${user!!.email}",
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
```
