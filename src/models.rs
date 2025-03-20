use serde::{Deserialize, Serialize};

// Status Response
#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub paint_window_handle: String,
    pub version: String,
}

// Connect Response
#[derive(Serialize)]
pub struct ConnectResponse {
    pub success: bool,
    pub paint_window_handle: String,
    pub error: Option<String>,
}

// Generic Operation Response
#[derive(Serialize)]
pub struct OperationResponse {
    pub success: bool,
    pub error: Option<String>,
}

// Draw Pixel Request
#[derive(Deserialize)]
pub struct DrawPixelRequest {
    pub x: i32,
    pub y: i32,
    pub color: Option<String>,
}

// Set Brush Size Request
#[derive(Deserialize)]
pub struct SetBrushSizeRequest {
    pub size: u32,
    pub tool: Option<String>,
}

// Fetch Image Request
#[derive(Deserialize)]
pub struct FetchImageRequest {
    pub path: String,
}

// Fetch Image Response
#[derive(Serialize)]
pub struct FetchImageResponse {
    pub status: String,
    pub data: String, // base64 encoded image data
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub error: Option<String>,
}

// Draw Line Request
#[derive(Deserialize)]
pub struct DrawLineRequest {
    pub start_x: i32,
    pub start_y: i32,
    pub end_x: i32,
    pub end_y: i32,
    pub color: String,
    pub thickness: u32,
}

// Draw Rectangle Request
#[derive(Deserialize)]
pub struct DrawRectangleRequest {
    pub start_x: i32,
    pub start_y: i32,
    pub width: i32,
    pub height: i32,
    pub filled: bool,
    pub color: String,
    pub thickness: u32,
}

// Draw Circle Request
#[derive(Deserialize)]
pub struct DrawCircleRequest {
    pub center_x: i32,
    pub center_y: i32,
    pub radius: i32,
    pub filled: bool,
    pub color: String,
    pub thickness: u32,
}

// Select Tool Request
#[derive(Deserialize)]
pub struct SelectToolRequest {
    pub tool: String,
}

// Set Color Request
#[derive(Deserialize)]
pub struct SetColorRequest {
    pub color: String,
}

// Save Request
#[derive(Deserialize)]
pub struct SaveRequest {
    pub filename: String,
    pub format: String,
}

// Image Recreation Request
#[derive(Deserialize)]
pub struct RecreateImageRequest {
    pub image_base64: String,
    pub output_filename: Option<String>,
    pub max_detail_level: Option<u32>,
}

// NEW MODELS FOR TEXT SUPPORT

// Add Text Request
#[derive(Deserialize)]
pub struct AddTextRequest {
    pub x: i32,
    pub y: i32,
    pub text: String,
    pub font_name: Option<String>,
    pub font_size: Option<u32>,
    pub font_style: Option<String>, // "regular", "bold", "italic", "bold_italic"
    pub color: Option<String>,
}

// NEW MODELS FOR IMAGE TRANSFORMATIONS

// Rotate Image Request
#[derive(Deserialize)]
pub struct RotateImageRequest {
    pub degrees: i32, // Typically 90, 180, or 270
    pub clockwise: bool,
}

// Flip Image Request
#[derive(Deserialize)]
pub struct FlipImageRequest {
    pub direction: String, // "horizontal" or "vertical"
}

// Scale Image Request
#[derive(Deserialize)]
pub struct ScaleImageRequest {
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub maintain_aspect_ratio: Option<bool>,
    pub percentage: Option<f32>, // e.g., 0.5 for 50%, 2.0 for 200%
}

// Crop Image Request
#[derive(Deserialize)]
pub struct CropImageRequest {
    pub start_x: i32,
    pub start_y: i32,
    pub width: i32,
    pub height: i32,
}

// NEW MODELS FOR CANVAS MANAGEMENT

// Create Canvas Request
#[derive(Deserialize)]
pub struct CreateCanvasRequest {
    pub width: i32,
    pub height: i32,
    pub background_color: Option<String>, // HTML color code, e.g., "#FFFFFF" for white
} 