# Whitehall Examples

This directory contains **12 focused examples** that teach Whitehall concepts **one at a time**. Each example is a single file (~50-70 lines) demonstrating exactly one concept without cognitive overload.

## 📚 Learning Path

Start from example 01 and work through sequentially. Each builds on concepts from previous examples.

### Fundamentals (01-04)

**01-button-counter** - Buttons and State
- Teaches: `var` state, `Button` onClick, `@if` conditionals
- Complexity: Single file, ~60 lines
- **Start here** if new to Whitehall

**02-task-list** - List Operations
- Teaches: `mutableListOf`, add/remove operations, `@for` loops
- Complexity: Single file, ~70 lines
- Builds on: State from example 01

**03-text-input** - TextField Binding
- Teaches: `TextField`, two-way binding with `value`/`onValueChange`
- Complexity: Single file, ~45 lines
- Builds on: State management

**04-form-validation** - Form Validation
- Teaches: Input validation, error messages, form submission
- Complexity: Single file, ~70 lines
- Builds on: Text input from example 03

### UI Patterns (05-10)

**05-navigation** - Screen Switching
- Teaches: Changing screens with state variables
- Complexity: Single file, ~75 lines
- Builds on: State and conditionals

**06-async-data** - Async Loading
- Teaches: `suspend` functions, loading states, `delay`
- Complexity: Single file, ~50 lines
- **First async example**

**07-dialogs** - Dialog Windows
- Teaches: `AlertDialog`, modal interactions, `onDismissRequest`
- Complexity: Single file, ~75 lines
- Builds on: State for show/hide

**08-animations** - Animated Visibility
- Teaches: `AnimatedVisibility`, `fadeIn`/`fadeOut`, `animateContentSize`
- Complexity: Single file, ~60 lines
- Imports: androidx.compose.animation.*

**09-theming** - Dark Mode
- Teaches: `MaterialTheme`, `darkColorScheme`/`lightColorScheme`, theme switching
- Complexity: Single file, ~70 lines
- Imports: androidx.compose.material3.* theming

**10-tabs** - Tab Switching
- Teaches: `TabRow`, `Tab` selection, content switching
- Complexity: Single file, ~60 lines
- Imports: androidx.compose.material3.TabRow

### Advanced Patterns (11-12)

**11-lazy-lists** - LazyColumn Performance
- Teaches: `LazyColumn` for efficient list rendering
- Complexity: Single file, ~40 lines
- Builds on: Lists from example 02
- **Performance focused**

**12-component-patterns** - Reusable UI Patterns
- Teaches: Extracting repeated UI patterns for reusability
- Complexity: Single file, ~55 lines
- Builds on: All previous concepts

## 🎯 Design Principles

### One Concept Per Example

Each example teaches **exactly one thing**:
- ✅ 03-text-input: TextField only
- ❌ Not: TextField + validation + forms + database

### No Cognitive Overload

Examples are **deliberately simple**:
- Single file (except when multi-file is the concept)
- 40-75 lines
- No mixing of unrelated concepts
- Clear focus stated in comments

### Progressive Complexity

Concepts build logically:
```
State → Lists → Input → Validation
  ↓
Navigation → Async → Dialogs
  ↓
Animations → Theming → Tabs
  ↓
Performance → Patterns
```

## 🚀 Next Steps

After completing all 12 examples, move to **`examples-complete/`** to see how these concepts combine into real applications:

- **weather-app** - Combines examples 06 (async) + 08 (animations) + 11 (lazy lists)
- **notes-app** - Combines examples 02 (lists) + 04 (validation) + 05 (navigation)
- **calculator** - State management patterns
- **settings-app** - Combines examples 09 (theming) + 10 (tabs) + 04 (forms)
- **profile-editor** - Form management and validation

## 🔨 Building Examples

Build all examples at once:
```bash
bash scripts/build-example-apps.sh
```

Build a single example:
```bash
cargo run -- build examples/01-button-counter/main.wh
```

## 📖 Documentation

- **LANGUAGE-REFERENCE.md** - Complete Whitehall language reference
- **PLAN-EXAMPLES-APPS.md** - Design philosophy and restructuring plan
- **REFACTOR-PROGRESS.md** - Implementation progress tracking

## 💡 Tips for Learning

1. **Read the file header** - Every example starts with comments explaining what it teaches
2. **Build and run** - See the app in action on Android
3. **Modify and experiment** - Change values, add buttons, break things!
4. **One concept at a time** - Don't skip ahead. Master each before moving on
5. **Check examples-complete/** - See how concepts combine in real apps

## 🤝 Contributing

When adding new examples, follow these rules:

1. **One concept only** - If teaching two things, make two examples
2. **Keep it simple** - 40-75 lines maximum
3. **Single file** - Unless multi-file IS the concept
4. **Clear header** - Explain what, why, and focus
5. **Test it builds** - Use `scripts/build-example-apps.sh`
