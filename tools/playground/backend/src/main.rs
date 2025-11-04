use axum::{
    extract::Json,
    response::Json as JsonResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
struct CompileRequest {
    code: String,
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
struct CompileResponse {
    success: bool,
    output: String,
    errors: Vec<CompileError>,
    warnings: Vec<CompileError>,
    ast: Option<String>,
}

async fn compile(Json(req): Json<CompileRequest>) -> JsonResponse<CompileResponse> {
    // For now, use a simple component name and package
    // TODO: Parse these from frontmatter if present
    let component_name = "PlaygroundComponent";
    let package = "com.example.playground";

    match whitehall::transpiler::transpile(&req.code, package, component_name, None) {
        Ok(kotlin_code) => {
            JsonResponse(CompileResponse {
                success: true,
                output: kotlin_code,
                errors: vec![],
                warnings: vec![],
                ast: None,
            })
        },
        Err(e) => {
            // Try to parse error message format: [Line X:Y] message
            let (line, column, message) = parse_error_position(&e);

            JsonResponse(CompileResponse {
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
            })
        },
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 Whitehall Playground backend running on http://localhost:3000");
    println!("   API endpoint: POST /api/compile");
    println!();

    axum::serve(listener, app).await.unwrap();
}
