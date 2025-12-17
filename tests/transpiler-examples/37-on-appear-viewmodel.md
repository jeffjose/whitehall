# OnAppear Prop with ViewModel

Tests onAppear prop with ViewModel pattern (triggered by lifecycle hook). The function reference is transformed to viewModel.functionName().

## Input

```whitehall
var items: List<String> = []
var page = 1
var isLoading = false

fun loadMore() {
  if (isLoading) return
  isLoading = true
  page = page + 1
}

$onMount {
  loadMore()
}

<Column>
  @for (item in items, key = {it}) {
    <Text>{item}</Text>
  }
  <Box onAppear={loadMore}>
    <Text>Loading more...</Text>
  </Box>
</Column>
```

## Output

**Primary File:** Test framework only validates primary output (wrapper component).

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.lifecycle.viewmodel.compose.viewModel

@Composable
fun PaginatedList() {
    val viewModel = viewModel<PaginatedListViewModel>()
    val uiState by viewModel.uiState.collectAsState()

    Column {
        uiState.items.forEach { item ->
            key(item) {
                Text(text = "${item}")
            }
        }
        Box {
            LaunchedEffect(Unit) {
                viewModel.loadMore()
            }
            Text(text = "Loading more...")
        }
    }
}
```

## Metadata

```
file: PaginatedList.wh
package: com.example.app.components
type_hint: component
multi_file: true
```
