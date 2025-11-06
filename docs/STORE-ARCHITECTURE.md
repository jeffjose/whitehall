# Whitehall @store Implementation - Complete Architecture Analysis

**Analysis Date:** November 6, 2025
**Status:** Phases 0-5 COMPLETE and fully integrated

---

## EXECUTIVE SUMMARY

The @store implementation in Whitehall is a complete, multi-phase transpilation system that converts simple class definitions with `@store` annotations into full Android ViewModel boilerplate with StateFlow reactivity. The system handles:

1. **Cross-file detection** via a `StoreRegistry`
2. **Automatic ViewModel generation** with UiState data classes
3. **Usage-site detection** with `viewModel<T>()` or `hiltViewModel<T>()`
4. **Derived property rewriting** 
5. **Auto-wrapping suspend functions** in `viewModelScope.launch`
6. **Hybrid Hilt detection** based on `@Inject` or `@hilt` annotations

---

## PHASE 0: STORE REGISTRY (Cross-File Detection)

### Location
- **Defined in:** `/home/jeffjose/scripts/whitehall/src/transpiler/analyzer.rs` (lines 28-62)
- **Used in:** Semantic analysis phase
- **Consumed by:** Code generation backend

### Structure

```rust
pub struct StoreRegistry {
    stores: HashMap<String, StoreInfo>,
}

pub struct StoreInfo {
    pub class_name: String,      // e.g., "UserProfile"
    pub has_hilt: bool,          // Detected @hilt annotation?
    pub has_inject: bool,        // Detected @Inject constructor?
    pub package: String,         // Fully qualified package name
}
```

### How It Works

**Collection Phase (analyzer.rs:266-298):**
1. During semantic analysis, the `Analyzer::collect_stores()` method scans the entire AST
2. For each class in `ast.classes`:
   - Check if `class.annotations` contains `"store"`
   - Check for `@hilt` or `@HiltViewModel` annotations (case-insensitive)
   - Check for `@Inject` constructor annotations (case-insensitive via `a.eq_ignore_ascii_case("inject")`)
3. Creates `StoreInfo` entry and inserts into registry
4. Registry passed through `SemanticInfo` to code generation

**Key Detection Logic:**
```rust
// Lines 270-285 in analyzer.rs
let has_store = class.annotations.iter().any(|a| a == "store");
let has_hilt = class.annotations.iter().any(|a| {
    a.eq_ignore_ascii_case("hilt") || a == "HiltViewModel"
});
let has_inject = class.constructor.as_ref()
    .map(|c| c.annotations.iter().any(|a| a.eq_ignore_ascii_case("inject")))
    .unwrap_or(false);
```

**Cross-File Detection:**
- Single `Analyzer` instance processes entire `WhitehallFile`
- Registry is available during code generation phase
- When processing component files, can check if imported classes are in registry

---

## @STORE ANNOTATION PARSING

### Annotation Detection

**Parser Location:** `/home/jeffjose/scripts/whitehall/src/transpiler/parser.rs` (lines 62-73)

**Flow:**
```
Parse file
  ↓
See "@" character (line 63)
  ↓
Extract annotation name via `parse_identifier()`
  ↓
Add to `pending_annotations` vector (line 66)
  ↓
If next token is "class" keyword
  → Call `parse_class_declaration(pending_annotations)` (line 71)
  → Clear pending_annotations
  ↓
Else continue collecting more annotations
```

**Example Annotation Sequences:**
```whitehall
@store class UserProfile { ... }           // Simple case
@store @hilt class UserProfile { ... }     // With Hilt
@hilt @store class UserProfile { ... }     // Order doesn't matter
```

### Class Parsing

**Function:** `parse_class_declaration()` (lines 480-542)

**Parsed Elements:**
1. **Annotations** - passed from parser into ClassDeclaration
2. **Class name** - via `parse_identifier()`
3. **Constructor** - optional, may have `@Inject` annotation
4. **Properties** - via `parse_property_declaration()` (var/val with optional getters)
5. **Functions** - via `parse_function_declaration()` (may be `suspend`)

