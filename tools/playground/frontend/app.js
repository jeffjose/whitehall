// Whitehall Playground - Main JavaScript

const BACKEND_URL = 'http://localhost:3000';
const COMPILE_DEBOUNCE_MS = 500;

let editor;
let compileTimeout;
let decorations = [];
let currentTab = 'kotlin';
let shikiCodeToHtml = null;

// Multi-file state
let files = {
    'Main.wh': `// The simplest Whitehall app - just render text!
<Text>Hello, Whitehall!</Text>`
};
let activeFile = 'Main.wh';
let compileResults = {}; // Store results per file
let activeOutputFile = 'Main.wh'; // Which output file is currently being viewed

// Sidebar collapse state
let fileSidebarCollapsed = false;
let outputSidebarCollapsed = false;

// Dynamically import Shiki
(async () => {
    try {
        const shiki = await import('https://esm.sh/shiki@1.0.0');
        shikiCodeToHtml = shiki.codeToHtml;
    } catch (error) {
        console.error('Failed to load Shiki:', error);
    }
})();

// File management functions
function renderFileTree() {
    console.log('renderFileTree called, files:', files);
    const fileTree = document.getElementById('file-tree');
    if (!fileTree) {
        console.error('file-tree element not found');
        return;
    }
    fileTree.innerHTML = Object.keys(files)
        .sort()
        .map(filename => `
            <div class="file-item ${filename === activeFile ? 'active' : ''}" data-filename="${filename}">
                <span class="file-item-icon">▹</span>
                <span class="file-item-name">${filename}</span>
                <div class="file-item-actions">
                    <button class="file-item-action-btn rename-btn" data-filename="${filename}" title="Rename">↻</button>
                </div>
            </div>
        `).join('');

    // Add click handlers for file items
    fileTree.querySelectorAll('.file-item').forEach(item => {
        item.addEventListener('click', (e) => {
            if (!e.target.closest('.file-item-actions')) {
                const filename = item.dataset.filename;
                switchToFile(filename);
            }
        });
    });

    // Add click handlers for rename buttons
    fileTree.querySelectorAll('.rename-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const filename = btn.dataset.filename;
            renameFile(filename);
        });
    });
}

function switchToFile(filename) {
    if (!files[filename]) return;

    // Save current file content before switching
    if (activeFile && editor) {
        files[activeFile] = editor.getValue();
    }

    activeFile = filename;

    // Update editor content
    if (editor) {
        editor.setValue(files[filename]);
    }

    // Update UI
    document.getElementById('active-file-label').textContent = filename;
    renderFileTree();

    // Also switch output view to this file
    activeOutputFile = filename;

    // Update output to show this file's result
    updateOutputForActiveFile();
    renderOutputFileTree();
}

function addNewFile() {
    const filename = prompt('Enter filename (e.g., Components.wh):', 'NewFile.wh');
    if (!filename) return;

    // Validate filename
    if (!filename.endsWith('.wh')) {
        alert('Filename must end with .wh');
        return;
    }

    if (files[filename]) {
        alert('File already exists');
        return;
    }

    // Add new file with template content
    files[filename] = `// ${filename}\n\n<Text>New component</Text>`;

    // Switch to new file
    switchToFile(filename);
    renderFileTree();
    compile();
}

function deleteFile() {
    if (Object.keys(files).length === 1) {
        alert('Cannot delete the last file');
        return;
    }

    if (!confirm(`Delete ${activeFile}?`)) {
        return;
    }

    delete files[activeFile];
    delete compileResults[activeFile];

    // Switch to first available file
    activeFile = Object.keys(files)[0];
    switchToFile(activeFile);
    renderFileTree();
    compile();
}

