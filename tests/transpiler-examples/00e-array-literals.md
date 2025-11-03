# Array Literal Syntax

Tests convenient array literal syntax `[...]` that transpiles to `listOf()` or `mutableListOf()`.

## Input

```whitehall
val numbers = [1, 2, 3, 4, 5]
val strings = ["Apple", "Banana", "Cherry"]
val mixed = [1, "two", 3]
var mutableNums = [10, 20, 30]
var mutableStrings = ["A", "B"]

val nested = [[1, 2], [3, 4]]

val multiline = [
  "One",
  "Two",
  "Three"
]

<Text>Done</Text>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.material3.Text
import androidx.compose.runtime.*

@Composable
fun ArrayLiterals() {
    var mutableNums by remember { mutableStateOf(mutableListOf(10, 20, 30)) }
    var mutableStrings by remember { mutableStateOf(mutableListOf("A", "B")) }

    val numbers = listOf(1, 2, 3, 4, 5)
    val strings = listOf("Apple", "Banana", "Cherry")
    val mixed = listOf(1, "two", 3)
    val nested = listOf(listOf(1, 2), listOf(3, 4))
    val multiline = listOf("One",
  "Two",
  "Three")

    Text(text = "Done")
}
```

## Metadata

```
file: ArrayLiterals.wh
package: com.example.app.components
```
