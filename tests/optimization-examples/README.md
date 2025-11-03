# Whitehall Optimization Examples

This directory contains examples showing how Whitehall's semantic analyzer and optimizer make **transparent automatic optimizations** without user intervention.

## Purpose

These examples demonstrate:
1. **What gets optimized** - Show input code and optimized output
2. **What doesn't get optimized** - Show when Whitehall correctly chooses NOT to optimize
3. **Why decisions are made** - Explain the analyzer's confidence scoring
4. **Performance impact** - Quantify the benefits of optimization

## Format

Each example markdown file has three sections:

### 1. Input
The Whitehall `.wh` source code the user writes

### 2. Unoptimized Output (Current)
What the current transpiler generates (Phase 0-4: always Compose)

### 3. Optimized Output (Phase 5+ Target)
What the analyzer/optimizer will generate when optimization is enabled

## Examples

### 01-static-list-recyclerview.md ✅
**Optimization:** RecyclerView instead of LazyColumn

**Scenario:** Immutable list with no mutations or event handlers

**Detection:**
- ✅ Collection is `val`
- ✅ Never mutated
- ✅ No event handlers
- ✅ Has key expression
- **Confidence:** 100/100

**Result:** 30-40% performance improvement, 40% less memory

---

### 02-dynamic-list-no-optimization.md ❌
**Optimization:** None (correctly stays with Compose)

**Scenario:** Mutable list with event handlers and state changes

**Detection:**
- ❌ Collection is `var` with `mutableStateOf`
- ❌ Mutated in lifecycle hooks
- ❌ Has event handlers that modify state
- **Confidence:** 0/100

**Result:** No optimization applied (correct decision!)

---

## Future Examples (Planned)

### 03-list-with-safe-handlers.md
- List is immutable but has onClick handlers
- Handlers don't mutate the list itself
- Should optimize? (Needs careful analysis)

### 04-text-heavy-rendering.md
- 1000+ Text composables
- Optimize to single TextView with spans

### 05-canvas-direct-drawing.md
- Custom drawing without Compose overhead
- Generate custom View with onDraw()

### 06-prop-list.md
- List passed as prop
- Conservative: don't optimize (parent might mutate)

## How to Use These

### During Development (Phase 1-4)
These examples guide implementation:
- Analyzer must detect the patterns shown
- Confidence scoring must match expected values
- Optimization hints must be generated correctly

### During Testing (Phase 5+)
Run transpiler on Input section, compare output:
- Phase 0-4: Should match "Unoptimized Output"
- Phase 5+: Should match "Optimized Output" when confidence >= 80

### For Documentation
These examples serve as:
- Living documentation of optimization behavior
- Reference for what gets optimized and why
- Performance benchmarking targets

## Implementation Status

| Phase | Status | Output Matches |
|-------|--------|----------------|
| Phase 0: Infrastructure | ✅ Complete | Unoptimized (Compose) |
| Phase 1: Usage Tracking | ⏳ Pending | Unoptimized (Compose) |
| Phase 2: Static Detection | ⏳ Pending | Unoptimized (Compose) |
| Phase 3: Hint Generation | ⏳ Pending | Unoptimized (Compose) |
| Phase 4: Planning | ⏳ Pending | Unoptimized (Compose) |
| Phase 5: RecyclerView | ⏳ Pending | **Optimized** (first change!) |

## Validation

To validate optimizer correctness:

```bash
# Phase 0-4: All examples should generate Compose
cargo test --test transpiler_examples_test

# Phase 5+: Example 01 should generate RecyclerView
cargo test optimization_examples

# Benchmark performance difference
cargo bench optimization_examples
```

## Adding New Examples

When adding a new optimization example:

1. Create `XX-optimization-name.md`
2. Include all three sections (Input, Unoptimized, Optimized)
3. Add analyzer decision log showing confidence scoring
4. Explain why optimization is safe (or unsafe)
5. Quantify performance impact if possible
6. Update this README with the new example

## Related Documentation

- **Architecture:** `docs/CODE-SEMANTICS.md` - Full optimization system design
- **Transpiler:** `docs/TRANSPILER.md` - Core transpiler documentation
- **Tests:** `tests/transpiler-examples/` - Syntax validation tests
