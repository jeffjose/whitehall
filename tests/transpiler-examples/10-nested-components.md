# Nested Components

Tests deep component nesting and props passing through multiple levels.

## Input

```whitehall
  @prop val title: String
  @prop val items: List<String>

<Scaffold
  topBar={
    <TopAppBar
      title={title}
      navigationIcon="back"
      onNavigationClick={() => goBack()}
    />
  }
>
  <Column padding={16} spacing={8}>
    <Text fontSize={20} fontWeight="bold">Items</Text>

    @for (item in items, key = { it }) {
      <Card>
        <Row padding={12} spacing={8}>
          <Icon name="check" />
          <Text>{item}</Text>
        </Row>
      </Card>
    }
  </Column>
</Scaffold>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.Icon
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun ItemList(
    title: String,
    items: List<String>
) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(title) },
                navigationIcon = "back",
                onNavigationClick = { goBack() }
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .padding(paddingValues)
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Text(
                text = "Items",
                fontSize = 20.sp,
                fontWeight = FontWeight.Bold
            )
            items.forEach { item ->
                key(item) {
                    Card {
                        Row(
                            modifier = Modifier.padding(12.dp),
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            Icon(name = "check")
                            Text(text = item)
                        }
                    }
                }
            }
        }
    }
}
```

## Metadata

```
file: ItemList.wh
package: com.example.app.components
```
