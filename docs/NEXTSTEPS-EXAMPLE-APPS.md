# Next Steps: Example Apps Roadmap

Progressive learning path for Whitehall examples beyond 1-18.

## Current State (Examples 1-18)

✅ **Basics (1-12)**: UI components, forms, navigation, async, theming, lists
✅ **FFI (13-15)**: Rust basic, Rust JSON, C++ basic
✅ **Advanced Async (16-18)**: Lifecycle hooks, computed values, multiple tasks

## Missing Core Features

From LANGUAGE-REFERENCE.md, these features are NOT yet demonstrated:

### Critical (Core Features)
- ❌ **Class-based ViewModels** - `class Store { var ... }` → auto-generates ViewModel
- ❌ **String resources (i18n)** - `R.string.*`, parameters, plurals
- ❌ **Helper composable functions** - Functions with markup bodies
- ❌ **Global singleton stores** - `@store object Settings { var ... }`

### Data & Syntax
- ❌ **Array syntax** - `[1, 2, 3]`, `var mutable = [10, 20]`
- ❌ **Ranges in use** - `1..100`, `0..20:2` (mentioned but not used)
- ❌ **Multi-line strings** - `"""..."""`
- ❌ **Safe navigation** - `user?.name ?: "Unknown"`
- ❌ **Ternary operator** - `count > 0 ? "Items" : "No items"`

### Patterns & Architecture
- ❌ **Pattern matching** - `@when` with sealed classes (have basic, not sealed)
- ❌ **Error handling** - Try/catch, Result types, error states
- ❌ **Form validation** - Complex multi-field validation
- ❌ **Navigation with arguments** - Passing data between screens

### Advanced
- ❌ **Hilt integration** - `@Inject`, `@HiltViewModel`
- ❌ **Mixed FFI** - Rust calling C++, complex interop
- ❌ **State persistence** - Saving/loading state
- ❌ **Import shortcuts** - `import $models.User`, `import $stores`

---

## Proposed Examples 19-27

### **Example 19: ViewModel Classes**
**Complexity**: Medium, ~80 lines
**Combines**: Class ViewModels + Derived properties + Complex state

```whitehall
// THE killer feature - class with var → auto ViewModel
class UserProfileStore {
  var firstName = ""
  var lastName = ""
  var email = ""
  var bio = ""
  var isLoading = false
  var saveSuccess = false
  var errorMessage = ""

  // Derived properties
  val fullName get() = "$firstName $lastName".trim()
  val isValid get() = firstName.isNotEmpty() &&
                       lastName.isNotEmpty() &&
                       email.contains("@")
  val bioLength get() = bio.length

  // Suspend function auto-wrapped in viewModelScope
  suspend fun saveProfile() {
    if (!isValid) {
      errorMessage = "Please complete all required fields"
      return
    }

    isLoading = true
    errorMessage = ""
    delay(2000) // Simulate API call
    isLoading = false
    saveSuccess = true
  }

  fun reset() {
    saveSuccess = false
    errorMessage = ""
  }
}

val store = UserProfileStore()  // Auto uses viewModel<UserProfileStore>()

<Column>
  <TextField bind:value={store.firstName} label="First Name" />
  <TextField bind:value={store.lastName} label="Last Name" />
  <TextField bind:value={store.email} label="Email" />

  <Text>Full Name: {store.fullName}</Text>

  @if (store.errorMessage.isNotEmpty()) {
    <Card backgroundColor="errorContainer">
      <Text color="error">{store.errorMessage}</Text>
    </Card>
  }

  <Button
    onClick={() => store.saveProfile()}
    enabled={store.isValid && !store.isLoading}
  >
    <Text>{store.isLoading ? "Saving..." : "Save Profile"}</Text>
  </Button>
</Column>
```

**Teaches**:
- Class → ViewModel auto-generation
- StateFlow for all var properties
- Derived properties (val get())
- State survives rotation
- Complex validation logic

---

### **Example 20: Helper Functions & Component Composition**
**Complexity**: Medium, ~90 lines
**Combines**: Helper functions + Safe navigation + Component reuse

