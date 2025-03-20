use axum::{
    routing::{get, post},
    Router, Json, http::StatusCode, response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod paint_integration;
mod models;
mod prompts;

use paint_integration::{PaintManager, PaintError, ImageMetadata};
use models::*;
use prompts::{get_prompt, get_all_prompts};

// Additional structures for prompt-related responses
#[derive(Serialize)]
struct PromptResponse {
    operation: String,
    prompt: String,
}

#[derive(Serialize)]
struct AllPromptsResponse {
    prompts: Vec<PromptResponse>,
}

#[derive(Deserialize)]
struct PromptRequest {
    operation: String,
}

#[derive(Serialize)]
struct LlmSystemPromptResponse {
    system_prompt: String,
}

#[derive(Serialize)]
struct LlmOperationExampleResponse {
    operation: String,
    example: String,
}

#[derive(Deserialize)]
struct LlmOperationExampleRequest {
    operation: String,
}

#[derive(Deserialize)]
struct RecreateImageRequest {
    image_base64: String,
    output_filename: Option<String>,
    max_detail_level: u8,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create shared Paint Manager
    let paint_manager = Arc::new(Mutex::new(PaintManager::new()));

    // Build our application with routes
    let app = Router::new()
        .route("/status", get(status_handler))
        .route("/connect", post(connect_handler))
        .route("/draw/line", post(draw_line_handler))
        .route("/draw/rectangle", post(draw_rectangle_handler))
        .route("/draw/circle", post(draw_circle_handler))
        .route("/draw/pixel", post(draw_pixel_handler))
        .route("/tool/select", post(select_tool_handler))
        .route("/color/set", post(set_color_handler))
        .route("/brush/size", post(set_brush_size_handler))
        .route("/save", post(save_handler))
        .route("/fetch", post(fetch_image_handler))
        .route("/recreate-image", post(recreate_image_handler))
        .route("/prompt", post(prompt_handler))
        .route("/prompts", get(all_prompts_handler))
        .route("/llm/system-prompt", get(llm_system_prompt_handler))
        .route("/llm/operation-example", post(llm_operation_example_handler))
        // Add new routes for text support, image transformations, and canvas management
        .route("/text/add", post(add_text_handler))
        .route("/image/rotate", post(rotate_image_handler))
        .route("/image/flip", post(flip_image_handler))
        .route("/image/scale", post(scale_image_handler))
        .route("/image/crop", post(crop_image_handler))
        .route("/canvas/create", post(create_canvas_handler))
        .with_state(paint_manager);

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Request handlers

async fn status_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
) -> impl IntoResponse {
    let paint_manager = state.lock().unwrap();
    let status = paint_manager.get_status();
    
    (StatusCode::OK, Json(status))
}

async fn connect_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    match paint_manager.connect() {
        Ok(hwnd) => {
            let response = ConnectResponse {
                success: true,
                paint_window_handle: format!("{:?}", hwnd),
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            let response = ConnectResponse {
                success: false,
                paint_window_handle: String::new(),
                error: Some(err.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

async fn draw_line_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<DrawLineRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.draw_line(
        payload.start_x, 
        payload.start_y, 
        payload.end_x, 
        payload.end_y, 
        &payload.color, 
        payload.thickness
    );
    
    handle_operation_result(result)
}

async fn draw_rectangle_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<DrawRectangleRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.draw_rectangle(
        payload.start_x, 
        payload.start_y, 
        payload.width, 
        payload.height, 
        payload.filled, 
        &payload.color, 
        payload.thickness
    );
    
    handle_operation_result(result)
}

async fn draw_circle_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<DrawCircleRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.draw_circle(
        payload.center_x, 
        payload.center_y, 
        payload.radius, 
        payload.filled, 
        &payload.color, 
        payload.thickness
    );
    
    handle_operation_result(result)
}

async fn draw_pixel_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<DrawPixelRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.draw_pixel(
        payload.x, 
        payload.y, 
        payload.color.as_deref()
    );
    
    handle_operation_result(result)
}

async fn select_tool_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<SelectToolRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.select_tool(&payload.tool);
    
    handle_operation_result(result)
}

async fn set_color_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<SetColorRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.set_color(&payload.color);
    
    handle_operation_result(result)
}

async fn set_brush_size_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<SetBrushSizeRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.set_brush_size(
        payload.size,
        payload.tool.as_deref()
    );
    
    handle_operation_result(result)
}

async fn save_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<SaveRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.save(&payload.filename, &payload.format);
    
    handle_operation_result(result)
}

