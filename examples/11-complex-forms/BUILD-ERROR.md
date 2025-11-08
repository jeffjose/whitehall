# Build Error Report: Example 11 - Complex Forms

## Build Status
✅ **Transpilation Successful**
❌ **Kotlin Compilation Failed** - Multiple errors

## Commands Tested
```bash
cargo run -- compile main.wh  # ✅ Succeeds
cargo run -- build main.wh     # ❌ Fails at Kotlin compilation
```

## Errors Found

### Error 1: Ternary operator not transformed in string templates
**Lines:** 127-131 in main.wh

**Whitehall Code:**
```whitehall
<Text fontSize={12}>Email: {email.isEmpty() ? "Not entered" : email}</Text>
```

**Generated (incorrect):**
```kotlin
Text(
    text = "Email: ${uiState.email.isEmpty() ? "Not entered" : uiState.email}",
    fontSize = 12.sp
)
```

**Expected:**
```kotlin
Text(
    text = "Email: ${if (uiState.email.isEmpty()) "Not entered" else uiState.email}",
    fontSize = 12.sp
)
```

**Issue:** Ternary operators inside string interpolations `{...}` are not being transformed to if/else
**Severity:** CRITICAL - Invalid Kotlin syntax
**Note:** Ternary operators work in normal prop values but not inside text content with interpolation

---

### Error 2: OutlinedTextField label prop expects composable lambda
**Lines:** 53-58 in main.wh

**Whitehall Code:**
```whitehall
<OutlinedTextField
  value={email}
  onValueChange={(value) => email = value}
  label="Email"
  isError={showErrors && !emailError.isEmpty()}
/>
```

**Generated (incorrect):**
```kotlin
OutlinedTextField(
    value = uiState.email,
    onValueChange = { value -> uiState.email = value },
    label = "Email",
    isError = uiState.showErrors && !uiState.emailError.isEmpty()
)
```

**Expected:**
```kotlin
OutlinedTextField(
    value = uiState.email,
    onValueChange = { value -> uiState.email = value },
    label = { Text("Email") },
    isError = uiState.showErrors && !uiState.emailError.isEmpty()
)
```

**Issue:** OutlinedTextField `label` prop expects `(() -> Unit)?` (composable lambda) but getting String
**Severity:** CRITICAL - Type mismatch
**Root Cause:** OutlinedTextField needs special handling like AlertDialog - label prop should wrap string in `{ Text(...) }`

---

### Error 3: ViewModel properties are vals - cannot be reassigned
**Lines:** 53-58 in main.wh

**Whitehall Code:**
```whitehall
onValueChange={(value) => email = value}
```

**Generated (incorrect):**
```kotlin
onValueChange = { value -> uiState.email = value }
```

**Expected:**
```kotlin
onValueChange = { value -> viewModel.updateEmail(value) }
// OR
onValueChange = { value -> email = value }  // if email is a local var
```

**Issue:** In ViewModel mode, state vars become val properties with private _mutableState backing. Direct assignment doesn't work.
**Severity:** HIGH - Breaks all form inputs
**Root Cause:** Lambda bodies with assignments need special ViewModel transformation

---

### Error 4: Checkbox onCheckedChange expects Boolean parameter
**Lines:** 103-106 in main.wh

**Whitehall Code:**
```whitehall
<Checkbox
  checked={agreeToTerms}
  onCheckedChange={(checked) => agreeToTerms = checked}
/>
```

**Generated:**
```kotlin
Checkbox(
    checked = uiState.agreeToTerms,
    onCheckedChange = { checked -> uiState.agreeToTerms = checked }
)
```

**Issue:** Same as Error 3 - val cannot be reassigned
**Severity:** HIGH

---

### Error 5: Function with complex logic and multiple returns
**Lines:** 33-43 in main.wh

