# Navigation with Reload Option

Tests the `reload` option for `$navigate` which forces navigation even when already at the destination.

## Input

```whitehall
  fun handleRefresh() {
    $navigate($routes.home, reload = true)
  }

  fun handleNormalNav() {
    $navigate($routes.settings)
  }

<Column gap={16}>
  <Text fontSize={24}>Navigation Demo</Text>

  <Button
    text="Refresh Home"
    onClick={handleRefresh}
  />

  <Button
    text="Go to Settings"
    onClick={handleNormalNav}
  />
</Column>
```

## Output

```kotlin
package com.example.app.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController
import com.example.app.navigateIfNeeded
import com.example.app.routes.Routes

@Composable
fun NavigationScreen(navController: NavController) {
    fun handleRefresh() {
        navController.navigate(Routes.Home) { launchSingleTop = true }
    }

    fun handleNormalNav() {
        navController.navigateIfNeeded(Routes.Settings)
    }

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = "Navigation Demo",
            fontSize = 24.sp
        )
        Button(onClick = { handleRefresh() }) {
            Text("Refresh Home")
        }
        Button(onClick = { handleNormalNav() }) {
            Text("Go to Settings")
        }
    }
}
```

## Metadata

```
file: NavigationScreen.wh
package: com.example.app.screens
type: screen
```
