# Dynamic List - No Optimization

Tests that mutable lists correctly stay as Compose (no optimization).

## Input

```whitehall
var items = listOf("One", "Two", "Three")

<Column>
  @for (item in items) {
    <Text text={item} />
  }
</Column>
```

## Unoptimized Output

```kotlin
package com.example

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier

@Composable
fun DynamicList() {
    var items = listOf("One", "Two", "Three")

    Column {
        items.forEach { item ->
            Text(text = item)
        }
    }
}
```

## Optimized Output

```kotlin
package com.example

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.*

@Composable
fun DynamicList() {
    var items by remember { mutableStateOf(listOf("One", "Two", "Three")) }

    Column {
        items.forEach { item ->
            Text(text = item)
        }
    }
}
```

## Metadata

```
file: DynamicList.wh
package: com.example
```
