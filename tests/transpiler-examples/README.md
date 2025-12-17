# Transpiler Test Examples

This directory contains markdown-based transpilation test cases. Each file demonstrates a specific feature or pattern and serves as both:
- **Executable test**: Validated by the test harness
- **Quick reference**: Documentation showing input → output transformations

## File Format

Each markdown file follows this structure:

```markdown
# Test Name

Brief description of what this test validates.

## Input

```whitehall
// Whitehall source code
```

## Output

```kotlin
// Expected Kotlin output
```
```

## Running Tests

```bash
cargo test transpiler_examples
```

The test harness:
1. Parses each markdown file
2. Extracts the Whitehall input from the `## Input` section
3. Extracts the expected Kotlin from the `## Output` section
4. Runs the transpiler on the input
5. Compares actual output with expected output

## Test Organization

Tests are organized by feature:

- `01-basic-component.md` - Simple component with props
- `02-control-flow-if.md` - @if/@else control flow
- `03-control-flow-for.md` - @for loops with keys
- `04-control-flow-when.md` - @when expressions
- `05-data-binding.md` - bind:value syntax
- `06-lifecycle-hooks.md` - $onMount and other hooks
- `07-routing-simple.md` - Basic screen with navigation
- `08-routing-params.md` - Route parameters
- `09-imports.md` - Import aliases ($lib, $models, etc.)
- `10-nested-components.md` - Deep component trees
- ... and more

## Adding New Tests

1. Create a new `.md` file in this directory
2. Follow the format above with `## Input` and `## Output` sections
3. Run `cargo test transpiler_examples` to validate
4. Tests serve as living documentation - keep them clear and focused

## Benefits

- **Easy Review**: Side-by-side input/output comparison
- **Living Docs**: Tests are documentation that never gets stale
- **Quick Reference**: Developers can quickly find examples
- **Maintainable**: Plain text format, easy to edit and diff
- **CI-Friendly**: Standard cargo test integration
