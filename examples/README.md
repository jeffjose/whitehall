# Whitehall Examples

This directory contains complete example applications demonstrating Whitehall syntax and patterns.

## Complete Applications

### Microblog
**Location:** `microblog/`

A Twitter/Threads-style microblogging app demonstrating:
- Authentication flow (login/signup)
- File-based routing with Navigation 2.8+
- Data fetching and state management
- Forms with validation
- Component composition and reuse
- All control flow patterns (`@if`, `@for`, `@when`)
- Data binding with `bind:value`
- Lifecycle hooks
- Event handling

**Purpose:** Validates Whitehall syntax with a realistic, complete application. Used to surface and finalize pending syntax decisions.

**Status:** 🚧 In Development

---

## Running Examples

Once the Whitehall compiler is built:

```bash
cd examples/microblog
whitehall run
```

For now, these are reference implementations showing proposed syntax.

---

## Other Examples

Smaller, focused examples are in `docs/syntax/examples/`:
- Individual components (Button, UserCard, Avatar)
- Routing examples
- Form examples (RegistrationForm)
- List examples (UserList, ShoppingCart)
