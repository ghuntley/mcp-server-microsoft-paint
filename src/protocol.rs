use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::error::Result;
use crate::core;

// Define handler type using Box<dyn Fn> to allow storing async functions
// This avoids type issues with different impl Future types
pub type MethodHandler = Box<dyn Fn(crate::PaintServerState, Option<Value>) -> 
    futures::future::BoxFuture<'static, Result<Value>> + Send + Sync>;

// Function to box the handlers properly to match the type
fn box_handler<F, Fut>(f: F) -> MethodHandler 
where
    F: Fn(crate::PaintServerState, Option<Value>) -> Fut + Send + Sync + 'static,
    Fut: futures::Future<Output = Result<Value>> + Send + 'static,
{
    Box::new(move |state, value| Box::pin(f(state, value)))
}

// === Request Parameters ===

#[derive(Deserialize, Debug)]
pub struct ConnectParams {
    pub client_id: String,
    pub client_name: String,
}

#[derive(Deserialize, Debug)]
pub struct SelectToolParams {
    pub tool: String, // Consider using an enum later: "pencil|brush|fill|text|eraser|select|shape"
    pub shape_type: Option<String>, // Consider enum: "rectangle|ellipse|line|..."
}

#[derive(Deserialize, Debug)]
pub struct SetColorParams {
    pub color: String, // Expecting "#RRGGBB"
}

#[derive(Deserialize, Debug)]
pub struct SetThicknessParams {
    pub level: u32, // Expecting 1-5
}

#[derive(Deserialize, Debug)]
pub struct SetBrushSizeParams {
    pub size: u32, // Expecting 1-30
    pub tool: Option<String>, // Consider enum: "pencil|brush"
}

#[derive(Deserialize, Debug)]
pub struct SetFillParams {
    pub fill_type: String, // Expecting "none|solid|outline"
}

#[derive(Deserialize, Debug)]
pub struct DrawPixelParams {
    pub x: i32,
    pub y: i32,
    pub color: Option<String>, // Optional color in #RRGGBB format
}

#[derive(Deserialize, Debug)]
pub struct DrawLineParams {
    pub start_x: i32,
    pub start_y: i32,
    pub end_x: i32,
    pub end_y: i32,
    pub color: Option<String>,     // Optional color in #RRGGBB format
    pub thickness: Option<u32>,    // Optional thickness level (1-5)
}

#[derive(Deserialize, Debug)]
pub struct DrawShapeParams {
    pub shape_type: String,        // "rectangle|ellipse|line|arrow|triangle|pentagon|hexagon"
    pub start_x: i32,
    pub start_y: i32,
    pub end_x: i32,
    pub end_y: i32,
    pub color: Option<String>,     // Optional color in #RRGGBB format
    pub thickness: Option<u32>,    // Optional thickness level (1-5)
    pub fill_type: Option<String>, // Optional fill type "none|solid|outline"
}

#[derive(Deserialize, Debug)]
pub struct DrawPolylineParams {
    pub points: Vec<Point>,         // Series of points to connect
    pub color: Option<String>,      // Optional color in #RRGGBB format
    pub thickness: Option<u32>,     // Optional thickness level (1-5)
    pub tool: Option<String>,       // Optional tool: "pencil" or "brush"
}

#[derive(Deserialize, Debug)]
pub struct AddTextParams {
    pub x: i32,                     // X position to place text
    pub y: i32,                     // Y position to place text
    pub text: String,               // Text content to add
    pub color: Option<String>,      // Optional color in #RRGGBB format
    pub font_name: Option<String>,  // Optional font name
    pub font_size: Option<u32>,     // Optional font size
    pub font_style: Option<String>, // Optional style: "regular", "bold", "italic", "bold_italic"
}

#[derive(Deserialize, Debug)]
pub struct CreateCanvasParams {
    pub width: u32,                 // Canvas width in pixels
    pub height: u32,                // Canvas height in pixels
    pub background_color: Option<String>, // Optional background color in #RRGGBB format
}

#[derive(Deserialize, Debug)]
pub struct SaveCanvasParams {
    pub file_path: String,         // Path where to save the file
    pub format: String,            // Format - "png", "jpeg", or "bmp"
}

#[derive(Deserialize, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

// Add more request parameter structs here...
// e.g., DrawLineParams, DrawPixelParams, AddTextParams, etc.

// === Response Payloads ===

#[derive(Serialize, Debug)]
pub struct SuccessResponse {
    pub status: String, // Always "success"
}

#[derive(Serialize, Debug)]
pub struct ConnectResponse {
    pub status: String, // Always "success"
    pub paint_version: String,
    pub canvas_width: u32,
    pub canvas_height: u32,
}

