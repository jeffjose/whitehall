# Lifecycle Hooks: onMount

Tests onMount lifecycle hook transpilation to LaunchedEffect. Component has lifecycle hook, which triggers ViewModel generation.

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

**Primary File:** Test framework only validates primary output (wrapper component).

```kotlin
package com.example.app.components

import androidx.compose.runtime.key
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import com.example.app.lib.api.ApiClient
import com.example.app.models.Post
import androidx.compose.foundation.layout.Column
import androidx.lifecycle.viewmodel.compose.viewModel

@Composable
fun FeedView() {
    val viewModel = viewModel<FeedViewViewModel>()
    val uiState by viewModel.uiState.collectAsState()

    Column {
        if (uiState.isLoading) {
            LoadingSpinner()
        } else {
            uiState.posts.forEach { post ->
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
type_hint: component
multi_file: true
```
