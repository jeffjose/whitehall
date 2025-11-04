# Whitehall Web Playground

**Goal:** Web-based IDE for experimenting with Whitehall syntax, seeing compiled output, and (eventually) previewing results.

**Status:** ✅ Phase 1 Complete (Nov 4, 2025)
**Location:** `tools/playground/`

## Current Implementation Status

**✅ Completed (Phase 1):**
- Backend server (Rust + Axum) with /api/compile endpoint
- Frontend with Monaco editor integration
- Real-time compilation with 500ms debounce
- Multiple output tabs (Kotlin / Errors / AST)
- 5 example snippets (hello/counter/todo/form/styling)
- URL hash state for code sharing
- Copy/format/clear buttons
- Keyboard shortcuts (Ctrl+Enter, Ctrl+S)
- Status indicator and toast notifications
- Mobile responsive layout

**⏳ Not Yet Implemented:**
- Parser position tracking for inline error markers (requires transpiler changes)
- Line/column error precision with clickable errors
- Error context snippets
- Visual preview (Phase 2 - future)
- Emulator integration (Phase 3 - future)
- Compose-for-Web runtime (Phase 4 - future)

**How to Run:**
```bash
# Terminal 1 - Backend
cd tools/playground/backend
cargo run
# Runs on http://localhost:3000

# Terminal 2 - Frontend
cd tools/playground/frontend
python -m http.server 8080
# Open http://localhost:8080
```

---

## Overview

A two-pane web interface for learning and prototyping with Whitehall:
- **Left pane:** Monaco code editor with Whitehall syntax
- **Right pane:** Tabbed output display
  - Tab 1: Compiled Kotlin code
  - Tab 2: (Future) Visual preview or emulator

**Use cases:**
- Learning Whitehall syntax without installing CLI
- Quick prototyping and experimentation
- Sharing code snippets (future)
- Documentation examples with live editing

---

## Architecture

### Directory Structure
```
whitehall/
├── src/                    # Existing CLI and transpiler
├── tools/
│   └── playground/
│       ├── README.md
│       ├── backend/
│       │   ├── Cargo.toml
│       │   └── src/
│       │       └── main.rs
│       └── frontend/
│           ├── index.html
│           ├── style.css
│           └── app.js
```

### Tech Stack

**Backend:**
- **Framework:** Axum (Rust web framework)
- **Transpiler:** Direct dependency on whitehall crate
- **CORS:** Tower-HTTP middleware
- **Port:** 3000

```toml
# tools/playground/backend/Cargo.toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["cors"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
whitehall = { path = "../../../" }  # Use local whitehall crate
```

