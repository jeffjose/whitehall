# Component with Single Prop

Tests a component with just one required prop.

## Input

```whitehall
  @prop val message: String

<Text>{message}</Text>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable

@Composable
fun SingleProp(
    message: String
) {
    Text(text = message)
}
```