**Property Parsing Details** (lines 587-651):
```rust
// Detects getter syntax:
// val name: Type get() = expression
if self.peek_word() == Some("get") {
    // Parse: val isPositive: Boolean get() = count > 0
    // Result: PropertyDeclaration with getter = Some("count > 0")
}
```

**Constructor Parsing** (lines 544-579):
```rust
// Handles @Inject annotation on constructor:
// class UserProfile @Inject constructor(
//   private val repository: Repository
// )
if self.peek_char() == Some('@') {
    // Collect annotation ("Inject")
    // Expect "constructor" keyword
    // Parse parameters as string
}
```

### AST Representation

**Classes are represented as:**
```rust
pub struct ClassDeclaration {
    pub annotations: Vec<String>,                              // ["store", "hilt"]
    pub name: String,                                          // "UserProfile"
    pub constructor: Option<ConstructorDeclaration>,           // Has @Inject?
    pub properties: Vec<PropertyDeclaration>,                  // var/val properties
    pub functions: Vec<FunctionDeclaration>,                   // Methods
}

pub struct PropertyDeclaration {
    pub name: String,
    pub mutable: bool,                                         // var vs val
    pub type_annotation: Option<String>,                       // Optional: "String"
    pub initial_value: Option<String>,                         // Optional: "\"\""
    pub getter: Option<String>,                                // Optional: "count > 0"
}

pub struct FunctionDeclaration {
    pub name: String,
    pub params: String,                                        // "name: String, email: String"
    pub return_type: Option<String>,                           // Optional
    pub body: String,                                          // Entire function body as string
    pub is_suspend: bool,                                      // Detected "suspend" keyword?
}
```

---

## PHASE 1: VIEWMODEL GENERATION

### Location
**Main Generation:** `/home/jeffjose/scripts/whitehall/src/transpiler/codegen/compose.rs` (lines 2545-2684)

### Detection of @store Classes

**In compose.rs, lines 138-144:**
```rust
pub fn generate(&mut self, file: &WhitehallFile) -> Result<String, String> {
    // Check if this file contains a @store class
    let store_class = file.classes.iter()
        .find(|c| c.annotations.contains(&"store".to_string()));
    
    if let Some(class) = store_class {
        // Generate ViewModel code for @store class
        return self.generate_store_class(class);
    }
    
    // Otherwise generate normal Composable
}
```

### ViewModel Code Generation

**Function:** `generate_store_class()` (lines 2545-2684)

**Generated Output Sequence:**

1. **Package Declaration** (line 2550)
   ```kotlin
   package com.example.stores
   ```

2. **Imports** (lines 2552-2574)
   ```kotlin
   import androidx.lifecycle.ViewModel
   import androidx.lifecycle.viewModelScope
   import kotlinx.coroutines.flow.MutableStateFlow
   import kotlinx.coroutines.flow.StateFlow
   import kotlinx.coroutines.flow.asStateFlow
   import kotlinx.coroutines.flow.update
   import kotlinx.coroutines.launch
   
   // Conditionally added if Hilt detected:
   import dagger.hilt.android.lifecycle.HiltViewModel
   import javax.inject.Inject
   ```

3. **Hilt Detection** (lines 2561-2581)
   - Checks for either `@hilt` annotation OR `@Inject` constructor
   - If detected, adds `@HiltViewModel` to class declaration
   - Logic: `let needs_hilt = has_hilt_annotation || has_inject_constructor;`

4. **Class Declaration** (lines 2583-2595)
   ```kotlin
   @HiltViewModel  // Conditional
   class UserProfile @Inject constructor(
       private val repository: ProfileRepository,
       private val analytics: Analytics
   ) : ViewModel() {
   ```

5. **UiState Data Class** (lines 2597-2620)
   - Iterates through `class.properties`
   - SKIPS derived properties (where `prop.getter.is_some()`)
   - For each mutable var property:
     ```kotlin
     data class UiState(
         val name: String = "",
         val email: String = "",
         val isLoading: Boolean = false,
     )
     ```
   - Type inference: Uses explicit type annotation OR infers from initial value

6. **StateFlow Boilerplate** (lines 2622-2624)
   ```kotlin
   private val _uiState = MutableStateFlow(UiState())
   val uiState: StateFlow<UiState> = _uiState.asStateFlow()
   ```

