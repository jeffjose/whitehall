# Minimal Component - Just Text

The simplest possible component: just a single text element with no props or state.

## Input

```whitehall
<Text>Hello, World!</Text>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable

@Composable
fun MinimalText() {
    Text(text = "Hello, World!")
}
```
