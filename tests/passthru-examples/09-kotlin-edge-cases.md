# Pass-Through: Kotlin Syntax Edge Cases (Phase 6)

Tests various Kotlin syntax edge cases including operator functions, infix functions, inline functions, and special characters in identifiers.

## Input

```whitehall
class MyStore {
    var items: List<Item> = []
}

data class Item(val value: Int) {
    operator fun plus(other: Item): Item = Item(value + other.value)
    operator fun compareTo(other: Item): Int = value.compareTo(other.value)
}

infix fun Item.combine(other: Item): Item = Item(this.value + other.value)

inline fun <reified T> Item.cast(): T? = value as? T

typealias ItemPredicate = (Item) -> Boolean

fun interface ItemFilter {
    fun test(item: Item): Boolean
}

fun createFilter(threshold: Int): ItemFilter = ItemFilter { it.value > threshold }
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

class MyStore : ViewModel() {
    data class UiState(
        val items: List<Item> = []
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var items: List<Item>
        get() = _uiState.value.items
        set(value) { _uiState.update { it.copy(items = value) } }

}

data class Item(val value: Int) {
    operator fun plus(other: Item): Item = Item(value + other.value)
    operator fun compareTo(other: Item): Int = value.compareTo(other.value)
}

infix fun Item.combine(other: Item): Item = Item(this.value + other.value)

inline fun <reified T> Item.cast(): T? = value as? T

typealias ItemPredicate = (Item) -> Boolean

fun interface ItemFilter {
    fun test(item: Item): Boolean
}

fun createFilter(threshold: Int): ItemFilter = ItemFilter { it.value > threshold }
```

## Metadata

```
file: MyStore.wh
package: com.example.app
```

## Status

🟢 **Phase 6:** Kotlin syntax edge cases test
✅ Operator functions: `operator fun plus`, `operator fun compareTo`
✅ Infix functions: `infix fun combine`
✅ Inline functions with reified types: `inline fun <reified T>`
✅ Type aliases for function types: `typealias ItemPredicate = (Item) -> Boolean`
✅ Fun interfaces (SAM interfaces)
✅ Lambda expressions in function bodies
