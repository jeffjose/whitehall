# Whitehall

A modern framework for native Android apps. Simple to write, fast to run.

> **Experimental.** APIs and syntax may change.

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

## Pillars

- **Minimal** — Less boilerplate, more clarity
- **Intuitive** — Helpful errors, fast iteration
- **Fast** — Native performance by default
- **Powerful** — Full control when you need it
- **Complete** — Routing, state, toolchain built in

## Quick Start

```bash
cargo install --git https://github.com/aspect-build/whitehall
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
