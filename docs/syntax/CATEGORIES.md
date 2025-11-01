# Syntax Design Categories

This directory contains syntax explorations and comparisons for Whitehall.

## Critical Categories (Priority 1)

These foundational decisions affect everything else:

### 1. Component Definition
**Why critical:** The basic unit of all Whitehall code.

Key decisions:
- How to declare components
- Props/parameters syntax
- State declaration
- Component composition
- Export/visibility

**File:** [01-component-definition.md](./01-component-definition.md)

---

### 2. UI/View DSL
**Why critical:** 90% of code will be UI declarations.

Key decisions:
- Layout primitives (Column, Row, etc.)
- Modifier syntax (padding, styling)
- Conditional rendering
- List rendering
- How close to mirror Jetpack Compose
- Event handlers

**File:** [02-ui-view-dsl.md](./02-ui-view-dsl.md)

---

### 3. Routing & Navigation
**Why critical:** Defines app structure and DX workflow.

Key decisions:
- File-based vs explicit routing
- Route parameters
- Navigation between screens
- Deep linking syntax
- Interop with Compose Navigation

**File:** [03-routing.md](./03-routing.md)

---

## Secondary Categories (Priority 2)

Will be addressed after foundational syntax is established:

- State Management (local, global, persistence)
- Type System (nullability, generics, data classes)
- Async/Concurrency (coroutines mapping)
- Interop & Imports
- Lifecycle & Effects
- Styling/Theming

---

## How to Use This Directory

Each file contains:
1. **Context** - What we're deciding and why it matters
2. **Options** - A/B/C comparisons of different syntax approaches
3. **Examples** - Real-world code in each syntax variant
4. **Trade-offs** - Pros/cons of each approach
5. **Decision** - Current choice (can evolve)

The `examples/` directory contains complete mini-apps in different syntax variants for holistic comparison.
