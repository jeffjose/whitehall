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

// Example snippets (ordered by increasing complexity)
const examples = {
    'hello': {
        name: '01. Hello World',
        code: `// The simplest Whitehall app - just render text!
<Text>Hello, Whitehall!</Text>`
    },
    'styling': {
        name: '02. Text Styling',
        code: `// Style text with fontSize, fontWeight, and color props
<Column padding={16} spacing={8}>
  <Text fontSize={24} fontWeight="bold">
    Large Bold Text
  </Text>
  <Text fontSize={16} color="#666">
    Hex colors work (#RGB or #RRGGBB)
  </Text>
  <Text fontWeight="bold" color="#2196F3">
    Combine multiple style props
  </Text>
</Column>`
    },
    'layout': {
        name: '03. Layouts',
        code: `// Column stacks items vertically, Row arranges horizontally
// Use spacing for gaps, padding for inner spacing
<Column padding={16} spacing={12}>
  <Text fontSize={20} fontWeight="bold">Column Layout</Text>
  <Text>Items stack vertically</Text>

  <Row spacing={8}>
    <Text>Row items →</Text>
    <Text color="#F44336">go</Text>
    <Text color="#4CAF50">horizontal</Text>
  </Row>
</Column>`
    },
    'padding': {
        name: '04. Padding Shortcuts',
        code: `// CSS-like padding shortcuts: p, px, py, pt, pb, pl, pr
<Column spacing={8}>
  <Text p={16} color="#2196F3">
    All sides (p=16)
  </Text>

  <Text px={20} py={8} color="#4CAF50">
    Horizontal & Vertical (px=20, py=8)
  </Text>

  <Card pt={4} pb={12} pl={8} pr={8}>
    <Text>Individual: top=4 bottom=12 left=8 right=8</Text>
  </Card>

  <Text fontWeight="bold">
    Multiple shortcuts combine into single padding()
  </Text>
</Column>`
    },
    'spacer': {
        name: '05. Spacer Shortcuts',
        code: `// Spacer with h (height) and w (width) shortcuts
<Column>
  <Text fontSize={20} fontWeight="bold">Spacer Examples</Text>

  <Text>First item</Text>
  <Spacer h={16} />
  <Text>16dp vertical space above</Text>

  <Spacer h={32} />
  <Text>32dp vertical space above</Text>

  <Spacer />
  <Text>Default 8dp space above</Text>

  <Row>
    <Text>Left</Text>
    <Spacer w={24} />
    <Text>24dp horizontal space</Text>
  </Row>
</Column>`
    },
    'button': {
        name: '17. Buttons & State',
        code: `// Reactive state: just declare 'var' and modify it!
var count = 0

<Column padding={16} spacing={12}>
  <Text fontSize={28} fontWeight="bold">{count}</Text>

  // Button text auto-wraps in Text (no need for <Text> tags)
  <Button onClick={() => count++}>
    Increment
  </Button>

  <Button onClick={() => count--}>
    Decrement
  </Button>

  <Button onClick={() => count = 0}>
    Reset
  </Button>
</Column>`
    },
    'binding': {
        name: '17. Two-Way Binding',
        code: `var name = ""

// bind:value creates two-way binding - changes flow both ways!
<Column padding={16} spacing={12}>
  <TextField
    bind:value={name}
    label="Enter your name"
  />

  // Interpolate with {expression}
  <Text fontSize={18}>
    Hello, {name.isEmpty() ? "stranger" : name}!
  </Text>
</Column>`
    },
    'multistate': {
        name: '17. Multiple State',
        code: `// Manage multiple reactive variables independently
var likes = 0
var dislikes = 0

<Column padding={16} spacing={12}>
  <Text fontSize={20} fontWeight="bold">Feedback</Text>

  // Nest layouts for complex UIs
  <Row spacing={16}>
    <Column spacing={4}>
      <Button onClick={() => likes++}>
        👍 Like
      </Button>
      <Text>{likes} likes</Text>
    </Column>

    <Column spacing={4}>
      <Button onClick={() => dislikes++}>
        👎 Dislike
      </Button>
      <Text>{dislikes} dislikes</Text>
    </Column>
  </Row>
</Column>`
    },
    'conditional': {
        name: '17. Conditionals',
        code: `var isLoggedIn = false

// @if for conditional rendering (like Kotlin 'if' expressions)
<Column padding={16} spacing={12}>
  @if (isLoggedIn) {
    <Text fontSize={20}>Welcome back! 👋</Text>
    <Button onClick={() => isLoggedIn = false}>
      Logout
    </Button>
  } else {
    <Text fontSize={20}>Please log in</Text>
    <Button onClick={() => isLoggedIn = true}>
      Login
    </Button>
  }
</Column>`
    },
    'list': {
        name: '17. Lists & Loops',
        code: `var items = ["Apple", "Banana", "Cherry", "Date"]

// @for loops over collections
<Column padding={16} spacing={8}>
  <Text fontSize={20} fontWeight="bold">Fruit List</Text>

  @for (fruit in items) {
    <Card padding={12} modifier={Modifier.fillMaxWidth()}>
      <Text>{fruit}</Text>
    </Card>
  }
</Column>`
    },
    'interactive-list': {
        name: '17. Interactive List',
        code: `var items = ["Apple", "Banana", "Cherry"]
var newItem = ""

<Column padding={16} spacing={8}>
  <Text fontSize={20} fontWeight="bold">Editable List</Text>

  <Row spacing={8}>
    <TextField
      bind:value={newItem}
      placeholder="Add item..."
      modifier={Modifier.weight(1f)}  // Fill available space
    />
    <Button onClick={() => {
      if (newItem.isNotEmpty()) {
        items = items + newItem  // Immutable list operations
        newItem = ""             // Clear input
      }
    }}>
      Add
    </Button>
  </Row>

  @for (item in items) {
    <Card padding={12} modifier={Modifier.fillMaxWidth()}>
      <Row spacing={8}>
        <Text modifier={Modifier.weight(1f)}>{item}</Text>
        // List subtraction removes items
        <Button onClick={() => items = items - item}>
          Delete
        </Button>
      </Row>
    </Card>
  }
</Column>`
    },
    'derived': {
        name: '17. Derived State',
        code: `// Use 'var' for mutable state, 'val' for derived/computed values
var price = 10
var quantity = 1

// Derived values auto-update when dependencies change
val total = price * quantity
val discount = if (quantity >= 5) 0.2 else 0.0
val finalPrice = total * (1 - discount)

<Column padding={16} spacing={12}>
  <Text fontSize={20} fontWeight="bold">Price Calculator</Text>

  <Text>Price: $\{price}</Text>
  <Text>Quantity: {quantity}</Text>

  <Row spacing={8}>
    <Button onClick={() => quantity++}>+</Button>
    <Button onClick={() => if (quantity > 1) quantity--}>-</Button>
  </Row>

  <Text>Subtotal: $\{total}</Text>
  <Text color="#4CAF50">
    Discount: {discount * 100}%
  </Text>
  <Text fontSize={18} fontWeight="bold">
    Total: $\{finalPrice}
  </Text>
</Column>`
    },
    'todo': {
        name: '17. Todo App',
        code: `var todos = ["Buy milk", "Write code"]
var newTodo = ""

// Complete todo app in ~30 lines!
<Column padding={16} spacing={12}>
  <Text fontSize={24} fontWeight="bold">My Todos</Text>

  <Row spacing={8}>
    <TextField
      bind:value={newTodo}
      placeholder="Add a new todo..."
      modifier={Modifier.weight(1f)}
    />
    <Button onClick={() => {
      if (newTodo.isNotEmpty()) {
        todos = todos + newTodo
        newTodo = ""
      }
    }}>
      Add
    </Button>
  </Row>

  @for (todo in todos) {
    <Card padding={12} modifier={Modifier.fillMaxWidth()}>
      <Row spacing={8}>
        <Text modifier={Modifier.weight(1f)}>{todo}</Text>
        <Button onClick={() => todos = todos - todo}>
          ✓
        </Button>
      </Row>
    </Card>
  }

  // Dynamic text based on list size
  <Text color="#888">
    {todos.size} items remaining
  </Text>
</Column>`
    },
    'form': {
        name: '17. Form Validation',
        code: `var name = ""
var email = ""
var age = ""
var agreed = false

// Derive validation state from inputs
val isValidEmail = email.contains("@") && email.contains(".")
val isValidAge = age.toIntOrNull() != null && age.toInt() >= 18
val canSubmit = name.isNotEmpty() && isValidEmail && isValidAge && agreed

<Column padding={16} spacing={12}>
  <Text fontSize={20} fontWeight="bold">Sign Up Form</Text>

  <TextField bind:value={name} label="Name" />

  <TextField bind:value={email} label="Email" />
  // Show validation errors conditionally
  @if (email.isNotEmpty() && !isValidEmail) {
    <Text color="error">Invalid email</Text>
  }

  <TextField bind:value={age} label="Age" />
  @if (age.isNotEmpty() && !isValidAge) {
    <Text color="error">Must be 18+</Text>
  }

  <Row spacing={8}>
    <Checkbox bind:checked={agreed} />
    <Text>I agree to terms</Text>
  </Row>

  // Button enabled state based on validation
  <Button enabled={canSubmit} onClick={() => {}}>
    {canSubmit ? "Submit" : "Complete all fields"}
  </Button>
</Column>`
    },
    'counter-list': {
        name: '17. List Mutations',
        code: `var counters = [0, 0, 0]

// Working with list indices for complex updates
<Column padding={16} spacing={12}>
  <Text fontSize={20} fontWeight="bold">Multiple Counters</Text>

  <Button onClick={() => counters = counters + 0}>
    Add Counter
  </Button>

  @for (i in counters.indices) {
    <Card padding={12} modifier={Modifier.fillMaxWidth()}>
      <Row spacing={12}>
        <Text fontSize={18} modifier={Modifier.weight(1f)}>
          Counter {i + 1}: {counters[i]}
        </Text>
        // Update specific index immutably
        <Button onClick={() => {
          val updated = counters.toMutableList()
          updated[i] = updated[i] + 1
          counters = updated
        }}>
          +
        </Button>
        // Filter by index to remove
        <Button onClick={() => {
          counters = counters.filterIndexed { idx, _ -> idx != i }
        }}>
          Remove
        </Button>
      </Row>
    </Card>
  }
</Column>`
    },
    'shopping': {
        name: '17. Complex Data',
        code: `// Work with maps and complex data structures
var cart = [
  { "name": "Laptop", "price": 999, "qty": 1 },
  { "name": "Mouse", "price": 25, "qty": 2 }
]

// Kotlin stdlib functions work in Whitehall
val total = cart.sumOf { it["price"] as Int * it["qty"] as Int }

<Column padding={16} spacing={12}>
  <Text fontSize={24} fontWeight="bold">Shopping Cart</Text>

  @for (item in cart) {
    <Card padding={12} modifier={Modifier.fillMaxWidth()}>
      <Column spacing={4}>
        <Text fontSize={16} fontWeight="bold">
          {item["name"]}
        </Text>
        <Row spacing={8}>
          <Text>$\{item["price"]} x {item["qty"]}</Text>
          <Text fontWeight="bold">
            = $\{item["price"] as Int * item["qty"] as Int}
          </Text>
        </Row>
      </Column>
    </Card>
  }

  <Card padding={16} modifier={Modifier.fillMaxWidth()}>
    <Row spacing={8}>
      <Text fontSize={20} fontWeight="bold" modifier={Modifier.weight(1f)}>
        Total:
      </Text>
      <Text fontSize={20} fontWeight="bold" color="#4CAF50">
        $\{total}
      </Text>
    </Row>
  </Card>
</Column>`
    },
    'tabs': {
        name: '17. Navigation',
        code: `// Simple tab navigation with conditional rendering
var activeTab = "home"

<Column padding={16} spacing={12}>
  <Text fontSize={24} fontWeight="bold">Multi-Tab App</Text>

  // Tab buttons update state
  <Row spacing={8}>
    <Button onClick={() => activeTab = "home"}>
      Home
    </Button>
    <Button onClick={() => activeTab = "profile"}>
      Profile
    </Button>
    <Button onClick={() => activeTab = "settings"}>
      Settings
    </Button>
  </Row>

  // Render different content based on active tab
  <Card padding={16} modifier={Modifier.fillMaxWidth()}>
    @if (activeTab == "home") {
      <Column spacing={8}>
        <Text fontSize={20} fontWeight="bold">Home</Text>
        <Text>Welcome to the home page!</Text>
      </Column>
    } else if (activeTab == "profile") {
      <Column spacing={8}>
        <Text fontSize={20} fontWeight="bold">Profile</Text>
        <Text>Name: John Doe</Text>
        <Text>Email: john@example.com</Text>
      </Column>
    } else {
      <Column spacing={8}>
        <Text fontSize={20} fontWeight="bold">Settings</Text>
        <Text>App version: 1.0.0</Text>
      </Column>
    }
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
        fontSize: 12,
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
    return examples.button.code;  // Start with button/state example
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
        kotlinPanel.innerHTML = `<pre id="kotlin-output" class="p-4 text-xs font-mono text-gray-300">${escapeHtml(code)}</pre>`;
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
            preEl.classList.add('p-4', 'text-xs', 'h-full', 'm-0');
            preEl.style.background = 'transparent';
        }
    } catch (error) {
        console.error('Shiki highlighting error:', error);
        // Fallback to plain text
        kotlinPanel.innerHTML = `<pre id="kotlin-output" class="p-4 text-xs font-mono text-gray-300">${escapeHtml(code)}</pre>`;
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
