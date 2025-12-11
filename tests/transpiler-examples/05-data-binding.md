# Data Binding: bind:value

Tests two-way data binding with bind:value syntax.

## Input

```whitehall
  var username = ""
  var password = ""

  fun handleLogin() {
    // Login logic
  }

<Column spacing={16}>
  <TextField
    label="Username"
    bind:value={username}
  />

  <TextField
    label="Password"
    bind:value={password}
    type="password"
  />

  <Button
    text="Login"
    onClick={handleLogin}
  />
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.*
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp

@Composable
fun LoginForm() {
    var username by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }

    fun handleLogin() {
        // Login logic
    }

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        TextField(
            label = { Text("Username") },
            value = username,
            onValueChange = { username = it }
        )
        TextField(
            label = { Text("Password") },
            value = password,
            onValueChange = { password = it },
            visualTransformation = PasswordVisualTransformation()
        )
        key(Unit) {
            Button(onClick = { handleLogin() }) {
                Text("Login")
            }
        }
    }
}
```

## Metadata

```
file: LoginForm.wh
package: com.example.app.components
```