function renameFile(oldFilename) {
    const newFilename = prompt('Rename file:', oldFilename);
    if (!newFilename || newFilename === oldFilename) return;

    // Validate filename
    if (!newFilename.endsWith('.wh')) {
        alert('Filename must end with .wh');
        return;
    }

    if (files[newFilename]) {
        alert('File already exists');
        return;
    }

    // Rename file
    files[newFilename] = files[oldFilename];
    delete files[oldFilename];

    // Update compile results
    if (compileResults[oldFilename]) {
        compileResults[newFilename] = compileResults[oldFilename];
        delete compileResults[oldFilename];
    }

    // Update active file if needed
    if (activeFile === oldFilename) {
        activeFile = newFilename;
        document.getElementById('active-file-label').textContent = newFilename;
    }

    renderFileTree();
}

function updateOutputForActiveFile() {
    const result = compileResults[activeFile];
    if (!result) return;

    if (result.success) {
        renderKotlinOutput(result.output);
        updateEditorErrors([]);
        renderErrorPanel([]);
    } else {
        updateEditorErrors(result.errors);
        renderErrorPanel(result.errors);
    }
}

// Output file tree management
function renderOutputFileTree() {
    const outputFileTree = document.getElementById('output-file-tree');
    const fileNames = Object.keys(compileResults).sort();

    if (fileNames.length === 0) {
        outputFileTree.innerHTML = '<div class="p-4 text-gray-500 text-sm text-center">No output yet</div>';
        return;
    }

    outputFileTree.innerHTML = fileNames.map(filename => {
        const result = compileResults[filename];
        const hasError = !result.success;
        const icon = hasError ? '✗' : '✓';
        const statusClass = hasError ? 'file-error' : 'file-success';

        // Convert .wh to .kt for display
        const displayName = filename.replace(/\.wh$/, '.kt');

        return `
            <div class="file-item ${filename === activeOutputFile ? 'active' : ''} ${statusClass}" data-output-filename="${filename}">
                <span class="file-item-icon">${icon}</span>
                <span class="file-item-name">${displayName}</span>
            </div>
        `;
    }).join('');

    // Add click handlers
    outputFileTree.querySelectorAll('.file-item').forEach(item => {
        item.addEventListener('click', () => {
            const filename = item.dataset.outputFilename;
            switchToOutputFile(filename);
        });
    });
}

function switchToOutputFile(filename) {
    if (!compileResults[filename]) return;

    activeOutputFile = filename;
    renderOutputFileTree();

    const result = compileResults[filename];

    // Update the displayed output
    if (currentTab === 'kotlin') {
        renderKotlinOutput(result.output);
    } else if (currentTab === 'ast' && result.ast) {
        document.getElementById('ast-output').textContent =
            JSON.stringify(JSON.parse(result.ast), null, 2);
    }
}

function updateOutputFileTreeVisibility() {
    const container = document.getElementById('output-file-tree-container');
    // Show output tree only on kotlin and ast tabs, hide on errors tab
    if (currentTab === 'kotlin' || currentTab === 'ast') {
        container.classList.remove('hidden');
    } else {
        container.classList.add('hidden');
    }
}

// Sidebar toggle functions
function toggleFileSidebar() {
    console.log('toggleFileSidebar called');
    fileSidebarCollapsed = !fileSidebarCollapsed;
    const sidebar = document.getElementById('file-sidebar');
    const toggleBtn = document.getElementById('toggle-file-sidebar-btn');
    console.log('sidebar:', sidebar, 'toggleBtn:', toggleBtn);

    if (fileSidebarCollapsed) {
        sidebar.classList.add('collapsed');
        toggleBtn.textContent = '▶';
        toggleBtn.title = 'Expand sidebar';
    } else {
        sidebar.classList.remove('collapsed');
        toggleBtn.textContent = '◀';
        toggleBtn.title = 'Collapse sidebar';
    }
}

