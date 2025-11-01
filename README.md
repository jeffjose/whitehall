# Whitehall

> A unified Rust toolchain for Android app development

**Status:** Early development - nothing works yet!

## What is Whitehall?

Whitehall aims to be to Android development what `cargo` is to Rust - a fast, modern, all-in-one CLI tool that handles project creation, building, dependencies, and deployment.

```bash
whitehall init my-app           # Create new project
whitehall build                 # Compile .whitehall files ’ Kotlin
whitehall run                   # Build and deploy to device
whitehall install <dependency>  # Add dependencies
```

## Why?

Android development is fragmented across multiple tools, languages, and slow build systems. Whitehall brings simplicity, speed, and modern developer experience to mobile development.

## Documentation

- [**VISION.md**](docs/VISION.md) - The full vision and goals
- [**ROADMAP.md**](docs/ROADMAP.md) - Development phases and milestones
- [**ARCHITECTURE.md**](docs/ARCHITECTURE.md) - Technical design and decisions

## Current Status

**Phase 0: Foundation** - Building the basic CLI structure

See [ROADMAP.md](docs/ROADMAP.md) for detailed progress.

## Development

```bash
# Once implemented:
cargo build
cargo install --path .
whitehall --help
```

## License

TBD

---

**Note:** This is an ambitious experimental project. Contributions and ideas welcome!
