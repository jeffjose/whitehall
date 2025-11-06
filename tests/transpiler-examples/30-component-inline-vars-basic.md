# Component Inline Vars - Basic (Phase 1.1)

Tests automatic ViewModel generation for components with inline `var` declarations. When a component has mutable state, it should auto-generate a ViewModel with StateFlow and a wrapper component.

**Note:** This generates TWO files - primary output shown below is the wrapper component.

## Input

```whitehall
var count = 0
var lastIncrement: String? = null

fun increment() {
  count++
  lastIncrement = "Incremented at ${System.currentTimeMillis()}"
}

fun decrement() {
  count--
  lastIncrement = null
}

fun reset() {
  count = 0
  lastIncrement = null
}

<Column spacing={16}>
  <Text text="Count: {count}" fontSize={32} />

  @if (lastIncrement != null) {
    <Text text={lastIncrement} fontSize={12} color="#666" />
  }

  <Row spacing={8}>
    <Button onClick={() => decrement()} text="-" />
    <Button onClick={() => increment()} text="+" />
  </Row>

  <Button onClick={() => reset()} text="Reset" />
</Column>
```

## Output

**Primary File (Counter.kt):** The test framework currently only validates the primary output.
The ViewModel file (CounterViewModel.kt) is also generated but not checked by this test.

```kotlin
package com.example.app.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel

@Composable
fun Counter() {
    val viewModel = viewModel<CounterViewModel>()
    val uiState by viewModel.uiState.collectAsState()

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = "Count: ${uiState.count}",
            fontSize = 32.sp
        )
        if (uiState.lastIncrement != null) {
            Text(
                text = uiState.lastIncrement,
                fontSize = 12.sp,
                color = Color(0xFF666666)
            )
        }
        Row(
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Button(onClick = { viewModel.decrement() }) {
                Text("-")
            }
            Button(onClick = { viewModel.increment() }) {
                Text("+")
            }
        }
        Button(onClick = { viewModel.reset() }) {
            Text("Reset")
        }
    }
}
```

## Metadata

```
file: Counter.wh
package: com.example.app.components
type_hint: component
multi_file: true
```
