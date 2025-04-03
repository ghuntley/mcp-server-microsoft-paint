// Placeholder for core server logic (command handlers) 

use crate::error::{Result, MspMcpError};
use crate::protocol::{ConnectParams, ConnectResponse, success_response, DrawPixelParams, DrawLineParams, DrawShapeParams, DrawPolylineParams, SelectToolParams, SetColorParams, SetThicknessParams, SetBrushSizeParams, SetFillParams, AddTextParams, CreateCanvasParams};
use crate::windows;
use crate::windows::{get_paint_hwnd, get_initial_canvas_dimensions, activate_paint_window, get_canvas_dimensions, draw_pixel_at, draw_line_at, draw_shape, draw_polyline, clear_canvas, select_region, copy_selection, paste_at, add_text, create_canvas};
use crate::PaintServerState; // Import the state struct from lib.rs
use log::{info, warn, error, debug};
use serde_json::{json, Value};
use std::time;
use tokio;

// Handler for the 'connect' method
pub async fn handle_connect(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling connect request...");

    // Deserialize parameters
    let connect_params: ConnectParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for connect".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    info!("Client connected: id={}, name={}", connect_params.client_id, connect_params.client_name);

    // Get HWND from state (should have been set during initialize)
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        match *hwnd_state {
            Some(h) => h,
            // This should ideally not happen if initialize succeeded
            None => return Err(MspMcpError::General("Paint HWND not found in state after initialize".to_string())),
        }
    };

    // Get initial canvas dimensions (still needed for connect response)
    let (width, height) = get_initial_canvas_dimensions(hwnd)?;

    // Create and return the response
    Ok(json!({
        "jsonrpc": "2.0",
        "id": 1, // Should be extracted from the request
        "result": {
            "paint_version": "windows11", // Assuming Win11 for now
            "canvas_width": width,
            "canvas_height": height
        }
    }))
}

// Handler for the 'activate_window' method
pub async fn handle_activate_window(
    state: PaintServerState,
    _params: Option<Value>, // No parameters needed for this command
) -> Result<Value> {
    info!("Handling activate_window request...");

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        // Check if we have a stored HWND
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => {
                // No HWND stored yet - client should call connect first
                return Err(MspMcpError::OperationNotSupported(
                    "No Paint window available. Call connect first.".to_string()));
            }
        }
    };

    // Call the windows module to activate the window
    activate_paint_window(hwnd)?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'get_canvas_dimensions' method
pub async fn handle_get_canvas_dimensions(
    state: PaintServerState,
    _params: Option<Value>, // No parameters needed for this command
) -> Result<Value> {
    info!("Handling get_canvas_dimensions request...");

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        // Check if we have a stored HWND
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => {
                // No HWND stored yet - client should call connect first
                return Err(MspMcpError::OperationNotSupported(
                    "No Paint window available. Call connect first.".to_string()));
            }
        }
    };

    // Call the windows module to get canvas dimensions
    let (width, height) = get_canvas_dimensions(hwnd)?;

    // Return dimensions in response
    Ok(json!({
        "jsonrpc": "2.0",
        "id": 1, // Should be extracted from the request
        "result": {
            "width": width,
            "height": height
        }
    }))
}

// Handler for the 'disconnect' method
pub async fn handle_disconnect(
    state: PaintServerState,
    _params: Option<Value>, // No parameters needed for this command
) -> Result<Value> {
    info!("Handling disconnect request...");

    // Optionally clear the HWND state to indicate we're no longer connected
    {
        let mut hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        *hwnd_state = None;
        info!("Cleared Paint HWND state on disconnect");
    }

    // Note: we don't actually close Paint, just clear our reference to it
    // If we wanted to close Paint, we could use WM_CLOSE or TerminateProcess

    // Return success response
    Ok(success_response())
}

// Handler for the 'get_version' method
pub async fn handle_get_version(
    _state: PaintServerState, // No state needed for this command
    _params: Option<Value>,   // No parameters needed for this command
) -> Result<Value> {
    info!("Handling get_version request...");

    // Return version information
    Ok(json!({
        "jsonrpc": "2.0",
        "id": 1, // Should be extracted from the request
        "result": {
            "protocol_version": "1.1",
            "server_version": env!("CARGO_PKG_VERSION"),
            "paint_version": "windows11"
        }
    }))
}

