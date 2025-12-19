use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::android_scaffold;
use crate::config::Config;
use crate::project::{discover_files, FileType, WhitehallFile};
use crate::routes;
use crate::transpiler;

/// App-level configuration parsed from main.wh's <App> component
#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    pub color_scheme: ColorScheme,
    pub dark_mode: DarkMode,
    pub imports: Vec<String>,  // Imports from main.wh (for store bindings)
}

#[derive(Debug, Clone, Default)]
pub enum ColorScheme {
    #[default]
    Default,           // Use Material3 default colors
    Dynamic,           // Use Android 12+ dynamic colors (wallpaper-based)
    Custom {           // Custom color palette
        primary: Option<String>,
        secondary: Option<String>,
        tertiary: Option<String>,
        background: Option<String>,
        surface: Option<String>,
        error: Option<String>,
    },
}

#[derive(Debug, Clone, Default)]
pub enum DarkMode {
    #[default]
    System,  // Follow system setting
    Light,   // Always light
    Dark,    // Always dark
    Binding {  // Dynamic binding to a store property
        store: String,     // e.g., "SettingsStore"
        property: String,  // e.g., "theme"
    },
}

/// Represents the result of a build operation
#[derive(Debug)]
pub struct BuildResult {
    pub files_transpiled: usize,
    pub output_dir: PathBuf,
    pub errors: Vec<BuildError>,
}

#[derive(Debug)]
pub struct BuildError {
    pub file: PathBuf,
    pub message: String,
}

/// Core build pipeline - used by build, watch, and run commands
///
/// # Arguments
/// * `config` - Parsed whitehall.toml configuration
/// * `clean` - If true, remove and recreate output directory
///
/// # Returns
/// BuildResult with success count and any errors
pub fn execute_build(config: &Config, clean: bool) -> Result<BuildResult> {
    let output_dir = Path::new(&config.build.output_dir);

    // 1. Clean output directory if requested
    if clean && output_dir.exists() {
        fs::remove_dir_all(output_dir)
            .context("Failed to clean output directory")?;
    }
    fs::create_dir_all(output_dir)
        .context("Failed to create output directory")?;

    // 2. Discover .wh files
    let files = discover_files(config)
        .context("Failed to discover source files")?;

    // 3. Build project-wide store registry for cross-file store detection
    let global_store_registry = build_store_registry(&files)?;

    // 4. Generate Android scaffold (only if clean or missing)
    let scaffold_exists = output_dir.join("app/build.gradle.kts").exists();
    if clean || !scaffold_exists {
        android_scaffold::generate(config, output_dir)
            .context("Failed to generate Android project scaffold")?;
    }

    // 4.5. Build FFI components if enabled
    crate::ffi_build::build_ffi(config, Path::new("."))
        .context("Failed to build FFI components")?;

    // 5. Transpile each file
    let mut errors = Vec::new();
    let mut success_count = 0;

    for file in &files {
        match transpile_file(file, config, output_dir, &global_store_registry) {
            Ok(_) => success_count += 1,
            Err(e) => errors.push(BuildError {
                file: file.path.clone(),
                message: e.to_string(),
            }),
        }
    }

    // 6. Generate Routes.kt from route structure
    if errors.is_empty() {
        generate_routes_file(config, output_dir)?;
    }

    // 7. Generate MainActivity if all files transpiled successfully
    if errors.is_empty() {
        generate_main_activity(config, output_dir, &files, &global_store_registry)?;
    }

    Ok(BuildResult {
        files_transpiled: success_count,
        output_dir: output_dir.to_path_buf(),
        errors,
    })
}

