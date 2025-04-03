# Implementation Status for MCP Server for Windows 11 Paint

This document tracks the implementation progress based on the specifications in the `specs/` directory.

## Legend

- `[ ]` Not Started
- `[~]` In Progress
- `[x]` Completed

## Core Protocol & Communication (STDIO JSON-RPC)

- `[ ]` Establish STDIO communication channel
- `[ ]` Implement JSON-RPC 2.0 message parsing (Requests)
- `[ ]` Implement JSON-RPC 2.0 message serialization (Responses/Notifications)
- `[ ]` Utilize `rust-mcp-sdk` for core communication handling

## Connection Management

- `[x]` Implement `connect` command
    - `[x]` Handle `client_id` and `client_name`
    - `[x]` Return `paint_version`, initial `canvas_width`, `canvas_height`
- `[x]` Implement `disconnect` command

## Windows 11 Paint Integration

- `[x]` Paint Application Detection
    - `[x]` Enumerate windows (`EnumWindows`) for "MSPaintApp" class or "Paint" title
    - `[x]` Launch Paint (`CreateProcess` with "mspaint.exe") if not running
- `[x]` Window Management
    - `[x]` Implement `activate_window` command
        - `[x]` Bring window to foreground (enhanced activation)
        - `[x]` Ensure window is not minimized (`ShowWindow`)
        - `[x]` Handle activation failures and retries
    - `[x]` Implement `get_canvas_dimensions` command
        - `[x]` Get window rectangle (`GetWindowRect`)
        - `[x]` Calculate canvas area based on Win11 UI layout
- `[x]` Mouse Event Simulation (`SendInput`)
    - `[x]` Implement mouse movement
    - `[x]` Implement mouse clicks (left button)
    - `[x]` Implement coordinate translation (client to screen, `ClientToScreen`)
    - `[x]` Implement normalized coordinates for High-DPI support
- `[x]` Keyboard Event Simulation (`SendInput`)
    - `[x]` Implement basic key presses
    - `[x]` Implement modifier keys (Ctrl, Alt, Shift)
    - `[x]` Implement special key codes (Enter, Tab, etc.)
    - `[x]` Handle key scan codes (`KEYEVENTF_SCANCODE`)
- `[~]` UI Element Interaction (Win11 Modern UI)
    - `[~]` Locate toolbar elements
    - `[ ]` Locate property panels
    - `[ ]` Interact with buttons, sliders, dropdowns
- `[ ]` Dialog Interaction
    - `[ ]` Identify dialog windows (class/title)
    - `[ ]` Implement navigation (Tab, Space, Enter)
    - `[ ]` Handle Font selection dialog
    - `[ ]` Handle Resize/Scale dialog
    - `[ ]` Handle Save/Open dialogs (if feasible)
    - `[ ]` Handle New Canvas dialog
- `[ ]` Menu Interaction
    - `[ ]` Access main menu items (Image, Rotate, Flip)
    - `[ ]` Navigate submenus

## Drawing Operations

- `[~]` Tool Selection (`select_tool` command)
    - `[x]` Pencil
    - `[x]` Brush
    - `[x]` Fill
    - `[x]` Text
    - `[x]` Eraser
    - `[x]` Select
    - `[x]` Shape (with `shape_type` parameter)
        - `[ ]` Rectangle
        - `[ ]` Ellipse
        - `[ ]` Line
        - `[ ]` Arrow
        - `[ ]` Triangle
        - `[ ]` Pentagon
        - `[ ]` Hexagon
- `[~]` Color Selection (`set_color` command)
    - `[x]` Handle `#RRGGBB` format
    - `[~]` Interact with Win11 color panel/picker
- `[~]` Thickness Selection (`set_thickness` command)
    - `[x]` Handle levels 1-5
    - `[~]` Interact with Win11 thickness controls
- `[~]` Brush Size Configuration (`set_brush_size` command)
    - `[x]` Handle pixel size (1-30px)
    - `[~]` Interact with Win11 size slider/presets
    - `[ ]` Map requested size to available options
- `[~]` Fill Type Selection (`set_fill` command for shapes)
    - `[x]` Handle `none|solid|outline`
    - `[~]` Interact with Win11 shape fill/outline options
- `[x]` Draw Pixel (`draw_pixel` command)
    - `[x]` Select pencil tool (1px)
    - `[~]` Set color (optional)
    - `[x]` Perform single click at coordinates
    - `[ ]` Handle zoom for precision (optional)
- `[x]` Draw Line (`draw_line` command)
    - `[x]` Simulate mouse drag
    - `[~]` Set color (optional)
    - `[~]` Set thickness (optional)
- `[~]` Draw Shape (`draw_shape` command)
    - `[x]` Select shape tool and type
    - `[~]` Set color (optional)
    - `[~]` Set thickness (optional)
    - `[~]` Set fill type (optional)
    - `[x]` Simulate mouse drag from start to end
- `[x]` Draw Polyline (`draw_polyline` command)
    - `[x]` Select pencil or brush tool
    - `[~]` Set color (optional)
    - `[~]` Set thickness (optional)
    - `[x]` Simulate sequence of mouse drags between points