// Handler for the 'draw_pixel' method
pub async fn handle_draw_pixel(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling draw_pixel request...");

    // Deserialize parameters
    let draw_params: DrawPixelParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for draw_pixel".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // If a color is specified, set it first
    if let Some(color) = &draw_params.color {
        windows::set_color(hwnd, color)?;
    }

    // Draw the pixel at the specified coordinates
    draw_pixel_at(hwnd, draw_params.x, draw_params.y)?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'draw_line' method
pub async fn handle_draw_line(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling draw_line request...");

    // Deserialize parameters
    let draw_params: DrawLineParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for draw_line".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // If a color is specified, set it first
    if let Some(color) = &draw_params.color {
        windows::set_color(hwnd, color)?;
    }

    // If thickness is specified, set it
    if let Some(thickness) = draw_params.thickness {
        windows::set_thickness(hwnd, thickness)?;
    }

    // Draw the line at the specified coordinates
    draw_line_at(
        hwnd, 
        draw_params.start_x, draw_params.start_y,
        draw_params.end_x, draw_params.end_y
    )?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'select_tool' method
pub async fn handle_select_tool(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling select_tool request...");

    // Deserialize parameters
    let tool_params: SelectToolParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for select_tool".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // Select the tool
    windows::select_tool(hwnd, &tool_params.tool)?;

    // If a shape type is specified, handle that as well
    if let Some(shape_type) = tool_params.shape_type {
        // TODO: Implement shape type selection
        info!("Would select shape type: {}", shape_type);
    }

    // Return success response
    Ok(success_response())
}

// Handler for the 'set_color' method
pub async fn handle_set_color(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling set_color request...");

    // Deserialize parameters
    let color_params: SetColorParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for set_color".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // Set the color
    windows::set_color(hwnd, &color_params.color)?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'set_thickness' method
pub async fn handle_set_thickness(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling set_thickness request...");

    // Deserialize parameters
    let thickness_params: SetThicknessParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for set_thickness".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // Set the thickness
    windows::set_thickness(hwnd, thickness_params.level)?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'set_brush_size' method
pub async fn handle_set_brush_size(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling set_brush_size request...");

    // Deserialize parameters
    let brush_params: SetBrushSizeParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for set_brush_size".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // Set the brush size
    windows::set_brush_size(hwnd, brush_params.size, brush_params.tool.as_deref())?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'set_fill' method
pub async fn handle_set_fill(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling set_fill request...");

    // Deserialize parameters
    let fill_params: SetFillParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for set_fill".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // Set the fill type
    windows::set_fill(hwnd, &fill_params.fill_type)?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'draw_shape' method
pub async fn handle_draw_shape(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling draw_shape request...");

    // Deserialize parameters
    let shape_params: DrawShapeParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for draw_shape".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // If a color is specified, set it first
    if let Some(color) = &shape_params.color {
        windows::set_color(hwnd, color)?;
    }

    // If a thickness is specified, set it
    if let Some(thickness) = shape_params.thickness {
        windows::set_thickness(hwnd, thickness)?;
    }

    // If a fill type is specified, set it
    if let Some(fill_type) = &shape_params.fill_type {
        windows::set_fill(hwnd, fill_type)?;
    }

    // Draw the shape
    draw_shape(
        hwnd,
        &shape_params.shape_type,
        shape_params.start_x, shape_params.start_y,
        shape_params.end_x, shape_params.end_y
    )?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'draw_polyline' method
pub async fn handle_draw_polyline(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling draw_polyline request...");

    // Deserialize parameters
    let polyline_params: DrawPolylineParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for draw_polyline".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // If a tool is specified, select it first (pencil or brush)
    if let Some(tool) = &polyline_params.tool {
        windows::select_tool(hwnd, tool)?;
    } else {
        // Default to pencil if no tool specified
        windows::select_tool(hwnd, "pencil")?;
    }

    // If a color is specified, set it
    if let Some(color) = &polyline_params.color {
        windows::set_color(hwnd, color)?;
    }

    // If a thickness is specified, set it
    if let Some(thickness) = polyline_params.thickness {
        windows::set_thickness(hwnd, thickness)?;
    }

    // Convert Point structs to (i32, i32) tuples for the Windows API
    let point_tuples: Vec<(i32, i32)> = polyline_params.points
        .iter()
        .map(|point| (point.x, point.y))
        .collect();

    // Draw the polyline
    draw_polyline(hwnd, &point_tuples)?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'clear_canvas' method
pub async fn handle_clear_canvas(
    state: PaintServerState,
    _params: Option<Value>, // No parameters needed
) -> Result<Value> {
    info!("Handling clear_canvas request...");

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // Clear the canvas
    clear_canvas(hwnd)?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'select_region' method
pub async fn handle_select_region(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling select_region request...");

    // Deserialize parameters - reusing DrawLineParams since it has the same structure
    let select_params: DrawLineParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for select_region".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // Select the region
    select_region(
        hwnd,
        select_params.start_x, select_params.start_y,
        select_params.end_x, select_params.end_y
    )?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'copy_selection' method
pub async fn handle_copy_selection(
    state: PaintServerState,
    _params: Option<Value>, // No parameters needed
) -> Result<Value> {
    info!("Handling copy_selection request...");

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // Copy the selection
    copy_selection(hwnd)?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'paste' method
pub async fn handle_paste(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling paste request...");

    // Deserialize parameters - we just need x, y coordinates
    let paste_params: DrawPixelParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for paste".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // Paste at the specified position
    paste_at(hwnd, paste_params.x, paste_params.y)?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'add_text' method
pub async fn handle_add_text(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling add_text request...");

    // Deserialize parameters
    let text_params: AddTextParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for add_text".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // Add text to the canvas
    add_text(
        hwnd,
        text_params.x,
        text_params.y,
        &text_params.text,
        text_params.color.as_deref(),
        text_params.font_name.as_deref(),
        text_params.font_size,
        text_params.font_style.as_deref()
    )?;

    // Return success response
    Ok(success_response())
}

// Handler for the 'create_canvas' method
pub async fn handle_create_canvas(
    state: PaintServerState,
    params: Option<Value>,
) -> Result<Value> {
    info!("Handling create_canvas request...");

    // Deserialize parameters
    let canvas_params: CreateCanvasParams = params
        .ok_or_else(|| MspMcpError::InvalidParameters("Missing params for create_canvas".to_string()))
        .and_then(|p| serde_json::from_value(p).map_err(MspMcpError::JsonError))?;

    // Get the Paint window handle from state
    let hwnd = {
        let hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        
        match *hwnd_state {
            Some(hwnd) => hwnd,
            None => return Err(MspMcpError::WindowNotFound),
        }
    };

    // Create a new canvas
    create_canvas(
        hwnd,
        canvas_params.width,
        canvas_params.height,
        canvas_params.background_color.as_deref()
    )?;

    // Get the updated canvas dimensions
    let (width, height) = get_canvas_dimensions(hwnd)?;

    // Return success response with the new dimensions
    Ok(json!({
        "status": "success",
        "canvas_width": width,
        "canvas_height": height
    }))
}

// Handler for the 'initialize' method
pub async fn handle_initialize(
    state: PaintServerState,
    _params: Option<Value>, // No parameters needed for initialization
) -> Result<Value> {
    info!("Handling initialize request...");
    
    // Find or launch Paint
    let hwnd = match windows::get_paint_hwnd() {
        Ok(h) => {
            info!("Found or launched Paint window: HWND={}", h);
            h
        },
        Err(e) => {
            error!("Failed to find or launch Paint: {}", e);
            return Err(e);
        }
    };
    
    // Store HWND in state
    {
        let mut hwnd_state = state.paint_hwnd.lock().map_err(|_| 
            MspMcpError::General("Failed to lock HWND state".to_string()))?;
        *hwnd_state = Some(hwnd);
        info!("Stored Paint HWND in server state");
    }
    
    // Return success
    Ok(json!({
        "jsonrpc": "2.0",
        "id": 1, // Should be overridden with actual request ID later
        "result": {
            "initialized": true
        }
    }))
}

// TODO: Add tests for handlers (might require mocking windows module) 