/// Build project-wide store registry by scanning all files for:
/// - Classes with var properties → ViewModel
/// - @store object → Singleton
/// - Components with inline var → ViewModel
fn build_store_registry(files: &[WhitehallFile]) -> Result<transpiler::StoreRegistry> {
    let mut registry = transpiler::StoreRegistry::new();

    for file in files {
        let source = fs::read_to_string(&file.path)
            .context(format!("Failed to read {} for store registry", file.path.display()))?;

        // Parse to extract reactive classes and component vars
        if let Ok(ast) = transpiler::parse_for_stores(&source) {
            // 1. Scan classes (for Class and Singleton sources)
            for class in &ast.classes {
                let has_store_annotation = class.annotations.contains(&"store".to_string());
                let is_object = class.is_object;
                let has_vars = class.properties.iter().any(|prop| prop.mutable);

                // Determine source type
                let source_type = if has_store_annotation && is_object {
                    transpiler::StoreSource::Singleton
                } else if has_vars {
                    transpiler::StoreSource::Class
                } else {
                    continue;  // Skip classes with no vars and not @store object
                };

                let has_hilt_annotation = class.annotations.iter().any(|a| a == "HiltViewModel" || a.eq_ignore_ascii_case("hilt"));
                let has_inject = class.constructor.as_ref()
                    .map(|c| c.annotations.iter().any(|a| a.eq_ignore_ascii_case("inject")))
                    .unwrap_or(false);
                let has_hilt = has_hilt_annotation || has_inject;

                let store_info = transpiler::StoreInfo {
                    class_name: class.name.clone(),
                    source: source_type,
                    has_vars,
                    has_hilt,
                    has_inject,
                    package: file.package_path.clone(),
                    route_params: vec![],  // Only components have route params
                };
                registry.insert(class.name.clone(), store_info);
            }

            // 2. Scan component inline vars (for ComponentInline source)
            // Check if this is a component/screen file and has var in state
            if file.file_type == FileType::Component || file.file_type == FileType::Screen {
                let has_component_vars = ast.state.iter().any(|state| state.mutable);

                if has_component_vars {
                    // Use component name as class_name (e.g., "CounterScreen" from CounterScreen.wh)
                    let component_name = file.component_name.clone();

                    let store_info = transpiler::StoreInfo {
                        class_name: component_name.clone(),
                        source: transpiler::StoreSource::ComponentInline,
                        has_vars: true,
                        has_hilt: false,  // Component inline vars don't support Hilt (yet)
                        has_inject: false,
                        package: file.package_path.clone(),
                        route_params: vec![],  // Will be filled during semantic analysis
                    };
                    registry.insert(component_name, store_info);
                }
            }
        }
    }

    Ok(registry)
}

/// Transpile a single .wh file to Kotlin
fn transpile_file(
    file: &WhitehallFile,
    _config: &Config,
    output_dir: &Path,
    global_store_registry: &transpiler::StoreRegistry,
) -> Result<()> {
    // Skip main.wh - it's handled separately in MainActivity generation
    if file.file_type == FileType::Main {
        return Ok(());
    }

    // Read source file
    let source = fs::read_to_string(&file.path)
        .context(format!("Failed to read {}", file.path.display()))?;

    // Determine component type for transpiler
    let component_type = match file.file_type {
        FileType::Screen => Some("screen"),
        FileType::Layout => Some("layout"),
        _ => None,
    };

    // Transpile to Kotlin with global store registry
    let result = transpiler::transpile_with_registry(
        &source,
        &file.package_path,
        &file.component_name,
        component_type,
        Some(global_store_registry),
    )
    .map_err(|e| anyhow::anyhow!("Compilation error: {}", e))?;

    // Handle single or multiple output files
    match result {
        transpiler::TranspileResult::Single(kotlin_code) => {
            // Single file output (standard case)
            let output_path = get_kotlin_output_path(output_dir, file);
            fs::create_dir_all(output_path.parent().unwrap())
                .context("Failed to create output directories")?;
            fs::write(&output_path, kotlin_code)
                .context(format!("Failed to write {}", output_path.display()))?;
        }
        transpiler::TranspileResult::Multiple(files) => {
            // Multiple file output (e.g., ComponentInline generates ViewModel + Component)
            for (suffix, content) in files {
                let output_path = get_kotlin_output_path_with_suffix(output_dir, file, &suffix);
                fs::create_dir_all(output_path.parent().unwrap())
                    .context("Failed to create output directories")?;
                fs::write(&output_path, content)
                    .context(format!("Failed to write {}", output_path.display()))?;
            }
        }
    }

    Ok(())
}

