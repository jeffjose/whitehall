# Box Layout with Stacking

Tests Box layout for overlaying and stacking components on z-axis.

## Input

```whitehall
import $models.User

  @prop val user: User
  @prop val isOnline: Boolean = false

<Box>
  <AsyncImage
    url={user.avatarUrl}
    width={80}
    height={80}
  />

  @if (isOnline) {
    <Box
      width={16}
      height={16}
      backgroundColor="green"
      alignment="bottomEnd"
    />
  }
</Box>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import com.example.app.models.User

@Composable
fun AvatarWithStatus(
    user: User,
    isOnline: Boolean = false
) {
    Box {
        AsyncImage(
            model = user.avatarUrl,
            contentDescription = null,
            modifier = Modifier.size(80.dp)
        )

        if (isOnline) {
            Box(
                modifier = Modifier
                    .size(16.dp)
                    .background(Color.Green)
                    .align(Alignment.BottomEnd)
            )
        }
    }
}
```

## Metadata

```
file: AvatarWithStatus.wh
package: com.example.app.components
```
