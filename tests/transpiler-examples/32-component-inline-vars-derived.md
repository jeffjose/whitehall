# Component Inline Vars - Derived State (Phase 1.1)

Tests that simple components with derived properties do NOT generate ViewModels. Component has only 2 functions, which doesn't meet the >= 3 threshold for ViewModel generation. Uses remember/mutableStateOf instead.

**Note:** This is a single-file component (no ViewModel generated).

## Input

```whitehall
var firstName = ""
var lastName = ""
var age = 0

val fullName: String get() = "$firstName $lastName"
val isAdult: Boolean get() = age >= 18
val displayName: String get() = if (fullName.isNotEmpty()) fullName else "Anonymous"

fun updateName(first: String, last: String) {
  firstName = first
  lastName = last
}

fun celebrateBirthday() {
  age++
}

<Column spacing={16}>
  <Text text="Name: {displayName}" fontSize={24} />
  <Text text="Age: {age}" fontSize={18} />

  @if (isAdult) {
    <Text text="✓ Adult" color="#4CAF50" />
  } else {
    <Text text="Minor" color="#FFA000" />
  }

  <Row spacing={8}>
    <TextField
      label="First Name"
      bind:value={firstName}
    />
    <TextField
      label="Last Name"
      bind:value={lastName}
    />
  </Row>

  <Button onClick={() => celebrateBirthday()} text="Birthday!" />
</Column>
```

## Output

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.*
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Composable
fun PersonForm() {
    var firstName by remember { mutableStateOf("") }
    var lastName by remember { mutableStateOf("") }
    var age by remember { mutableStateOf(0) }

    val fullName: String get() = "$firstName $lastName"
    val isAdult: Boolean get() = age >= 18
    val displayName: String get() = if (fullName.isNotEmpty()) fullName else "Anonymous"

    fun updateName(first: String, last: String) {
        firstName = first
          lastName = last
    }
    fun celebrateBirthday() {
        age++
    }

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
                text = "Name: ${displayName}",
                fontSize = 24.sp
            )
        Text(
            text = "Age: ${age}",
            fontSize = 18.sp
        )
        if (isAdult) {
            Text(
                    text = "✓ Adult",
                    color = Color(0xFF4CAF50)
                )
        } else {
            Text(
                    text = "Minor",
                    color = Color(0xFFFFA000)
                )
        }
        Row(
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            TextField(
                label = { Text("First Name") },
                value = firstName,
                onValueChange = { firstName = it }
            )
            TextField(
                label = { Text("Last Name") },
                value = lastName,
                onValueChange = { lastName = it }
            )
        }
        Button(onClick = { celebrateBirthday() }) {
            Text("Birthday!")
        }
    }
}
```

## Metadata

```
file: PersonForm.wh
package: com.example.app.components
type_hint: component
```
