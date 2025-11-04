# Whitehall Playground

Web-based playground for experimenting with Whitehall syntax and seeing compiled Kotlin output in real-time.

## Architecture

- **Backend:** Rust (Axum) API server that transpiles Whitehall code
- **Frontend:** Vanilla JavaScript with Monaco Editor (no build step)

## Development

### Backend

```bash
cd backend
cargo run
# Server runs on http://localhost:3000
```

### Frontend

```bash
cd frontend
python -m http.server 8080
# Open http://localhost:8080
```

## Features

- ✅ Real-time compilation with debouncing
- ✅ Monaco editor with syntax highlighting
- ✅ Inline error markers (red squiggly lines)
- ✅ Rich error panel with line/column and context
- ✅ Multiple output tabs (Kotlin / Errors / AST)
- ✅ Example snippets dropdown
- ✅ URL state for sharing code
- ✅ Copy/format/clear buttons
- ✅ Keyboard shortcuts (Ctrl+Enter, Ctrl+S)
- ✅ Mobile responsive

## API

### POST /api/compile

**Request:**
```json
{
  "code": "var count = 0\n<Text>{count}</Text>"
}
```

**Response:**
```json
{
  "success": true,
  "output": "...",
  "errors": [],
  "warnings": [],
  "ast": null
}
```

## Deployment

See `docs/WEB.md` for deployment options (Fly.io, Vercel, etc.)