```whitehall
// Data model
data class User(
  val id: String,
  val name: String,
  val email: String?,
  val role: String
)

var users = [
  User("1", "Alice", "alice@example.com", "Admin"),
  User("2", "Bob", null, "User"),
  User("3", "Carol", "carol@example.com", "User")
]

var selectedUserId = ""

// Helper function - auto-detected as @Composable
fun UserCard(user: User, isSelected: Boolean, onSelect: () -> Unit) {
  <Card
    p={16}
    backgroundColor={isSelected ? "primaryContainer" : "surface"}
    onClick={onSelect}
  >
    <Column spacing={8}>
      <Row spacing={8}>
        <Text fontSize={18} fontWeight="bold">{user.name}</Text>
        <Text fontSize={12} color="primary">{user.role}</Text>
      </Row>

      // Safe navigation operator
      <Text fontSize={14} color="#666">
        {user.email ?: "No email provided"}
      </Text>

      @if (isSelected) {
        <Text fontSize={12} color="primary">✓ Selected</Text>
      }
    </Column>
  </Card>
}

// Another helper - stats summary
fun UserStats(totalUsers: Int, admins: Int) {
  <Card p={12} backgroundColor="secondaryContainer">
    <Row spacing={16}>
      <Text>Total: {totalUsers}</Text>
      <Text>Admins: {admins}</Text>
      <Text>Users: {totalUsers - admins}</Text>
    </Row>
  </Card>
}

val adminCount = users.filter { it.role == "Admin" }.size

<Column spacing={16} p={20}>
  <Text fontSize={32} fontWeight="bold">User Directory</Text>

  <UserStats totalUsers={users.size} admins={adminCount} />

  @for (user in users) {
    <UserCard
      user={user}
      isSelected={user.id == selectedUserId}
      onSelect={() => selectedUserId = user.id}
    />
  }
</Column>
```

**Teaches**:
- Helper functions with markup (auto-@Composable)
- Safe navigation (`?.`) and elvis (`?:`) operators
- Ternary operator (`? :`)
- Component composition and reuse
- Passing callbacks to child components
- Clean code organization

---

### **Example 21: String Resources (i18n)**
**Complexity**: Medium, ~70 lines
**Combines**: i18n + Plurals + String formatting

Create `build/app/src/main/res/values/strings.xml`:
```xml
<resources>
    <string name="app_name">Shopping Cart</string>
    <string name="welcome">Welcome to our store!</string>
    <string name="greeting">Hello, %1$s!</string>
    <string name="price_format">$%1$.2f</string>
    <string name="add_to_cart">Add to Cart</string>
    <string name="checkout">Checkout</string>
    <string name="empty_cart">Your cart is empty</string>

    <plurals name="item_count">
        <item quantity="zero">No items</item>
        <item quantity="one">%d item</item>
        <item quantity="other">%d items</item>
    </plurals>
</resources>
```

```whitehall
import android.R

var userName = "Alice"
var cartItems = []
var selectedProduct = ""

data class Product(val name: String, val price: Double)

val products = [
  Product("Coffee", 4.99),
  Product("Tea", 3.49),
  Product("Pastry", 5.99)
]

fun addToCart(product: Product) {
  cartItems = cartItems + product
}

<Column p={20} spacing={16}>
  <Text fontSize={32} fontWeight="bold">{R.string.app_name}</Text>

  <Card p={16} backgroundColor="primaryContainer">
    <Text>{R.string.greeting(userName)}</Text>
  </Card>

  <Text fontSize={18} fontWeight="bold">Products:</Text>

  @for (product in products) {
    <Card p={12}>
      <Row spacing={8}>
        <Column spacing={4}>
          <Text fontSize={16}>{product.name}</Text>
          <Text fontSize={14}>{R.string.price_format(product.price)}</Text>
        </Column>
        <Button
          onClick={() => addToCart(product)}
          text={R.string.add_to_cart}
        />
      </Row>
    </Card>
  }

  <Card p={16} backgroundColor="secondaryContainer">
    @if (cartItems.isEmpty()) {
      <Text>{R.string.empty_cart}</Text>
    } else {
      <Column spacing={8}>
        <Text fontSize={18} fontWeight="bold">
          {R.plurals.item_count(cartItems.size)}
        </Text>
        <Button text={R.string.checkout} fillMaxWidth={true} />
      </Column>
    }
  </Card>
</Column>
```

