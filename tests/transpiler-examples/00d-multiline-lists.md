# Multiline List Declarations

Tests multiline list literal syntax with proper formatting.

## Input

```whitehall
val stringList = listOf(
  "Apple",
  "Banana",
  "Cherry"
)

val numberList = listOf(
  1,
  2,
  3
)

var mutableList = listOf(
  "A",
  "B"
)

<Text>Done</Text>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.material3.Text
import androidx.compose.runtime.*

@Composable
fun MultilineLists() {
    var mutableList by remember { mutableStateOf(listOf(
  "A",
  "B"
)) }

    val stringList = listOf(
  "Apple",
  "Banana",
  "Cherry"
)
    val numberList = listOf(
  1,
  2,
  3
)

    key(Unit) {
        Text(text = "Done")
    }
}
```

## Metadata

```
file: MultilineLists.wh
package: com.example.app.components
```
