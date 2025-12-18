# LazyColumn with items()

Tests LazyColumn for performance-optimized scrollable lists using items() function.

## Input

```whitehall
import $models.Post

  @prop val posts: List<Post>
  @prop val onPostClick: (String) -> Unit

<LazyColumn
  padding={16}
  gap={12}
>
  @for (post in posts, key = { it.id }) {
    <Card onClick={() => onPostClick(post.id)}>
      <Column padding={16}>
        <Text fontSize={18} fontWeight="bold">{post.title}</Text>
        <Text color="secondary">{post.author}</Text>
      </Column>
    </Card>
  }
</LazyColumn>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.app.models.Post

@Composable
fun PostFeed(
    posts: List<Post>,
    onPostClick: (String) -> Unit
) {
    LazyColumn(
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        items(posts, key = { it.id }) { post ->
            Card(onClick = { onPostClick(post.id) }) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = "${post.title}",
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Bold
                    )
                    Text(
                        text = "${post.author}",
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
file: PostFeed.wh
package: com.example.app.components
```
