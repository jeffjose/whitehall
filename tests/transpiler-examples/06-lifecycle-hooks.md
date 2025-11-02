# Lifecycle Hooks: onMount

Tests onMount lifecycle hook transpilation to LaunchedEffect.

## Input

```whitehall
import $lib.api.ApiClient
import $models.Post

  var posts: List<Post> = emptyList()
  var isLoading = true

  onMount {
    launch {
      ApiClient.getFeed()
        .onSuccess {
          posts = it
          isLoading = false
        }
        .onFailure {
          isLoading = false
        }
    }
  }

<Column>
  @if (isLoading) {
    <LoadingSpinner />
  } else {
    @for (post in posts, key = { it.id }) {
      <PostCard post={post} />
    }
  }
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.*
import com.example.app.lib.api.ApiClient
import com.example.app.models.Post
import kotlinx.coroutines.launch

@Composable
fun FeedView() {
    var posts by remember { mutableStateOf<List<Post>>(emptyList()) }
    var isLoading by remember { mutableStateOf(true) }

    val coroutineScope = rememberCoroutineScope()

    LaunchedEffect(Unit) {
        coroutineScope.launch {
              ApiClient.getFeed()
                .onSuccess {
                  posts = it
                  isLoading = false
                }
                .onFailure {
                  isLoading = false
                }
            }
    }

    Column {
        if (isLoading) {
            LoadingSpinner()
        } else {
            posts.forEach { post ->
                key(post.id) {
                    PostCard(post = post)
                }
            }
        }
    }
}
```

## Metadata

```
file: FeedView.wh
package: com.example.app.components
```
