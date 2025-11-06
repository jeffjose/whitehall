# Component Inline Vars - Derived State (Phase 1.1)

Tests automatic ViewModel generation with derived properties. Derived properties (val with getters) should NOT be included in UiState but should be generated as computed properties.

**Note:** This generates TWO files - primary output shown below is the wrapper component.

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

**Primary File:** Test framework only validates primary output.

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel

@Composable
fun PersonForm() {
    val viewModel = viewModel<PersonFormViewModel>()
    val uiState by viewModel.uiState.collectAsState()

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = "Name: ${viewModel.displayName}",
            fontSize = 24.sp
        )
        Text(
            text = "Age: ${uiState.age}",
            fontSize = 18.sp
        )

        if (viewModel.isAdult) {
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
                value = uiState.firstName,
                onValueChange = { viewModel.firstName = it }
            )
            TextField(
                label = { Text("Last Name") },
                value = uiState.lastName,
                onValueChange = { viewModel.lastName = it }
            )
        }

        Button(onClick = { viewModel.celebrateBirthday() }) {
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
multi_file: true
```