## Text Operations

- `[~]` Add Text (`add_text` command)
    - `[x]` Select text tool
    - `[x]` Click at position
    - `[x]` Simulate typing text content
    - `[x]` Handle finalization (click elsewhere / Enter)
    - `[~]` Set color (optional)
    - `[~]` Enhanced version parameters:
        - `[~]` Set font name (optional)
        - `[~]` Set font size (optional)
        - `[~]` Set font style (`regular|bold|italic|bold_italic`) (optional)
        - `[ ]` Interact with font selection dialog/panel

## Selection Operations

- `[x]` Select Region (`select_region` command)
    - `[x]` Select selection tool
    - `[x]` Simulate mouse drag for rectangle
- `[x]` Copy Selection (`copy_selection` command)
    - `[x]` Simulate Ctrl+C or menu command
- `[x]` Paste (`paste` command)
    - `[x]` Simulate Ctrl+V or menu command
    - `[x]` Position pasted content (if possible via click at `x`, `y`)

## Canvas Management

- `[x]` Clear Canvas (`clear_canvas` command)
    - `[x]` Select All (Ctrl+A)
    - `[x]` Press Delete key
- `[~]` Create New Canvas (`create_canvas` command)
    - `[x]` Trigger New command (Ctrl+N / menu)
    - `[~]` Interact with New Canvas dialog
    - `[~]` Set width and height
    - `[~]` Handle background color fill (optional)
- `[ ]` Save Canvas (`save` command)
    - `[ ]` Trigger Save command (Ctrl+S / menu)
    - `[ ]` Interact with Save dialog
    - `[ ]` Enter file path
    - `[ ]` Select format (`png|jpeg|bmp`)
    - `[ ]` Confirm save
- `[ ]` Fetch Image (`fetch_image` command)
    - `[ ]` Verify file existence
    - `[ ]` Read file contents securely
    - `[ ]` Validate image format (e.g., PNG)
    - `[ ]` Base64 encode image data
    - `[ ]` Return response with data, format, dimensions

## Image Transformations (New)

- `[ ]` Rotate Image (`rotate_image` command)
    - `[ ]` Select All (Ctrl+A) (if needed)
    - `[ ]` Trigger Rotate command via menu
    - `[ ]` Select direction (90°/180°/270°, clockwise/counter-clockwise)
- `[ ]` Flip Image (`flip_image` command)
    - `[ ]` Select All (Ctrl+A) (if needed)
    - `[ ]` Trigger Flip command via menu
    - `[ ]` Select direction (horizontal/vertical)
- `[ ]` Scale Image (`scale_image` command)
    - `[ ]` Trigger Resize command via menu/shortcut
    - `[ ]` Interact with Resize dialog
    - `[ ]` Set dimensions or percentage
    - `[ ]` Handle "Maintain aspect ratio" checkbox
    - `[ ]` Confirm resize
- `[ ]` Crop Image (`crop_image` command)
    - `[ ]` Requires prior selection (`select_region`)
    - `[ ]` Trigger Crop command via menu

## Image Recreation

- `[ ]` Implement `recreate_image` command
    - `[ ]` Decode base64 image data
    - `[ ]` Create new canvas (or clear existing)
    - `[ ]` Iterate through pixels (or segments) of source image
    - `[ ]` Use `draw_pixel` or optimized drawing method for each pixel/segment
    - `[ ]` Handle `max_detail_level` parameter (e.g., sampling)
    - `[ ]` Save result to `output_filename` (optional)

## Error Handling

- `[ ]` Implement standard error response format (`status: "error"`, `error` message/code)
- `[ ]` Define and use specific error codes (1000-1015+)
- `[ ]` Implement timeouts for UI operations
- `[ ]` Add logging for debugging

## Versioning

- `[x]` Implement `get_version` command
    - `[x]` Return `protocol_version`, `server_version`, `paint_version`

## Client Library (`PaintMcpClient` Rust Example)

*(This section tracks the conceptual mapping to the client library, not its implementation itself unless it's part of this specific project)*

- `[ ]` Map `connect` to `PaintMcpClient::new()` and `client.connect()`
- `[ ]` Map window commands to `activate_window`, `get_canvas_dimensions`
- `[ ]` Map canvas commands to `clear_canvas`, `create_canvas`
- `[ ]` Map tool selection to `select_tool` with `DrawingTool` enum
- `[ ]` Map color/style commands to `set_color`, `set_thickness`, `set_brush_size`, `set_shape_fill`
- `[ ]` Map drawing commands to `draw_line`, `draw_polyline`, `draw_pixel`, `draw_shape`
- `[ ]` Map text commands to `add_text`, `add_text_with_options`
- `[ ]` Map selection commands to `select_region`, `copy_selection`, `paste`
- `[ ]` Map transformation commands to `rotate_image`, `flip_image`, `scale_image`, `crop_image`
- `[ ]` Map file operations to `save_canvas`, `fetch_image`, `fetch_image_with_metadata`
- `[ ]` Map image recreation to `recreate_image`
- `[ ]` Map errors to `PaintMcpError` enum