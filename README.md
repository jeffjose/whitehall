# Whitehall

A Kotlin superset for Jetpack Compose with component markup, automatic state management, and zero-config tooling.

> **Experimental.** APIs and syntax may change.

## Overview

Whitehall extends Kotlin with ergonomic syntax for building Android UIs. Any valid Kotlin code is valid Whitehall code.

```whitehall
var count = 0

fun increment() {
    count++
}

<Column p={16}>
  <Text>Count: {count}</Text>
  <Button text="+" onClick={increment} />
</Column>
```

Transpiles to idiomatic Kotlin/Compose.

## Key Features

- **Component markup** — `<Text>`, `<Column>`, `<LazyColumn>`
- **Automatic reactivity** — `var` properties become `StateFlow`
- **Data binding** — `bind:value`, `bind:checked`
- **Control flow** — `@if`, `@for`, `@when`
- **Lifecycle hooks** — `$onMount`, `$onDispose`
- **Toolchain management** — automatic Java, Gradle, and Android SDK setup

## Quick Start

```bash
cargo install whitehall
whitehall init my-app && cd my-app
whitehall run
```

## Documentation

- [Language Reference](docs/LANGUAGE-REFERENCE.md) — Complete syntax guide
- [Architecture](docs/REF-OVERVIEW.md) — How Whitehall works
- [Transpiler](docs/REF-TRANSPILER.md) — Transpilation details
- [State Management](docs/REF-STATE-MANAGEMENT.md) — Reactivity patterns
- [Build System](docs/REF-BUILD-SYSTEM.md) — Build commands
- [Toolchain](docs/REF-TOOLCHAIN.md) — Toolchain management

## Contributing

Contributions welcome. See [Architecture](docs/REF-OVERVIEW.md) for internals.

## License

MIT
