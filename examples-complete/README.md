# Complete Application Examples

These applications combine multiple concepts from `examples/`.

**Study the focused examples first!** Each numbered example in `examples/` teaches ONE concept in isolation. Once you understand those building blocks, come here to see how they work together in real applications.

## Applications

### weather-app/
**Combines:** async-loading + animations + lazy-lists
- Fetches weather data from API (see example 5-async-loading)
- Loading animations and transitions (see example 7-animations)
- Forecast list with LazyColumn (see example 11-lazy-lists)

### notes-app/
**Combines:** CRUD operations + filtering + viewmodel
- Todo-style CRUD (see example 2-todo-list)
- Search and filtering (see example 12-search-filter)
- ViewModel architecture (see example 13-viewmodel-pattern)

### calculator/
**Combines:** viewmodel + component composition
- State machine pattern (see example 13-viewmodel-pattern)
- Reusable button components (see example 15-component-composition)

### settings-app/
**Combines:** theming + forms + tabs
- Dark mode toggle (see example 8-theming)
- Form validation (see example 4-form-validation)
- Organized with tabs (see example 9-tabs)

### profile-editor/
**Combines:** forms + validation + state management
- Form inputs (see example 3-text-input)
- Validation patterns (see example 4-form-validation)
- Save/cancel state management

## How to Use These Examples

1. **Don't start here** - Go through `examples/1-*` through `examples/17-*` first
2. **Look for comments** - Code includes references like `// See example 7 for animations`
3. **Compare and contrast** - See how simple concepts combine into full apps
4. **Build your own** - Use these as templates for your own applications

## Building

These are full Whitehall projects with `whitehall.toml` configuration:

```bash
cd weather-app
whitehall build
```
