# Pass-Through: Enum Class

Tests that enum classes defined after the main class are passed through unchanged.

## Input

```whitehall
class PokemonStore {
    var selectedType: PokemonType = PokemonType.NORMAL
    var pokemon: List<Pokemon> = []

    fun filterByType(type: PokemonType) {
        selectedType = type
    }
}

data class Pokemon(
    val id: Int,
    val name: String,
    val type: PokemonType
)

enum class PokemonType {
    NORMAL,
    FIRE,
    WATER,
    GRASS,
    ELECTRIC,
    ICE,
    FIGHTING,
    POISON,
    GROUND,
    FLYING,
    PSYCHIC,
    BUG,
    ROCK,
    GHOST,
    DRAGON,
    DARK,
    STEEL,
    FAIRY
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

class PokemonStore : ViewModel() {
    data class UiState(
        val selectedType: PokemonType = PokemonType.NORMAL,
        val pokemon: List<Pokemon> = []
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var selectedType: PokemonType
        get() = _uiState.value.selectedType
        set(value) { _uiState.update { it.copy(selectedType = value) } }

    var pokemon: List<Pokemon>
        get() = _uiState.value.pokemon
        set(value) { _uiState.update { it.copy(pokemon = value) } }

    fun filterByType(type: PokemonType) {
        selectedType = type
    }

}

data class Pokemon(
    val id: Int,
    val name: String,
    val type: PokemonType
)

enum class PokemonType {
    NORMAL,
    FIRE,
    WATER,
    GRASS,
    ELECTRIC,
    ICE,
    FIGHTING,
    POISON,
    GROUND,
    FLYING,
    PSYCHIC,
    BUG,
    ROCK,
    GHOST,
    DRAGON,
    DARK,
    STEEL,
    FAIRY
}
```

## Metadata

```
file: PokemonStore.wh
package: com.example.app
```

## Status

🔴 **Currently:** Errors with "Expected component, found: enum class"
🟢 **After Pass-Through:** Should pass through unchanged
