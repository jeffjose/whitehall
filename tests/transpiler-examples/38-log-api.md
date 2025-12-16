# Log API for Android Logging

Tests $log() transformation to Android Log calls. Auto-tags with component name when only message is provided.

## Input

```whitehall
var count = 0

fun increment() {
  $log.d("incrementing count")
  count = count + 1
  $log("Counter", "count is now ${count}")
}

<Button onClick={increment}>
  <Text>Count: {count}</Text>
</Button>
```

## Output

```kotlin
package com.example.app.components

import android.util.Log
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.*

@Composable
fun Counter() {
    var count by remember { mutableStateOf(0) }

    fun increment() {
        Log.d("Counter", "incrementing count")
        count = count + 1
        Log.i("Counter", "count is now ${count}")
    }

    Button(onClick = { increment() }) {
        Text(text = "Count: ${count}")
    }
}
```

## Metadata

```
file: Counter.wh
package: com.example.app.components
```
