use axum::{
    extract::Json,
    response::Json as JsonResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
#[serde(untagged)]
enum CompileRequest {
    // Single file (legacy format for backward compatibility)
    Single { code: String },
    // Multiple files (new format)
    Multi { files: HashMap<String, String> },
}

#[derive(Serialize)]
struct CompileError {
    message: String,
    line: Option<usize>,
    column: Option<usize>,
    length: Option<usize>,
    severity: String,
    context: Option<String>,
}

#[derive(Serialize)]
struct FileCompileResult {
    success: bool,
    output: String,
    errors: Vec<CompileError>,
    warnings: Vec<CompileError>,
    ast: Option<String>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum CompileResponse {
    // Single file response (legacy format for backward compatibility)
    Single {
        success: bool,
        output: String,
        errors: Vec<CompileError>,
        warnings: Vec<CompileError>,
        ast: Option<String>,
    },
    // Multiple files response (new format)
    Multi {
        success: bool,
        results: HashMap<String, FileCompileResult>,
    },
}

async fn compile(Json(req): Json<CompileRequest>) -> JsonResponse<CompileResponse> {
    match req {
        // Handle single file (legacy format)
        CompileRequest::Single { code } => {
            let result = compile_single_file(&code, "PlaygroundComponent");
            JsonResponse(CompileResponse::Single {
                success: result.success,
                output: result.output,
                errors: result.errors,
                warnings: result.warnings,
                ast: result.ast,
            })
        }
        // Handle multiple files (new format)
        CompileRequest::Multi { files } => {
            let mut results = HashMap::new();
            let mut overall_success = true;

            for (filename, code) in files {
                // Extract component name from filename (remove .wh extension)
                let component_name = filename
                    .strip_suffix(".wh")
                    .unwrap_or(&filename)
                    .replace("/", "_")
                    .replace("-", "_");

                let result = compile_single_file(&code, &component_name);
                if !result.success {
                    overall_success = false;
                }
                results.insert(filename, result);
            }

            JsonResponse(CompileResponse::Multi {
                success: overall_success,
                results,
            })
        }
    }
}

fn compile_single_file(code: &str, component_name: &str) -> FileCompileResult {
    let package = "com.example.playground";

    match whitehall::transpiler::transpile(code, package, component_name, None) {
        Ok(kotlin_code) => FileCompileResult {
            success: true,
            output: kotlin_code.primary_content().to_string(),
            errors: vec![],
            warnings: vec![],
            ast: None,
        },
        Err(e) => {
            // Try to parse error message format: [Line X:Y] message
            let (line, column, message) = parse_error_position(&e);

            FileCompileResult {
                success: false,
                output: String::new(),
                errors: vec![CompileError {
                    message,
                    line,
                    column,
                    length: Some(1),
                    severity: "error".to_string(),
                    context: None,
                }],
                warnings: vec![],
                ast: None,
            }
        }
    }
}

/// Parse error messages with format "[Line X:Y] message" and extract position
fn parse_error_position(error_str: &str) -> (Option<usize>, Option<usize>, String) {
    // Try to match pattern: [Line X:Y] message
    if let Some(start) = error_str.find("[Line ") {
        if let Some(end) = error_str[start..].find(']') {
            let pos_str = &error_str[start + 6..start + end]; // Skip "[Line "
            let message = error_str[start + end + 2..].to_string(); // Skip "] "

            // Parse "X:Y"
            if let Some(colon_pos) = pos_str.find(':') {
                let line_str = &pos_str[..colon_pos];
                let col_str = &pos_str[colon_pos + 1..];

                if let (Ok(line), Ok(col)) = (line_str.parse::<usize>(), col_str.parse::<usize>()) {
                    return (Some(line), Some(col), message);
                }
            }
        }
    }

    // Fallback: return original message without position
    (None, None, error_str.to_string())
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/compile", post(compile))
        .layer(CorsLayer::permissive());

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Error: Failed to bind to port 3000: {}", e);
            eprintln!("Hint: Port 3000 may already be in use. Try:");
            eprintln!("  - Kill existing process: lsof -ti:3000 | xargs kill");
            eprintln!("  - Or use a different port");
            std::process::exit(1);
        }
    };

    println!("🚀 Whitehall Playground backend running on http://localhost:3000");
    println!("   API endpoint: POST /api/compile");
    println!();

    axum::serve(listener, app).await.unwrap();
}