**Whitehall Code:**
```whitehall
fun validateForm(): Boolean {
  showErrors = true

  emailError = if (!validateEmail(email)) "Invalid email format" else ""
  passwordError = if (!validatePassword(password)) "Password must be at least 8 characters" else ""
  confirmPasswordError = if (password != confirmPassword) "Passwords don't match" else ""
  ageError = if (!validateAge(age)) "Age must be between 18 and 100" else ""

  return emailError.isEmpty() && passwordError.isEmpty() &&
         confirmPasswordError.isEmpty() && ageError.isEmpty() && agreeToTerms
}
```

**Issue:** Complex validation logic works in non-ViewModel mode but needs update functions in ViewModel mode
**Severity:** MEDIUM - Design pattern issue

---

## Whitehall Syntax Tested

✅ OutlinedTextField component
✅ Checkbox component
✅ Complex validation functions
✅ Multiple state variables
✅ Ternary operators in props (outside string templates)
✅ Lambda with parameter transformations
❌ Ternary operators inside string templates/interpolations
❌ OutlinedTextField label prop (needs lambda wrapper)
❌ State assignment in ViewModel mode

---

## Fixes Needed

### Fix #1: Transform ternary in string interpolations
**Priority:** HIGH
**Effort:** 2-3 hours

**Problem:** Ternary operators inside `{...}` in text content are not being transformed

**Current:**
```whitehall
<Text>Email: {email.isEmpty() ? "Not entered" : email}</Text>
```

Generates:
```kotlin
Text(text = "Email: ${email.isEmpty() ? \"Not entered\" : email}")
```

**Needed:**
```kotlin
Text(text = "Email: ${if (email.isEmpty()) "Not entered" else email}")
```

**Solution:**
Detect ternary operators in prop expressions and text interpolations, transform to if/else

### Fix #2: OutlinedTextField label as composable
**Priority:** CRITICAL
**Effort:** 1 hour

**Problem:** Material3 OutlinedTextField `label` param is `(() -> Unit)?`, not String

**Solution:**
1. Add special handling for OutlinedTextField (like TextField)
2. Wrap string label in lambda: `label = { Text("Email") }`

### Fix #3: ViewModel state updates
**Priority:** CRITICAL
**Effort:** 4-6 hours

**Problem:** ViewModel vars are vals, cannot be directly assigned in lambdas

**Solution Option 1 - Generate update functions:**
```kotlin
class MainViewModel : ViewModel() {
    private val _email = MutableStateFlow("")
    val email: StateFlow<String> = _email.asStateFlow()

    fun updateEmail(value: String) {
        _email.value = value
    }
}
```

Then transform:
```whitehall
onValueChange={(value) => email = value}
```

To:
```kotlin
onValueChange = { value -> viewModel.updateEmail(value) }
```

**Solution Option 2 - Use _mutableState directly:**
```kotlin
onValueChange = { value -> viewModel._email.value = value }
```

**Solution Option 3 - Don't use ViewModel for this example:**
Keep state as local vars (simpler for forms)

### Fix #4: Checkbox component registration
**Priority:** LOW
**Effort:** 5 minutes

Add Checkbox to component imports if not already registered.

---

## Potential Workaround Code

Use if/else instead of ternary in text:
```whitehall
<Text fontSize={12}>
  Email: {if (email.isEmpty()) "Not entered" else email}
</Text>
```

Use TextField instead of OutlinedTextField (may accept String label):
```whitehall
<TextField
  value={email}
  onValueChange={(value) => email = value}
  label="Email"
/>
```

Don't use ViewModel mode - keep as simple component:
```kotlin
@Composable
fun App() {
    var email by remember { mutableStateOf("") }
    // ... rest of code
}
```

---

## Key Insights

1. **String interpolation needs ternary transformation**: Text content with `{expr}` needs same ternary → if/else transformation as props

2. **Material3 TextField variants**: Both TextField and OutlinedTextField have `label: (() -> Unit)?` - need special handling

3. **ViewModel vs local state**: Forms with lots of inputs may be simpler with local state rather than ViewModel pattern

4. **Validation patterns**: Complex multi-field validation is easier with helper functions but needs careful state management
