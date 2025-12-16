# OnAppear Prop

Tests onAppear prop transpilation to LaunchedEffect(Unit). This is useful for triggering actions when a component becomes visible (e.g., infinite scroll pagination).

## Input

```whitehall
var items: List<String> = []
var page = 1

fun loadMore() {
  page = page + 1
}

<Column>
  @for (item in items, key = {it}) {
    <Text>{item}</Text>
  }
  <Box onAppear={loadMore}>
    <Text>Loading more...</Text>
  </Box>
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.*

@Composable
fun InfiniteList() {
    var items by remember { mutableStateOf<List<String>>(mutableListOf()) }
    var page by remember { mutableStateOf(1) }

    fun loadMore() {
        page = page + 1
    }

    Column {
        items.forEach { item ->
            key(item) {
                Text(text = "${item}")
            }
        }
        Box {
            LaunchedEffect(Unit) {
                loadMore
            }
            Text(text = "Loading more...")
        }
    }
}
```

## Metadata

```
file: InfiniteList.wh
package: com.example.app.components
```
