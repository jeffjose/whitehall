use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents a layout in the application (SvelteKit-style +layout.wh)
#[derive(Debug, Clone)]
pub struct Layout {
    /// Layout name derived from path: "Root" (for routes/), "Admin" (for routes/admin/)
    pub name: String,
    /// Composable function name: "RootLayout", "AdminLayout"
    pub composable_name: String,
    /// Directory path relative to src/routes (empty string for root)
    pub dir_path: String,
    /// Source file path
    pub source_path: PathBuf,
    /// Parent layout name (None for root layout)
    pub parent: Option<String>,
}

/// Represents a route in the application
#[derive(Debug, Clone)]
pub struct Route {
    /// Path pattern: "/", "/login", "/profile/:id"
    pub path: String,
    /// Route name for the sealed class: "Home", "Login", "Profile"
    pub name: String,
    /// Screen component name: "HomeScreen", "LoginScreen", "ProfileScreen"
    pub screen_name: String,
    /// Route parameters extracted from [param] folders
    pub params: Vec<RouteParam>,
    /// Source file path
    pub source_path: PathBuf,
    /// Layout chain for this route (outermost first): ["RootLayout", "AdminLayout"]
    pub layouts: Vec<String>,
    /// Layout override from @ syntax (None = inherit all, Some("") = no layout, Some("root") = only root)
    pub layout_override: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RouteParam {
    pub name: String,
    pub param_type: String,  // "String", "Int", etc. (default: String)
}

/// Scan src/routes/ directory and discover all layouts
pub fn discover_layouts() -> Result<Vec<Layout>> {
    let routes_dir = Path::new("src/routes");

    if !routes_dir.exists() {
        return Ok(Vec::new());
    }

    let mut layouts = Vec::new();

    // Find all +layout.wh files
    for entry in WalkDir::new(routes_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only process +layout.wh files (not @-prefixed ones for now)
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename == "+layout.wh" {
                    let layout = parse_layout_from_path(path)?;
                    layouts.push(layout);
                }
            }
        }
    }

    // Sort by dir_path length (root first, then nested)
    layouts.sort_by(|a, b| a.dir_path.len().cmp(&b.dir_path.len()));

    // Compute parent relationships
    let layout_names: Vec<(String, String)> = layouts.iter()
        .map(|l| (l.dir_path.clone(), l.name.clone()))
        .collect();

    for layout in &mut layouts {
        layout.parent = find_parent_layout(&layout.dir_path, &layout_names);
    }

    Ok(layouts)
}

/// Find the closest parent layout for a given directory path
fn find_parent_layout(dir_path: &str, layouts: &[(String, String)]) -> Option<String> {
    if dir_path.is_empty() {
        return None; // Root layout has no parent
    }

    // Walk up the directory tree to find nearest layout
    let mut current = dir_path.to_string();
    while let Some(pos) = current.rfind('/') {
        current = current[..pos].to_string();
        for (layout_dir, layout_name) in layouts {
            if layout_dir == &current {
                return Some(layout_name.clone());
            }
        }
    }

    // Check for root layout
    for (layout_dir, layout_name) in layouts {
        if layout_dir.is_empty() {
            return Some(layout_name.clone());
        }
    }

    None
}

/// Parse layout information from file path
fn parse_layout_from_path(path: &Path) -> Result<Layout> {
    // Strip src/routes/ prefix and +layout.wh suffix
    let relative = path
        .strip_prefix("src/routes")
        .or_else(|_| path.strip_prefix("src/routes/"))
        .map_err(|_| anyhow::anyhow!("Invalid layout path: {}", path.display()))?;

    // Get directory path (everything except +layout.wh)
    let dir_path = relative
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    // Build layout name from directory path
    let name = if dir_path.is_empty() {
        "Root".to_string()
    } else {
        // Convert "admin/settings" to "AdminSettings"
        dir_path
            .split('/')
            .filter(|s| !s.is_empty() && !s.starts_with('['))
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    Some(c) => c.to_uppercase().chain(chars).collect::<String>(),
                    None => String::new(),
                }
            })
            .collect::<String>()
    };

    let composable_name = format!("{}Layout", name);

    Ok(Layout {
        name,
        composable_name,
        dir_path,
        source_path: path.to_path_buf(),
        parent: None, // Set later after all layouts discovered
    })
}

