# Pass-Through: String Literals (Phase 3)

Tests that string literals in pass-through blocks are handled correctly. Braces, parens, and other structural characters inside strings should NOT affect parsing.

This test covers:
- Regular strings with braces: `"{ text }"`
- Multi-line strings with braces: `"""{ text }"""`
- Escaped quotes: `"She said \"hello\""`
- Character literals: `'{'`, `'}'`
- Mixed string types together

## Input

```whitehall
class StringStore {
    var messages: List<Message> = []
}

data class Message(
    val text: String = "{ default }",
    val quote: String = "She said \"Hello { world }\"",
    val path: String = "C:\\path\\{folder}\\file.txt",
    val delimiter: Char = '{',
    val json: String = """
        {
            "key": "value",
            "nested": { "data": true }
        }
    """,
    val regex: String = """^\{.*\}$"""
)
```

## Output

```kotlin
package com.example.app

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

class StringStore : ViewModel() {
    data class UiState(
        val messages: List<Message> = []
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var messages: List<Message>
        get() = _uiState.value.messages
        set(value) { _uiState.update { it.copy(messages = value) } }

}

data class Message(
    val text: String = "{ default }",
    val quote: String = "She said \"Hello { world }\"",
    val path: String = "C:\\path\\{folder}\\file.txt",
    val delimiter: Char = '{',
    val json: String = """
        {
            "key": "value",
            "nested": { "data": true }
        }
    """,
    val regex: String = """^\{.*\}$"""
)
```

## Metadata

```
file: StringStore.wh
package: com.example.app
```

## Status

🟢 **Phase 3:** String literal handling implemented
✅ All string types properly handled in pass-through blocks
