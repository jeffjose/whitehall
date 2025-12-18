# Inline Lambda Syntax

Tests inline lambda syntax with `() => expression` for event handlers and component props.

## Input

```whitehall
  @prop val items: List<String>
  @prop val onItemClick: (String) -> Unit

<Column gap={16}>
  @for (item in items, key = { it }) {
    <Card onClick={() => onItemClick(item)}>
      <Text>{item}</Text>
    </Card>
  }

  <Button
    text="Clear All"
    onClick={() => clearItems()}
  />
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.unit.dp

@Composable
fun ItemList(
    items: List<String>,
    onItemClick: (String) -> Unit
) {
    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        items.forEach { item ->
            key(item) {
                Card(onClick = { onItemClick(item) }) {
                    Text(text = "${item}")
                }
            }
        }
        Button(onClick = { clearItems() }) {
            Text("Clear All")
        }
    }
}
```

## Metadata

```
file: ItemList.wh
package: com.example.app.components
```