/// Scan src/routes/ directory and discover all routes
pub fn discover_routes() -> Result<Vec<Route>> {
    discover_routes_with_layouts(&discover_layouts()?)
}

/// Discover routes with layout information
pub fn discover_routes_with_layouts(layouts: &[Layout]) -> Result<Vec<Route>> {
    let routes_dir = Path::new("src/routes");

    if !routes_dir.exists() {
        return Ok(Vec::new());
    }

    let mut routes = Vec::new();

    // Find all +screen.wh files (including @-prefixed variants)
    for entry in WalkDir::new(routes_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // Match +screen.wh or +screen@xxx.wh
                if filename == "+screen.wh" || (filename.starts_with("+screen@") && filename.ends_with(".wh")) {
                    let route = parse_route_from_path_with_layouts(path, layouts)?;
                    routes.push(route);
                }
            }
        }
    }

    // Sort routes by path for consistent output
    routes.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(routes)
}

/// Parse route information from file path (legacy, without layouts)
fn parse_route_from_path(path: &Path) -> Result<Route> {
    parse_route_from_path_with_layouts(path, &[])
}

/// Parse route information from file path with layout chain computation
/// Examples:
/// - src/routes/+screen.wh → Route { path: "/", name: "Home", params: [] }
/// - src/routes/login/+screen.wh → Route { path: "/login", name: "Login", params: [] }
/// - src/routes/profile/[id]/+screen.wh → Route { path: "/profile/:id", name: "Profile", params: [id] }
/// - src/routes/auth/login/+screen@.wh → No layout (@ with empty = no layouts)
/// - src/routes/admin/+screen@root.wh → Only root layout (skip intermediate)
fn parse_route_from_path_with_layouts(path: &Path, layouts: &[Layout]) -> Result<Route> {
    // Extract filename to check for @ syntax
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("+screen.wh");

    // Parse @ syntax: +screen@.wh or +screen@root.wh
    let layout_override = if filename.starts_with("+screen@") && filename.ends_with(".wh") {
        let middle = &filename[8..filename.len()-3]; // Extract between @ and .wh
        Some(middle.to_string())
    } else {
        None
    };

    // Strip src/routes/ prefix
    let route_path = path
        .strip_prefix("src/routes")
        .or_else(|_| path.strip_prefix("src/routes/"))
        .map_err(|_| anyhow::anyhow!("Invalid route path: {}", path.display()))?;

    // Get path components (directories before +screen*.wh)
    let components: Vec<&str> = route_path
        .components()
        .filter_map(|c| {
            if let std::path::Component::Normal(os_str) = c {
                os_str.to_str()
            } else {
                None
            }
        })
        .filter(|s| !s.starts_with("+screen"))
        .collect();

    // Build route path and extract parameters
    let mut path_segments = Vec::new();
    let mut params = Vec::new();
    let mut name_parts = Vec::new();

    for component in &components {
        if component.starts_with('[') && component.ends_with(']') {
            // Parameter: [id] → :id
            let param_name = component.trim_start_matches('[').trim_end_matches(']');
            path_segments.push(format!(":{}", param_name));
            params.push(RouteParam {
                name: param_name.to_string(),
                param_type: "String".to_string(),  // Default to String
            });
        } else {
            // Regular path segment
            path_segments.push(component.to_string());
            // Use for route name
            let mut chars = component.chars();
            if let Some(first) = chars.next() {
                let capitalized = first.to_uppercase().chain(chars).collect::<String>();
                name_parts.push(capitalized);
            }
        }
    }

    // Build route path
    let route_path_str = if path_segments.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", path_segments.join("/"))
    };

    // Build route name
    let route_name = if name_parts.is_empty() {
        "Home".to_string()
    } else {
        name_parts.join("")
    };

    // Screen name = route name + "Screen"
    let screen_name = format!("{}Screen", route_name);

    // Compute layout chain for this route
    let dir_path = components.join("/");
    let layout_chain = compute_layout_chain(&dir_path, layouts, &layout_override);

    Ok(Route {
        path: route_path_str,
        name: route_name,
        screen_name,
        params,
        source_path: path.to_path_buf(),
        layouts: layout_chain,
        layout_override,
    })
}

