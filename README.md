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

- **Reduce line noise** — Write less boilerplate, focus on what matters
- **Developer experience first** — Intuitive syntax, helpful errors, fast iteration
- **Fast by default** — Native performance without configuration
- **Complex things are possible** — Escape hatches when you need them
- **Batteries included** — Routing, state, toolchain—all built in

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
