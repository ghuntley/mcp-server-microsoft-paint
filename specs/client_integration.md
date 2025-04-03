# Paint MCP Client Integration Specification

## Overview

The Paint MCP client library provides a programmatic interface for controlling Windows 11 Paint. It communicates with a separate MCP server process (also implemented using `rust-mcp-sdk`) via **STDIO (Standard Input/Output)**. The client library handles launching the server process and managing the communication channel. This specification outlines the integration requirements for client applications using this library.

## Client API

### Initialization

To begin interacting with Paint, the client application first creates an instance of the client library and then establishes a connection. The connection process typically involves locating the MCP server executable, launching it as a child process, and establishing communication over its standard input and standard output streams.

```rust
// Create a new Paint MCP client instance
// This might involve specifying the path to the server executable
let client = PaintMcpClient::new(/* potentially server path */)?;

// Connect to Windows 11 Paint
// This launches the server process and sets up STDIO communication.
// It likely performs the initial MCP handshake (initialize request/result).
client.connect()?;
```

**Note:** The client library internally uses the `rust-mcp-sdk` crate ([https://crates.io/crates/rust-mcp-sdk](https://crates.io/crates/rust-mcp-sdk)) to manage the JSON-RPC 2.0 messages exchanged over STDIO with the server process.

### Basic Operations

```rust
// Ensure Paint is visible and in foreground
client.activate_window()?;

// Get canvas dimensions
let (width, height) = client.get_canvas_dimensions()?;

// Clear canvas (creates new document)
client.clear_canvas()?;

// Create a new canvas with specific dimensions
client.create_canvas(1024, 768, Some("#FFFFFF"))?;
```

### Drawing Tools

```rust
// Available tools for Windows 11 Paint
pub enum DrawingTool {
    Pencil,
    Brush,
    Fill,
    Text,
    Eraser,
    Select,
    Shape(ShapeType),
}

// Select a drawing tool
client.select_tool(DrawingTool::Pencil)?;
client.select_tool(DrawingTool::Shape(ShapeType::Rectangle))?;
```

### Shape Drawing

```rust
// Available shape types in Windows 11 Paint
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    Triangle,
    Pentagon,
    Hexagon,
    // Other shapes supported by Windows 11 Paint
}

// Draw shapes with specific dimensions
client.draw_shape(ShapeType::Rectangle, x1, y1, x2, y2)?;

// Shape fill options
pub enum FillType {
    None,
    Solid,
    Outline,
}

// Set shape fill type
client.set_shape_fill(FillType::Solid)?;
```

### Color and Style

```rust
// Set active color with hex color code
client.set_color("#FF5733")?;

// Set brush/line thickness (1-5)
client.set_thickness(3)?;

// Set precise brush size in pixels (1-30px depending on tool)
client.set_brush_size(8)?;

// Set brush size with specific tool
client.set_brush_size_for_tool(12, DrawingTool::Brush)?;
```

### Freeform Drawing

```rust
// Draw a line from one point to another
client.draw_line(x1, y1, x2, y2)?;

// Draw a series of connected lines
client.draw_polyline(vec![(x1, y1), (x2, y2), (x3, y3)])?;

// Draw a single pixel at specific coordinates
client.draw_pixel(x, y, "#FF0000")?;  // With optional color parameter
```

### Text Operations

```rust
// Add simple text at specific position
client.add_text(x, y, "Hello World")?;

// Add text with enhanced font options
client.add_text_with_options(
    x, y, 
    "Hello World",
    Some("Arial"),             // font name
    Some(24),                  // font size
    Some(FontStyle::Bold),     // font style
    Some("#FF0000")            // text color
)?;

// Font style options
pub enum FontStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}
```

### Selection Operations

```rust
// Select a region
client.select_region(x1, y1, x2, y2)?;

// Copy selected region
client.copy_selection()?;

// Paste at position
client.paste(x, y)?;
```

### Image Transformations

```rust
// Rotate the image
client.rotate_image(90, true)?;  // 90 degrees clockwise

// Flip the image
client.flip_image(FlipDirection::Horizontal)?;
// or
client.flip_image(FlipDirection::Vertical)?;

// Resize/scale the image
client.scale_image(
    Some(800),                // new width
    Some(600),                // new height
    Some(true),               // maintain aspect ratio
    None                      // no percentage scaling
)?;

// Alternatively, scale by percentage
client.scale_image(
    None,                     // no fixed width
    None,                     // no fixed height
    None,                     // aspect ratio not applicable
    Some(50.0)                // scale to 50%
)?;

// Crop the image
client.crop_image(x, y, width, height)?;

// Flip direction options
pub enum FlipDirection {
    Horizontal,
    Vertical,
}
```

### File Operations

```rust
// Save canvas to a file
client.save_canvas("C:\\path\\to\\image.png", ImageFormat::Png)?;

// Fetch a saved image as bytes
let image_data: Vec<u8> = client.fetch_image("C:\\path\\to\\image.png")?;

// Fetch a saved image with metadata
let image_result = client.fetch_image_with_metadata("C:\\path\\to\\image.png")?;
println!("Image format: {}", image_result.format);
println!("Image dimensions: {}x{}", image_result.width, image_result.height);
println!("Image data length: {} bytes", image_result.data.len());
```

### Image Recreation

```rust
// Load an image from disk
let img_path = "C:\\path\\to\\source_image.jpg";
let img_data = std::fs::read(img_path)?;
let base64_data = base64::encode(&img_data);

// Recreate the image in Paint
// max_detail_level controls the level of detail - higher values (1-200) mean more detail but slower processing
client.recreate_image(&base64_data, Some("C:\\path\\to\\output.png"), Some(100))?;

// Or recreate without saving the result
client.recreate_image(&base64_data, None, None)?;
```

### Integration with Cursor

```rust
// When integrating with Cursor, you can use this pattern to handle images from the clipboard:

// 1. Get base64 image data from Cursor
let base64_image = cursor.get_clipboard_image()?;

// 2. Recreate the image in Paint
paint_client.connect()?;
paint_client.recreate_image(&base64_image, Some("C:\\path\\to\\recreated.png"), Some(150))?;

// 3. Optionally, apply additional edits to the recreated image
paint_client.select_tool(DrawingTool::Brush)?;
paint_client.set_color("#FF0000")?;
paint_client.draw_line(100, 100, 300, 300)?;
```

## Windows 11 Paint Integration Details

### Paint Version Detection

The client will detect and only support the Windows 11 version of Paint:

```rust
// Windows 11 Paint detection
let version = client.detect_paint_version()?;
assert_eq!(version, PaintVersion::Modern);
```

### Window Activation

Reliable window activation is crucial for consistent operation:

```rust
// Ensure Paint window is active
client.activate_window()?;
```

### Mouse Coordinate Mapping

Client coordinates (0,0 at top-left of canvas) are mapped to screen coordinates:

```rust
// Map client coordinates to screen coordinates
let screen_x = canvas_x + canvas_rect.left;
let screen_y = canvas_y + canvas_rect.top;
```

### Error Handling

All operations return a `Result` type with descriptive errors:

```rust
pub enum PaintMcpError {
    WindowNotFound,
    ActivationFailed,
    OperationTimeout,
    InvalidColor,
    UnsupportedOperation,
    TextInputFailed,
    FontSelectionFailed,
    TransformationFailed,
    CanvasCreationFailed,
    // Other error types
}
```

## Implementation Constraints

1. **Windows 11 Specific**: This implementation only supports Windows 11 Paint.
2. **Coordinate System**: All coordinates are relative to the canvas (0,0 at top-left).
3. **UI Interaction**: Operations use UI automation and may be affected by Paint window focus.
4. **Timing Constraints**: Operations include reasonable timeouts to handle UI responsiveness.
5. **Resolution Independence**: Functions work across different screen resolutions.
6. **Font Availability**: Text operations depend on fonts installed on the system.
7. **Transformation Limitations**: Some transformations may be limited by Paint's capabilities.

## Security Considerations

1. Client requires appropriate permissions to interact with Windows UI.
2. Screen content may be visible during automation operations.
3. Clipboard operations may affect system clipboard content.

## Performance Guidelines

1. Operations should complete within reasonable timeframes (typically < 500ms).
2. Complex operations (like polyline drawing) may take longer.
3. Clients should implement appropriate timeout handling.
4. Window activation adds overhead to operations.
5. Image transformations and canvas operations may take longer on larger images. 