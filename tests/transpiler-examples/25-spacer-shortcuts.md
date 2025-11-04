# Spacer Shortcuts

Test Spacer with h/w shorthand props for height and width.

## Metadata

```yaml
file: SpacerShortcuts.wh
package: com.example.app
```

## Input

```whitehall
<Column>
  <Text>First</Text>
  <Spacer h={16} />
  <Text>Second</Text>
  <Spacer w={24} />
  <Text>Third</Text>
  <Spacer />
  <Text>Fourth</Text>
</Column>
```

## Output

```kotlin
package com.example.app

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun SpacerShortcuts() {
    Column {
        Text(text = "First")
        Spacer(modifier = Modifier.height(16.dp))
        Text(text = "Second")
        Spacer(modifier = Modifier.width(24.dp))
        Text(text = "Third")
        Spacer(modifier = Modifier.height(8.dp))
        Text(text = "Fourth")
    }
}
```

## Notes

- **`h`** - height (vertical space): `<Spacer h={16} />` → `Spacer(modifier = Modifier.height(16.dp))`
- **`w`** - width (horizontal space): `<Spacer w={16} />` → `Spacer(modifier = Modifier.width(16.dp))`
- **No props** - defaults to 8dp height: `<Spacer />` → `Spacer(modifier = Modifier.height(8.dp))`
- Auto-adds `.dp` unit
- Consistent with Compose naming (Spacer, not Space)
