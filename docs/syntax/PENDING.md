# Pending Decisions & Topics

This document tracks syntax and design decisions that are **not yet finalized** but need to be addressed.

---

## High Priority

### 1. Auto-Import for Compose Primitives

**Question:** Should common Compose widgets be auto-imported?

**Options:**
- **Option A:** Auto-import common widgets (Column, Row, Text, Button, Card, etc.)
  - Pros: Less boilerplate, faster to write
  - Cons: Less explicit, might be confusing where they come from

- **Option B:** Require explicit imports
  - Pros: Clear, explicit, standard Kotlin
  - Cons: Verbose, repetitive

**Impact:** Affects every .wh file

**Status:** ⏸️ Awaiting decision

---

### ~~2. Data Flow & State Management~~

**Status:** ✅ DECIDED - See [Decision 008: State Management](./decisions/008-state-management.md)

**Implemented:**
- ✅ Inline component state: `var count = 0` (auto remember/mutableStateOf)
- ✅ Auto-ViewModel generation for complex components (Phase 1.1)
- ✅ @store classes for screen-level state
- ✅ Automatic StateFlow/UiState generation
- ✅ Hilt integration (@Inject or @hilt)
- ✅ Props and state hoisting patterns
- ✅ Suspend function auto-wrapping in viewModelScope

**See also:** `/docs/STORE.md` for complete documentation

---

### 3. Svelte Runes/Signals - Needed?

**Context:** Svelte 5 introduced "runes" for reactivity (`$state`, `$derived`, `$effect`)

**Questions:**
- Do we need similar reactivity primitives?
- Or is Compose's `remember { mutableStateOf() }` sufficient?
- Would `$state`, `$derived`, `$effect` make sense in Whitehall?

**Compose already has:**
- `var count by remember { mutableStateOf(0) }` - reactive state
- `val doubled = count * 2` - derived values (auto-recomputes)
- `LaunchedEffect`, `DisposableEffect` - side effects

**Potential Whitehall syntax:**
```whitehall
<script>
  // Option A: Use Compose patterns directly
  var count = 0  // Transpiles to mutableStateOf
  val doubled = count * 2  // Derived value

  // Option B: Svelte-style runes
  $state var count = 0
  $derived val doubled = count * 2
  $effect {
    console.log("Count changed: $count")
  }
</script>
```

**Status:** ⏸️ Need to decide if Compose patterns are enough

---

### 4. Event Handling

**Topics:**
- Event propagation (stop propagation, prevent default)
- Event modifiers (like Vue: `@click.stop`, `@click.prevent`)
- Custom events from child to parent
- Event delegation

**Compose approach:**
```kotlin
Button(onClick = { handleClick() })
```

**Potential Whitehall enhancements:**
```whitehall
<!-- Simple -->
<Button onClick={handleClick} />

<!-- With event object -->
<Button onClick={(event) => {
  event.stopPropagation()
  handleClick()
}} />

<!-- Modifiers (like Vue)? -->
<Button onClick.stop={handleClick} />
<Button onClick.prevent={handleClick} />
```

**Status:** ⏸️ Need to design event handling patterns

---

## Medium Priority

### 5. Mixing C++ / Native Code

**Question:** Can .wh files use JNI/C++ code?

**Scenarios:**
- Performance-critical code (image processing, crypto)
- Existing C++ libraries
- Native Android NDK

**Potential approach:**
```whitehall
import $app.native.ImageProcessor  // C++ via JNI

<script>
  fun processImage(bitmap: Bitmap): Bitmap {
    return ImageProcessor.process(bitmap)
  }
</script>
```

**Implementation:**
- Use standard Kotlin/JNI interop
- No special .wh syntax needed
- Just import the Kotlin wrapper

**Status:** ⏸️ Probably just use standard Kotlin/JNI - no special handling needed

---

### 6. Using Kotlin Compose As-Is with .wh Syntax

**Question:** Can I use raw Compose code inside .wh files?

**Potential approaches:**

**Option A: Escape hatch blocks**
```whitehall
<script>
  var items = listOf("A", "B", "C")
</script>

<!-- Mix .wh syntax with raw Compose -->
<Column>
  <Text>Header</Text>

  <!-- Escape to raw Compose -->
  @compose {
    LazyColumn(
      modifier = Modifier.weight(1f),
      state = rememberLazyListState()
    ) {
      items(items) { item ->
        Text(item)
      }
    }
  }
</Column>
```

**Option B: Just use Compose directly**
```whitehall
<script>
  fun CustomComposable() {
    LazyColumn(
      modifier = Modifier.weight(1f)
    ) {
      items(items) { item ->
        Text(item)
      }
    }
  }
</script>

<Column>
  <Text>Header</Text>
  {CustomComposable()}
</Column>
```

**Option C: No escape hatch - transpile everything**
- All UI must use .wh markup syntax
- Can call Compose functions from `<script>`
- But can't inline raw Compose in markup

**Status:** ⏸️ Need to decide if/how to support raw Compose

---

### ~~7. Lifecycle Hooks~~

**Status:** ✅ DECIDED - See [Decision 009: Lifecycle Hooks](./decisions/009-lifecycle-hooks.md)

**Implemented syntax:**
```whitehall
<script>
  onMount {
    // Run once when component mounts
    // For ViewModels: goes in init {} block with viewModelScope.launch
    // For simple components: transpiles to LaunchedEffect(Unit)
  }

  onDispose {
    // Cleanup when component unmounts
    // For ViewModels: not supported (ViewModels have onCleared)
    // For simple components: transpiles to DisposableEffect(Unit)
  }
</script>
```

**Smart handling:**
- Complex components with lifecycle → ViewModel with init block
- Simple components with lifecycle → LaunchedEffect/DisposableEffect

**Status:** ✅ Fully implemented and tested (38/38 tests passing)

---

## Lower Priority

### 8. Component Name Conventions

**Questions:**
- PascalCase enforced?
- Filename must match component name?
- One component per file?

**Status:** ⏸️ Pending

---

### 9. Private Components

**Question:** Support file-scoped private components?

```whitehall
private component Helper { }

component Public {
  <Helper />
}
```

**Status:** ⏸️ Pending

---

### 10. CSS/Styling System

**Question:** Do we need a `<style>` section?

**Options:**
- Inline styles only (Compose Modifiers)
- CSS-like `<style>` section that transpiles to Modifiers
- No styling in .wh (use Compose themes)

**Status:** ⏸️ Pending (lower priority)

---

### 11. Animation & Transitions

**How to expose Compose animations?**

```whitehall
<script>
  var visible = true
</script>

<!-- Option A: Declarative -->
<AnimatedVisibility visible={visible}>
  <Text>Hello</Text>
</AnimatedVisibility>

<!-- Option B: Modifier-based -->
<Text animate:fadeIn={visible}>Hello</Text>
```

**Status:** ⏸️ Pending (Phase 2+)

---

### 12. Testing Support

**How to write tests for .wh components?**

```kotlin
// Can we test .wh components like Compose?
@Test
fun testButton() {
  composeTestRule.setContent {
    MyButton(text = "Click")
  }
  composeTestRule.onNodeWithText("Click").performClick()
}
```

**Status:** ⏸️ Pending (Phase 2+)

---

## How to Track Decisions

When a topic is decided:
1. Create a decision document in `docs/syntax/decisions/`
2. Update this file to link to the decision
3. Remove from pending list

**Example:**
```markdown
### ~~1. Auto-Import for Compose Primitives~~

**Status:** ✅ Decided - See [Decision 008](./decisions/008-auto-imports.md)
```
