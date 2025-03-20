/// Formats a system prompt for LLM integration
pub fn format_system_prompt() -> String {
    let intro = "You are an AI assistant that helps users control Microsoft Paint on Windows 11 through the Paint MCP (Microsoft Paint Control Protocol) API. \n\
Your job is to understand user requests about drawing and creating images, then translate them into appropriate API calls.\n\n\
The Paint MCP API provides the following operations:\n\n";

    let operations = "1. **GET /status** - Check the current status of the Paint MCP server\n\
2. **POST /connect** - Connect to Windows 11 Paint or launch a new instance\n\
3. **POST /draw/line** - Draw a straight line between two points\n\
4. **POST /draw/rectangle** - Draw a rectangle with specified position and size\n\
5. **POST /draw/circle** - Draw a circle with specified center and radius\n\
6. **POST /draw/pixel** - Draw a single pixel at specific coordinates\n\
7. **POST /tool/select** - Select a specific drawing tool\n\
8. **POST /color/set** - Set the active color\n\
9. **POST /brush/size** - Set the brush or pencil size\n\
10. **POST /save** - Save the drawing to a file\n\
11. **POST /fetch** - Retrieve a saved image file with metadata\n\
12. **POST /recreate-image** - Recreate an uploaded image using Paint\n\
13. **POST /text/add** - Add text to the drawing with font settings\n\
14. **POST /image/rotate** - Rotate the image by specified degrees\n\
15. **POST /image/flip** - Flip the image horizontally or vertically\n\
16. **POST /image/scale** - Resize/scale the image to new dimensions\n\
17. **POST /image/crop** - Crop the image to the specified region\n\
18. **POST /canvas/create** - Create a new canvas with specified dimensions\n";

    let instructions = "\nWhen a user describes what they want to draw, you should:\n\
1. Understand their request and break it down into a series of Paint MCP operations\n\
2. Generate the corresponding API calls with appropriate parameters\n\
3. Explain what each API call does in a clear and concise manner\n\n\
For example, if a user asks \"Draw a red circle with a blue rectangle inside it\", you might suggest:\n\
1. Connect to Paint using POST /connect\n\
2. Set color to red using POST /color/set with {\"color\": \"#FF0000\"}\n\
3. Draw a circle using POST /draw/circle with appropriate parameters\n\
4. Set color to blue using POST /color/set with {\"color\": \"#0000FF\"}\n\
5. Draw a smaller rectangle inside the circle using POST /draw/rectangle\n\n\
If a user wants to recreate an image in Paint, they can upload an image and use the POST /recreate-image endpoint with the base64-encoded image data.\n\n\
You can request more detailed information about any of these operations using the GET /prompt endpoint.\n\n\
Remember: All coordinates are relative to the canvas, with (0,0) at the top-left corner.";

    format!("{}{}{}", intro, operations, instructions)
}

/// Formats a prompt for a specific operation to help LLM understand its usage
pub fn format_operation_example(operation: &str) -> Option<String> {
    let prompt_text = get_prompt(operation)?;
    
    // Create a sample JSON request based on the operation
    let sample_json = match operation {
        "status" => "GET /status",
        "connect" => "POST /connect",
        "draw_line" => "POST /draw/line\nContent-Type: application/json\n\n{\n  \"start_x\": 100,\n  \"start_y\": 100,\n  \"end_x\": 300,\n  \"end_y\": 200,\n  \"color\": \"#0000FF\",\n  \"thickness\": 2\n}",
        "draw_rectangle" => "POST /draw/rectangle\nContent-Type: application/json\n\n{\n  \"start_x\": 50,\n  \"start_y\": 50,\n  \"width\": 200,\n  \"height\": 150,\n  \"filled\": true,\n  \"color\": \"#FF0000\",\n  \"thickness\": 1\n}",
        "draw_circle" => "POST /draw/circle\nContent-Type: application/json\n\n{\n  \"center_x\": 200,\n  \"center_y\": 200,\n  \"radius\": 50,\n  \"filled\": false,\n  \"color\": \"#00FF00\",\n  \"thickness\": 2\n}",
        "draw_pixel" => "POST /draw/pixel\nContent-Type: application/json\n\n{\n  \"x\": 150,\n  \"y\": 150,\n  \"color\": \"#FF00FF\"\n}",
        "select_tool" => "POST /tool/select\nContent-Type: application/json\n\n{\n  \"tool\": \"brush\"\n}",
        "set_color" => "POST /color/set\nContent-Type: application/json\n\n{\n  \"color\": \"#00FFFF\"\n}",
        "set_brush_size" => "POST /brush/size\nContent-Type: application/json\n\n{\n  \"size\": 8,\n  \"tool\": \"brush\"\n}",
        "save" => "POST /save\nContent-Type: application/json\n\n{\n  \"filename\": \"C:\\\\drawings\\\\my_drawing.png\",\n  \"format\": \"png\"\n}",
        "fetch_image" => "POST /fetch\nContent-Type: application/json\n\n{\n  \"path\": \"C:\\\\drawings\\\\my_drawing.png\"\n}",
        "recreate_image" => "POST /recreate-image\nContent-Type: application/json\n\n{\n  \"image_base64\": \"iVBORw0KGgoAAAANSUhEUgAAAAUAAAAFCAYAAACNbyblAAAAHElEQVQI12P4...\",\n  \"output_filename\": \"C:\\\\drawings\\\\recreated.png\",\n  \"max_detail_level\": 100\n}",
        "add_text" => "POST /text/add\nContent-Type: application/json\n\n{\n  \"x\": 100,\n  \"y\": 100,\n  \"text\": \"Hello, World!\",\n  \"font_name\": \"Arial\",\n  \"font_size\": 24,\n  \"font_style\": \"bold\",\n  \"color\": \"#000000\"\n}",
        "rotate_image" => "POST /image/rotate\nContent-Type: application/json\n\n{\n  \"degrees\": 90,\n  \"clockwise\": true\n}",
        "flip_image" => "POST /image/flip\nContent-Type: application/json\n\n{\n  \"direction\": \"horizontal\"\n}",
        "scale_image" => "POST /image/scale\nContent-Type: application/json\n\n{\n  \"width\": 800,\n  \"height\": 600,\n  \"maintain_aspect_ratio\": true,\n  \"percentage\": null\n}",
        "crop_image" => "POST /image/crop\nContent-Type: application/json\n\n{\n  \"start_x\": 50,\n  \"start_y\": 50,\n  \"width\": 400,\n  \"height\": 300\n}",
        "create_canvas" => "POST /canvas/create\nContent-Type: application/json\n\n{\n  \"width\": 1024,\n  \"height\": 768,\n  \"background_color\": \"#FFFFFF\"\n}",
        _ => return None,
    };
    
    let result = format!(
        "Operation: {}\n\n{}\n\nExample API Call:\n\n{}\n",
        operation,
        prompt_text.trim(),
        sample_json
    );
    
    Some(result)
}

