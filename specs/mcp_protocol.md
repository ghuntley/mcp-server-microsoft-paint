# MCP Protocol Specification for Windows 11 Paint

## Overview

The Microsoft Paint Control Protocol (MCP) provides a standardized interface for programmatically controlling Windows 11 Paint. This specification defines the communication protocol between client applications and the Paint MCP server.

## Protocol Design

The MCP protocol is a JSON-based RPC protocol over HTTP/WebSockets. All commands and responses are formatted as JSON objects.

## Connection Management

### Connect Request

```json
{
  "command": "connect",
  "params": {
    "client_id": "unique-client-identifier",
    "client_name": "Sample App"
  }
}
```

### Connect Response

```json
{
  "status": "success",
  "paint_version": "windows11",
  "canvas_width": 800,
  "canvas_height": 600
}
```

### Disconnect Request

```json
{
  "command": "disconnect"
}
```

### Disconnect Response

```json
{
  "status": "success"
}
```

## Drawing Operations

### Select Tool

```json
{
  "command": "select_tool",
  "params": {
    "tool": "pencil|brush|fill|text|eraser|select|shape",
    "shape_type": "rectangle|ellipse|line|arrow|triangle|pentagon|hexagon"
  }
}
```

### Set Color

```json
{
  "command": "set_color",
  "params": {
    "color": "#RRGGBB"
  }
}
```

### Set Thickness

```json
{
  "command": "set_thickness",
  "params": {
    "level": 1-5
  }
}
```

### Set Brush Size

```json
{
  "command": "set_brush_size",
  "params": {
    "size": 1-30,           // Pixel size (1-30px depending on tool)
    "tool": "pencil|brush"  // Optional - defaults to current tool
  }
}
```

### Set Fill Type

```json
{
  "command": "set_fill",
  "params": {
    "type": "none|solid|outline"
  }
}
```

### Draw Line

```json
{
  "command": "draw_line",
  "params": {
    "start_x": 100,
    "start_y": 100,
    "end_x": 200,
    "end_y": 200,
    "color": "#RRGGBB",  // Optional
    "thickness": 2       // Optional
  }
}
```

### Draw Pixel

```json
{
  "command": "draw_pixel",
  "params": {
    "x": 150,
    "y": 150,
    "color": "#RRGGBB"  // Optional
  }
}
```

### Draw Shape

```json
{
  "command": "draw_shape",
  "params": {
    "shape": "rectangle|ellipse|line|arrow|triangle|pentagon|hexagon",
    "start_x": 100,
    "start_y": 100,
    "end_x": 300,
    "end_y": 200,
    "color": "#RRGGBB",     // Optional
    "thickness": 2,         // Optional
    "fill_type": "none|solid|outline"  // Optional
  }
}
```

### Draw Polyline

```json
{
  "command": "draw_polyline",
  "params": {
    "points": [
      {"x": 100, "y": 100},
      {"x": 150, "y": 50},
      {"x": 200, "y": 100}
    ],
    "color": "#RRGGBB",  // Optional
    "thickness": 2       // Optional
  }
}
```

### Add Text

```json
{
  "command": "add_text",
  "params": {
    "x": 100,
    "y": 100,
    "text": "Hello World",
    "color": "#RRGGBB"  // Optional
  }
}
```

### Enhanced Add Text (New)

```json
{
  "command": "add_text",
  "params": {
    "x": 100,
    "y": 100,
    "text": "Hello World",
    "font_name": "Arial",           // Optional
    "font_size": 24,                // Optional
    "font_style": "bold",           // Optional: regular|bold|italic|bold_italic
    "color": "#RRGGBB"              // Optional
  }
}
```

## Selection Operations

### Select Region

```json
{
  "command": "select_region",
  "params": {
    "start_x": 100,
    "start_y": 100,
    "end_x": 300,
    "end_y": 200
  }
}
```

### Copy Selection

```json
{
  "command": "copy_selection"
}
```

### Paste

```json
{
  "command": "paste",
  "params": {
    "x": 150,
    "y": 150
  }
}
```

## Canvas Management

### Clear Canvas