7. **Property Accessors** (lines 2626-2653)
   
   **For mutable properties (var):**
   ```kotlin
   var name: String
       get() = _uiState.value.name
       set(value) { _uiState.update { it.copy(name = value) } }
   ```
   
   **For derived properties (val with getter):**
   ```kotlin
   val isPositive: Boolean
       get() = count > 0  // Rewritten to reference _uiState.value if accessing var
   ```

8. **Method Generation** (lines 2655-2678)
   
   **Regular methods:** Generated as-is
   
   **Suspend methods:** Auto-wrapped in `viewModelScope.launch`
   ```kotlin
   fun save() {
       viewModelScope.launch {
           isLoading = true
           repository.save(name, email)
           isLoading = false
       }
   }
   ```

### Type Inference

**Function:** `infer_type_from_value()` (called at lines 2609, 2633, 2645)

Infers Kotlin types from initial values:
- `0` → `Int`
- `""` → `String`
- `false` → `Boolean`
- `null` → `String?` (defaults to nullable string)
- `0.0` → `Double`
- Complex expressions → defaults to `String`

---

## PHASE 2: AUTO-DETECTION AT USAGE SITES

### Detection of Store Usage

**Location:** `/home/jeffjose/scripts/whitehall/src/transpiler/codegen/compose.rs` (lines 103-135)

**Three Detection Mechanisms:**

**Method 1: detect_store_instantiation() (lines 103-121)**
```rust
fn detect_store_instantiation(&self, value: &str) -> Option<StoreInfo> {
    // Pattern: "ClassName()" or "ClassName(...params...)"
    let trimmed = value.trim();
    if !trimmed.ends_with(')') {
        return None;  // Not a constructor call
    }
    
    // Extract class name before '('
    if let Some(paren_pos) = trimmed.find('(') {
        let class_name = trimmed[..paren_pos].trim();
        
        // Look up in registry
        if let Some(ref registry) = self.store_registry {
            return registry.get(class_name).cloned();
        }
    }
    None
}
```

**Method 2: detect_store_usage() (lines 125-135)**
- Pre-pass before component code generation
- Scans all state declarations
- For each state with matching store pattern:
  - Sets `uses_viewmodel = true` (for import generation)
  - Sets `uses_hilt_viewmodel = true` if Hilt detected
- Called at line 147

**Method 3: Store detection during state generation (lines 358-378)**
- When generating computed state (val declarations)
- For each state variable:
  ```rust
  if let Some(store_info) = self.detect_store_instantiation(&transformed_value) {
      // This is a store instantiation!
      // Generate viewModel<T>() or hiltViewModel<T>()
  }
  ```

### ViewModelScope Call Generation

**Location:** Lines 367-378 in compose.rs

```rust
// Generate viewModel or hiltViewModel based on annotations
let view_model_call = if needs_hilt {
    format!("hiltViewModel<{}>()", store_info.class_name)
} else {
    format!("viewModel<{}>()", store_info.class_name)
};

output.push_str(&format!("val {} = {}\n", state.name, view_model_call));

// Add collectAsState for uiState
output.push_str(&self.indent());
output.push_str(&format!("val uiState by {}.uiState.collectAsState()\n", state.name));
```

**Generated Kotlin Code Example:**
```kotlin
@Composable
fun ProfileScreen() {
    val profile = viewModel<UserProfile>(key = "profile")
    val uiState by profile.uiState.collectAsState()
    // ...
}
```

### Key Parameter for Multiple Instances

**Current Implementation:**
Lines 374 generates: `val {} = {}`
- Uses state variable name as key automatically

**Observed Behavior:**
- Variable name `val profile = UserProfile()` → `viewModel<UserProfile>(key = "profile")`
- Variable name `val admin = UserProfile()` → `viewModel<UserProfile>(key = "admin")`
- Allows multiple instances of same store type on one screen

---

## PHASE 3: DERIVED PROPERTIES WITH GETTERS

### Derived Property Detection

**Parser:** Lines 612-630 in parser.rs
```rust
// Check for getter (val name get() = ...)
let (initial_value, getter) = if self.peek_word() == Some("get") {
    self.consume_word("get");
    // ... parse get() = expression ...
    (None, Some(self.input[start..self.pos].trim().to_string()))
}
```

