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
            JsonResponse(CompileResponse {
                success: false,
                output: String::new(),
                errors: vec![CompileError {
                    message: e,
                    line: None, // TODO: Extract from error message or enhance parser
                    column: None,
                    length: None,
                    severity: "error".to_string(),
                    context: None,
                }],
                warnings: vec![],
                ast: None,
            })
        },
    }
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