**Teaches**:
- String resources (`R.string.*`)
- String formatting with parameters
- Plurals (`R.plurals.*`)
- Localization patterns
- Production-ready i18n

---

### **Example 22: Collections, Arrays & Ranges**
**Complexity**: Medium, ~80 lines
**Combines**: Array syntax + Ranges + Data manipulation

```whitehall
// Array syntax
var numbers = [1, 2, 3, 4, 5]
var names = ["Alice", "Bob", "Carol"]
var selectedIndices = []

// Ranges
val smallRange = 1..10
val evenNumbers = 0..20:2
val countdown = 10..1:-1

var startRange = 1
var endRange = 100
var stepSize = 5

// Computed from ranges
val customRange = startRange..endRange:stepSize

<Column p={20} spacing={16}>
  <Text fontSize={32} fontWeight="bold">Collections Demo</Text>

  <Card p={16}>
    <Column spacing={8}>
      <Text fontSize={18} fontWeight="bold">Array Operations:</Text>
      <Text>Numbers: {numbers.joinToString(", ")}</Text>
      <Text>Sum: {numbers.sum()}</Text>
      <Text>Average: {numbers.average()}</Text>
      <Text>Max: {numbers.maxOrNull() ?: 0}</Text>

      <Row spacing={8}>
        <Button onClick={() => numbers = numbers + (numbers.size + 1)}>
          <Text>Add Number</Text>
        </Button>
        <Button
          onClick={() => numbers = numbers.dropLast(1)}
          enabled={numbers.isNotEmpty()}
        >
          <Text>Remove Last</Text>
        </Button>
      </Row>
    </Column>
  </Card>

  <Card p={16}>
    <Column spacing={8}>
      <Text fontSize={18} fontWeight="bold">Range Examples:</Text>
      <Text fontSize={14}>Simple (1..10): {smallRange.take(5).joinToString(", ")}...</Text>
      <Text fontSize={14}>Even (0..20:2): {evenNumbers.take(5).joinToString(", ")}...</Text>
      <Text fontSize={14}>Countdown (10..1:-1): {countdown.take(5).joinToString(", ")}...</Text>
    </Column>
  </Card>

  <Card p={16}>
    <Column spacing={12}>
      <Text fontSize={18} fontWeight="bold">Custom Range Builder:</Text>

      <Row spacing={8}>
        <TextField
          value={startRange.toString()}
          onValueChange={(v) => startRange = v.toIntOrNull() ?: 1}
          label="Start"
        />
        <TextField
          value={endRange.toString()}
          onValueChange={(v) => endRange = v.toIntOrNull() ?: 100}
          label="End"
        />
        <TextField
          value={stepSize.toString()}
          onValueChange={(v) => stepSize = v.toIntOrNull() ?: 1}
          label="Step"
        />
      </Row>

      <Text>Result: {customRange.take(10).joinToString(", ")}...</Text>
      <Text fontSize={12} color="#666">Showing first 10 of {customRange.count()} items</Text>
    </Column>
  </Card>

  <Card p={16}>
    <Column spacing={8}>
      <Text fontSize={18} fontWeight="bold">Name Filter:</Text>

      @for ((index, name) in names.withIndex()) {
        <Row spacing={8}>
          <Checkbox
            checked={selectedIndices.contains(index)}
            onCheckedChange={(checked) =>
              selectedIndices = if (checked)
                selectedIndices + index
              else
                selectedIndices.filter { it != index }
            }
          />
          <Text>{name}</Text>
        </Row>
      }

      @if (selectedIndices.isNotEmpty()) {
        <Text fontSize={14} color="primary">
          Selected: {selectedIndices.map { names[it] }.joinToString(", ")}
        </Text>
      }
    </Column>
  </Card>
</Column>
```

**Teaches**:
- Array syntax `[1, 2, 3]`
- Range syntax `1..10`, `0..20:2`, `10..1:-1`
- Array manipulation (add, remove, filter, map)
- Collection methods (sum, average, max)
- Dynamic ranges based on state
- withIndex() for iteration with indices

