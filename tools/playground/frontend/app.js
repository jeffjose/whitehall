// Whitehall Playground - Main JavaScript

const BACKEND_URL = 'http://localhost:3000';
const COMPILE_DEBOUNCE_MS = 500;

let editor;
let compileTimeout;
let decorations = [];
let currentTab = 'kotlin';
let lastCompileResult = null;
let shikiCodeToHtml = null;

// Dynamically import Shiki
(async () => {
    try {
        const shiki = await import('https://esm.sh/shiki@1.0.0');
        shikiCodeToHtml = shiki.codeToHtml;
    } catch (error) {
        console.error('Failed to load Shiki:', error);
    }
})();

// Example snippets
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
        code: `var todos = ["Buy milk", "Write code", "Test Whitehall"]
var newTodo = ""

fun addTodo() {
  if (newTodo.isNotEmpty()) {
    todos = todos + newTodo
    newTodo = ""
  }
}

<Column padding={16} spacing={8}>
  <Text fontSize={24} fontWeight="bold">My Todos</Text>

  <Row spacing={8}>
    <TextField
      bind:value={newTodo}
      placeholder="Add a new todo..."
      modifier={Modifier.weight(1f)}
    />
    <Button onClick={addTodo}>
      <Text>Add</Text>
    </Button>
  </Row>

  @for (todo in todos) {
    <Card padding={12} modifier={Modifier.fillMaxWidth()}>
      <Text>{todo}</Text>
    </Card>
  }
</Column>`
    },
    'form': {
        name: 'Form with Binding',
        code: `var name = ""
var email = ""
var agreeToTerms = false

<Column padding={16} spacing={12}>
  <Text fontSize={20} fontWeight="bold">Sign Up Form</Text>

  <TextField
    bind:value={name}
    label="Name"
  />

  <TextField
    bind:value={email}
    label="Email"
  />

  <Row spacing={8}>
    <Checkbox bind:checked={agreeToTerms} />
    <Text>I agree to the terms and conditions</Text>
  </Row>

  <Button
    onClick={() => {}}
    enabled={agreeToTerms}
  >
    <Text>Submit</Text>
  </Button>
</Column>`
    },
    'styling': {
        name: 'Styling & Modifiers',
        code: `<Column
  modifier={Modifier
    .fillMaxSize()
    .padding(16.dp)
    .background(Color(0xFFF5F5F5))
  }
  horizontalAlignment="CenterHorizontally"
>
  <Text
    text="Welcome to Whitehall"
    fontSize={28}
    fontWeight="bold"
    color={Color(0xFF1976D2)}
  />

  <Spacer modifier={Modifier.height(16.dp)} />

  <Card
    modifier={Modifier.fillMaxWidth()}
    elevation={4.dp}
  >
    <Column padding={16}>
      <Text text="This is a card" fontSize={18} />
      <Text text="With some content" color={Color.Gray} />
    </Column>
  </Card>
</Column>`
    }
};

// Initialize Monaco Editor
require.config({
    paths: { 'vs': 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' }
});

require(['vs/editor/editor.main'], function() {
    editor = monaco.editor.create(document.getElementById('editor'), {
        value: getInitialCode(),
        language: 'kotlin', // Close enough for now
        theme: 'vs-dark',
        automaticLayout: true,
        minimap: { enabled: true },
        fontSize: 14,
        fontFamily: "'Fira Code', 'Consolas', 'Courier New', monospace",
        fontLigatures: false, // Disable ligatures to fix number rendering
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        wordWrap: 'on',
        renderWhitespace: 'none',
        letterSpacing: 0,
    });

    // Compile on change (debounced)
    editor.onDidChangeModelContent(() => {
        clearTimeout(compileTimeout);
        compileTimeout = setTimeout(() => {
            compile();
        }, COMPILE_DEBOUNCE_MS);
        updateURL(); // Update URL hash with code
    });

    // Initial compilation
    compile();

    // Keyboard shortcuts
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, compile);
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, formatCode);
});

// Get initial code from URL hash or default example
function getInitialCode() {
    const hash = window.location.hash.slice(1);
    if (hash) {
        try {
            return decodeURIComponent(atob(hash));
        } catch (e) {
            console.error('Invalid URL hash');
        }
    }
    return examples.counter.code;
}

