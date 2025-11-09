# Complete Application Examples

These applications combine multiple concepts from `examples/`.

**Study the focused examples first!** Each numbered example in `examples/` teaches ONE concept in isolation. Once you understand those building blocks, come here to see how they work together in real applications.

## Applications

### weather-app/
**Combines:** async-loading + animations + lazy-lists
- Fetches weather data from API (see example 06-async-data)
- Loading animations and transitions (see example 08-animations)
- Forecast list with LazyColumn (see example 11-lazy-lists)

### notes-app/
**Combines:** CRUD operations + lists + state management
- Todo-style CRUD operations (see example 02-task-list)
- List rendering and management (see example 11-lazy-lists)
- State management patterns

### calculator/
**Combines:** state management + component patterns
- State machine pattern for calculator logic
- Reusable button components (see example 12-component-patterns)
- Complex state updates

### settings-app/
**Combines:** theming + forms + tabs
- Dark mode toggle (see example 09-theming)
- Form validation (see example 04-form-validation)
- Organized with tabs (see example 10-tabs)

### profile-editor/
**Combines:** forms + validation + state management
- Form inputs (see example 03-text-input)
- Validation patterns (see example 04-form-validation)
- Save/cancel state management

## How to Use These Examples

1. **Don't start here** - Go through `examples/01-*` through `examples/12-*` first
2. **Look for comments** - Code includes references like `// See example 08 for animations`
3. **Compare and contrast** - See how simple concepts combine into full apps
4. **Build your own** - Use these as templates for your own applications

## Building

These are full Whitehall projects with `whitehall.toml` configuration:

```bash
cd weather-app
cargo run --manifest-path ../../Cargo.toml -- build .
```

Or build from project root:
```bash
cargo run -- build examples-complete/weather-app
```

## Learning Path

```
examples/           → Learn concepts one at a time
  ↓
examples-complete/  → See how concepts work together
  ↓
Your own apps!      → Build something amazing
```

## What Makes These "Complete"?

Unlike focused examples that teach one concept, these apps:
- **Combine multiple patterns** from different examples
- **Handle edge cases** and error states
- **Use realistic data structures** and state management
- **Show production patterns** for organizing code

## From Examples to Complete Apps

See how focused examples evolve:

**Example 02 (task-list)**: Simple add/remove
```whitehall
var tasks = mutableListOf("Task 1")
<Button onClick={() => tasks.add("New")} />
```

**Complete App (notes-app)**: Full CRUD + categories + search
```whitehall
data class Note(id, title, content, category)
var notes = mutableStateListOf<Note>()
fun addNote() { /* validation, categories, timestamps */ }
fun deleteNote(id) { /* confirm dialog, cleanup */ }
```

The focused example teaches the concept. The complete app shows real-world usage.