---

### **Example 23: Global State & Settings**
**Complexity**: Medium, ~85 lines
**Combines**: @store object + Multi-line strings + Persistence patterns

```whitehall
// Global singleton - NOT a ViewModel, app-wide state
@store object AppSettings {
  var darkMode = false
  var notificationsEnabled = true
  var language = "en"
  var fontSize = 16
  var username = ""

  val availableLanguages = ["en", "es", "fr", "de", "ja"]

  fun reset() {
    darkMode = false
    notificationsEnabled = true
    language = "en"
    fontSize = 16
  }
}

// Multi-line string example
val privacyPolicy = """
Privacy Policy

1. Data Collection
We collect minimal user data.

2. Data Usage
Your data is never shared.

3. Contact
Email: privacy@example.com
"""

var showPolicy = false

<Column p={20} spacing={16}>
  <Text fontSize={32} fontWeight="bold">App Settings</Text>

  <Card p={16}>
    <Column spacing={12}>
      <Text fontSize={18} fontWeight="bold">User Preferences</Text>

      <TextField
        bind:value={AppSettings.username}
        label="Username"
      />

      <Row spacing={8}>
        <Text>Dark Mode</Text>
        <Switch bind:checked={AppSettings.darkMode} />
      </Row>

      <Row spacing={8}>
        <Text>Notifications</Text>
        <Switch bind:checked={AppSettings.notificationsEnabled} />
      </Row>

      <Column spacing={4}>
        <Text>Font Size: {AppSettings.fontSize}sp</Text>
        <Row spacing={8}>
          <Button
            onClick={() => AppSettings.fontSize = Math.max(12, AppSettings.fontSize - 2)}
            text="-"
          />
          <Button
            onClick={() => AppSettings.fontSize = Math.min(24, AppSettings.fontSize + 2)}
            text="+"
          />
        </Row>
      </Column>

      <Column spacing={4}>
        <Text>Language</Text>
        @for (lang in AppSettings.availableLanguages) {
          <Row spacing={8}>
            <Checkbox
              checked={AppSettings.language == lang}
              onCheckedChange={(checked) =>
                if (checked) AppSettings.language = lang
              }
            />
            <Text>{lang.uppercase()}</Text>
          </Row>
        }
      </Column>
    </Column>
  </Card>

  <Card p={16} backgroundColor="secondaryContainer">
    <Column spacing={8}>
      <Text fontSize={14} fontWeight="bold">Current Settings:</Text>
      <Text fontSize={12}>Dark Mode: {AppSettings.darkMode ? "On" : "Off"}</Text>
      <Text fontSize={12}>Notifications: {AppSettings.notificationsEnabled ? "On" : "Off"}</Text>
      <Text fontSize={12}>Language: {AppSettings.language}</Text>
      <Text fontSize={12}>Font Size: {AppSettings.fontSize}sp</Text>
    </Column>
  </Card>

  <Button
    onClick={() => showPolicy = !showPolicy}
    text={showPolicy ? "Hide Policy" : "Show Privacy Policy"}
    fillMaxWidth={true}
  />

  @if (showPolicy) {
    <Card p={16} backgroundColor="surfaceVariant">
      <Text fontSize={12} fontFamily="monospace">{privacyPolicy}</Text>
    </Card>
  }

  <Button
    onClick={() => AppSettings.reset()}
    text="Reset to Defaults"
  />
</Column>
```

**Teaches**:
- Global singleton with `@store object`
- App-wide state (NOT ViewModel)
- Multi-line strings `"""..."""`
- Settings persistence patterns
- Shared state across screens

---

### **Example 24: Error Handling & Validation**
**Complexity**: Medium-High, ~95 lines
**Combines**: Try/catch + Sealed classes + Pattern matching + Complex validation

