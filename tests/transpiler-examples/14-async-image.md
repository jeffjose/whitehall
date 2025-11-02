# AsyncImage with Loading and Error States

Tests AsyncImage with placeholder, loading, and error state handling.

## Input

```whitehall
  @prop val url: String
  @prop val contentDescription: String? = null
  @prop val size: Int = 120

<AsyncImage
  url={url}
  contentDescription={contentDescription}
  width={size}
  height={size}
  placeholder="ic_placeholder"
  error="ic_error"
  crossfade={true}
/>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import coil.request.ImageRequest
import androidx.compose.ui.platform.LocalContext
import coil.compose.rememberAsyncImagePainter

@Composable
fun NetworkImage(
    url: String,
    contentDescription: String? = null,
    size: Int = 120
) {
    AsyncImage(
        model = ImageRequest.Builder(LocalContext.current)
            .data(url)
            .crossfade(true)
            .placeholder(R.drawable.ic_placeholder)
            .error(R.drawable.ic_error)
            .build(),
        contentDescription = contentDescription,
        modifier = Modifier.size(size.dp),
        contentScale = ContentScale.Crop
    )
}
```

## Metadata

```
file: NetworkImage.wh
package: com.example.app.components
```
