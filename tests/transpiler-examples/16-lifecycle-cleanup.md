# Lifecycle Cleanup with onDispose

Tests onDispose hook for cleanup when component leaves composition.

## Input

```whitehall
import $lib.websocket.WebSocketClient

  var messages: List<String> = emptyList()
  var connection: WebSocketClient? = null

  onMount {
    connection = WebSocketClient.connect("ws://api.example.com")
    connection?.onMessage { msg ->
      messages = messages + msg
    }
  }

  onDispose {
    connection?.disconnect()
    connection = null
  }

<Column spacing={8}>
  @for (message in messages, key = { it }) {
    <Text>{message}</Text>
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
import com.example.app.lib.websocket.WebSocketClient

@Composable
fun MessageList() {
    var messages by remember { mutableStateOf<List<String>>(emptyList()) }
    var connection by remember { mutableStateOf<WebSocketClient?>(null) }

    DisposableEffect(Unit) {
        connection = WebSocketClient.connect("ws://api.example.com")
        connection?.onMessage { msg ->
            messages = messages + msg
        }

        onDispose {
            connection?.disconnect()
            connection = null
        }
    }

    Column(
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        messages.forEach { message ->
            key(message) {
                Text(text = "${message}")
            }
        }
    }
}
```

## Metadata

```
file: MessageList.wh
package: com.example.app.components
```
