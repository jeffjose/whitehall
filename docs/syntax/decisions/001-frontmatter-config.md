# Decision 001: Frontmatter Configuration for Single-File Apps

**Status:** Decided
**Date:** 2025-11-01
**Decider:** User preference

## Context

Single-file apps need a way to specify configuration (app name, package, dependencies) without a separate `Whitehall.toml` file.

## Options Considered

1. **TOML frontmatter** (like Python's uv)
   ```whitehall
   /// [app]
   /// name = "My App"
   /// package = "com.example.myapp"
   ///
   /// [dependencies]
   /// coil = "2.5.0"
   ```

2. **JavaDoc-style annotations**
   ```whitehall
   /**
    * @app MyApp
    * @package com.example.myapp
    * @dependency coil:2.5.0
    */
   ```

3. **Structured config section**
   ```whitehall
   <config>
     app {
       name = "My App"
       package = "com.example.myapp"
     }
   </config>
   ```

## Decision

**Use TOML frontmatter (Option 1)** - Lines starting with `///` at the top of the file.

## Rationale

- Familiar to Python developers using uv
- Standard TOML syntax (no new format to learn)
- Easy to parse (just strip `///` prefix and parse as TOML)
- Visually distinct from code
- Can be copy-pasted to/from `Whitehall.toml`
- Triple-slash comments are not commonly used in other contexts

## Implementation Notes

- Frontmatter must be at the very top of the file
- Only lines starting with `///` (exactly three slashes) are frontmatter
- Empty frontmatter lines: `///` (no space needed)
- Parsing stops at first non-frontmatter line
- Use `toml` crate for parsing (same as `Whitehall.toml`)

## Examples

### Minimal single-file app:
```whitehall
/// [app]
/// name = "Hello"
/// package = "hello"

component App()

<Text>Hello, World!</Text>
```

### With dependencies:
```whitehall
/// [app]
/// name = "Image Viewer"
/// package = "com.example.imageviewer"
/// minSdk = 24
/// targetSdk = 34
///
/// [dependencies]
/// coil = "2.5.0"
/// androidx.compose.material3 = "1.2.0"

component App()

// ... app code
```

### Full configuration:
```whitehall
/// [app]
/// name = "Production App"
/// package = "com.company.product"
/// version = "1.0.0"
/// minSdk = 26
/// targetSdk = 34
///
/// [dependencies]
/// retrofit = "2.9.0"
/// room = "2.6.0"
///
/// [build]
/// optimize_level = "aggressive"

component App()

// ... app code
```

## Future Considerations

- May want to support `///!` for special compiler directives
- Could add validation for required frontmatter fields
- Might add shortcuts like `/// @dependency coil` as alternative to TOML