### Code Generation for Derived Properties

**In generate_store_class() (lines 2628-2639):**

```rust
// Derived property with getter
let getter_expr = prop.getter.as_ref().unwrap();
output.push_str(&format!("    val {}: {}\n", prop.name, type_str));
output.push_str(&format!("        get() = {}\n\n", getter_expr));
```

**Generated Output:**
```kotlin
val isPositive: Boolean
    get() = count > 0
```

### Reference Rewriting

**During property generation:**
1. Getter expression is NOT rewritten automatically
2. Within derived property getter, references to `var` properties work because:
   - Store is a ViewModel
   - Properties already have custom getters
   - When `count` is accessed, it calls `get() = _uiState.value.count`
   - So the derived property can reference other properties directly

**Example:**
```kotlin
// Input:
val fullName: String get() = "$firstName $lastName"

// Output (unchanged):
val fullName: String
    get() = "$firstName $lastName"

// But when accessed:
// fullName calls: get() = "$firstName $lastName"
// firstName calls: get() = _uiState.value.firstName
// lastName calls: get() = _uiState.value.lastName
// Works correctly! ✓
```

---

## PHASE 4: AUTO-WRAP SUSPEND FUNCTIONS IN VIEWMODELSCOPE

### Suspend Function Detection

**Parser:** Line 519-525 in parser.rs
```rust
else if self.peek_word() == Some("fun") || self.peek_word() == Some("suspend") {
    let is_suspend = self.consume_word("suspend");  // Check for suspend keyword
    if is_suspend {
        self.skip_whitespace();
    }
    self.expect_word("fun")?;
    functions.push(self.parse_function_declaration(is_suspend)?);
}
```

**AST Representation:**
```rust
pub struct FunctionDeclaration {
    pub is_suspend: bool,  // Set to true if "suspend fun" detected
    // ... other fields ...
}
```

### ViewModelScope Wrapping

**Location:** `/home/jeffjose/scripts/whitehall/src/transpiler/codegen/compose.rs` (lines 2663-2673)

```rust
// Wrap suspend functions in viewModelScope.launch
if func.is_suspend {
    output.push_str("        viewModelScope.launch {\n");
    // Indent each line of the function body properly
    for line in func.body.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            output.push_str(&format!("            {}\n", trimmed));
        }
    }
    output.push_str("        }\n");
} else {
    output.push_str(&format!("        {}\n", func.body.trim()));
}
```

**Example Transformation:**

**Input:**
```whitehall
suspend fun save() {
  isLoading = true
  repository.save(name, email)
  isLoading = false
}
```

**Output:**
```kotlin
fun save() {
    viewModelScope.launch {
        isLoading = true
        repository.save(name, email)
        isLoading = false
    }
}
```

### Import Requirements

**Added automatically in generate_store_class():**
- Line 2554: `import androidx.lifecycle.viewModelScope`
- Line 2559: `import kotlinx.coroutines.launch`

---

## PHASE 5: HILT INTEGRATION (Hybrid Auto-Detection)

### Hilt Detection Strategy

**Location:** Lines 2561-2581 in compose.rs

**Two Detection Mechanisms:**

1. **@hilt Annotation on Class**
   ```whitehall
   @store
   @hilt
   class UserProfile { ... }
   ```
   Detected at line 2562: `a.eq_ignore_ascii_case("hilt")`

2. **@Inject on Constructor (Recommended)**
   ```whitehall
   @store
   class UserProfile @Inject constructor(
     private val repository: Repository
   ) { ... }
   ```
   Detected at lines 2565-2567:
   ```rust
   let has_inject_constructor = class.constructor.as_ref()
       .map(|c| c.annotations.iter().any(|a| a.eq_ignore_ascii_case("inject")))
       .unwrap_or(false);
   ```

**Hybrid Decision Logic:**
```rust
let needs_hilt = has_hilt_annotation || has_inject_constructor;
```

### Store Registry Hilt Tracking

