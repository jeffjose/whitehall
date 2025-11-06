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

**Primary File:** Test framework only validates primary output (wrapper component).

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.app.lib.websocket.WebSocketClient

@Composable
fun MessageList() {
    val viewModel = viewModel<MessageListViewModel>()
    val uiState by viewModel.uiState.collectAsState()

    Column(
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        uiState.messages.forEach { message ->
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
multi_file: true
```