/// Get the output path for a transpiled Kotlin file
fn get_kotlin_output_path(output_dir: &Path, file: &WhitehallFile) -> PathBuf {
    let package_path = file.package_path.replace('.', "/");
    output_dir
        .join("app/src/main/kotlin")
        .join(package_path)
        .join(format!("{}.kt", file.component_name))
}

/// Get the output path for a transpiled Kotlin file with a suffix
/// Empty suffix returns the standard path (e.g., "Counter.kt")
/// Non-empty suffix creates a variant (e.g., "CounterViewModel.kt" for suffix="ViewModel")
fn get_kotlin_output_path_with_suffix(output_dir: &Path, file: &WhitehallFile, suffix: &str) -> PathBuf {
    let package_path = file.package_path.replace('.', "/");
    let filename = if suffix.is_empty() {
        format!("{}.kt", file.component_name)
    } else {
        format!("{}{}.kt", file.component_name, suffix)
    };
    output_dir
        .join("app/src/main/kotlin")
        .join(package_path)
        .join(filename)
}

/// Generate MainActivity.kt
fn generate_main_activity(
    config: &Config,
    output_dir: &Path,
    files: &[WhitehallFile],
    global_store_registry: &transpiler::StoreRegistry,
) -> Result<()> {
    // Discover routes to determine if we need NavHost setup
    let discovered_routes = routes::discover_routes()?;

    // Check if there's a main.wh file
    let main_file = files.iter().find(|f| f.file_type == FileType::Main);

    // Priority: Routes > main.wh > default
    // When routes exist, always use NavHost (main.wh becomes app config + layout)
    let main_content = if !discovered_routes.is_empty() {
        // Parse main.wh for <App> configuration (theme, dark mode, etc.)
        let app_config = parse_app_config(main_file);
        // Generate MainActivity with NavHost for routing + theme from main.wh
        generate_navhost_main_activity(config, &discovered_routes, &app_config)
    } else if let Some(main_file) = main_file {
        // No routes - use transpiled main.wh content as the App composable
        let source = fs::read_to_string(&main_file.path)?;
        let result = transpiler::transpile_with_registry(&source, &config.android.package, "App", None, Some(global_store_registry))
            .map_err(|e| anyhow::anyhow!(e))?;

        // Handle Multiple results (e.g., when main.wh has inline vars → generates ViewModel)
        match &result {
            transpiler::TranspileResult::Multiple(files) => {
                // Write secondary files (e.g., AppViewModel.kt)
                // Note: main.wh is transpiled with component_name="App", so we use "App" for filenames
                let package_path = config.android.package.replace('.', "/");
                for (suffix, content) in files {
                    if !suffix.is_empty() {
                        let filename = format!("App{}.kt", suffix);
                        let output_path = output_dir
                            .join("app/src/main/kotlin")
                            .join(&package_path)
                            .join(filename);
                        fs::create_dir_all(output_path.parent().unwrap())
                            .context("Failed to create output directories")?;
                        fs::write(&output_path, content)
                            .context(format!("Failed to write {}", output_path.display()))?;
                    }
                }
            }
            transpiler::TranspileResult::Single(_) => {
                // Single file, no extra files to write
            }
        }

        // Get primary content (main.wh should generate single file or wrapper component)
        result.primary_content().to_string()
    } else {
        // Generate default MainActivity with basic content
        generate_default_main_activity(config)
    };

    // If we transpiled main.wh (and no routes), we need to wrap it in MainActivity
    // When routes exist, main_content is already a complete MainActivity
    let activity_content = if main_file.is_some() && discovered_routes.is_empty() {
        // Extract imports and code from transpiled main content
        let lines: Vec<&str> = main_content.lines().collect();
        let mut app_imports = Vec::new();
        let mut app_code_lines = Vec::new();
        let mut in_header = true;

        for line in lines {
            let trimmed = line.trim();
            if in_header && (trimmed.is_empty() || trimmed.starts_with("package ") || trimmed.starts_with("import ")) {
                if trimmed.starts_with("import ") {
                    app_imports.push(line.to_string());
                }
                // Skip package and empty lines in header
            } else {
                in_header = false;
                app_code_lines.push(line);
            }
        }

        let app_code = app_code_lines.join("\n");

        // Deduplicate app imports and remove MaterialTheme since it's in the template
        let mut unique_imports: Vec<String> = app_imports.into_iter()
            .filter(|imp| !imp.contains("androidx.compose.material3.MaterialTheme"))
            .collect();
        unique_imports.sort();
        unique_imports.dedup();

        let app_imports_str = if unique_imports.is_empty() {
            String::new()
        } else {
            format!("\n{}", unique_imports.join("\n"))
        };

        format!(
            r#"package {}

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme{}

class MainActivity : ComponentActivity() {{
    override fun onCreate(savedInstanceState: Bundle?) {{
        super.onCreate(savedInstanceState)
        setContent {{
            MaterialTheme {{
                App()
            }}
        }}
    }}
}}

{}"#,
            config.android.package, app_imports_str, app_code
        )
    } else {
        main_content
    };

    // Write MainActivity.kt
    let package_path = config.android.package.replace('.', "/");
    let output_path = output_dir
        .join("app/src/main/kotlin")
        .join(package_path)
        .join("MainActivity.kt");

    fs::create_dir_all(output_path.parent().unwrap())?;
    fs::write(output_path, activity_content)?;

    Ok(())
}