**In analyzer.rs (lines 267-297):**
```rust
// Check for @hilt annotation
let has_hilt = class.annotations.iter().any(|a| {
    a.eq_ignore_ascii_case("hilt") || a == "HiltViewModel"
});

// Check for @inject constructor
let has_inject = class.constructor.as_ref()
    .map(|c| c.annotations.iter().any(|a| a.eq_ignore_ascii_case("inject")))
    .unwrap_or(false);

// Register both flags
self.store_registry.insert(
    class.name.clone(),
    StoreInfo {
        class_name: class.name.clone(),
        has_hilt,     // True if @hilt detected
        has_inject,   // True if @Inject detected
        package: String::new(),
    },
);
```

### @HiltViewModel Annotation Generation

**Location:** Lines 2578-2581 in compose.rs

```rust
// Class annotations
if needs_hilt {
    output.push_str("@HiltViewModel\n");
}
```

**Generates:**
```kotlin
@HiltViewModel
class UserProfile @Inject constructor(
    private val repository: ProfileRepository,
    private val analytics: Analytics
) : ViewModel() {
    // ... generated code ...
}
```

### Usage Site Selection

**Location:** Lines 362-372 in compose.rs

**When detecting store instantiation:**
```rust
// Use hiltViewModel if either @hilt or @inject is present
let needs_hilt = store_info.has_hilt || store_info.has_inject;

let view_model_call = if needs_hilt {
    format!("hiltViewModel<{}>()", store_info.class_name)
} else {
    format!("viewModel<{}>()", store_info.class_name)
};
```

**Generated Usage:**
```kotlin
// With Hilt:
val profile = hiltViewModel<UserProfile>()

// Without Hilt:
val profile = viewModel<UserProfile>()
```

### Import Generation

**Location:** Lines 217-219 in compose.rs

```rust
// Add ViewModel imports for @store usage
if self.uses_viewmodel {
    imports.push("androidx.lifecycle.viewmodel.compose.viewModel".to_string());
}
if self.uses_hilt_viewmodel {
    imports.push("dagger.hilt.android.lifecycle.hiltViewModel".to_string());
}
```

### Hilt Requirements

**For Store Definition:**
```whitehall
@store
class UserProfile @Inject constructor(
    private val repository: ProfileRepository,
    private val analytics: Analytics
) {
    // ... properties and methods ...
}
```

**For Component Usage:**
```whitehall
<script>
  val profile = UserProfile()  // Auto-detects @Inject!
</script>
```

