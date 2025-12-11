# Image Component with Web-Style Props

Tests the Image component which maps to AsyncImage with web-friendly prop aliases.

## Input

```whitehall
@prop val imageUrl: String

<Image src={imageUrl} alt="Random photo" fit="cover" width="100%" height={200} />
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage

@Composable
fun ImageTest(
    imageUrl: String
) {
    AsyncImage(
        modifier = Modifier.fillMaxWidth().height(200.dp),
        model = imageUrl,
        contentDescription = "Random photo",
        contentScale = ContentScale.Crop
    )
}
```

## Metadata

```
file: ImageTest.wh
package: com.example.app.components
```

## Notes

Web-style to Compose mappings:
- `src` → `model`
- `alt` → `contentDescription`
- `fit="cover"` → `contentScale = ContentScale.Crop`
- `width="100%"` → `Modifier.fillMaxWidth()`
- `height={200}` → `Modifier.height(200.dp)`
