# Import Aliases

Tests $ prefix import aliases mapping to configured package paths.

## Input

```whitehall
import $lib.api.ApiClient
import $models.User
import $models.Post
import $components.Avatar
import $components.PostCard

  @prop val userId: String

<Column>
  <Avatar url="https://example.com/avatar.jpg" />
  <Text>User Profile</Text>
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import com.example.app.components.Avatar
import com.example.app.components.PostCard
import com.example.app.lib.api.ApiClient
import com.example.app.models.Post
import com.example.app.models.User

@Composable
fun UserHeader(
    userId: String
) {
    Column {
        Avatar(url = "https://example.com/avatar.jpg")
        Text(text = "User Profile")
    }
}
```

## Metadata

```
file: UserHeader.wh
package: com.example.app.components
```

## Notes

Import alias mapping is configured in `whitehall.toml`:

```toml
[imports.aliases]
app = "com.example.app"
lib = "com.example.app.lib"
models = "com.example.app.models"
components = "com.example.app.components"
```