/// Compute the layout chain for a route based on its directory path
/// Returns composable names in order from outermost to innermost
fn compute_layout_chain(dir_path: &str, layouts: &[Layout], layout_override: &Option<String>) -> Vec<String> {
    // Handle @ override syntax
    if let Some(override_val) = layout_override {
        if override_val.is_empty() {
            // +screen@.wh = no layouts at all
            return Vec::new();
        }
        // +screen@root.wh = only layouts up to and including "root"
        // Find the specified layout and return only its chain
        let target = override_val.to_lowercase();
        if let Some(layout) = layouts.iter().find(|l| l.name.to_lowercase() == target) {
            let mut chain = Vec::new();
            collect_layout_chain_up(layout, layouts, &mut chain);
            return chain;
        }
        // If target not found, return empty
        return Vec::new();
    }

    // Normal case: collect all layouts from root to this directory
    let mut chain = Vec::new();
    let mut current_path = String::new();

    // Always check for root layout first
    if let Some(root) = layouts.iter().find(|l| l.dir_path.is_empty()) {
        chain.push(root.composable_name.clone());
    }

    // Then check each path segment
    if !dir_path.is_empty() {
        for segment in dir_path.split('/') {
            if current_path.is_empty() {
                current_path = segment.to_string();
            } else {
                current_path = format!("{}/{}", current_path, segment);
            }

            if let Some(layout) = layouts.iter().find(|l| l.dir_path == current_path) {
                chain.push(layout.composable_name.clone());
            }
        }
    }

    chain
}

/// Collect layout chain upward from a layout to root
fn collect_layout_chain_up(layout: &Layout, layouts: &[Layout], chain: &mut Vec<String>) {
    // First collect parent chain
    if let Some(parent_name) = &layout.parent {
        if let Some(parent) = layouts.iter().find(|l| &l.name == parent_name) {
            collect_layout_chain_up(parent, layouts, chain);
        }
    }
    // Then add this layout
    chain.push(layout.composable_name.clone());
}