#[derive(Serialize, Debug)]
pub struct GetVersionResponse {
    pub status: String, // Always "success"
    pub protocol_version: String,
    pub server_version: String,
    pub paint_version: String,
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub status: String, // Always "error"
    pub error: ErrorDetails,
}

#[derive(Serialize, Debug)]
pub struct ErrorDetails {
    pub code: i32,
    pub message: String,
}

// Add more response structs here...
// e.g., GetCanvasDimensionsResponse, FetchImageResponse, etc.


// === Utility ===

// Helper function to create a standard success response
pub fn success_response() -> serde_json::Value {
    serde_json::to_value(SuccessResponse { status: "success".to_string() }).unwrap_or_default()
}

// Helper function to create a standard error response
pub fn error_response(code: i32, message: String) -> serde_json::Value {
    serde_json::to_value(ErrorResponse {
        status: "error".to_string(),
        error: ErrorDetails { code, message },
    })
    .unwrap_or_default()
}

// Basic tests for struct serialization/deserialization
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_params_deserialization() {
        let json = r#"{
            "client_id": "test-client",
            "client_name": "Test App"
        }"#;
        let params: ConnectParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.client_id, "test-client");
        assert_eq!(params.client_name, "Test App");
    }

    #[test]
    fn test_connect_response_serialization() {
        let response = ConnectResponse {
            status: "success".to_string(),
            paint_version: "windows11".to_string(),
            canvas_width: 1024,
            canvas_height: 768,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"success\""));
        assert!(json.contains("\"paint_version\":\"windows11\""));
        assert!(json.contains("\"canvas_width\":1024"));
        assert!(json.contains("\"canvas_height\":768"));
    }

     #[test]
    fn test_error_response_serialization() {
        let response_val = error_response(1001, "Window not found".to_string());
        let json_string = serde_json::to_string(&response_val).unwrap();
        assert!(json_string.contains("\"status\":\"error\""));
        assert!(json_string.contains("\"code\":1001"));
        assert!(json_string.contains("\"message\":\"Window not found\""));
    }
    
    #[test]
    fn test_draw_polyline_params_deserialization() {
        let json = r###"{
            "points": [
                {"x": 10, "y": 20},
                {"x": 30, "y": 40},
                {"x": 50, "y": 60}
            ],
            "color": "#FF0000",
            "thickness": 2,
            "tool": "pencil"
        }"###;
        
        let params: DrawPolylineParams = serde_json::from_str(json).unwrap();
        
        assert_eq!(params.points.len(), 3);
        assert_eq!(params.points[0].x, 10);
        assert_eq!(params.points[0].y, 20);
        assert_eq!(params.points[1].x, 30);
        assert_eq!(params.points[1].y, 40);
        assert_eq!(params.points[2].x, 50);
        assert_eq!(params.points[2].y, 60);
        
        assert_eq!(params.color.as_deref(), Some("#FF0000"));
        assert_eq!(params.thickness, Some(2));
        assert_eq!(params.tool.as_deref(), Some("pencil"));
    }

    // Add more tests for other structs...
}

// Map of method names to handler functions
pub fn get_method_handler(method: &str) -> Option<MethodHandler> {
    match method {
        "initialize" => Some(box_handler(core::handle_initialize)),
        "connect" => Some(box_handler(core::handle_connect)),
        "activate_window" => Some(box_handler(core::handle_activate_window)),
        "get_canvas_dimensions" => Some(box_handler(core::handle_get_canvas_dimensions)),
        "disconnect" => Some(box_handler(core::handle_disconnect)),
        "get_version" => Some(box_handler(core::handle_get_version)),
        // Drawing commands
        "draw_pixel" => Some(box_handler(core::handle_draw_pixel)),
        "draw_line" => Some(box_handler(core::handle_draw_line)),
        "draw_shape" => Some(box_handler(core::handle_draw_shape)),
        "draw_polyline" => Some(box_handler(core::handle_draw_polyline)),
        // Text operations
        "add_text" => Some(box_handler(core::handle_add_text)),
        // Selection operations
        "select_region" => Some(box_handler(core::handle_select_region)),
        "copy_selection" => Some(box_handler(core::handle_copy_selection)),
        "paste" => Some(box_handler(core::handle_paste)),
        // Canvas operations
        "clear_canvas" => Some(box_handler(core::handle_clear_canvas)),
        "create_canvas" => Some(box_handler(core::handle_create_canvas)),
        // Tool settings
        "select_tool" => Some(box_handler(core::handle_select_tool)),
        "set_color" => Some(box_handler(core::handle_set_color)),
        "set_thickness" => Some(box_handler(core::handle_set_thickness)),
        "set_brush_size" => Some(box_handler(core::handle_set_brush_size)),
        "set_fill" => Some(box_handler(core::handle_set_fill)),
        // Unknown method
        _ => None,
    }
} 