// Compile Whitehall code
async function compile() {
    const code = editor.getValue();

    // Update status
    setStatus('compiling');

    try {
        const response = await fetch(`${BACKEND_URL}/api/compile`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ code }),
        });

        const result = await response.json();
        lastCompileResult = result;

        if (result.success) {
            // Success - render with syntax highlighting
            await renderKotlinOutput(result.output);

            document.getElementById('ast-output').textContent =
                result.ast ? JSON.stringify(JSON.parse(result.ast), null, 2) : 'AST view not available';

            updateEditorErrors([]);
            setStatus('success');

            // Hide error count badge
            document.getElementById('error-count').classList.add('hidden');

            // Clear error panel when successful
            renderErrorPanel([]);
        } else {
            // Errors - keep last valid output visible, show errors in panel
            updateEditorErrors(result.errors);
            renderErrorPanel(result.errors);
            setStatus('error');

            // Show error count badge
            const errorCountEl = document.getElementById('error-count');
            errorCountEl.textContent = result.errors.length;
            errorCountEl.classList.remove('hidden');

            // Auto-switch to Errors tab to show what went wrong
            switchTab('errors');
        }
    } catch (error) {
        console.error('Compilation error:', error);
        setStatus('error');
        renderErrorPanel([{
            message: `Connection error: ${error.message}`,
            severity: 'error',
        }]);
    }
}

// Render Kotlin output with Shiki syntax highlighting
async function renderKotlinOutput(code) {
    const kotlinPanel = document.getElementById('kotlin-panel');

    // If Shiki hasn't loaded yet, show plain text
    if (!shikiCodeToHtml) {
        kotlinPanel.innerHTML = `<pre id="kotlin-output" class="p-4 text-sm font-mono text-gray-300">${escapeHtml(code)}</pre>`;
        return;
    }

    try {
        const html = await shikiCodeToHtml(code, {
            lang: 'kotlin',
            theme: 'github-dark'
        });

        // Replace the panel content with highlighted HTML
        kotlinPanel.innerHTML = html;

        // Add padding and full height styling to match the original
        const preEl = kotlinPanel.querySelector('pre');
        if (preEl) {
            preEl.classList.add('p-4', 'text-sm', 'h-full', 'm-0');
            preEl.style.background = 'transparent';
        }
    } catch (error) {
        console.error('Shiki highlighting error:', error);
        // Fallback to plain text
        kotlinPanel.innerHTML = `<pre id="kotlin-output" class="p-4 text-sm font-mono text-gray-300">${escapeHtml(code)}</pre>`;
    }
}

// Update Monaco editor with error decorations
function updateEditorErrors(errors) {
    // Clear previous decorations
    decorations = editor.deltaDecorations(decorations, []);

    if (!errors || errors.length === 0) {
        monaco.editor.setModelMarkers(editor.getModel(), 'whitehall', []);
        return;
    }

    // Add new decorations for errors with position info
    const newDecorations = errors
        .filter(err => err.line && err.column)
        .map(err => ({
            range: new monaco.Range(
                err.line,
                err.column,
                err.line,
                err.column + (err.length || 1)
            ),
            options: {
                isWholeLine: false,
                className: 'error-decoration',
                hoverMessage: { value: err.message },
                glyphMarginClassName: 'error-glyph',
            }
        }));

    decorations = editor.deltaDecorations([], newDecorations);

    // Set Monaco markers
    monaco.editor.setModelMarkers(
        editor.getModel(),
        'whitehall',
        errors
            .filter(err => err.line && err.column)
            .map(err => ({
                startLineNumber: err.line,
                startColumn: err.column,
                endLineNumber: err.line,
                endColumn: err.column + (err.length || 1),
                message: err.message,
                severity: monaco.MarkerSeverity.Error,
            }))
    );
}

