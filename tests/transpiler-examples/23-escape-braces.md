# Escape Braces (Svelte-style)

Test double-brace escaping for literal braces in text: `{{expr}}` → literal `{expr}`

## Metadata

```yaml
file: EscapeBraces.wh
package: com.example.app
```

## Input

```whitehall
var value = 42

<Column spacing={8}>
  <Text>Interpolation: {value}</Text>
  <Text>Literal braces: {{value}}</Text>
  <Text>Mixed: The value is {{value}} not {value}</Text>
  <Text>Multiple: {{a}} {{b}} {c}</Text>
</Column>
```

## Output

```kotlin
package com.example.app

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.unit.dp

@Composable
fun EscapeBraces() {
    var value by remember { mutableStateOf(42) }

    Column(
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        Text(text = "Interpolation: ${value}")
        Text(text = "Literal braces: {value}")
        Text(
            text = "Mixed: The value is {value} not ${value}"
        )
        Text(text = "Multiple: {a} {b} ${c}")
    }
}
```

## Notes

- `{expr}` → interpolates the expression
- `{{expr}}` → literal text `{expr}` (double braces escape to single)
- Follows Svelte's escape pattern
- Works in any text content