```whitehall
sealed class LoadingState<out T> {
  object Idle : LoadingState<Nothing>()
  object Loading : LoadingState<Nothing>()
  data class Success<T>(val data: T) : LoadingState<T>()
  data class Error(val message: String) : LoadingState<Nothing>()
}

data class FormData(
  val email: String,
  val password: String,
  val confirmPassword: String
)

var email = ""
var password = ""
var confirmPassword = ""
var state: LoadingState<String> = LoadingState.Idle()

// Validation
val emailError = if (email.isEmpty())
    ""
  else if (!email.contains("@"))
    "Invalid email format"
  else
    ""

val passwordError = if (password.isEmpty())
    ""
  else if (password.length < 8)
    "Password must be at least 8 characters"
  else
    ""

val confirmError = if (confirmPassword.isEmpty())
    ""
  else if (password != confirmPassword)
    "Passwords do not match"
  else
    ""

val hasErrors = emailError.isNotEmpty() ||
                 passwordError.isNotEmpty() ||
                 confirmError.isNotEmpty()

val isFormComplete = email.isNotEmpty() &&
                      password.isNotEmpty() &&
                      confirmPassword.isNotEmpty()

suspend fun submitForm() {
  if (hasErrors || !isFormComplete) {
    state = LoadingState.Error("Please fix form errors")
    return
  }

  state = LoadingState.Loading()

  try {
    delay(2000) // Simulate API call

    // Simulate random success/failure
    if (Math.random() > 0.3) {
      state = LoadingState.Success("Account created successfully!")
    } else {
      throw Exception("Network error: Could not reach server")
    }
  } catch (e: Exception) {
    state = LoadingState.Error(e.message ?: "Unknown error occurred")
  }
}

<Column p={20} spacing={16}>
  <Text fontSize={32} fontWeight="bold">Sign Up</Text>

  <Card p={16}>
    <Column spacing={12}>
      <Column spacing={4}>
        <TextField
          bind:value={email}
          label="Email"
        />
        @if (emailError.isNotEmpty()) {
          <Text fontSize={12} color="error">{emailError}</Text>
        }
      </Column>

      <Column spacing={4}>
        <TextField
          bind:value={password}
          label="Password"
          type="password"
        />
        @if (passwordError.isNotEmpty()) {
          <Text fontSize={12} color="error">{passwordError}</Text>
        }
      </Column>

      <Column spacing={4}>
        <TextField
          bind:value={confirmPassword}
          label="Confirm Password"
          type="password"
        />
        @if (confirmError.isNotEmpty()) {
          <Text fontSize={12} color="error">{confirmError}</Text>
        }
      </Column>
    </Column>
  </Card>

  // Pattern matching with sealed class
  @when (state) {
    is LoadingState.Idle -> {
      <Card p={12} backgroundColor="surfaceVariant">
        <Text fontSize={12}>Fill out the form to create an account</Text>
      </Card>
    }
    is LoadingState.Loading -> {
      <Card p={16} backgroundColor="primaryContainer">
        <Row spacing={8}>
          <CircularProgressIndicator />
          <Text>Creating account...</Text>
        </Row>
      </Card>
    }
    is LoadingState.Success -> {
      <Card p={16} backgroundColor="secondaryContainer">
        <Column spacing={8}>
          <Text fontSize={16} fontWeight="bold" color="primary">✓ Success</Text>
          <Text>{state.data}</Text>
        </Column>
      </Card>
    }
    is LoadingState.Error -> {
      <Card p={16} backgroundColor="errorContainer">
        <Column spacing={8}>
          <Text fontSize={16} fontWeight="bold" color="error">✗ Error</Text>
          <Text color="error">{state.message}</Text>
        </Column>
      </Card>
    }
  }

  <Button
    onClick={() => submitForm()}
    enabled={!hasErrors && isFormComplete && state !is LoadingState.Loading}
    fillMaxWidth={true}
  >
    <Text>Create Account</Text>
  </Button>

  @if (state is LoadingState.Error || state is LoadingState.Success) {
    <Button
      onClick={() => {
        state = LoadingState.Idle()
        email = ""
        password = ""
        confirmPassword = ""
      }}
      text="Reset Form"
    />
  }
</Column>
```

**Teaches**:
- Sealed classes for state modeling
- Pattern matching with `@when`
- Try/catch error handling
- Complex multi-field validation
- Real-time validation feedback
- State machine patterns (Idle → Loading → Success/Error)