/// Generate MainActivity with NavHost for routing
fn generate_navhost_main_activity(config: &Config, routes: &[routes::Route], app_config: &AppConfig) -> String {
    // Generate composable calls for each route
    let mut composable_entries = Vec::new();

    for route in routes {
        let screen_call = if route.params.is_empty() {
            // No parameters: ProfileScreen(navController)
            format!("{}(navController)", route.screen_name)
        } else {
            // With parameters: extract route via toRoute<T>() for type-safe navigation
            // it.toRoute<Routes.Photo>().id, it.toRoute<Routes.Photo>().name, etc.
            let params_str = route
                .params
                .iter()
                .map(|p| format!("it.toRoute<Routes.{}>().{}", route.name, p.name))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}(navController, {})", route.screen_name, params_str)
        };

        // Wrap screen with layout chain (outermost first)
        // e.g., layouts = ["RootLayout", "AdminLayout"] becomes:
        // RootLayout { AdminLayout { HomeScreen(navController) } }
        let wrapped_screen = if route.layouts.is_empty() {
            screen_call
        } else {
            let mut result = screen_call;
            // Wrap from innermost to outermost
            for layout in route.layouts.iter().rev() {
                result = format!("{} {{ {} }}", layout, result);
            }
            result
        };

        // Use instant transitions (no animation) by default for snappy navigation
        let composable = format!(
            "        composable<Routes.{}>(
            enterTransition = {{ EnterTransition.None }},
            exitTransition = {{ ExitTransition.None }},
            popEnterTransition = {{ EnterTransition.None }},
            popExitTransition = {{ ExitTransition.None }}
        ){{ {} }}",
            route.name, wrapped_screen
        );
        composable_entries.push(composable);
    }

    let composables = composable_entries.join("\n");

    // Find Home route for start destination (default to first route)
    let start_destination = routes
        .iter()
        .find(|r| r.name == "Home")
        .map(|r| format!("Routes.{}", r.name))
        .unwrap_or_else(|| format!("Routes.{}", routes[0].name));

    // Generate imports based on color scheme
    // Generate store binding imports if needed
    let binding_imports = if let DarkMode::Binding { store, .. } = &app_config.dark_mode {
        // Find the import for this store and resolve it
        let store_import = app_config.imports.iter()
            .find(|imp| imp.ends_with(store))
            .map(|imp| {
                // Resolve $stores.X to package.stores.X
                if imp.starts_with('$') {
                    let rest = &imp[1..];
                    format!("{}.{}", config.android.package, rest)
                } else {
                    imp.clone()
                }
            })
            .unwrap_or_else(|| format!("{}.stores.{}", config.android.package, store));

        format!(r#"
import {}
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue"#, store_import)
    } else {
        String::new()
    };

    let theme_imports = match &app_config.color_scheme {
        ColorScheme::Dynamic => format!(r#"import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.platform.LocalContext{}"#, binding_imports),
        ColorScheme::Default | ColorScheme::Custom { .. } => {
            match &app_config.dark_mode {
                DarkMode::Binding { .. } => format!(r#"import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme{}"#, binding_imports),
                DarkMode::System => r#"import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme"#.to_string(),
                _ => "import androidx.compose.material3.MaterialTheme".to_string(),
            }
        }
    };

    // Generate dark theme check based on dark_mode setting
    let dark_theme_check = match &app_config.dark_mode {
        DarkMode::System => "val darkTheme = isSystemInDarkTheme()".to_string(),
        DarkMode::Light => "val darkTheme = false".to_string(),
        DarkMode::Dark => "val darkTheme = true".to_string(),
        DarkMode::Binding { store, property } => format!(
            r#"val settingsState by {}.state.collectAsState()
            val systemDarkTheme = isSystemInDarkTheme()
            val darkTheme = when (settingsState.{}) {{
                "light" -> false
                "dark" -> true
                else -> systemDarkTheme
            }}"#,
            store, property
        ),
    };

    // Generate color scheme setup based on color_scheme setting
    let color_scheme_setup = match &app_config.color_scheme {
        ColorScheme::Dynamic => format!(
            r#"{}
            val context = LocalContext.current
            val colorScheme = when {{
                Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {{
                    if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
                }}
                darkTheme -> darkColorScheme()
                else -> lightColorScheme()
            }}"#,
            dark_theme_check
        ),
        ColorScheme::Default => match &app_config.dark_mode {
            DarkMode::System | DarkMode::Binding { .. } => format!(
                r#"{}
            val colorScheme = if (darkTheme) darkColorScheme() else lightColorScheme()"#,
                dark_theme_check
            ),
            DarkMode::Light => "val colorScheme = lightColorScheme()".to_string(),
            DarkMode::Dark => "val colorScheme = darkColorScheme()".to_string(),
        },
        ColorScheme::Custom { .. } => {
            // TODO: Support custom colors
            "val colorScheme = lightColorScheme()".to_string()
        }
    };

    // Generate MaterialTheme wrapper - use colorScheme param if we computed one
    let material_theme_start = if matches!(app_config.color_scheme, ColorScheme::Default)
        && matches!(app_config.dark_mode, DarkMode::System)
        && !matches!(app_config.color_scheme, ColorScheme::Dynamic) {
        // Simple case: default theme with system dark mode - just use MaterialTheme {}
        "MaterialTheme {".to_string()
    } else {
        "MaterialTheme(colorScheme = colorScheme) {".to_string()
    };

    // For simple default case, skip the color scheme setup entirely
    let needs_color_scheme_setup = !matches!(
        (&app_config.color_scheme, &app_config.dark_mode),
        (ColorScheme::Default, DarkMode::System)
    );

    let set_content_body = if needs_color_scheme_setup {
        format!(
            r#"            {}
            {}
                val navController = rememberNavController()
                CompositionLocalProvider(LocalNavController provides navController) {{
                    NavHost(
                        navController = navController,
                        startDestination = {}
                    ) {{
{}
                    }}
                }}
            }}"#,
            color_scheme_setup,
            material_theme_start,
            start_destination,
            composables
        )
    } else {
        format!(
            r#"            MaterialTheme {{
                val navController = rememberNavController()
                CompositionLocalProvider(LocalNavController provides navController) {{
                    NavHost(
                        navController = navController,
                        startDestination = {}
                    ) {{
{}
                    }}
                }}
            }}"#,
            start_destination,
            composables
        )
    };

    // Only include layouts import if any route uses layouts
    let has_layouts = routes.iter().any(|r| !r.layouts.is_empty());
    let layouts_import = if has_layouts {
        format!("import {}.layouts.*\n", config.android.package)
    } else {
        String::new()
    };

    format!(
        r#"package {}

import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.Button
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.navigation.NavController
{}
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.toRoute
import {}.routes.Routes
import {}.screens.*
{}
// Global navigation controller accessible via LocalNavController.current
val LocalNavController = staticCompositionLocalOf<NavController> {{
    error("NavController not provided - ensure you're inside a CompositionLocalProvider")
}}

// Global error state for app-wide error handling
private val globalError = mutableStateOf<String?>(null)

// Safe navigation extension that catches errors instead of crashing
fun NavController.navigateSafe(route: String) {{
    try {{
        navigate(route)
    }} catch (e: IllegalArgumentException) {{
        Log.e("Navigation", "Route not found: $route", e)
        globalError.value = route
    }}
}}

// Error screen for navigation failures
@Composable
fun ErrorScreen(
    route: String?,
    onGoBack: () -> Unit
) {{
    Surface(
        modifier = Modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background
    ) {{
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(32.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {{
            Text(
                text = "Route doesn't exist",
                fontSize = 24.sp,
                color = MaterialTheme.colorScheme.onBackground
            )
            Spacer(modifier = Modifier.height(16.dp))
            Text(
                text = "/$route",
                fontSize = 16.sp,
                color = MaterialTheme.colorScheme.primary,
                fontFamily = FontFamily.Monospace
            )
            Spacer(modifier = Modifier.height(32.dp))
            Button(onClick = onGoBack) {{
                Text("Go Back")
            }}
        }}
    }}
}}

class MainActivity : ComponentActivity() {{
    override fun onCreate(savedInstanceState: Bundle?) {{
        super.onCreate(savedInstanceState)

        setContent {{
            // Check for navigation errors first
            val failedRoute = globalError.value
            if (failedRoute != null) {{
                // Show error screen for navigation failures
                MaterialTheme {{
                    ErrorScreen(
                        route = failedRoute,
                        onGoBack = {{
                            globalError.value = null
                        }}
                    )
                }}
            }} else {{
{}
            }}
        }}
    }}
}}
"#,
        config.android.package,
        theme_imports,
        config.android.package,
        config.android.package,
        layouts_import,
        set_content_body
    )
}

