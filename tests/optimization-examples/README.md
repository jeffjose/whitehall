# Whitehall Optimization Examples

Testable examples showing transparent automatic optimizations.

## Format

Each example has:
- **Input**: Whitehall `.wh` source code
- **Unoptimized Output**: What the transpiler generates without optimization (Compose)
- **Optimized Output**: What the transpiler generates with optimization enabled (RecyclerView/other)
- **Metadata**: File info, optimization type, confidence score

This format allows testing both modes:
- Phase 0-4: Test against Unoptimized Output
- Phase 5+: Test against Optimized Output

## Examples

### 01-static-list-optimization.md ✅

**Optimize**: RecyclerView instead of LazyColumn

**When**:
- Collection is `val` (immutable)
- Never mutated
- No event handlers
- Confidence: 100/100

**Result**: 30-40% faster, 40% less memory

---

### 02-dynamic-list-no-optimization.md ❌

**Optimize**: None (stays Compose - correct!)

**When**:
- Collection is `var` with `mutableStateOf`
- Has mutations
- Has event handlers
- Confidence: 0/100

**Result**: No optimization (Compose is better here)

---

## Testing

```bash
# Phase 0-4: Should generate Unoptimized Output
cargo test --test optimization_examples

# Phase 5+: Should generate Optimized Output when optimization enabled
cargo test --test optimization_examples --features optimizations
```

## Adding Examples

1. Create `XX-name.md`
2. Add Input, Unoptimized Output, Optimized Output, Metadata sections
3. Keep it concise and testable
4. Update this README

Both outputs are always testable - no time-based "future" code!
