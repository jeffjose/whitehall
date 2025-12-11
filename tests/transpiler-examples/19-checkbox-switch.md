# Checkbox and Switch with bind:checked

Tests boolean form inputs with two-way data binding using bind:checked.

## Input

```whitehall
  var isEnabled = false
  var acceptTerms = false
  var notifications = true

  fun handleSubmit() {
    if (acceptTerms) {
      // Submit logic
    }
  }

<Column spacing={16}>
  <Row spacing={8}>
    <Checkbox bind:checked={isEnabled} />
    <Text>Enable feature</Text>
  </Row>

  <Row spacing={8}>
    <Checkbox bind:checked={acceptTerms} />
    <Text>I accept the terms and conditions</Text>
  </Row>

  <Row spacing={8}>
    <Text>Push notifications</Text>
    <Switch bind:checked={notifications} />
  </Row>

  <Button
    text="Submit"
    onClick={handleSubmit}
    enabled={acceptTerms}
  />
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.unit.dp

@Composable
fun SettingsForm() {
    var isEnabled by remember { mutableStateOf(false) }
    var acceptTerms by remember { mutableStateOf(false) }
    var notifications by remember { mutableStateOf(true) }

    fun handleSubmit() {
        if (acceptTerms) {
              // Submit logic
            }
    }

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Row(
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Checkbox(
                checked = isEnabled,
                onCheckedChange = { isEnabled = it }
            )
            key(Unit) {
                Text(text = "Enable feature")
            }
        }
        Row(
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Checkbox(
                checked = acceptTerms,
                onCheckedChange = { acceptTerms = it }
            )
            key(Unit) {
                Text(
                    text = "I accept the terms and conditions"
                )
            }
        }
        Row(
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            key(Unit) {
                Text(text = "Push notifications")
            }
            Switch(
                checked = notifications,
                onCheckedChange = { notifications = it }
            )
        }
        Button(
            onClick = { handleSubmit() },
            enabled = acceptTerms
        ) {
            Text("Submit")
        }
    }
}
```

## Metadata

```
file: SettingsForm.wh
package: com.example.app.components
```