---

### **Example 25: Navigation with Arguments**
**Complexity**: Medium-High, ~100 lines
**Combines**: Navigation + Data passing + Deep state

```whitehall
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController

data class Product(
  val id: String,
  val name: String,
  val price: Double,
  val description: String
)

val products = [
  Product("1", "Laptop", 999.99, "High-performance laptop"),
  Product("2", "Mouse", 29.99, "Wireless ergonomic mouse"),
  Product("3", "Keyboard", 79.99, "Mechanical keyboard with RGB")
]

// Two screens: ProductList and ProductDetail
fun ProductListScreen(onProductClick: (String) -> Unit) {
  <Column p={20} spacing={16}>
    <Text fontSize={32} fontWeight="bold">Products</Text>

    @for (product in products) {
      <Card p={16} onClick={() => onProductClick(product.id)}>
        <Column spacing={4}>
          <Text fontSize={18} fontWeight="bold">{product.name}</Text>
          <Text fontSize={16} color="primary">${product.price}</Text>
        </Column>
      </Card>
    }
  </Column>
}

fun ProductDetailScreen(productId: String, onBack: () -> Unit) {
  val product = products.find { it.id == productId }

  @if (product == null) {
    <Column p={20}>
      <Text>Product not found</Text>
      <Button onClick={onBack} text="Go Back" />
    </Column>
  } else {
    <Column p={20} spacing={16}>
      <Button onClick={onBack} text="← Back" />

      <Card p={20}>
        <Column spacing={12}>
          <Text fontSize={32} fontWeight="bold">{product.name}</Text>
          <Text fontSize={24} color="primary">${product.price}</Text>
          <Text fontSize={16}>{product.description}</Text>

          <Button text="Add to Cart" fillMaxWidth={true} />
        </Column>
      </Card>
    </Column>
  }
}

// Main navigation setup
val navController = rememberNavController()

<NavHost navController={navController} startDestination="list">
  <composable route="list">
    <ProductListScreen
      onProductClick={(id) => navController.navigate("detail/$id")}
    />
  </composable>

  <composable route="detail/{productId}">
    val productId = it.arguments?.getString("productId") ?: ""
    <ProductDetailScreen
      productId={productId}
      onBack={() => navController.popBackStack()}
    />
  </composable>
</NavHost>
```

**Teaches**:
- Navigation with arguments
- Multiple screens in one file
- Data passing between screens
- Safe navigation (null handling)
- Navigation patterns (list → detail)

---

### **Example 26: Advanced FFI - Mixed Rust & C++**
**Complexity**: High, multi-file
**Combines**: Rust calling C++ + Complex FFI patterns

Structure:
```
src/ffi/
  cpp/
    image_processor.cpp    # C++ image processing
  rust/
    src/
      lib.rs               # Rust calls C++ via extern
```

```cpp
// src/ffi/cpp/image_processor.cpp
#include <string>
#include <vector>

// @ffi
int get_image_width(const std::string& path) {
    return 1920;  // Simplified
}

// @ffi
std::vector<int> get_rgb_histogram(const std::string& path) {
    return {100, 150, 200};  // R, G, B averages
}
```

```rust
// src/ffi/rust/src/lib.rs
use whitehall::ffi;

// Link to C++ library
#[link(name = "image_processor")]
extern "C" {
    fn get_image_width(path: *const std::os::raw::c_char) -> i32;
}

#[ffi]
pub fn analyze_image(path: String) -> String {
    unsafe {
        let c_path = std::ffi::CString::new(path).unwrap();
        let width = get_image_width(c_path.as_ptr());
        format!("Image width: {}px", width)
    }
}

#[ffi]
pub fn process_batch(count: i32) -> i32 {
    // Rust processing logic
    count * 2
}
```

