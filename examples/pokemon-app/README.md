# Pokédex App

A complete Whitehall example app that demonstrates:

- HTTP API calls with OkHttp
- JSON parsing with kotlinx.serialization
- ViewModel/Store pattern with state management
- List with LazyColumn and proper keys
- Navigation with route parameters
- Nice layouts with Material 3
- Loading states and error handling
- Responsive cards and spacing
- Progress bars with visual stats

## Features

- **List Screen**: Shows 50 Pokemon with IDs
- **Detail Screen**: Shows sprite, height, weight, types, and base stats
- **State Management**: Uses PokemonStore ViewModel for API calls
- **Error Handling**: Retry button on failures
- **Nice UI**: Material 3 colors, proper spacing, rounded cards

## API

Uses the free [PokéAPI](https://pokeapi.co/):
- List: `https://pokeapi.co/api/v2/pokemon?limit=50`
- Detail: `https://pokeapi.co/api/v2/pokemon/{id}`

## Dependencies

Add to `build.gradle.kts`:

```kotlin
dependencies {
    implementation("com.squareup.okhttp3:okhttp:4.12.0")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.6.0")
}
```

## Build

```bash
cd examples/pokemon-app
whitehall build
cd build
./gradlew assembleDebug
```

## Run

```bash
whitehall run
```

## Structure

```
src/
├── main.wh                         # Navigation setup
├── stores/
│   └── PokemonStore.wh            # API calls + state
├── screens/
│   ├── PokemonListScreen.wh       # List view
│   └── PokemonDetailScreen.wh     # Detail view with stats
└── components/
    └── PokemonCard.wh             # List item card
```

## Whitehall Features Used

- `var` state with auto ViewModel generation
- `suspend fun` with auto `viewModelScope.launch`
- `io { }` dispatcher for network calls
- `bind:value` two-way binding
- `@for` loops with keys
- `@if/@else` conditionals
- CSS-like padding shortcuts (`p={16}`, `px={20}`)
- `spacing={8}` for Column/Row
- `fillMaxWidth={true}` modifier
- Hex colors (`#6200EE`) and theme colors (`color="secondary"`)
- `onMount` lifecycle hook
- Route parameters (`$screen.params.id`)
- Navigation (`navigate()`, `navigateBack()`)
- AsyncImage for remote images
