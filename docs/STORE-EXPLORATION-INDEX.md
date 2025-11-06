# Whitehall @store Implementation - Exploration Index

This directory contains comprehensive documentation of the @store implementation in Whitehall, generated from a complete codebase exploration.

## Document Guide

### 1. [STORE-ARCHITECTURE.md](STORE-ARCHITECTURE.md) - MAIN DOCUMENT
**Size:** 29 KB, 1018 lines
**Audience:** Architects, implementers, refactorers
**Content:**
- Executive summary of all 5 phases
- Phase-by-phase breakdown with exact code locations
- Complete data flow diagrams
- Working example with generated output
- Key architectural insights
- Testing & verification procedures
- Refactoring opportunities for future phases

**Start here if:** You want complete understanding or need to refactor

### 2. [STORE-CODE-REFERENCE.md](STORE-CODE-REFERENCE.md) - QUICK REFERENCE
**Size:** 16 KB, 400+ lines
**Audience:** Developers, debuggers
**Content:**
- File structure with line numbers
- Critical code sections with full snippets
- Key decision points
- Troubleshooting guide
- Testing commands

**Start here if:** You need to find specific code or debug

### 3. [STATE-MANAGEMENT.md](STATE-MANAGEMENT.md) - DESIGN DOCUMENT
**Size:** Original design and roadmap
**Audience:** Product designers, decision makers
**Content:**
- Complete design decisions for Phases 0-5
- Examples of input/output transformations
- Migration path and breaking changes
- Future phases (6-7) planning
- Global state design considerations

**Start here if:** You want to understand the design rationale

## Phase Reference

| Phase | Status | Document | Code |
|-------|--------|----------|------|
| 0 | ✅ Complete | STORE-ARCHITECTURE.md | analyzer.rs:28-298 |
| 1 | ✅ Complete | STORE-ARCHITECTURE.md | compose.rs:2545-2684 |
| 2 | ✅ Complete | STORE-ARCHITECTURE.md | compose.rs:103-378 |
| 3 | ✅ Complete | STORE-ARCHITECTURE.md | parser.rs:612-630 |
| 4 | ✅ Complete | STORE-ARCHITECTURE.md | parser.rs:519-525 |
| 5 | ✅ Complete | STORE-ARCHITECTURE.md | analyzer.rs:276-285 |

## Key Concepts

### Store Registry
- **What:** HashMap of @store classes with metadata
- **Where:** analyzer.rs:28-62
- **Why:** Enable cross-file detection
- **How:** Built during semantic analysis

### Two Generation Paths
1. **Store Definition:** File has @store class → generate ViewModel
2. **Store Usage:** File uses store → detect and generate viewModel<T>()

### Hilt Hybrid Detection
- **Automatic:** Either @hilt annotation OR @Inject constructor enables Hilt
- **Transparent:** No annotation needed at usage site
- **Smart:** Selects hiltViewModel<T>() vs viewModel<T>()

### StateFlow Pattern
```kotlin
data class UiState(val name: String = "")
private val _uiState = MutableStateFlow(UiState())  // private, mutable
val uiState: StateFlow<UiState> = _uiState.asStateFlow()  // public, immutable
```

## Quick Navigation

### I want to understand...

**The architecture:**
→ Read STORE-ARCHITECTURE.md Executive Summary + Phase 0

**How store registry works:**
→ Read STORE-CODE-REFERENCE.md Section 1-2 + STORE-ARCHITECTURE.md Phase 0

**How @store classes are parsed:**
→ Read STORE-CODE-REFERENCE.md Section 3-4 + STORE-ARCHITECTURE.md @STORE Annotation Parsing

**How ViewModel code is generated:**
→ Read STORE-CODE-REFERENCE.md Section 8 + STORE-ARCHITECTURE.md Phase 1

**How stores are detected at usage sites:**
→ Read STORE-CODE-REFERENCE.md Section 6-7 + STORE-ARCHITECTURE.md Phase 2

**How Hilt is integrated:**
→ Read STORE-ARCHITECTURE.md Phase 5 + STORE-CODE-REFERENCE.md Decision 1

**The complete example:**
→ Read STORE-ARCHITECTURE.md Working Example: Counter Store

**Where specific code is:**
→ Use STORE-CODE-REFERENCE.md File Structure + Critical Code Locations

