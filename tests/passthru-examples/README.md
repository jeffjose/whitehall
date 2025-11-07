# Pass-Through Test Examples

This directory contains test cases for the **pass-through architecture** - the ability for Whitehall to pass arbitrary Kotlin syntax through unchanged.

## Philosophy

**Whitehall is a superset of Kotlin.**

Any syntax that Whitehall doesn't explicitly recognize and transform should pass through unchanged. This makes Whitehall flexible and future-proof.

## Current Status

🔴 **These tests currently FAIL** - They demonstrate Kotlin syntax that should work but doesn't yet.

🟢 **After Pass-Through Implementation** - These will pass, showing that arbitrary Kotlin code works in `.wh` files.

## Test Cases

### 01-data-class.md
Data classes defined after the main class

**Currently:** Errors with "Expected component, found: data class"
**After:** Passes through unchanged

### 02-sealed-class.md
Sealed class hierarchies for state management

**Currently:** Errors with "Expected component, found: data class"
**After:** Passes through unchanged

### 03-enum-class.md
Enum classes for type-safe constants

**Currently:** Errors with "Expected component, found: enum class"
**After:** Passes through unchanged

### 04-typealias-and-helpers.md
Type aliases and top-level helper functions

**Currently:** Errors with "Expected component, found: typealias"
**After:** Passes through unchanged

### 05-mixed-constructs.md
Multiple Kotlin constructs mixed together

**Currently:** Errors with "Expected component, found: typealias"
**After:** Passes through unchanged

## Running Tests

```bash
# Run just pass-through tests
cargo test --test passthru_examples_test

# Run with output
cargo test --test passthru_examples_test -- --nocapture

# Run all tests including pass-through
./scripts/test-examples.sh
```

## Expected Behavior After Implementation

All tests should:
1. Parse without errors
2. Pass through Kotlin constructs unchanged
3. Generate valid Kotlin code
4. Maintain declaration order

## Implementation

See `docs/PASSTHRU.md` for the full implementation plan.

---

**Status:** 🔴 Tests created, implementation pending
**Created:** 2025-11-07
**Related:** docs/PASSTHRU.md, docs/GAPS.md
