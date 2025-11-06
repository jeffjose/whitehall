# Padding/Margin Shortcuts

Test CSS-like padding and margin shorthand props: `p`, `px`, `py`, `pt`, `pb`, `pl`, `pr` (and `m*` equivalents).

## Metadata

```yaml
file: PaddingShortcuts.wh
package: com.example.app
```

## Input

```whitehall
<Column>
  <Text p={16}>All sides</Text>

  <Text px={20} py={8}>
    Horizontal & Vertical
  </Text>

  <Text pt={4} pb={12}>
    Top & Bottom
  </Text>

  <Card pl={8} pr={16}>
    <Text>Left & Right</Text>
  </Card>
</Column>
```

## Output

```kotlin
package com.example.app

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Card
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun PaddingShortcuts() {
    Column {
        Text(
            text = "All sides",
            modifier = Modifier
            .padding(16.dp)
        )
        Text(
            text = "Horizontal & Vertical",
            modifier = Modifier
            .padding(horizontal = 20.dp, vertical = 8.dp)
        )
        Text(
            text = "Top & Bottom",
            modifier = Modifier
            .padding(top = 4.dp, bottom = 12.dp)
        )
        Card(
            modifier = Modifier
            .padding(start = 8.dp, end = 16.dp)
        ) {
            Text(text = "Left & Right")
        }
    }
}
```

## Notes

- **`p`** - padding all sides
- **`px`** - padding horizontal (start + end)
- **`py`** - padding vertical (top + bottom)
- **`pt`, `pb`, `pl`, `pr`** - padding top, bottom, left (start), right (end)
- **`m*`** variants work identically (Compose doesn't have true margin, so they map to padding)
- Multiple shortcuts on same component are combined into single `padding()` call
- Auto-adds `.dp` unit
- Works on any component that supports modifiers
