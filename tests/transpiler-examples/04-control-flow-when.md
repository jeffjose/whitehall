# Control Flow: When Expression

Tests @when for multi-branch conditionals.

## Input

```whitehall
  @prop val status: LoadingState

<Column>
  @when {
    status == LoadingState.LOADING -> <LoadingSpinner />
    status == LoadingState.ERROR -> <ErrorView message="Failed to load" />
    status == LoadingState.EMPTY -> <Text>No items found</Text>
    else -> <ContentView />
  }
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.runtime.Composable
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text

@Composable
fun StatusView(
    status: LoadingState
) {
    Column {
        when {
            status == LoadingState.LOADING -> LoadingSpinner()
            status == LoadingState.ERROR -> ErrorView(message = "Failed to load")
            status == LoadingState.EMPTY -> Text(text = "No items found")
            else -> ContentView()
        }
    }
}
```
