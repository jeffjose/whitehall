# Whitehall Example Apps

This directory contains real-world example applications to test and demonstrate Whitehall features.

## Examples

### 1. Counter (Minimal)
**Location**: `counter/`
**Features**: Basic state management, button clicks, simple UI
**Complexity**: ⭐ Beginner

### 2. Todo List
**Location**: `todo-list/`
**Features**: Lists, components, dynamic state, conditional rendering
**Complexity**: ⭐⭐ Intermediate

### 3. Weather App
**Location**: `weather-app/`
**Features**: Cards, loading states, simulated data
**Complexity**: ⭐⭐ Intermediate

### 4. Profile Card
**Location**: `profile-card/`
**Features**: Reusable components, props, conditional text
**Complexity**: ⭐⭐ Intermediate

### 5. Settings Screen
**Location**: `settings-screen/`
**Features**: Forms, switches, complex layouts
**Complexity**: ⭐⭐⭐ Advanced

## Testing Examples

Build any example:
\`\`\`bash
cd examples/counter
whitehall build
\`\`\`

Build from root with --manifest-path:
\`\`\`bash
whitehall build --manifest-path examples/counter/whitehall.toml
\`\`\`