/// Generate a default MainActivity with basic "Hello, Whitehall!" content
fn generate_default_main_activity(config: &Config) -> String {
    format!(
        r#"package {}

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier

class MainActivity : ComponentActivity() {{
    override fun onCreate(savedInstanceState: Bundle?) {{
        super.onCreate(savedInstanceState)
        setContent {{
            MaterialTheme {{
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {{
                    Text("Hello, Whitehall!")
                }}
            }}
        }}
    }}
}}
"#,
        config.android.package
    )
}

/// Parse main.wh to extract <App> configuration
/// Returns AppConfig with theme settings, or default if no App component found
fn parse_app_config(main_file: Option<&WhitehallFile>) -> AppConfig {
    let Some(file) = main_file else {
        return AppConfig::default();
    };

    let Ok(source) = fs::read_to_string(&file.path) else {
        return AppConfig::default();
    };

    let mut config = AppConfig::default();

    // Parse imports (e.g., "import $stores.SettingsStore")
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            config.imports.push(trimmed[7..].trim().to_string());
        }
    }

    // Simple parsing: look for <App ...> and extract props
    // Format: <App colorScheme="dynamic" darkMode="system">
    // Or: <App colorScheme="dynamic" darkMode={SettingsStore.theme}>
    if let Some(app_start) = source.find("<App") {
        let app_section = &source[app_start..];
        let app_end = app_section.find('>').unwrap_or(app_section.len());
        let app_tag = &app_section[..app_end];

        // Parse colorScheme prop
        if let Some(cs_start) = app_tag.find("colorScheme=") {
            let after_eq = &app_tag[cs_start + 12..];
            if let Some(value) = extract_prop_value(after_eq) {
                config.color_scheme = match value.as_str() {
                    "dynamic" => ColorScheme::Dynamic,
                    _ => ColorScheme::Default,
                };
            }
        }

        // Parse darkMode prop
        if let Some(dm_start) = app_tag.find("darkMode=") {
            let after_eq = &app_tag[dm_start + 9..];
            if let Some((value, is_binding)) = extract_prop_value_with_type(after_eq) {
                if is_binding {
                    // Parse binding like "SettingsStore.theme"
                    if let Some((store, property)) = parse_store_binding(&value) {
                        config.dark_mode = DarkMode::Binding { store, property };
                    } else {
                        config.dark_mode = DarkMode::System;
                    }
                } else {
                    config.dark_mode = match value.as_str() {
                        "light" => DarkMode::Light,
                        "dark" => DarkMode::Dark,
                        "system" | _ => DarkMode::System,
                    };
                }
            }
        }
    }

    config
}

