# Decision 005: Data Binding

**Status:** ✅ Decided
**Date:** 2025-11-01
**Decider:** User preference

## Context

How to handle data binding between components and form inputs. Two main approaches:
1. **Two-way binding** - Automatic synchronization (Svelte's `bind:`)
2. **One-way binding** - Explicit handlers (React/Compose style)

**Compose uses one-way binding** (state hoisting):
```kotlin
var text by remember { mutableStateOf("") }
TextField(
  value = text,
  onValueChange = { text = it }
)
```

---

## Part 1: Text Input Binding

### Option A: Two-Way Binding (Svelte-style)

```whitehall
<script>
  var email = ""
  var password = ""
</script>

<Input bind:value={email} label="Email" />
<Input bind:value={password} label="Password" />

<!-- Value is automatically synced both ways -->
<Text>Email: {email}</Text>
```

**Pros:**
- Very concise (no onChange handler)
- Obvious what's happening (`bind:` makes it clear)
- Less boilerplate
- Familiar from Svelte/Vue

**Cons:**
- "Magic" - not explicit
- Different from Compose pattern
- Harder to add validation/transforms

**Transpiles to:**
```kotlin
var email by remember { mutableStateOf("") }
var password by remember { mutableStateOf("") }

OutlinedTextField(
  value = email,
  onValueChange = { email = it },
  label = { Text("Email") }
)
OutlinedTextField(
  value = password,
  onValueChange = { password = it },
  label = { Text("Password") }
)

Text("Email: $email")
```

---

### Option B: One-Way Binding (Explicit)

```whitehall
<script>
  var email = ""
  var password = ""
</script>

<Input
  value={email}
  onValueChange={(newValue) => email = newValue}
  label="Email"
/>

<Input
  value={password}
  onValueChange={(newValue) => password = newValue}
  label="Password"
/>
```

**Pros:**
- Explicit control flow
- Matches Compose exactly
- Easy to add validation
- No magic

**Cons:**
- Verbose for simple cases
- Repetitive pattern

---

### Option C: Hybrid (bind: as shorthand)

```whitehall
<script>
  var email = ""
  var password = ""
  var username = ""

  fun validateUsername(value: String) {
    username = value.lowercase().trim()
  }
</script>

<!-- Simple cases: use bind: -->
<Input bind:value={email} label="Email" />
<Input bind:value={password} label="Password" />

<!-- Complex cases: use explicit handlers -->
<Input
  value={username}
  onValueChange={validateUsername}
  label="Username"
/>
```

**Pros:**
- Best of both worlds
- Concise for simple cases
- Explicit when needed
- `bind:` is optional shorthand

**Cons:**
- Two ways to do the same thing
- When to use which?

---

## Part 2: Other Form Elements

### Checkboxes

**Option A (bind:):**
```whitehall
<script>
  var agreedToTerms = false
  var subscribeNewsletter = false
</script>

<Checkbox bind:checked={agreedToTerms} label="I agree to terms" />
<Checkbox bind:checked={subscribeNewsletter} label="Subscribe to newsletter" />
```

**Option B (explicit):**
```whitehall
<Checkbox
  checked={agreedToTerms}
  onCheckedChange={(value) => agreedToTerms = value}
  label="I agree to terms"
/>
```

---

### Radio Buttons

**Option A (bind:):**
```whitehall
<script>
  var selectedPlan = "free"
</script>

<RadioGroup bind:value={selectedPlan}>
  <Radio value="free" label="Free" />
  <Radio value="pro" label="Pro" />
  <Radio value="enterprise" label="Enterprise" />
</RadioGroup>
```

**Option B (explicit):**
```whitehall
<RadioGroup
  value={selectedPlan}
  onValueChange={(value) => selectedPlan = value}
>
  <Radio value="free" label="Free" />
  <Radio value="pro" label="Pro" />
  <Radio value="enterprise" label="Enterprise" />
</RadioGroup>
```

---

### Sliders

**Option A (bind:):**
```whitehall
<script>
  var volume = 50
</script>

<Slider bind:value={volume} min={0} max={100} />
<Text>Volume: {volume}%</Text>
```

**Option B (explicit):**
```whitehall
<Slider
  value={volume}
  onValueChange={(value) => volume = value}
  min={0}
  max={100}
/>
```

---

### Switches/Toggles

**Option A (bind:):**
```whitehall
<script>
  var darkMode = false
  var notifications = true
</script>

<Switch bind:checked={darkMode} label="Dark Mode" />
<Switch bind:checked={notifications} label="Enable Notifications" />
```

**Option B (explicit):**
```whitehall
<Switch
  checked={darkMode}
  onCheckedChange={(value) => darkMode = value}
  label="Dark Mode"
/>
```

---

## Part 3: Validation and Transforms

### With Two-Way Binding

**Problem:** How to add validation?

```whitehall
<script>
  var email = ""
  var emailError: String? = null

  // Need to intercept changes somehow
  fun onEmailChange(value: String) {
    email = value
    emailError = if (value.contains("@")) null else "Invalid email"
  }
</script>

<!-- Can't use bind: here, need explicit handler -->
<Input
  value={email}
  onValueChange={onEmailChange}
  error={emailError}
  label="Email"
/>
```

**Or custom binding?**
```whitehall
<!-- Custom directive? -->
<Input
  bind:value={email}
  bind:error={emailError}
  validate={(value) => value.contains("@") ? null : "Invalid email"}
  label="Email"
/>
```

---

### With Explicit Binding

```whitehall
<script>
  var email = ""
  var emailError: String? = null

  fun handleEmailChange(value: String) {
    email = value
    emailError = if (value.contains("@")) null else "Invalid email"
  }
</script>

<Input
  value={email}
  onValueChange={handleEmailChange}
  error={emailError}
  label="Email"
/>
```

**Pros:**
- Clear control flow
- Easy to add any logic
- Explicit

**Cons:**
- More verbose

---

## Part 4: Binding to Complex Objects

### Nested Properties

```whitehall
<script>
  var user = User(
    name = "",
    email = "",
    address = Address(
      street = "",
      city = ""
    )
  )
</script>

<!-- Option A: bind to properties -->
<Input bind:value={user.name} label="Name" />
<Input bind:value={user.email} label="Email" />
<Input bind:value={user.address.street} label="Street" />
<Input bind:value={user.address.city} label="City" />

<!-- Option B: explicit (Compose-style) -->
<Input
  value={user.name}
  onValueChange={(value) => user = user.copy(name = value)}
  label="Name"
/>
<Input
  value={user.address.street}
  onValueChange={(value) => user = user.copy(
    address = user.address.copy(street = value)
  )}
  label="Street"
/>
```

**Issue with nested binding:**
- Compose requires immutable updates (copy)
- `bind:value={user.name}` doesn't work for data classes
- Need explicit copy calls

---

## Part 5: Real-World Comparison

### Simple Form (Login)

**With bind: (Option A/C):**
```whitehall
<script>
  var email = ""
  var password = ""

  fun handleLogin() {
    AuthRepository.login(email, password)
  }
</script>

<Column spacing={16}>
  <Input bind:value={email} label="Email" keyboardType="email" />
  <Input bind:value={password} label="Password" type="password" />
  <Button text="Login" onClick={handleLogin} />
</Column>
```

**With explicit (Option B):**
```whitehall
<script>
  var email = ""
  var password = ""

  fun handleLogin() {
    AuthRepository.login(email, password)
  }
</script>

<Column spacing={16}>
  <Input
    value={email}
    onValueChange={(value) => email = value}
    label="Email"
    keyboardType="email"
  />
  <Input
    value={password}
    onValueChange={(value) => password = value}
    label="Password"
    type="password"
  />
  <Button text="Login" onClick={handleLogin} />
</Column>
```

**Difference:** 2 lines saved with `bind:`

---

### Complex Form (User Profile with Validation)

**With explicit:**
```whitehall
<script>
  var name = ""
  var email = ""
  var age = 18

  var nameError: String? = null
  var emailError: String? = null
  var ageError: String? = null

  fun validateName(value: String) {
    name = value
    nameError = when {
      value.isEmpty() -> "Name is required"
      value.length < 2 -> "Name too short"
      else -> null
    }
  }

  fun validateEmail(value: String) {
    email = value
    emailError = when {
      !value.contains("@") -> "Invalid email"
      else -> null
    }
  }

  fun validateAge(value: Int) {
    age = value
    ageError = when {
      value < 13 -> "Must be 13 or older"
      value > 120 -> "Invalid age"
      else -> null
    }
  }

  val isValid = nameError == null && emailError == null && ageError == null
</script>

<Column spacing={16}>
  <Input
    value={name}
    onValueChange={validateName}
    error={nameError}
    label="Name"
  />
  <Input
    value={email}
    onValueChange={validateEmail}
    error={emailError}
    label="Email"
    keyboardType="email"
  />
  <Slider
    value={age}
    onValueChange={validateAge}
    min={13}
    max={120}
    label="Age: {age}"
  />

  <Button text="Save" onClick={handleSave} disabled={!isValid} />
</Column>
```

**With bind: (doesn't help much here):**
```whitehall
<!-- Still need validators, so bind: provides no benefit -->
<Input
  value={name}
  onValueChange={validateName}
  error={nameError}
  label="Name"
/>
```

---

## FINAL DECISION

**Support `bind:` as optional syntactic sugar with smart transpilation.**

### Core Principles

1. **`bind:` is pure syntactic sugar** - Always transpiles to Compose's value + onChange pattern
2. **Smart transpilation** - Handles nested properties with automatic `.copy()` generation
3. **Optional** - Can always use explicit handlers instead
4. **No magic props** - No `validate` or other invented props; just transpiles to standard Compose

### Syntax

```whitehall
<!-- Simple variable binding -->
<Input bind:value={email} label="Email" />
<Checkbox bind:checked={agreed} />
<Switch bind:checked={darkMode} />
<Slider bind:value={volume} min={0} max={100} />

<!-- Nested property binding (smart transpilation) -->
<Input bind:value={user.name} label="Name" />
<Input bind:value={user.address.street} label="Street" />

<!-- Explicit handlers (always supported) -->
<Input
  value={email}
  onValueChange={(v) => email = v}
  label="Email"
/>

<!-- Validation with explicit handler -->
<Input
  value={username}
  onValueChange={(v) => {
    username = v
    usernameError = if (v.length >= 3) null else "Too short"
  }}
  error={usernameError}
  label="Username"
/>
```

### Supported Bind Directives

- `bind:value` → `value` + `onValueChange`
- `bind:checked` → `checked` + `onCheckedChange`
- Component-specific bindings follow same pattern

### Smart Transpilation Examples

#### Simple Variable

**Input:**
```whitehall
<script>
  var email = ""
</script>

<Input bind:value={email} />
```

**Output:**
```kotlin
var email by remember { mutableStateOf("") }

OutlinedTextField(
  value = email,
  onValueChange = { email = it }
)
```

#### Nested Property (Single Level)

**Input:**
```whitehall
<script>
  var user = User(name = "", email = "")
</script>

<Input bind:value={user.name} label="Name" />
<Input bind:value={user.email} label="Email" />
```

**Output:**
```kotlin
var user by remember { mutableStateOf(User(name = "", email = "")) }

OutlinedTextField(
  value = user.name,
  onValueChange = { user = user.copy(name = it) },
  label = { Text("Name") }
)
OutlinedTextField(
  value = user.email,
  onValueChange = { user = user.copy(email = it) },
  label = { Text("Email") }
)
```

#### Nested Property (Deep)

**Input:**
```whitehall
<script>
  var user = User(
    name = "",
    address = Address(
      street = "",
      city = ""
    )
  )
</script>

<Input bind:value={user.address.street} label="Street" />
<Input bind:value={user.address.city} label="City" />
```

**Output:**
```kotlin
var user by remember { mutableStateOf(
  User(
    name = "",
    address = Address(street = "", city = "")
  )
) }

OutlinedTextField(
  value = user.address.street,
  onValueChange = {
    user = user.copy(
      address = user.address.copy(street = it)
    )
  },
  label = { Text("Street") }
)
OutlinedTextField(
  value = user.address.city,
  onValueChange = {
    user = user.copy(
      address = user.address.copy(city = it)
    )
  },
  label = { Text("City") }
)
```

#### Checkbox Binding

**Input:**
```whitehall
<script>
  var agreed = false
  var subscribed = false
</script>

<Checkbox bind:checked={agreed} label="I agree" />
<Switch bind:checked={subscribed} label="Subscribe" />
```

**Output:**
```kotlin
var agreed by remember { mutableStateOf(false) }
var subscribed by remember { mutableStateOf(false) }

Checkbox(
  checked = agreed,
  onCheckedChange = { agreed = it },
  label = { Text("I agree") }
)
Switch(
  checked = subscribed,
  onCheckedChange = { subscribed = it },
  label = { Text("Subscribe") }
)
```

### Limitations

**Cannot bind to:**

1. **Expressions**
   ```whitehall
   <Input bind:value={email.lowercase()} />  // ❌ Error
   ```

2. **Array/List indices**
   ```whitehall
   <Input bind:value={items[0]} />           // ❌ Error
   ```

3. **Computed properties (get-only)**
   ```whitehall
   val fullName get() = "$firstName $lastName"
   <Input bind:value={fullName} />           // ❌ Error: no setter
   ```

**For these cases, use explicit handlers:**
```whitehall
<Input
  value={email.lowercase()}
  onValueChange={(v) => email = v.lowercase()}
/>
```

### Validation Pattern (No Special Props)

**Just use explicit handlers:**

```whitehall
<script>
  var email = ""
  var emailError: String? = null

  fun handleEmailChange(value: String) {
    email = value
    emailError = when {
      value.isEmpty() -> "Email required"
      !value.contains("@") -> "Invalid email"
      else -> null
    }
  }
</script>

<Input
  value={email}
  onValueChange={handleEmailChange}
  error={emailError}
  label="Email"
/>
```

**No `validate` prop** - Keep it simple, use explicit logic.

---

## Future Enhancements (Phase 2)

These are NOT in Phase 1 but could be added later:

### 1. Debouncing

```whitehall
<!-- Potential future syntax -->
<Input
  bind:value={searchQuery}
  debounce={300}
  placeholder="Search..."
/>
```

Transpiles to debounced state updates.

### 2. Custom Validation Helpers

```whitehall
<!-- Potential future syntax -->
<Input
  bind:value={email}
  validators={[required, email]}
  error={emailError}
/>
```

Or just stick with explicit handlers (simpler).

### 3. Transform Modifiers

```whitehall
<!-- Potential future syntax -->
<Input
  bind:value={username}
  transform={(v) => v.lowercase().trim()}
/>
```

Phase 1: Use explicit handlers. Phase 2: Could add sugar.

---

## Alternative: Kotlin Property Delegates?

**Could we support this?**
```whitehall
<script>
  var email by mutableStateOf("")
  var password by mutableStateOf("")
</script>

<!-- Auto-detect mutableStateOf and generate binding? -->
<Input value={email} label="Email" />
```

**Problem:** Still need onChange for Compose, can't infer it

---

## Edge Cases

### Multiple bindings on same element?

```whitehall
<!-- Is this valid? -->
<CustomInput
  bind:value={text}
  bind:isFocused={hasFocus}
/>
```

**Answer:** Yes, each bind: maps to different prop pairs

---

### Binding with default value?

```whitehall
<Input bind:value={email} defaultValue="user@example.com" />
```

**Issue:** `defaultValue` doesn't exist in Compose. Just use initial state:
```whitehall
<script>
  var email = "user@example.com"
</script>
<Input bind:value={email} />
```

---

### Conditional binding?

```whitehall
<Input bind:value={isEditMode ? editValue : displayValue} />
```

**Answer:** ❌ Error - can't bind to expression. Use explicit:
```whitehall
<Input
  value={isEditMode ? editValue : displayValue}
  onValueChange={(v) => if (isEditMode) editValue = v else displayValue = v}
/>
```

---

## Implementation Notes

### Compiler Requirements

The compiler must:

1. **Detect binding patterns** - Parse `bind:value={expression}`
2. **Analyze expression type:**
   - Simple variable: `email` → direct assignment
   - Property access: `user.name` → generate `.copy()`
   - Deep nesting: `user.address.street` → nested `.copy()`
3. **Generate appropriate onValueChange handler**
4. **Validate bindable expressions** - Error on expressions, indices, computed props

### Type Inference

The compiler must infer types from the bound variable to match Compose component signatures:

```whitehall
var count: Int = 0
<Slider bind:value={count} />  // Knows to use Int type
```

### Error Messages

```
error: Cannot bind to expression
  ┌─ LoginForm.wh:12:20
  │
12│   <Input bind:value={email.lowercase()} />
  │                      ^^^^^^^^^^^^^^^^^^^ expression not allowed in binding
  │
  = help: use explicit handler: onValueChange={(v) => email = v.lowercase()}
```

```
error: Cannot bind to computed property
  ┌─ ProfileForm.wh:8:20
  │
8 │   <Input bind:value={fullName} />
  │                      ^^^^^^^^^ property has no setter
  │
  = note: computed property is: val fullName get() = "$firstName $lastName"
  = help: bind to individual properties instead or use explicit handler
```

---

## Open Questions for Future

1. **Custom bind: directives for components?**
   ```whitehall
   <!-- Component author defines bindable props -->
   component CustomSlider {
     @prop var value: Float
     @prop var onValueChange: (Float) -> Unit
   }

   <!-- Auto-generates bind:value support -->
   <CustomSlider bind:value={volume} />
   ```

2. **Two-way binding with parent components?**
   ```whitehall
   <!-- Parent -->
   <ChildComponent bind:selectedItem={currentItem} />

   <!-- Child can update parent's state? -->
   ```
   Compose doesn't do this - always use callbacks. Keep it explicit.
