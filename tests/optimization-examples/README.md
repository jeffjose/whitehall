# Whitehall Optimization Examples

Testable examples showing transparent automatic optimizations.

## Format

Each example has:
- **Input**: Whitehall `.wh` source code
- **Output** (Current/Unoptimized): What Phase 0-4 generates (always Compose)
- **Output** (Future/Optimized): What Phase 5+ will generate (RecyclerView when safe)
- **Metadata**: File info, optimization type, confidence score

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
# Phase 0-4: Should generate Compose (Current Output section)
cargo test --test optimization_examples

# Phase 5+: Example 01 should generate RecyclerView (Future Output section)
cargo test --test optimization_examples --features phase5
```

## Adding Examples

1. Create `XX-name.md`
2. Add Input, Output (current), Output (future), Metadata sections
3. Keep it concise and testable
4. Update this README