```json
{
  "command": "clear_canvas"
}
```

### Create New Canvas (New)

```json
{
  "command": "create_canvas",
  "params": {
    "width": 1024,
    "height": 768,
    "background_color": "#FFFFFF"  // Optional, defaults to white
  }
}
```

### Save Canvas

```json
{
  "command": "save",
  "params": {
    "path": "C:\\path\\to\\image.png",
    "format": "png|jpeg|bmp"
  }
}
```

### Fetch Image

```json
{
  "command": "fetch_image",
  "params": {
    "path": "C:\\path\\to\\image.png"
  }
}
```

### Fetch Image Response

```json
{
  "status": "success",
  "data": "base64_encoded_image_data_here",
  "format": "png",
  "width": 800,
  "height": 600
}
```

## Image Transformations (New)

### Rotate Image

```json
{
  "command": "rotate_image",
  "params": {
    "degrees": 90,            // Typically 90, 180, or 270
    "clockwise": true         // Optional, defaults to true
  }
}
```

### Flip Image

```json
{
  "command": "flip_image",
  "params": {
    "direction": "horizontal" // horizontal|vertical
  }
}
```

### Scale Image

```json
{
  "command": "scale_image",
  "params": {
    "width": 800,                      // Optional if percentage is provided
    "height": 600,                     // Optional if percentage is provided
    "maintain_aspect_ratio": true,     // Optional, defaults to false
    "percentage": 50                   // Optional, scale as percentage (e.g., 50 for 50%, 200 for 200%)
  }
}
```

### Crop Image

```json
{
  "command": "crop_image",
  "params": {
    "start_x": 50,
    "start_y": 50,
    "width": 400,
    "height": 300
  }
}
```

## Image Recreation

### Recreate Image Request

```json
{
  "command": "recreate_image",
  "params": {
    "image_base64": "base64_encoded_image_data_here",
    "output_filename": "C:\\path\\to\\output.png",  // Optional
    "max_detail_level": 100                         // Optional, 1-200
  }
}
```

### Recreate Image Response

```json
{
  "status": "success",
  "error": null
}
```

## Error Response

All operations may return an error response:

```json
{
  "status": "error",
  "error": "Error message describing what went wrong"
}
```

## Window Management

### Activate Window

```json
{
  "command": "activate_window"
}
```

### Get Canvas Dimensions

```json
{
  "command": "get_canvas_dimensions"
}
```

Response:

```json
{
  "status": "success",
  "width": 800,
  "height": 600
}
```

## Error Handling

All responses include a `status` field indicating success or failure. In case of failure, an `error` field provides details:

```json
{
  "status": "error",
  "error": {
    "code": 1001,
    "message": "Paint window not found"
  }
}
```

### Error Codes

| Code | Description |
|------|-------------|
| 1000 | General error |
| 1001 | Paint window not found |
| 1002 | Operation timeout |
| 1003 | Invalid parameters |
| 1004 | Invalid color format |
| 1005 | Invalid tool |
| 1006 | Invalid shape |
| 1007 | Window activation failed |
| 1008 | Operation not supported in Windows 11 Paint |
| 1009 | File not found |
| 1010 | Permission denied accessing file |
| 1011 | Invalid image format |
| 1012 | Text input failed |
| 1013 | Font selection failed |
| 1014 | Image transformation failed |
| 1015 | Canvas creation failed |

## Protocol Extensions

The MCP protocol may be extended with new commands as Windows 11 Paint evolves. Clients should gracefully handle unknown commands and parameters.

## Windows 11 Paint-Specific Considerations

1. All coordinates are relative to the canvas (0,0 at top-left)
2. Color values must be in the format `#RRGGBB`
3. Shape operations honor the current fill settings
4. Operations that require dialog interactions (save/open) may be limited
5. Image transformations work on the entire canvas/image or current selection
6. Font availability depends on what's installed on the system

## Versioning

This protocol specification is versioned:

```json
{
  "command": "get_version"
}
```

Response:

```json
{
  "status": "success",
  "protocol_version": "1.1",
  "server_version": "1.1.0",
  "paint_version": "windows11"
}
``` 