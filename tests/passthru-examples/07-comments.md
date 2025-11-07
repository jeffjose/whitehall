# Pass-Through: Comments (Phase 4)

Tests that comments in pass-through blocks are handled correctly. Braces, parens, and other structural characters inside comments should NOT affect parsing.

This test covers:
- Line comments with braces: `// { comment }`
- Block comments with braces: `/* { comment } */`
- Comment markers inside strings: `"// not a comment"`
- String markers inside comments: `/* "not a string" */`
- URLs in comments: `// http://example.com`
- Mixed strings, comments, and code

## Input

```whitehall
class CommentStore {
    var items: List<Item> = []
}

data class Item(
    val id: Int,  // Unique identifier { not a brace }
    /* This field has { braces } in comment */
    val name: String = "Name",  // Field description
    val url: String = "http://example.com",  // URL in comment: http://test.com
    val code: String = "// not a real comment",  /* String with comment markers */
    val template: String = """
        // This looks like { a comment } but it's in a string
        /* Also not { a comment } */
    """,
    val count: Int  /* Multi-line comment with {
        braces and (parens) across
        multiple lines { test }
    } end of comment */
)

typealias ItemId = Int

enum class Status {
    ACTIVE,  // Active status { not a brace }
    INACTIVE  /* Inactive { also not a brace } */
}
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

class CommentStore : ViewModel() {
    data class UiState(
        val items: List<Item> = []
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var items: List<Item>
        get() = _uiState.value.items
        set(value) { _uiState.update { it.copy(items = value) } }

}

data class Item(
    val id: Int,  // Unique identifier { not a brace }
    /* This field has { braces } in comment */
    val name: String = "Name",  // Field description
    val url: String = "http://example.com",  // URL in comment: http://test.com
    val code: String = "// not a real comment",  /* String with comment markers */
    val template: String = """
        // This looks like { a comment } but it's in a string
        /* Also not { a comment } */
    """,
    val count: Int  /* Multi-line comment with {
        braces and (parens) across
        multiple lines { test }
    } end of comment */
)

typealias ItemId = Int

enum class Status {
    ACTIVE,  // Active status { not a brace }
    INACTIVE  /* Inactive { also not a brace } */
}
```

## Metadata

```
file: CommentStore.wh
package: com.example.app
```

## Status

🟢 **Phase 4:** Comment handling implemented
✅ All comment types properly handled in pass-through blocks

## Edge Cases Covered

1. ✅ Line comments with braces: `// { text }`
2. ✅ Block comments with braces: `/* { text } */`
3. ✅ Multi-line block comments with braces
4. ✅ Comment markers in strings: `"// not a comment"`
5. ✅ String markers in comments: `/* "not a string" */`
6. ✅ URLs in comments: `// http://example.com`
7. ✅ Mixed strings and comments on same line
8. ✅ Inline block comments: `return /* comment */ 42`
9. ✅ Documentation-style comments: `/* * ... */`
10. ✅ Comments at different nesting levels
