# Whitehall Web Playground

**Goal:** Web-based IDE for experimenting with Whitehall syntax, seeing compiled output, and (eventually) previewing results.

**Status:** Planning phase
**Priority:** After end-to-end testing is complete

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

### **Phase 1: MVP - Compiled Output Only** ⭐ START HERE
**Goal:** Basic playground with code editor and Kotlin output

**Time estimate:** 2-3 hours

**Features:**
- Left pane: Monaco editor with Whitehall syntax
- Right pane: Compiled Kotlin code display
- POST /api/compile endpoint
- Debounced compilation (500ms after typing stops)
- Error display with messages
- Copy-to-clipboard button

**Implementation steps:**
1. Setup project structure (10 min)
   - Create `tools/playground/` directory
   - Create backend Cargo.toml
   - Create frontend files (HTML/CSS/JS)

2. Backend API (30-45 min)
   - Axum server setup
   - /api/compile endpoint
   - Use whitehall transpiler crate
   - Error handling and JSON responses
   - CORS middleware

3. Frontend UI (45-60 min)
   - Split-pane layout (CSS Grid)
   - Integrate Monaco Editor from CDN
   - Fetch API to call backend
   - Display compiled output or errors
   - Add loading spinner
   - Copy button functionality

4. Testing (15-30 min)
   - Test with various Whitehall examples
   - Handle edge cases (empty input, syntax errors)
   - Test debouncing behavior

**Success criteria:**
- ✅ Can type Whitehall code in left pane
- ✅ See compiled Kotlin in right pane after 500ms
- ✅ Errors displayed clearly
- ✅ Can copy output to clipboard

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

### For MVP (Now)
**Start with Phase 1 only:**
- Compiled Kotlin output view
- 2-3 hours of work
- Validates transpiler via web UI
- Useful for documentation and learning

### After MVP is Working
**Consider Phase 2 if:**
- Want visual feedback for learners
- Documentation would benefit from previews
- Have 4-6 hours to invest

**Skip Phase 3 unless:**
- Building a commercial service
- Have infrastructure budget
- Need real Android execution

**Consider Phase 4 for:**
- Whitehall 2.0+ (multi-platform vision)
- Long-term roadmap feature
- Major version milestone

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

### Phase 1 Complete When:
- ✅ Can type Whitehall code and see Kotlin output
- ✅ Compilation happens automatically (debounced)
- ✅ Errors displayed clearly
- ✅ Copy button works
- ✅ Deployed and accessible via URL

### Phase 2 Complete When:
- ✅ Visual preview shows component layout
- ✅ Styling approximates Material3
- ✅ Preview updates in sync with code
- ✅ Handles common components (Column, Row, Text, Button)

---

## Open Questions

1. **Syntax highlighting:** Should we create custom Monaco language definition for Whitehall?
2. **Examples:** What snippets should we include? (Counter, Todo, Form, Navigation)
3. **Domain:** Should we host at play.whitehall.dev or whitehall.dev/playground?
4. **Sharing:** Do we want URL-based sharing (save to DB) or just local-only?
5. **Analytics:** Track usage metrics (compile counts, popular examples)?

---

**Next Steps:**
1. Complete end-to-end testing (Priority 1)
2. After E2E testing, implement Phase 1 playground
3. Gather feedback on compiled output view
4. Decide if Phase 2 (preview) is worth the investment