// Render error panel
function renderErrorPanel(errors) {
    const panel = document.getElementById('errors-panel');

    if (!errors || errors.length === 0) {
        panel.innerHTML = '<div class="p-4 text-gray-400 text-center">✓ No errors - code compiled successfully!</div>';
        return;
    }

    // Add a header with error count and helpful message
    const header = `
        <div class="p-4 bg-red-900 bg-opacity-20 border-b border-red-800 mb-4">
            <div class="flex items-center gap-2 text-red-400 font-semibold mb-2">
                <span class="text-2xl">⚠️</span>
                <span>${errors.length} Compilation ${errors.length === 1 ? 'Error' : 'Errors'}</span>
            </div>
            <div class="text-sm text-gray-400">
                ${errors.some(e => e.line) ? 'Click on an error to jump to the line in the editor.' : 'Fix the errors below to see your compiled Kotlin code.'}
            </div>
        </div>
    `;

    const errorItems = errors.map(err => `
        <div class="error-item ${err.line ? 'cursor-pointer' : ''}" ${err.line ? `onclick="jumpToLine(${err.line})"` : ''}>
            <div class="error-header">
                <span class="error-icon">❌</span>
                ${err.line ? `<span class="error-location">Line ${err.line}${err.column ? `:${err.column}` : ''}</span>` : '<span class="error-location">Syntax Error</span>'}
            </div>
            <div class="error-message">${escapeHtml(err.message)}</div>
            ${err.context ? `<pre class="error-context">${escapeHtml(err.context)}</pre>` : ''}
        </div>
    `).join('');

    panel.innerHTML = header + errorItems;
}

// Jump to line in editor
function jumpToLine(line) {
    if (!line) return;
    editor.revealLineInCenter(line);
    editor.setPosition({ lineNumber: line, column: 1 });
    editor.focus();
}

// Set status indicator
function setStatus(status) {
    const indicator = document.getElementById('status-indicator');
    indicator.className = 'w-3 h-3 rounded-full';
    indicator.classList.add(status);
}

// Switch tabs
function switchTab(tab) {
    currentTab = tab;

    // Update tab buttons
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.tab === tab);
    });

    // Show/hide panels
    document.getElementById('kotlin-panel').classList.toggle('hidden', tab !== 'kotlin');
    document.getElementById('errors-panel').classList.toggle('hidden', tab !== 'errors');
    document.getElementById('ast-panel').classList.toggle('hidden', tab !== 'ast');
}

// Format code (basic indentation)
function formatCode() {
    const code = editor.getValue();
    // For now, just normalize whitespace
    // TODO: Implement proper Whitehall formatter
    const formatted = code.split('\n').map(line => line.trim()).join('\n');
    editor.setValue(formatted);
    showToast('Code formatted');
}

// Clear editor
function clearEditor() {
    if (confirm('Clear all code?')) {
        editor.setValue('');
        compile();
    }
}

// Copy Kotlin output
function copyOutput() {
    // Get the text content from the Kotlin panel (works with both plain text and Shiki HTML)
    const kotlinPanel = document.getElementById('kotlin-panel');
    const output = kotlinPanel.textContent || kotlinPanel.innerText;

    if (!output || output.includes('Compilation failed')) {
        showToast('Nothing to copy');
        return;
    }

    navigator.clipboard.writeText(output).then(() => {
        showToast('Copied to clipboard!');
    });
}

// Share code (copy URL with code)
function shareCode() {
    const url = window.location.href;
    navigator.clipboard.writeText(url).then(() => {
        showToast('Link copied to clipboard!');
    });
}

// Update URL hash with code
function updateURL() {
    const code = editor.getValue();
    const encoded = btoa(encodeURIComponent(code));
    window.history.replaceState(null, '', '#' + encoded);
}

// Load example
function loadExample(key) {
    if (!key) return;
    const example = examples[key];
    if (example) {
        editor.setValue(example.code);
        compile();
        // Keep the dropdown showing the selected example
        document.getElementById('examples').value = key;
    }
}

// Show toast notification
function showToast(message) {
    const toast = document.getElementById('toast');
    const messageEl = document.getElementById('toast-message');
    messageEl.textContent = message;
    toast.classList.remove('hidden');
    setTimeout(() => {
        toast.classList.add('hidden');
    }, 2000);
}

// Escape HTML
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Event listeners
document.addEventListener('DOMContentLoaded', () => {
    // Tab buttons
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.addEventListener('click', () => switchTab(btn.dataset.tab));
    });

    // Action buttons
    document.getElementById('format-btn').addEventListener('click', formatCode);
    document.getElementById('clear-btn').addEventListener('click', clearEditor);
    document.getElementById('copy-btn').addEventListener('click', copyOutput);
    document.getElementById('share-btn').addEventListener('click', shareCode);

    // Examples dropdown
    document.getElementById('examples').addEventListener('change', (e) => loadExample(e.target.value));
});
