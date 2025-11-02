# Routing: Simple Navigation

Tests basic $routes navigation without parameters.

## Input

```whitehall
  fun handleLoginClick() {
    navigate($routes.login)
  }

  fun handleSignupClick() {
    navigate($routes.signup)
  }

<Column spacing={16}>
  <Text fontSize={24}>Welcome!</Text>

  <Button
    text="Login"
    onClick={handleLoginClick}
  />

  <Button
    text="Sign Up"
    onClick={handleSignupClick}
    variant="outlined"
  />
</Column>
```

## Output

```kotlin
package com.example.app.screens

import androidx.compose.runtime.Composable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController

@Composable
fun WelcomeScreen(navController: NavController) {
    fun handleLoginClick() {
        navController.navigate(Routes.Login)
    }

    fun handleSignupClick() {
        navController.navigate(Routes.Signup)
    }

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = "Welcome!",
            fontSize = 24.sp
        )

        Button(
            onClick = { handleLoginClick() }
        ) {
            Text("Login")
        }

        Button(
            onClick = { handleSignupClick() },
            variant = "outlined"
        ) {
            Text("Sign Up")
        }
    }
}
```

## Metadata

```
file: WelcomeScreen.wh
package: com.example.app.screens
type: screen
```
