# Control Flow: For Loop with Key

Tests @for loops with keys and empty blocks.

## Input

```whitehall
import $models.Post

  @prop val posts: List<Post>

<Column gap={16}>
  @for (post in posts, key = { it.id }) {
    <Card onClick={() => $navigate($routes.post.detail(id = post.id))}>
      <Column padding={12}>
        <Text fontSize={16} fontWeight="bold">{post.title}</Text>
        <Text color="secondary">{post.excerpt}</Text>
      </Column>
    </Card>
  } empty {
    <Text color="secondary">No posts yet</Text>
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
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.app.LocalNavController
import com.example.app.models.Post
import com.example.app.routes.Routes

@Composable
fun PostList(
    posts: List<Post>
) {
    val navController = LocalNavController.current
    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        if (posts.isEmpty()) {
            Text(
                text = "No posts yet",
                color = MaterialTheme.colorScheme.secondary
            )
        } else {
            posts.forEach { post ->
                key(post.id) {
                    Card(
                        onClick = { navController.navigate(Routes.Post.Detail(id = post.id)) { launchSingleTop = true } }
                    ) {
                        Column(modifier = Modifier.padding(12.dp)) {
                            Text(
                                text = "${post.title}",
                                fontSize = 16.sp,
                                fontWeight = FontWeight.Bold
                            )
                            Text(
                                text = "${post.excerpt}",
                                color = MaterialTheme.colorScheme.secondary
                            )
                        }
                    }
                }
            }
        }
    }
}
```

## Metadata

```
file: PostList.wh
package: com.example.app.components
```
