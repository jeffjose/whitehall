# Control Flow: If/Else

Tests @if/@else conditional rendering.

## Input

```whitehall
  @prop val isLoading: Boolean
  @prop val error: String?
  @prop val data: String?

<Column>
  @if (isLoading) {
    <LoadingSpinner />
  } else if (error != null) {
    <ErrorView message={error} />
  } else {
    <Text>{data ?: "No data"}</Text>
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
fun DataView(
    isLoading: Boolean,
    error: String? = null,
    data: String? = null
) {
    Column {
        if (isLoading) {
            LoadingSpinner()
        } else if (error != null) {
            ErrorView(message = error)
        } else {
            Text(text = data ?: "No data")
        }
    }
}
```

## Metadata

```
file: StatusView.wh
package: com.example.app.components
```