async fn fetch_image_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<FetchImageRequest>,
) -> impl IntoResponse {
    let paint_manager = state.lock().unwrap();
    
    match paint_manager.fetch_image_with_metadata(&payload.path) {
        Ok(metadata) => {
            let response = FetchImageResponse {
                status: "success".to_string(),
                data: metadata.data,
                format: metadata.format,
                width: metadata.width,
                height: metadata.height,
                error: None,
            };
            (StatusCode::OK, Json(response))
        },
        Err(err) => {
            let response = FetchImageResponse {
                status: "error".to_string(),
                data: "".to_string(),
                format: "".to_string(),
                width: 0,
                height: 0,
                error: Some(err.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

async fn prompt_handler(
    Json(payload): Json<PromptRequest>,
) -> impl IntoResponse {
    match get_prompt(&payload.operation) {
        Some(prompt_text) => {
            let response = PromptResponse {
                operation: payload.operation,
                prompt: prompt_text.to_string(),
            };
            (StatusCode::OK, Json(response))
        },
        None => {
            (
                StatusCode::NOT_FOUND, 
                Json(serde_json::json!({
                    "error": format!("No prompt found for operation: {}", payload.operation)
                }))
            )
        }
    }
}

async fn all_prompts_handler() -> impl IntoResponse {
    let prompts_map = get_all_prompts();
    
    let prompts = prompts_map.iter()
        .map(|(op, text)| PromptResponse {
            operation: op.to_string(),
            prompt: text.trim().to_string(),
        })
        .collect::<Vec<_>>();
    
    let response = AllPromptsResponse { prompts };
    (StatusCode::OK, Json(response))
}

async fn llm_system_prompt_handler() -> impl IntoResponse {
    let system_prompt = prompts::format_system_prompt();
    
    let response = LlmSystemPromptResponse {
        system_prompt,
    };
    
    (StatusCode::OK, Json(response))
}

async fn llm_operation_example_handler(
    Json(payload): Json<LlmOperationExampleRequest>,
) -> impl IntoResponse {
    match prompts::format_operation_example(&payload.operation) {
        Some(example) => {
            let response = LlmOperationExampleResponse {
                operation: payload.operation,
                example,
            };
            (StatusCode::OK, Json(response))
        },
        None => {
            (
                StatusCode::NOT_FOUND, 
                Json(serde_json::json!({
                    "error": format!("No example found for operation: {}", payload.operation)
                }))
            )
        }
    }
}

// Handler for recreating images in Paint
async fn recreate_image_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<models::RecreateImageRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    
    let result = paint_manager.recreate_image(
        &payload.image_base64,
        payload.output_filename.as_deref(),
        payload.max_detail_level,
    );
    
    handle_operation_result(result)
}

// New handlers for text support, image transformations, and canvas management

async fn add_text_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<AddTextRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.add_text(
        payload.x,
        payload.y,
        &payload.text,
        payload.font_name.as_deref(),
        payload.font_size.as_ref(),
        payload.font_style.as_deref(),
        payload.color.as_deref(),
    );
    
    handle_operation_result(result)
}

async fn rotate_image_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<RotateImageRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.rotate_image(payload.degrees, payload.clockwise);
    
    handle_operation_result(result)
}

async fn flip_image_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<FlipImageRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.flip_image(&payload.direction);
    
    handle_operation_result(result)
}

async fn scale_image_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<ScaleImageRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.scale_image(
        payload.width,
        payload.height,
        payload.maintain_aspect_ratio,
        payload.percentage,
    );
    
    handle_operation_result(result)
}

async fn crop_image_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<CropImageRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.crop_image(
        payload.start_x,
        payload.start_y,
        payload.width,
        payload.height,
    );
    
    handle_operation_result(result)
}

async fn create_canvas_handler(
    state: axum::extract::State<Arc<Mutex<PaintManager>>>,
    Json(payload): Json<CreateCanvasRequest>,
) -> impl IntoResponse {
    let mut paint_manager = state.lock().unwrap();
    let result = paint_manager.create_canvas(
        payload.width,
        payload.height,
        payload.background_color.as_deref(),
    );
    
    handle_operation_result(result)
}

// Helper function for consistent response handling
fn handle_operation_result(result: Result<(), PaintError>) -> impl IntoResponse {
    match result {
        Ok(()) => {
            let response = OperationResponse {
                success: true,
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        Err(err) => {
            let response = OperationResponse {
                success: false,
                error: Some(err.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}