/// Generate Routes.kt sealed interface
pub fn generate_routes_kt(routes: &[Route], package: &str) -> String {
    let mut output = String::new();

    // Package declaration
    output.push_str(&format!("package {}.routes\n\n", package));

    // Imports
    output.push_str("import kotlinx.serialization.Serializable\n\n");

    // Sealed interface
    output.push_str("sealed interface Routes {\n");

    for route in routes {
        output.push_str("    @Serializable\n");

        if route.params.is_empty() {
            // Object route (no parameters)
            output.push_str(&format!("    data object {} : Routes\n\n", route.name));
        } else {
            // Data class route (with parameters)
            let params_str = route
                .params
                .iter()
                .map(|p| format!("val {}: {}", p.name, p.param_type))
                .collect::<Vec<_>>()
                .join(", ");

            output.push_str(&format!(
                "    data class {}({}) : Routes\n\n",
                route.name, params_str
            ));
        }
    }

    output.push_str("}\n");

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_root_route() {
        let path = Path::new("src/routes/+screen.wh");
        let route = parse_route_from_path(path).unwrap();

        assert_eq!(route.path, "/");
        assert_eq!(route.name, "Home");
        assert_eq!(route.screen_name, "HomeScreen");
        assert_eq!(route.params.len(), 0);
    }

    #[test]
    fn test_parse_simple_route() {
        let path = Path::new("src/routes/login/+screen.wh");
        let route = parse_route_from_path(path).unwrap();

        assert_eq!(route.path, "/login");
        assert_eq!(route.name, "Login");
        assert_eq!(route.screen_name, "LoginScreen");
        assert_eq!(route.params.len(), 0);
    }

    #[test]
    fn test_parse_route_with_param() {
        let path = Path::new("src/routes/profile/[id]/+screen.wh");
        let route = parse_route_from_path(path).unwrap();

        assert_eq!(route.path, "/profile/:id");
        assert_eq!(route.name, "Profile");
        assert_eq!(route.screen_name, "ProfileScreen");
        assert_eq!(route.params.len(), 1);
        assert_eq!(route.params[0].name, "id");
        assert_eq!(route.params[0].param_type, "String");
    }

    #[test]
    fn test_parse_nested_route() {
        let path = Path::new("src/routes/post/create/+screen.wh");
        let route = parse_route_from_path(path).unwrap();

        assert_eq!(route.path, "/post/create");
        assert_eq!(route.name, "PostCreate");
        assert_eq!(route.screen_name, "PostCreateScreen");
        assert_eq!(route.params.len(), 0);
    }

    #[test]
    fn test_generate_routes_kt() {
        let routes = vec![
            Route {
                path: "/".to_string(),
                name: "Home".to_string(),
                screen_name: "HomeScreen".to_string(),
                params: vec![],
                source_path: PathBuf::from("src/routes/+screen.wh"),
                layouts: vec![],
                layout_override: None,
            },
            Route {
                path: "/login".to_string(),
                name: "Login".to_string(),
                screen_name: "LoginScreen".to_string(),
                params: vec![],
                source_path: PathBuf::from("src/routes/login/+screen.wh"),
                layouts: vec![],
                layout_override: None,
            },
            Route {
                path: "/profile/:id".to_string(),
                name: "Profile".to_string(),
                screen_name: "ProfileScreen".to_string(),
                params: vec![RouteParam {
                    name: "id".to_string(),
                    param_type: "String".to_string(),
                }],
                source_path: PathBuf::from("src/routes/profile/[id]/+screen.wh"),
                layouts: vec![],
                layout_override: None,
            },
        ];

        let output = generate_routes_kt(&routes, "com.example.app");

        assert!(output.contains("package com.example.app.routes"));
        assert!(output.contains("sealed interface Routes"));
        assert!(output.contains("data object Home : Routes"));
        assert!(output.contains("data object Login : Routes"));
        assert!(output.contains("data class Profile(val id: String) : Routes"));
    }

    #[test]
    fn test_parse_layout_from_path() {
        // Root layout
        let path = Path::new("src/routes/+layout.wh");
        let layout = parse_layout_from_path(path).unwrap();
        assert_eq!(layout.name, "Root");
        assert_eq!(layout.composable_name, "RootLayout");
        assert_eq!(layout.dir_path, "");

        // Nested layout
        let path = Path::new("src/routes/admin/+layout.wh");
        let layout = parse_layout_from_path(path).unwrap();
        assert_eq!(layout.name, "Admin");
        assert_eq!(layout.composable_name, "AdminLayout");
        assert_eq!(layout.dir_path, "admin");

        // Deeply nested layout
        let path = Path::new("src/routes/admin/settings/+layout.wh");
        let layout = parse_layout_from_path(path).unwrap();
        assert_eq!(layout.name, "AdminSettings");
        assert_eq!(layout.composable_name, "AdminSettingsLayout");
        assert_eq!(layout.dir_path, "admin/settings");
    }

    #[test]
    fn test_layout_override_parsing() {
        // No override
        let path = Path::new("src/routes/login/+screen.wh");
        let route = parse_route_from_path(path).unwrap();
        assert_eq!(route.layout_override, None);

        // Empty override (no layouts)
        let path = Path::new("src/routes/auth/+screen@.wh");
        let route = parse_route_from_path(path).unwrap();
        assert_eq!(route.layout_override, Some("".to_string()));

        // Named override
        let path = Path::new("src/routes/admin/users/+screen@root.wh");
        let route = parse_route_from_path(path).unwrap();
        assert_eq!(route.layout_override, Some("root".to_string()));
    }

    #[test]
    fn test_compute_layout_chain() {
        let layouts = vec![
            Layout {
                name: "Root".to_string(),
                composable_name: "RootLayout".to_string(),
                dir_path: "".to_string(),
                source_path: PathBuf::from("src/routes/+layout.wh"),
                parent: None,
            },
            Layout {
                name: "Admin".to_string(),
                composable_name: "AdminLayout".to_string(),
                dir_path: "admin".to_string(),
                source_path: PathBuf::from("src/routes/admin/+layout.wh"),
                parent: Some("Root".to_string()),
            },
        ];

        // Root route gets root layout
        let chain = compute_layout_chain("", &layouts, &None);
        assert_eq!(chain, vec!["RootLayout"]);

        // Admin route gets both layouts
        let chain = compute_layout_chain("admin", &layouts, &None);
        assert_eq!(chain, vec!["RootLayout", "AdminLayout"]);

        // Admin route with @ (no layouts)
        let chain = compute_layout_chain("admin", &layouts, &Some("".to_string()));
        assert_eq!(chain, Vec::<String>::new());

        // Admin route with @root (only root)
        let chain = compute_layout_chain("admin", &layouts, &Some("root".to_string()));
        assert_eq!(chain, vec!["RootLayout"]);
    }
}