/// Returns a detailed prompt for a specific operation
pub fn get_prompt(operation: &str) -> Option<String> {
    let prompt = match operation {
        "status" => "Gets the current status of the Paint MCP server, including whether it's connected to Paint, the window handle, and version information.",
        "connect" => "Connects to Windows 11 Paint or launches a new instance if not already running.",
        "draw_line" => "Draws a straight line between two points with the specified color and thickness.",
        "draw_rectangle" => "Draws a rectangle with the specified position, size, color, and thickness. The filled parameter determines if the rectangle is filled or just outlined.",
        "draw_circle" => "Draws a circle with the specified center point, radius, color, and thickness. The filled parameter determines if the circle is filled or just outlined.",
        "draw_pixel" => "Draws a single pixel at the specified coordinates with the optional color.",
        "select_tool" => "Selects a drawing tool in Paint. Valid tools include 'pencil', 'brush', 'fill', 'text', 'eraser', 'select', and shape names.",
        "set_color" => "Sets the active color for drawing operations. Color should be provided as a hex value (e.g., #FF0000 for red).",
        "set_brush_size" => "Sets the brush or pencil size for drawing. Size ranges from 1-30 pixels depending on the tool.",
        "save" => "Saves the current drawing to a file with the specified filename and format.",
        "fetch_image" => "Retrieves a saved image file with metadata, returning base64-encoded image data, format, and dimensions.",
        "recreate_image" => "Recreates an uploaded image using Paint's drawing capabilities. Requires base64-encoded image data in the request. The max_detail_level parameter (1-200) controls the level of detail (higher values mean more detail but slower processing). An optional output_filename can be provided to save the result.",
        "add_text" => "Add text to the drawing at the specified position. You can configure the font name, size, style, and color. Font styles include 'regular', 'bold', 'italic', and 'bold_italic'.",
        "rotate_image" => "Rotate the entire image by the specified degrees. Currently supports 90, 180, or 270 degree rotations. The 'clockwise' parameter determines the direction of rotation.",
        "flip_image" => "Flip the image either horizontally or vertically. The 'direction' parameter must be either 'horizontal' or 'vertical'.",
        "scale_image" => "Resize the image to new dimensions. You can specify the width and height directly, or use a percentage scale. The 'maintain_aspect_ratio' parameter determines if the aspect ratio should be preserved.",
        "crop_image" => "Crop the image to the specified region defined by starting position, width, and height.",
        "create_canvas" => "Create a new canvas with the specified dimensions. You can optionally set a background color (default is white).",
        _ => return None,
    };
    
    Some(prompt.to_string())
}

/// Gets all available operation prompts
pub fn get_all_prompts() -> Vec<(String, String)> {
    let operations = vec![
        "status", "connect", "draw_line", "draw_rectangle", "draw_circle",
        "draw_pixel", "select_tool", "set_color", "set_brush_size", "save",
        "fetch_image", "recreate_image", 
        // Add new operations
        "add_text", "rotate_image", "flip_image", "scale_image", "crop_image", "create_canvas"
    ];
    
    operations
        .into_iter()
        .filter_map(|op| {
            get_prompt(op).map(|prompt| (op.to_string(), prompt))
        })
        .collect()
} 