```whitehall
// src/main.wh
import $ffi.cpp.ImageProcessor
import $ffi.rust.ImageAnalyzer

var imagePath = "/path/to/image.jpg"
var batchSize = 10

<Column p={20} spacing={16}>
  <Text fontSize={32} fontWeight="bold">Image Processing</Text>

  <Card p={16}>
    <Column spacing={8}>
      <Text fontSize={18} fontWeight="bold">C++ Direct:</Text>
      <Text>Width: {ImageProcessor.getImageWidth(imagePath)}px</Text>

      val histogram = ImageProcessor.getRgbHistogram(imagePath)
      <Text>RGB: R={histogram[0]}, G={histogram[1]}, B={histogram[2]}</Text>
    </Column>
  </Card>

  <Card p={16}>
    <Column spacing={8}>
      <Text fontSize={18} fontWeight="bold">Rust (calls C++):</Text>
      <Text>{ImageAnalyzer.analyzeImage(imagePath)}</Text>
      <Text>Batch result: {ImageAnalyzer.processBatch(batchSize)}</Text>
    </Column>
  </Card>
</Column>
```

**Teaches**:
- Mixed FFI (Rust calling C++)
- Complex foreign function interfaces
- Unsafe Rust for C interop
- Multiple FFI languages in one app

---

### **Example 27: Complete App - Task Manager**
**Complexity**: High, multi-file, ~200+ lines
**Combines**: Everything learned - production-ready app

Structure:
```
src/
  main.wh              # Main nav + routing
  stores/
    TaskStore.wh       # ViewModel
  components/
    TaskCard.wh        # Reusable component
  models/
    Task.wh           # Data models
```

```whitehall
// src/models/Task.wh
sealed class TaskStatus {
  object Todo : TaskStatus()
  object InProgress : TaskStatus()
  object Done : TaskStatus()
}

data class Task(
  val id: String,
  val title: String,
  val description: String,
  val status: TaskStatus,
  val priority: Int,
  val dueDate: Long?
)
```

```whitehall
// src/stores/TaskStore.wh
class TaskStore {
  var tasks = []
  var filter = "all"  // "all", "todo", "done"
  var isLoading = false
  var searchQuery = ""

  val filteredTasks get() = tasks.filter { task ->
    val matchesSearch = searchQuery.isEmpty() ||
                        task.title.contains(searchQuery, ignoreCase = true)
    val matchesFilter = when (filter) {
      "all" -> true
      "todo" -> task.status is TaskStatus.Todo
      "done" -> task.status is TaskStatus.Done
      else -> true
    }
    matchesSearch && matchesFilter
  }

  val stats get() = mapOf(
    "total" to tasks.size,
    "todo" to tasks.count { it.status is TaskStatus.Todo },
    "done" to tasks.count { it.status is TaskStatus.Done }
  )

  suspend fun loadTasks() {
    isLoading = true
    delay(1000)
    // Load from API or local DB
    isLoading = false
  }

  fun addTask(title: String, description: String) {
    val newTask = Task(
      id = UUID.randomUUID().toString(),
      title = title,
      description = description,
      status = TaskStatus.Todo(),
      priority = 0,
      dueDate = null
    )
    tasks = tasks + newTask
  }

  fun toggleStatus(taskId: String) {
    tasks = tasks.map { task ->
      if (task.id == taskId) {
        task.copy(status = when (task.status) {
          is TaskStatus.Todo -> TaskStatus.Done()
          is TaskStatus.Done -> TaskStatus.Todo()
          else -> task.status
        })
      } else task
    }
  }

  fun deleteTask(taskId: String) {
    tasks = tasks.filter { it.id != taskId }
  }
}
```

```whitehall
// src/components/TaskCard.wh
fun TaskCard(
  task: Task,
  onToggle: () -> Unit,
  onDelete: () -> Unit
) {
  <Card p={16}>
    <Column spacing={8}>
      <Row spacing={8}>
        <Checkbox
          checked={task.status is TaskStatus.Done}
          onCheckedChange={(checked) => onToggle()}
        />
        <Column spacing={4}>
          <Text
            fontSize={16}
            fontWeight="bold"
            textDecoration={task.status is TaskStatus.Done ? "lineThrough" : "none"}
          >
            {task.title}
          </Text>
          <Text fontSize={12} color="#666">{task.description}</Text>
        </Column>
      </Row>

      <Row spacing={8}>
        @when (task.status) {
          is TaskStatus.Todo -> {
            <Text fontSize={10} color="orange">TODO</Text>
          }
          is TaskStatus.Done -> {
            <Text fontSize={10} color="green">✓ DONE</Text>
          }
        }

        <Button onClick={onDelete} text="Delete" />
      </Row>
    </Column>
  </Card>
}
```