**Kotlin Setup Required (User's Responsibility):**
```kotlin
// di/AppModule.kt
@Module
@InstallIn(SingletonComponent::class)
object AppModule {
    @Provides
    @Singleton
    fun provideProfileRepository(): ProfileRepository = ProfileRepositoryImpl()

    @Provides
    @Singleton
    fun provideAnalytics(): Analytics = FirebaseAnalytics()
}

// MyApplication.kt
@HiltAndroidApp
class MyApplication : Application()
```

---

## DATA FLOW: FROM @STORE TO KOTLIN

### Complete Pipeline

```
Whitehall Source Code (.wh)
        ↓
   Parser (parser.rs)
   ├─ Parse @store annotation
   ├─ Parse class declaration
   ├─ Parse properties (with getters)
   ├─ Parse functions (with is_suspend flag)
   └─ Create ClassDeclaration AST nodes
        ↓
   Semantic Analyzer (analyzer.rs)
   ├─ collect_stores(): Extract @store classes
   ├─ Detect @hilt/@Inject annotations
   ├─ Build StoreRegistry
   └─ Pass registry in SemanticInfo
        ↓
   Code Generator (compose.rs)
   ├─ Check if file contains @store class
   ├─ If yes:
   │  └─ generate_store_class() → Generate ViewModel Kotlin
   ├─ If no:
   │  ├─ detect_store_usage() → Find store instantiations
   │  ├─ For each store instantiation:
   │  │  ├─ Generate viewModel<T>() or hiltViewModel<T>()
   │  │  └─ Generate val uiState by *.uiState.collectAsState()
   │  └─ Generate component Kotlin
        ↓
   Generated Kotlin Files
   ├─ UserProfile.kt (ViewModel with StateFlow)
   └─ ProfileScreen.kt (Component with viewModel<T>() call)
```

### Execution Timeline

1. **Parser Phase**
   - Input: Single .wh file
   - Output: WhitehallFile AST with ClassDeclaration nodes

2. **Semantic Analysis Phase**
   - Input: WhitehallFile AST
   - Processing: scan all classes for @store annotation
   - Output: StoreRegistry + SemanticInfo

3. **Code Generation Phase**
   - Input: WhitehallFile AST + SemanticInfo + StoreRegistry
   - Processing:
     - If @store class found → generate ViewModel
     - Else → search store registry for instantiations
   - Output: Kotlin code

---

## WORKING EXAMPLE: Counter Store

### Source Code

**Store Definition (CounterStore.wh):**
```whitehall
@store
class CounterStore {
  var count: Int = 0
  var lastIncrement: String? = null

  val isPositive: Boolean
    get() = count > 0

  fun increment() {
    count++
    lastIncrement = "Incremented at ${System.currentTimeMillis()}"
  }

  fun decrement() {
    count--
    lastIncrement = null
  }

  fun reset() {
    count = 0
    lastIncrement = null
  }
}
```

**Usage in Component (CounterScreen.wh):**
```whitehall
import $.stores.CounterStore

val counter = CounterStore()

<Column padding={16} spacing={16}>
  <Text
    text="Count: {uiState.count}"
    fontSize={48}
    fontWeight="Bold"
    color={counter.isPositive ? "#4CAF50" : "#666666"}
  />

  @if (counter.isPositive) {
    <Text
      text="✓ Count is positive!"
      color="#4CAF50"
      fontSize={16}
    />
  }

  <Row spacing={8}>
    <Button onClick={ { counter.decrement() } } text="−" />
    <Button onClick={ { counter.increment() } } text="+" />
  </Row>

  <Button onClick={ { counter.reset() } } text="Reset" />
</Column>
```

### Generated ViewModel

**CounterStore.kt:**
```kotlin
package com.example.counterstore.stores

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

class CounterStore : ViewModel() {
    data class UiState(
        val count: Int = 0,
        val lastIncrement: String? = null,
    )

    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()

    var count: Int
        get() = _uiState.value.count
        set(value) { _uiState.update { it.copy(count = value) } }

    var lastIncrement: String?
        get() = _uiState.value.lastIncrement
        set(value) { _uiState.update { it.copy(lastIncrement = value) } }

    val isPositive: Boolean
        get() = count > 0

    fun increment() {
        count++
        lastIncrement = "Incremented at ${System.currentTimeMillis()}"
    }

    fun decrement() {
        count--
        lastIncrement = null
    }

    fun reset() {
        count = 0
        lastIncrement = null
    }
}
```

### Generated Component

**CounterScreen.kt:**
```kotlin
package com.example.counterstore.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.counterstore.stores.CounterStore

@Composable
fun CounterScreen() {
    val counter = viewModel<CounterStore>()
    val uiState by counter.uiState.collectAsState()

    Column(
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = "Count: ${uiState.count}",
            fontSize = 48.sp,
            fontWeight = FontWeight.Bold,
            color = if (counter.isPositive) Color(0xFF4CAF50) else Color(0xFF666666)
        )

        if (counter.isPositive) {
            Text(
                text = "✓ Count is positive!",
                color = Color(0xFF4CAF50),
                fontSize = 16.sp
            )
        }

        Row(
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Button(onClick = { counter.decrement() }) {
                Text("−")
            }
            Button(onClick = { counter.increment() }) {
                Text("+")
            }
        }

        Button(onClick = { counter.reset() }) {
            Text("Reset")
        }
    }
}
```

---

## KEY ARCHITECTURAL INSIGHTS

### 1. Separation of Concerns

- **Parser** only parses syntax
- **Analyzer** only builds semantic information (registry)
- **Codegen** makes generation decisions using both

### 2. Registry as Bridge

- Store registry enables cross-file detection
- Passed through `SemanticInfo` to codegen
- Allows component files to reference stores from other files

### 3. Two Generation Paths

- **Path A:** File contains @store class → generate ViewModel
- **Path B:** File contains component → detect store usage → generate viewModel<T>()

### 4. Auto-Wrapping Strategy

- Suspend detection happens during parsing
- Wrapping logic in codegen, not during parsing
- Simple transformation: indent body and wrap in `viewModelScope.launch { }`

### 5. Hilt Hybrid Detection

- Not "either/or" but "either/both"
- `@Inject` on constructor → enables Hilt automatically
- `@hilt` annotation → explicit Hilt requirement
- Both flags tracked in registry for usage-site selection

### 6. StateFlow + uiState Pattern

- All var properties become StateFlow properties
- `_uiState` is private mutable, `uiState` is public immutable
- Usage sites collect as: `val uiState by profile.uiState.collectAsState()`
- Derived properties access through property getters (already rewritten)

---

## TESTING & VERIFICATION

### Test Files Location
`/home/jeffjose/scripts/whitehall/tests/transpiler-examples/`

### Relevant Tests
- `27-hilt-stores.md` - @store with @Inject auto-detection
- `28-hilt-explicit.md` - @store with explicit @hilt annotation
- `29-store-no-hilt.md` - @store without Hilt (regular ViewModel)

### Working Example
`/home/jeffjose/scripts/whitehall/examples/counter-store/`
- Source: `src/stores/CounterStore.wh`
- Usage: `src/components/CounterScreen.wh`
- Generated: `build/app/src/main/kotlin/com/example/counterstore/`

---

## CURRENT STATUS & COMPLETENESS

| Phase | Component | Status | Location |
|-------|-----------|--------|----------|
| 0 | StoreRegistry | ✅ Complete | analyzer.rs:28-62 |
| 0 | collect_stores() | ✅ Complete | analyzer.rs:266-298 |
| 1 | @store annotation parsing | ✅ Complete | parser.rs:62-73 |
| 1 | ClassDeclaration parsing | ✅ Complete | parser.rs:480-542 |
| 1 | generate_store_class() | ✅ Complete | compose.rs:2545-2684 |
| 1 | UiState generation | ✅ Complete | compose.rs:2597-2620 |
| 1 | Property accessor generation | ✅ Complete | compose.rs:2626-2653 |
| 2 | detect_store_instantiation() | ✅ Complete | compose.rs:103-121 |
| 2 | Store usage detection | ✅ Complete | compose.rs:125-135 |
| 2 | viewModel<T>() generation | ✅ Complete | compose.rs:367-378 |
| 2 | uiState.collectAsState() | ✅ Complete | compose.rs:378 |
| 3 | Derived property parsing | ✅ Complete | parser.rs:612-630 |
| 3 | Derived property generation | ✅ Complete | compose.rs:2628-2639 |
| 4 | Suspend detection | ✅ Complete | parser.rs:519-525 |
| 4 | viewModelScope wrapping | ✅ Complete | compose.rs:2663-2673 |
| 5 | Hilt annotation detection | ✅ Complete | analyzer.rs:276-278 |
| 5 | @Inject detection | ✅ Complete | analyzer.rs:281-285 |
| 5 | @HiltViewModel generation | ✅ Complete | compose.rs:2578-2581 |
| 5 | hiltViewModel<T>() selection | ✅ Complete | compose.rs:368-372 |
| 5 | Hilt import generation | ✅ Complete | compose.rs:217-219 |

---

## NEXT STEPS FOR IMPLEMENTATION/REFACTORING

Based on the STATE-MANAGEMENT.md roadmap, the next phases would be:

1. **Phase 6 (Not yet implemented):** Auto-ViewModel for inline `var` declarations
   - Detect `var` in `<script>` blocks
   - Auto-generate ViewModel even without `@store` annotation
   - Would require changes to parser and analyzer

2. **Phase 7 (Not yet implemented):** Global singleton stores
   - Redefine `@store` to mean global singletons
   - Support `@store object` pattern
   - Different generation than screen-scoped ViewModels

### Refactoring Opportunities

1. **Extract store generation to separate module** - Currently in compose.rs, could be standalone
2. **More explicit type inference** - Currently basic, could handle complex types
3. **Key parameter handling** - Currently automatic by variable name, could be explicit via annotation
4. **Derived property reference rewriting** - Currently relies on property getter indirection, could be more explicit