function toggleOutputSidebar() {
    outputSidebarCollapsed = !outputSidebarCollapsed;
    const container = document.getElementById('output-file-tree-container');
    const toggleBtn = document.getElementById('toggle-output-sidebar-btn');

    if (outputSidebarCollapsed) {
        container.classList.add('collapsed');
        toggleBtn.textContent = '▶';
        toggleBtn.title = 'Expand sidebar';
    } else {
        container.classList.remove('collapsed');
        toggleBtn.textContent = '◀';
        toggleBtn.title = 'Collapse sidebar';
    }
}

// Example snippets - ordered for learning progression
// Each example builds on concepts from previous ones
const examples = {
    'hello': {
        name: '01. Hello World',
        code: `// The simplest Whitehall app - just render text!
<Text>Hello, Whitehall!</Text>`
    },
    'variables': {
        name: '02. Variables',
        code: `// Variables hold data. Use {curly braces} to display them.
var name = "World"
var version = 1

// Interpolation: embed variables in text with {expression}
<Column padding={16} gap={8}>
  <Text>Hello, {name}!</Text>
  <Text>Version: {version}</Text>
  <Text>Name has {name.length} characters</Text>
</Column>`
    },
    'button': {
        name: '03. Buttons & State',
        code: `// 'var' creates reactive state - changes automatically update the UI
var count = 0

<Column padding={16} gap={12}>
  <Text fontSize={24}>{count}</Text>

  // onClick runs when the button is pressed
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
    'conditional': {
        name: '04. Conditionals',
        code: `// @if shows or hides content based on a condition
var isLoggedIn = false

<Column padding={16} gap={12}>
  @if (isLoggedIn) {
    <Text fontSize={20}>Welcome back!</Text>
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
        name: '05. Lists & Loops',
        code: `// Arrays use [square brackets]
var fruits = ["Apple", "Banana", "Cherry"]

// @for loops over each item in a collection
<Column padding={16} gap={8}>
  <Text fontSize={20} fontWeight="bold">Fruits</Text>

  @for (fruit in fruits) {
    <Text>• {fruit}</Text>
  }

  <Text color="#888">{fruits.size} items</Text>
</Column>`
    },
    'binding': {
        name: '06. Text Input',
        code: `// bind:value creates two-way binding
// When user types, the variable updates. When variable changes, input updates.
var name = ""

<Column padding={16} gap={12}>
  <TextField
    bind:value={name}
    label="Enter your name"
  />

  <Text>Hello, {name.isEmpty() ? "stranger" : name}!</Text>
</Column>`
    },
    'derived': {
        name: '07. Derived State',
        code: `// 'var' = mutable state (can change)
// 'val' = derived/computed (auto-updates when dependencies change)
var quantity = 1
var price = 10

// These recalculate automatically when quantity or price changes
val subtotal = price * quantity
val tax = subtotal * 0.1
val total = subtotal + tax

<Column padding={16} gap={12}>
  <Text fontSize={20} fontWeight="bold">Order</Text>

  <Row gap={8}>
    <Button onClick={() => if (quantity > 1) quantity--}>-</Button>
    <Text fontSize={18}>{quantity}</Text>
    <Button onClick={() => quantity++}>+</Button>
  </Row>

  <Text>Subtotal: \${subtotal}</Text>
  <Text>Tax (10%): \${tax}</Text>
  <Text fontWeight="bold">Total: \${total}</Text>
</Column>`
    },
    'layout': {
        name: '08. Layouts',
        code: `// Column stacks children vertically
// Row arranges children horizontally
// spacing = gap between items, padding = space inside container
<Column padding={16} gap={16}>
  <Text fontSize={20} fontWeight="bold">Layouts</Text>

  <Text>Column (vertical):</Text>
  <Column gap={4}>
    <Text>First</Text>
    <Text>Second</Text>
    <Text>Third</Text>
  </Column>

  <Text>Row (horizontal):</Text>
  <Row gap={8}>
    <Text>Left</Text>
    <Text>•</Text>
    <Text>Right</Text>
  </Row>
</Column>`
    },
    'styling': {
        name: '09. Styling',
        code: `// Style props: fontSize, fontWeight, color
// Colors: hex (#FF5722) or theme names (primary, error)
<Column padding={16} gap={8}>
  <Text fontSize={24} fontWeight="bold">
    Large Bold
  </Text>

  <Text fontSize={14} color="#666666">
    Small gray text
  </Text>

  <Text color="#2196F3">
    Blue text (hex color)
  </Text>

  <Text color="primary">
    Theme primary color
  </Text>

  <Text color="error">
    Theme error color
  </Text>
</Column>`
    },
    'interactive-list': {
        name: '10. Interactive List',
        code: `// Combine state, loops, and input for dynamic lists
var items = ["Apple", "Banana"]
var newItem = ""

<Column padding={16} gap={8}>
  <Text fontSize={20} fontWeight="bold">Shopping List</Text>

  <Row gap={8}>
    <TextField
      bind:value={newItem}
      label="Add item"
      modifier={Modifier.weight(1f)}
    />
    <Button onClick={() => {
      if (newItem.isNotEmpty()) {
        items = items + newItem
        newItem = ""
      }
    }}>
      Add
    </Button>
  </Row>

  @for (item in items) {
    <Row gap={8}>
      <Text modifier={Modifier.weight(1f)}>{item}</Text>
      <Button onClick={() => items = items - item}>
        Remove
      </Button>
    </Row>
  }
</Column>`
    },
    'form': {
        name: '11. Form Validation',
        code: `// Combine derived state with conditionals for validation
var email = ""
var password = ""

// Validation rules as derived state
val isValidEmail = email.contains("@")
val isValidPassword = password.length >= 8
val canSubmit = isValidEmail && isValidPassword

<Column padding={16} gap={12}>
  <Text fontSize={20} fontWeight="bold">Login</Text>

  <TextField bind:value={email} label="Email" />
  @if (email.isNotEmpty() && !isValidEmail) {
    <Text color="error">Must contain @</Text>
  }

  <TextField bind:value={password} label="Password" type="password" />
  @if (password.isNotEmpty() && !isValidPassword) {
    <Text color="error">Must be 8+ characters</Text>
  }

  <Button enabled={canSubmit} onClick={() => {}}>
    {canSubmit ? "Login" : "Fill in all fields"}
  </Button>
</Column>`
    },
    'todo': {
        name: '12. Todo App',
        code: `// Capstone: combines everything you've learned!
var todos = ["Learn Whitehall", "Build an app"]
var newTodo = ""

<Column padding={16} gap={12}>
  <Text fontSize={24} fontWeight="bold">My Todos</Text>

  <Row gap={8}>
    <TextField
      bind:value={newTodo}
      label="New todo"
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
      <Row gap={8}>
        <Text modifier={Modifier.weight(1f)}>{todo}</Text>
        <Button onClick={() => todos = todos - todo}>
          Done
        </Button>
      </Row>
    </Card>
  }

  <Text color="#888">{todos.size} remaining</Text>
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

    // Initialize file tree
    renderFileTree();

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
            const decoded = decodeURIComponent(atob(hash));
            // Try to parse as JSON (multi-file format)
            try {
                const parsed = JSON.parse(decoded);
                if (parsed.files) {
                    // Multi-file format: { files: {...}, activeFile: "..." }
                    files = parsed.files;
                    activeFile = parsed.activeFile || Object.keys(files)[0];
                    return files[activeFile];
                }
            } catch (e) {
                // Not JSON, treat as single file
                files = { 'Main.wh': decoded };
                activeFile = 'Main.wh';
                return decoded;
            }
        } catch (e) {
            console.error('Invalid URL hash');
        }
    }
    // Default example - start with Hello World
    files = { 'Main.wh': examples.hello.code };
    activeFile = 'Main.wh';
    return examples.hello.code;
}

// Compile Whitehall code (multi-file support)
async function compile() {
    // Save current editor content before compiling
    if (activeFile && editor) {
        files[activeFile] = editor.getValue();
    }

    // Update status
    setStatus('compiling');

    try {
        const response = await fetch(`${BACKEND_URL}/api/compile`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ files }),
        });

        const result = await response.json();

        // Handle multi-file response
        if (result.results) {
            // Multi-file response
            compileResults = result.results;

            // Count total errors across all files
            let totalErrors = 0;
            for (const fileResult of Object.values(result.results)) {
                if (fileResult.errors) {
                    totalErrors += fileResult.errors.length;
                }
            }

            // Update UI based on active file's result
            const activeResult = compileResults[activeFile];
            if (activeResult) {
                if (activeResult.success) {
                    await renderKotlinOutput(activeResult.output);
                    updateEditorErrors([]);
                    renderErrorPanel([]);

                    // Show AST if available
                    document.getElementById('ast-output').textContent =
                        activeResult.ast ? JSON.stringify(JSON.parse(activeResult.ast), null, 2) : 'AST view not available';
                } else {
                    updateEditorErrors(activeResult.errors);
                    renderErrorPanel(activeResult.errors);
                }
            }

            // Update status and error count
            if (result.success && totalErrors === 0) {
                setStatus('success');
                document.getElementById('error-count').classList.add('hidden');
            } else {
                setStatus('error');
                const errorCountEl = document.getElementById('error-count');
                errorCountEl.textContent = totalErrors;
                errorCountEl.classList.remove('hidden');

                // Auto-switch to Errors tab
                if (totalErrors > 0 && !activeResult.success) {
                    switchTab('errors');
                }
            }

            // Update output file tree
            renderOutputFileTree();
            updateOutputFileTreeVisibility();
        } else {
            // Fallback: single file response (legacy)
            compileResults[activeFile] = result;

            if (result.success) {
                await renderKotlinOutput(result.output);
                updateEditorErrors([]);
                renderErrorPanel([]);
                setStatus('success');
                document.getElementById('error-count').classList.add('hidden');
            } else {
                updateEditorErrors(result.errors);
                renderErrorPanel(result.errors);
                setStatus('error');

                const errorCountEl = document.getElementById('error-count');
                errorCountEl.textContent = result.errors.length;
                errorCountEl.classList.remove('hidden');
                switchTab('errors');
            }

            // Update output file tree
            renderOutputFileTree();
            updateOutputFileTreeVisibility();
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

    // Update output file tree visibility
    updateOutputFileTreeVisibility();
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

// Clear editor (clears current file only)
function clearEditor() {
    if (confirm(`Clear ${activeFile}?`)) {
        if (editor) {
            editor.setValue('');
            files[activeFile] = '';
            compile();
        }
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

// Update URL hash with code (supports multi-file)
function updateURL() {
    // Save current editor content
    if (activeFile && editor) {
        files[activeFile] = editor.getValue();
    }

    // Encode all files as JSON
    const state = {
        files,
        activeFile
    };
    const encoded = btoa(encodeURIComponent(JSON.stringify(state)));
    window.history.replaceState(null, '', '#' + encoded);
}

// Load example
function loadExample(key) {
    if (!key) return;
    const example = examples[key];
    if (example) {
        // Support both old format (code) and new format (files)
        if (example.files) {
            files = { ...example.files }; // Copy files object
            activeFile = Object.keys(files)[0]; // First file
        } else if (example.code) {
            // Legacy format - convert to files
            files = { 'Main.wh': example.code };
            activeFile = 'Main.wh';
        }

        // Update editor
        if (editor && files[activeFile]) {
            editor.setValue(files[activeFile]);
        }

        // Update UI
        document.getElementById('active-file-label').textContent = activeFile;
        renderFileTree();
        compile();

        // Keep the dropdown showing the selected example
        document.getElementById('examples').value = key;

        // Update nav button states
        updateNavButtons();
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
    console.log('DOMContentLoaded fired');

    // Tab buttons
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.addEventListener('click', () => switchTab(btn.dataset.tab));
    });

    // Action buttons
    document.getElementById('format-btn').addEventListener('click', formatCode);
    document.getElementById('clear-btn').addEventListener('click', clearEditor);
    document.getElementById('copy-btn').addEventListener('click', copyOutput);
    document.getElementById('share-btn').addEventListener('click', shareCode);

    // File management buttons
    const addFileBtn = document.getElementById('add-file-btn');
    const deleteFileBtn = document.getElementById('delete-file-btn');
    const toggleFileSidebarBtn = document.getElementById('toggle-file-sidebar-btn');
    const toggleOutputSidebarBtn = document.getElementById('toggle-output-sidebar-btn');

    console.log('Buttons found:', {addFileBtn, deleteFileBtn, toggleFileSidebarBtn, toggleOutputSidebarBtn});

    if (addFileBtn) addFileBtn.addEventListener('click', addNewFile);
    if (deleteFileBtn) deleteFileBtn.addEventListener('click', deleteFile);

    // Sidebar toggle buttons
    if (toggleFileSidebarBtn) toggleFileSidebarBtn.addEventListener('click', toggleFileSidebar);
    if (toggleOutputSidebarBtn) toggleOutputSidebarBtn.addEventListener('click', toggleOutputSidebar);

    // Examples dropdown and navigation
    document.getElementById('examples').addEventListener('change', (e) => loadExample(e.target.value));
    document.getElementById('prev-example-btn').addEventListener('click', prevExample);
    document.getElementById('next-example-btn').addEventListener('click', nextExample);

    // Logo click - go home
    document.getElementById('logo').addEventListener('click', goHome);

    // Set initial dropdown to hello and update nav buttons
    document.getElementById('examples').value = 'hello';
    updateNavButtons();

    console.log('All event listeners attached');
});

// Get ordered list of example keys
function getExampleKeys() {
    return Object.keys(examples);
}

// Update prev/next button states based on current position
function updateNavButtons() {
    const keys = getExampleKeys();
    const dropdown = document.getElementById('examples');
    const currentIndex = keys.indexOf(dropdown.value);
    const prevBtn = document.getElementById('prev-example-btn');
    const nextBtn = document.getElementById('next-example-btn');

    // Disable prev if at first example
    if (currentIndex <= 0) {
        prevBtn.disabled = true;
        prevBtn.classList.add('opacity-50', 'cursor-not-allowed');
        prevBtn.classList.remove('hover:bg-gray-600');
    } else {
        prevBtn.disabled = false;
        prevBtn.classList.remove('opacity-50', 'cursor-not-allowed');
        prevBtn.classList.add('hover:bg-gray-600');
    }

    // Disable next if at last example
    if (currentIndex >= keys.length - 1) {
        nextBtn.disabled = true;
        nextBtn.classList.add('opacity-50', 'cursor-not-allowed');
        nextBtn.classList.remove('hover:bg-gray-600');
    } else {
        nextBtn.disabled = false;
        nextBtn.classList.remove('opacity-50', 'cursor-not-allowed');
        nextBtn.classList.add('hover:bg-gray-600');
    }
}

// Navigate to previous example
function prevExample() {
    const keys = getExampleKeys();
    const dropdown = document.getElementById('examples');
    const currentIndex = keys.indexOf(dropdown.value);
    if (currentIndex > 0) {
        loadExample(keys[currentIndex - 1]);
    }
}

// Navigate to next example
function nextExample() {
    const keys = getExampleKeys();
    const dropdown = document.getElementById('examples');
    const currentIndex = keys.indexOf(dropdown.value);
    if (currentIndex < keys.length - 1) {
        loadExample(keys[currentIndex + 1]);
    }
}

// Go to first example (home)
function goHome() {
    const keys = getExampleKeys();
    loadExample(keys[0]);
}
