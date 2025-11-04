# Color Support

Tests hex color support with automatic RGBA→ARGB conversion for Android.

## Input

```whitehall
<Column padding={16} spacing={12}>
  <Text fontSize={24} fontWeight="bold" color="#1976D2">
    Color Examples
  </Text>

  <Text color="#F00">
    3-char hex: #F00
  </Text>

  <Text color="#4CAF50">
    6-char hex: #4CAF50
  </Text>

  <Text color="#FF000080">
    8-char RGBA: #FF000080 (50% alpha)
  </Text>

  <Column backgroundColor="#F5F5F5" padding={8}>
    <Text>backgroundColor prop</Text>
  </Column>

  <Column
    modifier={Modifier
      .fillMaxWidth()
      .background("#E3F2FD")
      .padding(8.dp)
    }
  >
    <Text>Modifier with hex color</Text>
  </Column>
</Column>
```

## Output

```kotlin
package com.example.app

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun Colors() {
    Column(
        modifier = Modifier.padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        Text(
            text = "Color Examples",
            fontSize = 24.sp,
            fontWeight = FontWeight.Bold,
            color = Color(0xFF1976D2)
        )
        Text(
            text = "3-char hex: #F00",
            color = Color(0xFFFF0000)
        )
        Text(
            text = "6-char hex: #4CAF50",
            color = Color(0xFF4CAF50)
        )
        Text(
            text = "8-char RGBA: #FF000080 (50% alpha)",
            color = Color(0x80FF0000)
        )
        Column(
            modifier = Modifier.background(Color(0xFFF5F5F5)),
            modifier = Modifier.padding(8.dp)
        ) {
            Text(text = "backgroundColor prop")
        }
        Column(
            modifier = Modifier
            .fillMaxWidth()
            .background(Color(0xFFE3F2FD))
            .padding(8.dp)
        ) {
            Text(text = "Modifier with hex color")
        }
    }
}
```

## Features

- **3-char hex**: `#RGB` → `Color(0xFFRRGGBB)` (expanded with full alpha)
- **6-char hex**: `#RRGGBB` → `Color(0xFFRRGGBB)` (full alpha)
- **8-char RGBA**: `#RRGGBBAA` → `Color(0xAARRGGBB)` (web RGBA → Android ARGB)
- **Error handling**: Invalid hex colors fail compilation with clear error messages
- **Validation**: Checks for valid hex characters (0-9, A-F) and correct length (3, 6, or 8)

## Metadata

```
file: Colors.wh
package: com.example.app
```