**How to debug an issue:**
→ Use STORE-CODE-REFERENCE.md Troubleshooting

### I want to...

**Refactor the store implementation:**
→ Read STORE-ARCHITECTURE.md Key Architectural Insights + Refactoring Opportunities

**Add a new feature:**
→ Read STORE-ARCHITECTURE.md Next Steps for Implementation + STORE-CODE-REFERENCE.md Key Decision Points

**Write tests:**
→ Read STORE-ARCHITECTURE.md Testing & Verification

**Understand the design decisions:**
→ Read STATE-MANAGEMENT.md Decisions Made sections

## File Map

```
src/transpiler/
├── ast.rs              : ClassDeclaration, PropertyDeclaration structures
├── parser.rs           : @store annotation & class parsing
├── analyzer.rs         : StoreRegistry & semantic analysis
└── codegen/
    └── compose.rs      : ViewModel & usage site generation

tests/
└── transpiler-examples/
    ├── 27-hilt-stores.md          : @store with @Inject
    ├── 28-hilt-explicit.md         : @store with @hilt
    └── 29-store-no-hilt.md         : Regular @store

examples/
└── counter-store/
    ├── src/stores/CounterStore.wh      : Store definition
    ├── src/components/CounterScreen.wh : Store usage
    └── build/                           : Generated code

docs/
├── STATE-MANAGEMENT.md          : Design document & roadmap
├── STORE-ARCHITECTURE.md        : Complete analysis (THIS REPO)
├── STORE-CODE-REFERENCE.md      : Code snippets & quick reference (THIS REPO)
└── STORE-EXPLORATION-INDEX.md   : This file
```

## Key Code Locations

### Parser (parser.rs)
- Annotation detection: 62-81
- Class parsing: 480-542
- Property with getters: 587-651
- Constructor with @Inject: 544-579

### Analyzer (analyzer.rs)
- StoreRegistry: 28-62
- collect_stores(): 266-298

### Codegen (compose.rs)
- detect_store_instantiation(): 103-121
- Store usage detection: 125-135, 358-378
- viewModel<T>() generation: 367-378
- ViewModel generation: 2545-2684
- Hilt detection: 2561-2581
- Suspend wrapping: 2663-2673

## Testing

Run all tests:
```bash
cd /home/jeffjose/scripts/whitehall
cargo test transpiler_examples_test
```

Run store tests only:
```bash
cargo test transpiler_examples_test -- 27\|28\|29
```

Run working example:
```bash
whitehall run examples/counter-store/
```

## Implementation Status

| Feature | Status | Phase |
|---------|--------|-------|
| @store annotation parsing | ✅ | 0 |
| Store registry | ✅ | 0 |
| ViewModel generation | ✅ | 1 |
| UiState data class | ✅ | 1 |
| Property accessors | ✅ | 1 |
| Store usage detection | ✅ | 2 |
| viewModel<T>() generation | ✅ | 2 |
| collectAsState() | ✅ | 2 |
| Derived properties | ✅ | 3 |
| Suspend function wrapping | ✅ | 4 |
| @Inject detection | ✅ | 5 |
| @HiltViewModel generation | ✅ | 5 |
| hiltViewModel<T>() selection | ✅ | 5 |

## Notes

- **Exploration Date:** November 6, 2025
- **Analyzed Commits:** 
  - f86ad5c: Update STATE-MANAGEMENT.md: Lifecycle hooks already implemented
  - 7cd0943: Update STATE-MANAGEMENT.md: Phase 5 complete
  - acf1f38: Simplify toolchain clean prompt to y/n
  - 6a9f61b: Fix type inference for @store properties
  - c4f7716: Add additional tests for Hilt scenarios
  
- **Confidence Level:** VERY HIGH
  - Implementation is production-ready
  - Well-tested with multiple test cases
  - Demonstrated with working example
  - Clean architecture with clear separation of concerns

## Next Steps

For Phases 6-7 planning, see:
- STATE-MANAGEMENT.md: "Proposed Design Change" section
- STORE-ARCHITECTURE.md: "Next Steps for Implementation/Refactoring"

For refactoring guidance, see:
- STORE-ARCHITECTURE.md: "Refactoring Opportunities"
- STORE-CODE-REFERENCE.md: "Key Decision Points"
