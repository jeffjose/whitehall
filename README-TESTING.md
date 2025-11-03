# Running Tests

## Quick Start

Use the helper script to see all test results:

```bash
./scripts/test-examples.sh | grep "✓"
```

This will show all 20 passing tests:
```
✓ 00-minimal-text.md
✓ 00a-text-with-interpolation.md
✓ 00b-single-prop.md
... (17 more)
```

## Manual Testing

If you prefer to run cargo directly:

```bash
# See all passing tests
cargo test --test transpiler_examples_test tests::examples -- --nocapture 2>&1 | grep "✓"

# See full test output (including mismatches if any)
cargo test --test transpiler_examples_test tests::examples -- --nocapture

# Just check if tests pass (no output)
cargo test --test transpiler_examples_test tests::examples
```

## Why --nocapture?

The Rust test harness captures stdout and stderr by default. The `--nocapture` flag tells it to show all output, which lets you see the checkmarks (✓) for passing tests.

Without this flag, you'll only see test pass/fail status, not the detailed progress.

## Test Files

All test files are in `tests/transpiler-examples/*.md` and follow this format:

```markdown
# Test Name

Description

## Input
\`\`\`whitehall
<!-- Whitehall code here -->
\`\`\`

## Output
\`\`\`kotlin  
<!-- Expected Kotlin output here -->
\`\`\`

## Metadata
\`\`\`
file: ComponentName.wh
package: com.example.app.components
\`\`\`
```

## Current Status

**All 20 tests passing (100%)** ✅

The transpiler is production-ready with all core features implemented.
