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

// Example snippets (ordered by increasing complexity)
// Each example can have multiple files
const examples = {
    'hello': {
        name: '01. Hello World',
        files: {
            'Main.wh': `// The simplest Whitehall app - just render text!
<Text>Hello, Whitehall!</Text>`
        }
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
    'navigation': {
        name: '17. File-Based Routing',
        files: {
            'App.wh': `// App.wh - Main entry point with NavHost
// File-based routing: routes/[name]/+screen.wh

// Back stack for navigation
var backStack = ["home"]
var userId = ""

// Current route from back stack
val currentRoute = backStack[backStack.size - 1]

// Navigation helpers
fun navigateTo(route: String) {
  backStack = backStack + route
}

fun navigateBack() {
  if (backStack.size > 1) {
    backStack = backStack.dropLast(1)
  }
}

// NavHost: loads screen from routes/[name]/+screen.wh
<Column padding={16} spacing={16}>
  <Text>Route: /{currentRoute}</Text>
  <Text>Stack: {backStack.joinToString(" → ")}</Text>

  // Render current route's screen
  @if (currentRoute == "home") {
    // Would import: routes/home/+screen.wh
    <HomeScreen />
  } else if (currentRoute == "profile") {
    // Would import: routes/profile/+screen.wh
    <ProfileScreen />
  } else if (currentRoute == "detail") {
    // Would import: routes/detail/+screen.wh
    <DetailScreen userId={userId} />
  } else {
    // Would import: routes/settings/+screen.wh
    <SettingsScreen />
  }
</Column>`,
            'routes/home/+screen.wh': `// routes/home/+screen.wh
// Home route screen

<Column spacing={8}>
  <Text fontSize={24}>Home Screen</Text>
  <Text>Path: /home</Text>

  <Button onClick={() => navigateTo("profile")}>
    Go to Profile →
  </Button>

  <Button onClick={() => navigateTo("settings")}>
    Go to Settings →
  </Button>
</Column>`,
            'routes/profile/+screen.wh': `// routes/profile/+screen.wh
// Profile route screen

<Column spacing={8}>
  <Text fontSize={24}>Profile Screen</Text>
  <Text>Path: /profile</Text>

  <Button onClick={() => navigateBack()}>
    ← Back
  </Button>

  <Text>User: John Doe</Text>
  <Text>Email: john@example.com</Text>

  <Button onClick={() => {
    userId = "123"
    navigateTo("detail")
  }}>
    View Details →
  </Button>
</Column>`,
            'routes/detail/+screen.wh': `// routes/detail/+screen.wh
// Detail route screen with parameter

@prop val userId: String

<Column spacing={8}>
  <Text fontSize={24}>Detail Screen</Text>
  <Text>Path: /detail</Text>

  <Button onClick={() => navigateBack()}>
    ← Back
  </Button>

  <Text>User ID: {userId}</Text>
  <Text>Name: John Doe</Text>
  <Text>Role: Developer</Text>
</Column>`,
            'routes/settings/+screen.wh': `// routes/settings/+screen.wh
// Settings route screen

var notifications = true

<Column spacing={8}>
  <Text fontSize={24}>Settings Screen</Text>
  <Text>Path: /settings</Text>

  <Button onClick={() => navigateBack()}>
    ← Back
  </Button>

  <Row spacing={8}>
    <Checkbox bind:checked={notifications} />
    <Text>Enable notifications</Text>
  </Row>

  <Text>Version: 1.0.0</Text>
</Column>`
        }
    },
    'multifile': {
        name: '18. Multi-File Project',
        files: {
            'Main.wh': `// Multi-file example: Main screen uses components from other files
// Note: Import functionality is coming soon!

var count = 0
var name = ""

<Column padding={16} spacing={12}>
  <Text fontSize={24} fontWeight="bold">Multi-File Demo</Text>

  <Text>Count: {count}</Text>
  <Button onClick={() => count++}>
    Increment
  </Button>

  <TextField bind:value={name} label="Your name" />
  <Text>Hello, {name}!</Text>
</Column>`,
            'Components.wh': `// Reusable components
// This file would contain shared components

<Column padding={12}>
  <Text fontWeight="bold">Custom Component</Text>
  <Text>This demonstrates multi-file structure</Text>
</Column>`,
            'Utils.wh': `// Utility functions and helpers
// This file would contain helper functions

<Text>Utility helpers would go here</Text>`
        }
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
    // Default example
    files = { 'Main.wh': examples.button.code };
    activeFile = 'Main.wh';
    return examples.button.code;
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

    // Examples dropdown
    document.getElementById('examples').addEventListener('change', (e) => loadExample(e.target.value));

    console.log('All event listeners attached');
});
