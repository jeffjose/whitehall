# Range Literals

Tests Kotlin range syntax support in variable assignments and @for loops.

## Input

```whitehall
// Basic ranges (Kotlin-native syntax)
val simple = 1..10
val stepped = (0..100 step 2)
val countdown = (10 downTo 1)

// In @for loops
<Column spacing={8}>
  @for (i in 1..5) {
    <Text>Item {i}</Text>
  }

  @for (n in 0..10 step 2) {
    <Text>Even: {n}</Text>
  }
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.unit.dp

@Composable
fun RangeDemo() {
    val simple = 1..10
    val stepped = (0..100 step 2)
    val countdown = (10 downTo 1)

    Column(
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        1..5.forEach { i ->
            Text(text = "Item ${i}")
        }
        0..10 step 2.forEach { n ->
            Text(text = "Even: ${n}")
        }
    }
}
```

## Metadata

```
file: RangeDemo.wh
package: com.example.app.components
```