/// Parse a store binding expression like "SettingsStore.theme" into (store, property)
fn parse_store_binding(expr: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = expr.split('.').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

/// Extract a quoted prop value from a string starting after the '='
fn extract_prop_value(s: &str) -> Option<String> {
    extract_prop_value_with_type(s).map(|(v, _)| v)
}

/// Extract a prop value and whether it's a binding (in curly braces)
/// Returns (value, is_binding)
fn extract_prop_value_with_type(s: &str) -> Option<(String, bool)> {
    let s = s.trim_start();
    if s.starts_with('"') {
        // "value" - static string
        let end = s[1..].find('"')?;
        Some((s[1..=end].to_string(), false))
    } else if s.starts_with('{') {
        // {value} - expression/binding
        let end = s[1..].find('}')?;
        let inner = s[1..=end].trim();
        // Handle {"value"} case - still a static value
        if inner.starts_with('"') && inner.ends_with('"') {
            Some((inner[1..inner.len()-1].to_string(), false))
        } else {
            // Expression binding like {SettingsStore.theme}
            Some((inner.to_string(), true))
        }
    } else {
        None
    }
}

/// Generate Routes.kt file from route directory structure
fn generate_routes_file(config: &Config, output_dir: &Path) -> Result<()> {
    // Discover routes from src/routes/ directory
    let discovered_routes = routes::discover_routes()?;

    // If no routes found, skip generation
    if discovered_routes.is_empty() {
        return Ok(());
    }

    // Generate Routes.kt content
    let routes_content = routes::generate_routes_kt(&discovered_routes, &config.android.package);

    // Write to build/app/src/main/kotlin/{package}/routes/Routes.kt
    let package_path = config.android.package.replace('.', "/");
    let routes_dir = output_dir
        .join("app/src/main/kotlin")
        .join(package_path)
        .join("routes");

    fs::create_dir_all(&routes_dir)?;
    let routes_file = routes_dir.join("Routes.kt");
    fs::write(routes_file, routes_content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_kotlin_output_path() {
        let file = WhitehallFile {
            path: PathBuf::from("src/components/Button.wh"),
            file_type: FileType::Component,
            component_name: "Button".to_string(),
            package_path: "com.example.app.components".to_string(),
        };

        let output_dir = Path::new("build");
        let result = get_kotlin_output_path(output_dir, &file);

        assert_eq!(
            result,
            PathBuf::from("build/app/src/main/kotlin/com/example/app/components/Button.kt")
        );
    }

    #[test]
    fn test_default_main_activity_generation() {
        use crate::config::{AndroidConfig, BuildConfig, Config, FfiConfig, ProjectConfig, ToolchainConfig};

        let config = Config {
            project: ProjectConfig {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
            },
            android: AndroidConfig {
                min_sdk: 24,
                target_sdk: 34,
                package: "com.example.test".to_string(),
            },
            build: BuildConfig::default(),
            toolchain: ToolchainConfig::default(),
            ffi: FfiConfig::default(),
        };

        let content = generate_default_main_activity(&config);

        assert!(content.contains("package com.example.test"));
        assert!(content.contains("class MainActivity"));
        assert!(content.contains("Hello, Whitehall!"));
    }
}
