# Basic Component with Props

Tests a simple component with required and optional props, including default values.

## Input

```whitehall
import $models.User

  @prop val url: String
  @prop val size: Int = 48
  @prop val onClick: (() -> Unit)? = null

<AsyncImage
  url={url}
  width={size}
  height={size}
  modifier={onClick?.let { Modifier.clickable { it() } } ?: Modifier}
/>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.foundation.clickable
import com.example.app.models.User
import coil.compose.AsyncImage

@Composable
fun Avatar(
    url: String,
    size: Int = 48,
    onClick: (() -> Unit)? = null
) {
    AsyncImage(
        url = url,
        width = size,
        height = size,
        modifier = onClick?.let { Modifier.clickable { it() } } ?: Modifier
    )
}
```

## Metadata

```
file: Avatar.wh
package: com.example.app.components
```
