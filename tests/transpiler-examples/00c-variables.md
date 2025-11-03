# Variable Declarations

Tests various variable declaration styles: val, var, strings, numbers, and single-line lists.

## Input

```whitehall
val stringVal = "Hello"
val numberVal = 42
val boolVal = true
val listVal = listOf("Apple", "Banana", "Cherry")
var stringVar = "World"
var numberVar = 100
var listVar = listOf("A", "B", "C")

<Column>
  <Text text={stringVal} />
  <Text text={numberVal.toString()} />
  <Text text={boolVal.toString()} />
  <Text text={listVal.size.toString()} />
  <Text text={stringVar} />
  <Text text={numberVar.toString()} />
  <Text text={listVar.size.toString()} />
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.*

@Composable
fun Variables() {
    var stringVar by remember { mutableStateOf("World") }
    var numberVar by remember { mutableStateOf(100) }
    var listVar by remember { mutableStateOf(listOf("A", "B", "C")) }

    val stringVal = "Hello"
    val numberVal = 42
    val boolVal = true
    val listVal = listOf("Apple", "Banana", "Cherry")

    Column {
        Text(text = stringVal)
        Text(text = numberVal.toString())
        Text(text = boolVal.toString())
        Text(text = listVal.size.toString())
        Text(text = stringVar)
        Text(text = numberVar.toString())
        Text(text = listVar.size.toString())
    }
}
```

## Metadata

```
file: Variables.wh
package: com.example.app.components
```