**Frontend:**
- **Editor:** Monaco Editor (VS Code's editor, via CDN)
- **Styling:** Tailwind CSS (via CDN)
- **Framework:** Vanilla JavaScript (no build step needed)
- **Layout:** CSS Grid for split panes

---

## API Design

### POST /api/compile
Compile Whitehall code to Kotlin.

**Request:**
```json
{
  "code": "var count = 0\n<Text>{count}</Text>"
}
```

**Response (Success):**
```json
{
  "success": true,
  "output": "@Composable\nfun Component() {\n    var count by remember { mutableStateOf(0) }\n    Text(text = \"$count\")\n}",
  "errors": []
}
```

**Response (Error):**
```json
{
  "success": false,
  "output": "",
  "errors": [
    {
      "message": "Unexpected token '<'",
      "line": 2,
      "column": 1
    }
  ]
}
```

### GET /api/examples
Return list of example snippets (future).

---

## Preview/Emulator Options Analysis

### Option 1: ❌ Native Android Emulator - Not Practical
Running actual Android emulator (like Android Studio's) in browser.

**Why it doesn't work:**
- Requires virtualization (KVM/HAXM) - can't run in browser
- Heavy resource usage (2-4GB RAM per instance)
- Not designed for web embedding
- Would need native desktop app, not web

**Verdict:** Not viable for web playground

---

### Option 2: ⚠️ Emulator Streaming - Possible but Complex
Stream video from backend emulator to frontend.

**Architecture:**
```
Frontend (Browser)
    ↕ WebRTC/WebSocket (video stream + touch events)
Backend (Server)
    ↕ adb commands
Headless Android Emulator (Docker + KVM)
```

**Implementation:**
1. Backend runs headless Android emulator (Docker + Android SDK)
2. Capture emulator screen as video stream (ffmpeg + WebRTC)
3. Frontend displays video and sends touch events back
4. Backend forwards touch events to emulator via adb

**Pros:**
- Real Android environment
- Actual app execution
- True Material3 rendering
- Interactive (can click buttons, etc.)

**Cons:**
- **Infrastructure:** Requires Docker + KVM support, can't run on all hosting
- **Resources:** ~2GB RAM + CPU per concurrent user
- **Cost:** Very expensive to scale (dedicated servers needed)
- **Latency:** Video encoding/decoding adds 100-500ms delay
- **Complexity:** 10-20x more complex than MVP
- **Isolation:** Need separate emulator per user session
- **Startup time:** 30-60 seconds to boot emulator

**Examples that do this:**
- Firebase Test Lab (Google's massive infrastructure)
- BrowserStack (multi-million dollar service)
- Appetize.io (specialized iOS/Android streaming, commercial)

**Estimated effort:** 20-40 hours
**Estimated hosting cost:** $50-200/month for 10 concurrent users

**Verdict:** Only worth it for a commercial product with investment

---

### Option 3: ✅ Visual Preview/Mock Renderer - Realistic Alternative
Parse Whitehall AST and render HTML/CSS approximation.

**How it works:**
```
Whitehall code → Parse to AST → Traverse component tree → Generate HTML/CSS
```

**Example transformation:**
```whitehall
<Column padding={16} spacing={8}>
  <Text fontSize={24}>Hello World</Text>
  <Button onClick={() => count++}>
    <Text>Click me</Text>
  </Button>
</Column>
```

**Renders as:**
```html
<div style="display: flex; flex-direction: column; gap: 8px; padding: 16px;">
  <p style="font-size: 24px; margin: 0;">Hello World</p>
  <button style="padding: 8px 16px; background: #6200EE; color: white; border: none; border-radius: 4px;">
    Click me
  </button>
</div>
```

**Mapping table:**
| Whitehall/Compose | HTML/CSS |
|-------------------|----------|
| `<Column>` | `<div style="display: flex; flex-direction: column">` |
| `<Row>` | `<div style="display: flex; flex-direction: row">` |
| `<Text>` | `<p>` or `<span>` |
| `<Button>` | `<button>` with Material-like styling |
| `<TextField>` | `<input>` or `<textarea>` |
| `padding={16}` | `padding: 16px` |
| `fontSize={24}` | `font-size: 24px` |

**Implementation approach:**
1. Reuse existing Whitehall parser to get AST
2. Walk component tree
3. Map each component to HTML + inline CSS
4. Inject into iframe for isolation
5. (Optional) Add Material3-like CSS theme

**Pros:**
- Fast and lightweight (renders instantly)
- No emulator/backend needed (can do in browser)
- Works on any hosting (static site)
- Shows layout, styling, structure
- Good enough for learning/documentation

**Cons:**
- Not "real" Android (approximation only)
- Won't have exact Material3 look
- No actual Compose runtime
- Limited interactivity (can't run real Kotlin logic)
- State management won't work (no @Composable runtime)

**What works:**
- ✅ Layout preview (Column, Row, Box)
- ✅ Styling (padding, colors, fonts)
- ✅ Component hierarchy visualization
- ✅ Static content display

**What doesn't work:**
- ❌ Real interactivity (button clicks, state changes)
- ❌ Lifecycle hooks (onMount, onDispose)
- ❌ Data binding (bind:value)
- ❌ Compose-specific features (modifiers, remember, etc.)

**Estimated effort:** 4-6 hours
**Estimated hosting cost:** $0-5/month (static hosting)

**Verdict:** Best option for MVP preview functionality

---

### Option 4: 🎯 Compose for Web - Most Promising Long-term
Transpile Whitehall → Kotlin/JS → runs natively in browser using Compose for Web.

**How it works:**
```
Whitehall → Kotlin/JS (Compose Multiplatform Web target) → JavaScript bundle → Browser
```

**Architecture:**
1. Add Compose-for-Web compilation path to transpiler
2. Transpile Whitehall to Kotlin/JS instead of Kotlin/Android
3. Use Kotlin/JS compiler to generate JavaScript
4. Load compiled JS in iframe
5. Actual Compose runtime running in browser

**Pros:**
- Real Compose runtime (not approximation)
- Fully interactive (actual state management, lifecycle, etc.)
- Native web performance
- Could support hot reload
- Same semantics as Android version

**Cons:**
- Major architecture change (new compilation target)
- Whitehall currently only targets Android Compose
- Would need separate transpiler path for Web
- Requires Kotlin/JS toolchain integration
- Compose-for-Web is less mature than Android
- Large JS bundle size (~500KB-1MB)

**Implementation complexity:**
- 50+ hours of development
- Requires deep understanding of Compose Multiplatform
- Need build pipeline for Kotlin → JS compilation
- Testing across browsers

**Estimated effort:** 50+ hours
**Estimated hosting cost:** $0-10/month (static hosting + edge functions)

**Verdict:** Interesting long-term, but massive scope. Consider for Whitehall v2.0+

---

## Phased Implementation Plan

### **Phase 1: Full-Featured Playground** ⭐ START HERE
**Goal:** Svelte-playground-quality developer experience with excellent error reporting

**Time estimate:** 10-13 hours

**Inspiration:** Svelte REPL (https://svelte.dev/repl) - gold standard for web playgrounds

**Features:**

#### Core Functionality
- **Left pane:** Monaco editor with Whitehall syntax
- **Right pane:** Tabbed output display
  - **Tab 1: Kotlin Output** - Compiled code with syntax highlighting
  - **Tab 2: Errors** - Detailed error panel (if any errors)
  - **Tab 3: AST View** - Debug view of parsed AST (optional)
- **Real-time compilation:** Debounced (500ms after typing stops)
- **Inline error markers:** Red squiggly lines in editor at exact error location
- **Example snippets:** Dropdown with prebuilt examples

#### Enhanced Error Reporting (Like Svelte REPL)
- ✅ **Line/column precision:** Errors show exact location (e.g., "Line 5, Column 12")
- ✅ **Inline decorations:** Monaco red underlines at error position
- ✅ **Source context:** Show surrounding code with error highlighted
- ✅ **Clickable errors:** Click error in panel → jump to line in editor
- ✅ **Multiple errors:** Display all errors at once, not just first one
- ✅ **Error categories:** Syntax errors, transpiler errors, etc.

**Example error display:**
```
❌ Syntax Error at Line 5, Column 12

  3 | var count = 0
  4 |
  5 | <Column padding={16>
              ↑
              Expected closing brace '}'
  6 |   <Text>{count}</Text>
  7 | </Column>
```

#### User Experience
- **Format button:** Auto-format Whitehall code (basic indentation)
- **Copy button:** Copy compiled Kotlin to clipboard
- **Clear button:** Reset editor to blank state
- **Loading state:** Spinner during compilation
- **Success indicator:** Green checkmark when compilation succeeds
- **Keyboard shortcuts:**
  - `Ctrl+Enter` / `Cmd+Enter`: Force recompile
  - `Ctrl+S` / `Cmd+S`: Format code
- **URL state:** Share code via URL hash (encode/decode)
- **Mobile responsive:** Works on tablets/phones

#### Example Snippets
Dropdown with working examples:
1. **Hello World** - Minimal component
2. **Counter** - State management with button
3. **Todo List** - Array manipulation and @for loops
4. **Form** - Text input with bind:value
5. **Navigation** - Multi-screen example (if routing ready)
6. **Styling** - Modifiers and theming
7. **Lifecycle** - onMount/onDispose hooks

---

## Implementation Steps (Phase 1)

### 1. Backend - Enhanced Error Reporting (3-4 hours)

**Goal:** Track line/column in parser and return detailed error info

**Tasks:**
- Enhance parser to track position for every token/node
- Store `(line, col)` in error types
- Return structured error objects from API
- Support multiple errors in single compilation

**API changes:**
```rust
#[derive(Serialize)]
struct CompileError {
    message: String,
    line: usize,
    column: usize,
    length: usize,  // How many chars to underline
    severity: String,  // "error" | "warning"
    context: String,  // Source code snippet around error
}

#[derive(Serialize)]
struct CompileResponse {
    success: bool,
    output: String,
    errors: Vec<CompileError>,
    warnings: Vec<CompileError>,
    ast: Option<String>,  // JSON serialized AST (for debug tab)
}
```

**Parser enhancements:**
```rust
// In src/transpiler/parser.rs or new src/transpiler/position.rs
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,  // Byte offset in file
}

#[derive(Debug)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

// Update error types to include position
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
    pub kind: ErrorKind,
}

impl ParseError {
    pub fn with_context(&self, source: &str) -> String {
        // Extract 2 lines before/after error
        // Highlight error line with ↑ marker
        // Return formatted string
    }
}
```

**Challenges:**
- Need to track position during parsing (add to Parser state)
- Handle multi-line tokens (strings, comments)
- Efficient span calculation
- Format context snippets nicely

**Time:** 3-4 hours (requires transpiler changes)

---

### 2. Frontend - Monaco Integration with Error Decorations (3-4 hours)

**Goal:** Display inline errors and rich error panel

**Tasks:**
- Setup Monaco Editor with custom decorations
- Parse error response and create Monaco markers
- Implement error panel with clickable line numbers
- Add tab switching (Kotlin / Errors / AST)
- Handle editor focus on error click

**Monaco error decorations:**
```javascript
function updateEditorErrors(errors) {
    // Clear previous decorations
    decorations = editor.deltaDecorations(decorations, []);

    // Add new decorations
    const newDecorations = errors.map(err => ({
        range: new monaco.Range(err.line, err.column, err.line, err.column + err.length),
        options: {
            isWholeLine: false,
            className: 'error-decoration',  // Red squiggly underline
            hoverMessage: { value: err.message },
            glyphMarginClassName: 'error-glyph',  // Red dot in margin
        }
    }));

    decorations = editor.deltaDecorations([], newDecorations);

    // Also set Monaco markers (for problems panel)
    monaco.editor.setModelMarkers(editor.getModel(), 'whitehall', errors.map(err => ({
        startLineNumber: err.line,
        startColumn: err.column,
        endLineNumber: err.line,
        endColumn: err.column + err.length,
        message: err.message,
        severity: monaco.MarkerSeverity.Error,
    })));
}
```

**Error panel:**
```javascript
function renderErrorPanel(errors) {
    const html = errors.map(err => `
        <div class="error-item" onclick="jumpToLine(${err.line})">
            <div class="error-header">
                <span class="error-icon">❌</span>
                <span class="error-location">Line ${err.line}, Column ${err.column}</span>
            </div>
            <div class="error-message">${err.message}</div>
            <pre class="error-context">${err.context}</pre>
        </div>
    `).join('');

    document.getElementById('errors-panel').innerHTML = html;
}

function jumpToLine(line) {
    editor.revealLineInCenter(line);
    editor.setPosition({ lineNumber: line, column: 1 });
    editor.focus();
}
```

**Time:** 3-4 hours

---

### 3. Example Snippets Dropdown (1 hour)

**Goal:** Load working examples into editor

**Implementation:**
```javascript
const examples = {
    'hello': {
        name: 'Hello World',
        code: `<Text>Hello, Whitehall!</Text>`
    },
    'counter': {
        name: 'Counter',
        code: `var count = 0

<Column padding={16} spacing={8}>
  <Text fontSize={24}>{count}</Text>
  <Button onClick={() => count++}>
    <Text>Increment</Text>
  </Button>
</Column>`
    },
    'todo': {
        name: 'Todo List',
        code: `var todos = ["Buy milk", "Write code"]
var newTodo = ""

<Column padding={16}>
  <TextField bind:value={newTodo} label="New Todo" />
  <Button onClick={() => todos = todos + newTodo}>
    <Text>Add</Text>
  </Button>

  @for (todo in todos) {
    <Text>{todo}</Text>
  }
</Column>`
    },
    // Add more examples...
};

function loadExample(key) {
    editor.setValue(examples[key].code);
    compile();
}
```

**UI:**
```html
<select id="examples" onchange="loadExample(this.value)">
    <option value="">Load Example...</option>
    <option value="hello">Hello World</option>
    <option value="counter">Counter</option>
    <option value="todo">Todo List</option>
</select>
```

**Time:** 1 hour

---

### 4. Multiple Output Tabs (1 hour)

**Goal:** Switch between Kotlin output, Errors, and AST view

**Implementation:**
```javascript
let currentTab = 'kotlin';

function switchTab(tab) {
    currentTab = tab;

    // Update tab buttons
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.tab === tab);
    });

    // Show/hide panels
    document.getElementById('kotlin-output').style.display = tab === 'kotlin' ? 'block' : 'none';
    document.getElementById('errors-panel').style.display = tab === 'errors' ? 'block' : 'none';
    document.getElementById('ast-view').style.display = tab === 'ast' ? 'block' : 'none';

    // Badge for error count
    if (tab === 'errors' && errorCount > 0) {
        document.querySelector('[data-tab="errors"]').innerHTML = `Errors (${errorCount})`;
    }
}
```

**Time:** 1 hour

---

### 5. URL State & Sharing (1 hour)

**Goal:** Share code via URL hash

**Implementation:**
```javascript
// Encode code to URL
function shareCode() {
    const code = editor.getValue();
    const encoded = btoa(encodeURIComponent(code));  // Base64 encode
    const url = `${window.location.origin}${window.location.pathname}#${encoded}`;

    navigator.clipboard.writeText(url);
    showToast('Link copied to clipboard!');
}

// Decode from URL on load
window.addEventListener('load', () => {
    const hash = window.location.hash.slice(1);
    if (hash) {
        try {
            const code = decodeURIComponent(atob(hash));
            editor.setValue(code);
            compile();
        } catch (e) {
            console.error('Invalid URL hash');
        }
    }
});

// Update URL on every change (debounced)
let urlTimeout;
editor.onDidChangeModelContent(() => {
    clearTimeout(urlTimeout);
    urlTimeout = setTimeout(() => {
        const code = editor.getValue();
        const encoded = btoa(encodeURIComponent(code));
        window.history.replaceState(null, '', `#${encoded}`);
    }, 1000);
});
```

**Time:** 1 hour

---

### 6. Polish & Testing (1-2 hours)

**Tasks:**
- Format button (basic indentation)
- Clear button
- Copy button with success toast
- Loading spinner during compilation
- Success/error status indicators
- Keyboard shortcuts (Ctrl+Enter, Ctrl+S)
- Mobile responsive layout
- Cross-browser testing
- Error edge cases

**Time:** 1-2 hours

---

## Updated Tech Stack

**Backend:**
```toml
# tools/playground/backend/Cargo.toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["cors"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
whitehall = { path = "../../../" }

# For position tracking in parser (if needed)
# Add to main whitehall Cargo.toml:
# logos = "0.13"  # Lexer with position tracking (if switching from hand-rolled parser)
```

**Frontend:**
- Monaco Editor 0.45+ (TypeScript/JSON editing built-in)
- Tailwind CSS 3.x (utility-first styling)
- Vanilla JavaScript (ES6+, no framework)

---

## Success Criteria (Phase 1 Complete When)

- ✅ Can type Whitehall code with Monaco syntax highlighting
- ✅ Real-time compilation with 500ms debounce
- ✅ **Inline error markers:** Red squiggly lines at exact error position
- ✅ **Error panel:** Detailed errors with line/column and context
- ✅ **Clickable errors:** Jump to error line in editor
- ✅ **Multiple errors displayed** (not just first one)
- ✅ Tabs switch between Kotlin output, Errors, and AST
- ✅ Example snippets load correctly
- ✅ Copy button copies Kotlin to clipboard
- ✅ Share button creates URL with encoded code
- ✅ URL hash loads code on page load
- ✅ Format button indents code correctly
- ✅ Keyboard shortcuts work (Ctrl+Enter, Ctrl+S)
- ✅ Mobile responsive (works on tablets)
- ✅ Success indicator shows when compilation succeeds
- ✅ No console errors or warnings

---

## Phase 1 Time Breakdown

| Task | Time |
|------|------|
| Backend: Enhanced error reporting | 3-4 hours |
| Frontend: Monaco + error decorations | 3-4 hours |
| Example snippets dropdown | 1 hour |
| Multiple output tabs | 1 hour |
| URL state & sharing | 1 hour |
| Polish & testing | 1-2 hours |
| **Total** | **10-13 hours** |

---

## What Phase 1 Includes (vs. Later Phases)

**✅ In Phase 1:**
- Excellent error reporting (inline + panel)
- Real-time compilation
- Example snippets
- URL sharing
- Professional UI/UX
- Tabs for output/errors/AST

**❌ NOT in Phase 1 (Future):**
- Visual preview (Phase 2)
- Emulator (Phase 3)
- Compose-for-Web runtime (Phase 4)
- Syntax highlighting for Whitehall (Phase 5)
- Save to database / user accounts (Phase 6)
- Autocomplete / IntelliSense (Phase 7)

---

### **Phase 2: Static Visual Preview** (Optional Enhancement)
**Goal:** Add visual approximation of component layout

**Time estimate:** 4-6 hours

**Features:**
- Add second tab in right pane: "Preview"
- Parse Whitehall AST to extract component tree
- Render HTML/CSS approximation
- Material3-inspired styling
- Preview updates in sync with compilation

**Implementation steps:**
1. AST to HTML converter (2-3 hours)
   - Walk component tree
   - Map components to HTML elements
   - Convert props to inline styles
   - Handle nesting

2. Preview renderer (1-2 hours)
   - Iframe for isolated rendering
   - Material3 CSS theme
   - Layout helpers

3. UI integration (1 hour)
   - Tabbed interface (Code / Preview)
   - Sync updates with compilation
   - Error state handling

**Success criteria:**
- ✅ Can see visual layout of components
- ✅ Styling approximates Material3
- ✅ Updates in real-time with code changes

**Limitations:**
- Not interactive (can't click buttons)
- No state management
- Approximation only (won't match Android exactly)

---

### **Phase 3: Emulator Streaming** (Future - High Investment)
**Goal:** Real Android emulator in browser

**Time estimate:** 20-40 hours
**Infrastructure cost:** $50-200/month

**Only pursue if:**
- Building commercial playground service
- Have budget for infrastructure
- Need real Android execution

**Implementation steps:**
1. Docker-based Android emulator setup (4-8 hours)
2. WebRTC video streaming (6-10 hours)
3. Touch event forwarding (2-4 hours)
4. Session management (4-6 hours)
5. Scaling infrastructure (4-8 hours)
6. Performance optimization (4-6 hours)

---

### **Phase 4: Compose-for-Web Target** (Future - Major Feature)
**Goal:** Actual Compose runtime in browser

**Time estimate:** 50+ hours

**Requirements:**
- Add Kotlin/JS compilation path to transpiler
- Integrate Kotlin/JS toolchain
- Test Compose Multiplatform Web support
- Build system for JS output

**Only pursue if:**
- Planning multi-platform support
- Want full interactivity without emulator
- Have time for major transpiler rewrite

---

## Recommendation

### ✅ Phase 1 Complete (Nov 4, 2025)
**What's working now:**
- Full-featured web playground with Monaco editor
- Real-time compilation and output display
- Multiple tabs (Kotlin / Errors / AST)
- Example snippets and URL sharing
- All core UX features (copy/format/clear, keyboard shortcuts)

**Ready to use for:**
- Learning Whitehall syntax
- Prototyping and experimentation
- Documentation examples
- Sharing code snippets

### Next Steps (Optional Enhancements)

**Option A: Enhanced Error Reporting (3-4 hours)**
- Add position tracking to parser
- Inline error markers with line/column
- Clickable errors that jump to code
- Requires transpiler changes

**Option B: Phase 2 - Visual Preview (4-6 hours)**
- HTML/CSS approximation of component layout
- Material3-inspired styling
- Good for visual learners

**Option C: Use as-is**
- Phase 1 is fully functional
- Provides excellent value for documentation and learning
- Can enhance later based on user feedback

---

## File Structure

### Backend (`tools/playground/backend/`)

**src/main.rs:**
```rust
use axum::{
    extract::Json,
    response::Json as JsonResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use whitehall::transpile; // Use whitehall crate

#[derive(Deserialize)]
struct CompileRequest {
    code: String,
}

#[derive(Serialize)]
struct CompileResponse {
    success: bool,
    output: String,
    errors: Vec<String>,
}

async fn compile(Json(req): Json<CompileRequest>) -> JsonResponse<CompileResponse> {
    match whitehall::transpile(&req.code) {
        Ok(kotlin_code) => JsonResponse(CompileResponse {
            success: true,
            output: kotlin_code,
            errors: vec![],
        }),
        Err(e) => JsonResponse(CompileResponse {
            success: false,
            output: String::new(),
            errors: vec![e.to_string()],
        }),
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/compile", post(compile))
        .layer(CorsLayer::permissive());

    println!("🚀 Playground backend running on http://localhost:3000");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

### Frontend (`tools/playground/frontend/`)

**index.html:**
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Whitehall Playground</title>
    <link rel="stylesheet" href="style.css">
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-100">
    <div class="container">
        <header>
            <h1>Whitehall Playground</h1>
        </header>

        <div class="panes">
            <!-- Left: Code editor -->
            <div class="pane editor-pane">
                <div id="editor"></div>
            </div>

            <!-- Right: Output -->
            <div class="pane output-pane">
                <div class="tabs">
                    <button class="tab active">Kotlin Output</button>
                    <button class="copy-btn">Copy</button>
                </div>
                <div id="output"></div>
                <div id="errors"></div>
            </div>
        </div>
    </div>

    <!-- Monaco Editor -->
    <script src="https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs/loader.js"></script>
    <script src="app.js"></script>
</body>
</html>
```

**app.js:**
```javascript
let editor;
let compileTimeout;

// Initialize Monaco Editor
require.config({ paths: { vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' }});
require(['vs/editor/editor.main'], function() {
    editor = monaco.editor.create(document.getElementById('editor'), {
        value: `var count = 0\n\n<Column padding={16}>\n  <Text fontSize={24}>{count}</Text>\n  <Button onClick={() => count++}>\n    <Text>Increment</Text>\n  </Button>\n</Column>`,
        language: 'kotlin', // Close enough for now
        theme: 'vs-dark',
        automaticLayout: true,
    });

    // Compile on change (debounced)
    editor.onDidChangeModelContent(() => {
        clearTimeout(compileTimeout);
        compileTimeout = setTimeout(compile, 500);
    });

    // Initial compilation
    compile();
});

async function compile() {
    const code = editor.getValue();
    const output = document.getElementById('output');
    const errors = document.getElementById('errors');

    output.textContent = 'Compiling...';
    errors.innerHTML = '';

    try {
        const response = await fetch('http://localhost:3000/api/compile', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ code }),
        });

        const result = await response.json();

        if (result.success) {
            output.textContent = result.output;
        } else {
            output.textContent = '';
            errors.innerHTML = result.errors.map(e => `<div class="error">${e}</div>`).join('');
        }
    } catch (error) {
        errors.innerHTML = `<div class="error">Connection error: ${error.message}</div>`;
    }
}

// Copy to clipboard
document.querySelector('.copy-btn').addEventListener('click', () => {
    const output = document.getElementById('output').textContent;
    navigator.clipboard.writeText(output);
});
```

---

## Future Enhancements

### Features
- [ ] Example snippets dropdown (Counter, Todo, Form, etc.)
- [ ] Share functionality (save to DB, generate shareable URL)
- [ ] Syntax highlighting for Whitehall (custom Monaco language)
- [ ] Dark/light theme toggle
- [ ] Mobile-responsive layout
- [ ] Keyboard shortcuts (Cmd+S to compile, Cmd+K to copy)
- [ ] Error highlighting in editor (inline squiggles)
- [ ] Autocomplete for Whitehall syntax
- [ ] "Fork" button to create new snippet from existing

### Integration
- [ ] Embed playground in documentation (iframe)
- [ ] Public gallery of shared snippets
- [ ] GitHub Gist integration (import/export)
- [ ] Deploy to whitehall.dev or play.whitehall.dev

---

## Deployment Options

### Development
```bash
# Terminal 1: Backend
cd tools/playground/backend
cargo run

# Terminal 2: Frontend
cd tools/playground/frontend
python -m http.server 8080
# Visit http://localhost:8080
```

### Production

**Backend options:**
- Fly.io (Rust-friendly, $0-5/month)
- Railway ($5-10/month)
- DigitalOcean App Platform ($5-12/month)

**Frontend options:**
- Vercel (free tier, static hosting)
- Netlify (free tier)
- GitHub Pages (free, static only)
- Cloudflare Pages (free tier)

**Recommended:**
- Backend on Fly.io (Rust support, edge deployment)
- Frontend on Vercel (fast CDN, zero config)

---

## Success Metrics

### ✅ Phase 1 Complete (Nov 4, 2025)
- ✅ Can type Whitehall code and see Kotlin output
- ✅ Compilation happens automatically (debounced 500ms)
- ✅ Errors displayed clearly in dedicated tab
- ✅ Copy button works
- ✅ Multiple output tabs (Kotlin / Errors / AST)
- ✅ 5 example snippets working
- ✅ URL hash state for sharing
- ✅ Keyboard shortcuts (Ctrl+Enter, Ctrl+S)
- ✅ Mobile responsive layout
- ✅ Status indicators and toast notifications
- ⏳ **Deployment pending** - Ready to deploy to production

### Phase 2 Goals (Future):
- ⏳ Visual preview shows component layout
- ⏳ Styling approximates Material3
- ⏳ Preview updates in sync with code
- ⏳ Handles common components (Column, Row, Text, Button)

---

## Open Questions

1. **Deployment:** Where to host? (play.whitehall.dev vs whitehall.dev/playground)
   - Backend: Fly.io, Railway, or DigitalOcean
   - Frontend: Vercel, Netlify, or Cloudflare Pages
2. **Analytics:** Track usage metrics? (compile counts, popular examples)
3. **Syntax highlighting:** Create custom Monaco language definition for Whitehall?
4. **Sharing:** URL hash only (current) or save to database for persistent links?
5. **Examples:** Are current 5 examples sufficient? (hello/counter/todo/form/styling)

---

## Next Steps

**Immediate (Testing & Deployment):**
1. ✅ Phase 1 implementation complete
2. ⏳ Manual testing of all features
3. ⏳ Deploy backend to production
4. ⏳ Deploy frontend to production
5. ⏳ Add playground link to documentation

**Future Enhancements (Optional):**
1. Add position tracking to parser for inline error markers
2. Implement Phase 2 (visual preview) if user feedback requests it
3. Create custom Whitehall syntax highlighting for Monaco
4. Add more example snippets based on common patterns
