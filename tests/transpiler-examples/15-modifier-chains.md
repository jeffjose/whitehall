# Modifier Chains and Conditional Modifiers

Tests chaining multiple modifiers and conditional modifier application.

## Input

```whitehall
  @prop val text: String
  @prop val isClickable: Boolean = false
  @prop val fillWidth: Boolean = false
  @prop val onClick: (() -> Unit)? = null

<Card
  fillMaxWidth={fillWidth}
  modifier={isClickable && onClick != null ? Modifier.clickable { onClick() } : Modifier}
>
  <Text
    padding={16}
    fillMaxWidth={true}
  >
    {text}
  </Text>
</Card>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun ActionCard(
    text: String,
    isClickable: Boolean = false,
    fillWidth: Boolean = false,
    onClick: (() -> Unit)? = null
) {
    Card(
        modifier = Modifier
            .let { if (fillWidth) it.fillMaxWidth() else it }
            .let { if (isClickable && onClick != null) it.clickable { onClick() } else it }
    ) {
        Text(
            text = "${text}",
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        )
    }
}
```

## Metadata

```
file: ActionCard.wh
package: com.example.app.components
```
