# Text with Simple Interpolation

Tests string interpolation in text content.

## Input

```whitehall
  var name = "World"

<Text>Hello, {name}!</Text>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.runtime.*
import androidx.compose.material3.Text

@Composable
fun TextWithInterpolation() {
    var name by remember { mutableStateOf("World") }

    Text(text = "Hello, $name!")
}
```