```whitehall
// src/main.wh
import $stores.TaskStore
import $components.TaskCard
import $models.Task
import $models.TaskStatus

val store = TaskStore()

var newTaskTitle = ""
var newTaskDescription = ""
var showAddDialog = false

$onMount {
  launch { store.loadTasks() }
}

<Column p={20} spacing={16}>
  <Row spacing={8}>
    <Text fontSize={32} fontWeight="bold">Tasks</Text>
    <Button onClick={() => showAddDialog = true} text="+ Add" />
  </Row>

  <Card p={12} backgroundColor="secondaryContainer">
    <Row spacing={16}>
      <Text>Total: {store.stats["total"]}</Text>
      <Text>Todo: {store.stats["todo"]}</Text>
      <Text>Done: {store.stats["done"]}</Text>
    </Row>
  </Card>

  <TextField
    bind:value={store.searchQuery}
    label="Search tasks"
  />

  <Row spacing={8}>
    <Button
      onClick={() => store.filter = "all"}
      text="All"
    />
    <Button
      onClick={() => store.filter = "todo"}
      text="Todo"
    />
    <Button
      onClick={() => store.filter = "done"}
      text="Done"
    />
  </Row>

  @if (store.isLoading) {
    <CircularProgressIndicator />
  } else if (store.filteredTasks.isEmpty()) {
    <Card p={24}>
      <Text>No tasks found</Text>
    </Card>
  } else {
    @for (task in store.filteredTasks) {
      <TaskCard
        task={task}
        onToggle={() => store.toggleStatus(task.id)}
        onDelete={() => store.deleteTask(task.id)}
      />
    }
  }

  @if (showAddDialog) {
    <AlertDialog
      onDismissRequest={() => showAddDialog = false}
      title={<Text>Add New Task</Text>}
      text={
        <Column spacing={12}>
          <TextField
            bind:value={newTaskTitle}
            label="Title"
          />
          <TextField
            bind:value={newTaskDescription}
            label="Description"
          />
        </Column>
      }
      confirmButton={
        <Button
          onClick={() => {
            store.addTask(newTaskTitle, newTaskDescription)
            newTaskTitle = ""
            newTaskDescription = ""
            showAddDialog = false
          }}
          text="Add"
        />
      }
      dismissButton={
        <Button
          onClick={() => showAddDialog = false}
          text="Cancel"
        />
      }
    />
  }
</Column>
```

**Teaches**:
- Multi-file app structure
- ViewModel class with complex logic
- Reusable components
- Sealed classes for state
- Stats/computed properties
- CRUD operations
- Search & filtering
- Dialogs for input
- Production patterns

---

## Summary

| Example | Focus | Features Combined | Complexity |
|---------|-------|-------------------|------------|
| 19 | ViewModel Classes | Class → ViewModel, StateFlow, derived props | Medium |
| 20 | Helper Functions | @Composable helpers, safe nav (?.), ternary (?:) | Medium |
| 21 | i18n | String resources, plurals, formatting | Medium |
| 22 | Collections | Arrays `[]`, ranges `..`, data ops | Medium |
| 23 | Global State | @store object, multi-line strings | Medium |
| 24 | Error Handling | Sealed classes, @when, try/catch, validation | Medium-High |
| 25 | Navigation | Routes, arguments, multi-screen | Medium-High |
| 26 | Advanced FFI | Rust + C++ mixed, complex interop | High |
| 27 | Complete App | Task manager - everything combined | High |

**Progressive climb**:
- 19-21: Core missing features (ViewModel, helpers, i18n)
- 22-23: Data patterns (collections, global state)
- 24-25: Robust patterns (errors, navigation)
- 26: Advanced FFI
- 27: Capstone - complete production app

All features from LANGUAGE-REFERENCE.md will be covered!
