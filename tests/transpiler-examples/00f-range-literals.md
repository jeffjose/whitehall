# Range Literals

Tests range literal syntax with auto-conversion to `.toList()`.

## Input

```whitehall
// Basic range converts to (1..10).toList()
val simple = 1..10

// Range with step converts to (0 rangeTo 10 step 2).toList()
val evens = 0..10:2

// Range with countdown converts to (10 downTo 1).toList()
val countdown = 10..1:-1

<Column spacing={8}>
  <Text>Simple: {simple.size} items</Text>
  <Text>Evens: {evens.size} items</Text>
  <Text>Countdown: {countdown.size} items</Text>
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
    val simple = (1..10).toList()
    val evens = (0 rangeTo 10 step 2).toList()
    val countdown = (10 downTo 1).toList()

    Column(
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Text(text = "Simple: ${simple.size} items")
        Text(text = "Evens: ${evens.size} items")
        Text(
            text = "Countdown: ${countdown.size} items"
        )
    }
}
```

## Metadata

```
file: RangeDemo.wh
package: com.example.app.components
```
