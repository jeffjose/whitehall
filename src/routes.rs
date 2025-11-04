use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
}

#[derive(Debug, Clone)]
pub struct RouteParam {
    pub name: String,
    pub param_type: String,  // "String", "Int", etc. (default: String)
}

/// Scan src/routes/ directory and discover all routes
pub fn discover_routes() -> Result<Vec<Route>> {
    let routes_dir = Path::new("src/routes");

    if !routes_dir.exists() {
        // No routes directory - this is fine, just return empty
        return Ok(Vec::new());
    }

    let mut routes = Vec::new();

    // Find all +screen.wh files
    for entry in WalkDir::new(routes_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only process +screen.wh files
        if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some("+screen.wh") {
            let route = parse_route_from_path(path)?;
            routes.push(route);
        }
    }

    // Sort routes by path for consistent output
    routes.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(routes)
}

/// Parse route information from file path
/// Examples:
/// - src/routes/+screen.wh → Route { path: "/", name: "Home", params: [] }
/// - src/routes/login/+screen.wh → Route { path: "/login", name: "Login", params: [] }
/// - src/routes/profile/[id]/+screen.wh → Route { path: "/profile/:id", name: "Profile", params: [id] }
fn parse_route_from_path(path: &Path) -> Result<Route> {
    // Strip src/routes/ prefix
    let route_path = path
        .strip_prefix("src/routes")
        .or_else(|_| path.strip_prefix("src/routes/"))
        .map_err(|_| anyhow::anyhow!("Invalid route path: {}", path.display()))?;

    // Get path components (directories before +screen.wh)
    let components: Vec<&str> = route_path
        .components()
        .filter_map(|c| {
            if let std::path::Component::Normal(os_str) = c {
                os_str.to_str()
            } else {
                None
            }
        })
        .filter(|s| *s != "+screen.wh")
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
    let route_path = if path_segments.is_empty() {
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

    Ok(Route {
        path: route_path,
        name: route_name,
        screen_name,
        params,
        source_path: path.to_path_buf(),
    })
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
            },
            Route {
                path: "/login".to_string(),
                name: "Login".to_string(),
                screen_name: "LoginScreen".to_string(),
                params: vec![],
                source_path: PathBuf::from("src/routes/login/+screen.wh"),
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
            },
        ];

        let output = generate_routes_kt(&routes, "com.example.app");

        assert!(output.contains("package com.example.app.routes"));
        assert!(output.contains("sealed interface Routes"));
        assert!(output.contains("data object Home : Routes"));
        assert!(output.contains("data object Login : Routes"));
        assert!(output.contains("data class Profile(val id: String) : Routes"));
    }
}
