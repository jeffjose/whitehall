# Pass-Through: Data Class After Main Class

Tests that data classes defined after the main class are passed through unchanged.

## Input

```whitehall
class PokemonStore {
    var pokemon: List<Pokemon> = []

    suspend fun loadPokemon() {
        pokemon = listOf(Pokemon(1, "Pikachu"))
    }
}

data class Pokemon(
    val id: Int,
    val name: String
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

class PokemonStore : ViewModel() {
    data class UiState(
        val pokemon: List<Pokemon> = []
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var pokemon: List<Pokemon>
        get() = _uiState.value.pokemon
        set(value) { _uiState.update { it.copy(pokemon = value) } }

    fun loadPokemon() {
        viewModelScope.launch {
            pokemon = listOf(Pokemon(1, "Pikachu"))
        }
    }

}

data class Pokemon(
    val id: Int,
    val name: String
)
```

## Metadata

```
file: PokemonStore.wh
package: com.example.app
```

## Status

🔴 **Currently:** Errors with "Expected component, found: data class"
🟢 **After Pass-Through:** Should pass through unchanged
