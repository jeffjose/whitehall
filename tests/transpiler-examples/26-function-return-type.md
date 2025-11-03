# Function Return Type Annotations

Tests optional return type annotations on functions.

## Input

```whitehall
fun getMessage(): String {
  return "Hello, World!"
}

fun getCount(x: Int): Int {
  return x + 1
}

<Column>
  <Text>{getMessage()}</Text>
  <Text>{getCount(5)}</Text>
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable

@Composable
fun FunctionReturnType() {
    fun getMessage(): String {
        return "Hello, World!"
    }

    fun getCount(x: Int): Int {
        return x + 1
    }

    Column {
        Text(text = getMessage())
        Text(text = getCount(5))
    }
}
```

## Metadata

```yaml
file: FunctionReturnType.wh
package: com.example.app.components
```
