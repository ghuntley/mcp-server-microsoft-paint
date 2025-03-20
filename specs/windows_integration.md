# Windows 11 Integration Specification

## Paint Application Detection

The MCP server will locate Windows 11 Paint (mspaint.exe) using the following methods:

1. Enumerate windows using `EnumWindows` to find windows with class name "MSPaintApp" or title containing "Paint"
2. If no instance is found, launch Paint using `CreateProcess` with "mspaint.exe"

## Window Manipulation

Once the Paint window is located:

1. Bring the window to the foreground using enhanced activation methods
2. Ensure the window is not minimized using `ShowWindow`
3. Get window dimensions with `GetWindowRect`
4. Calculate the canvas area within the window based on Windows 11 Paint's modern UI layout

## Drawing Operations

Drawing operations will be performed by:

1. Selecting the appropriate tool from Windows 11 Paint's modern toolbar using mouse events
2. Setting appropriate color and thickness parameters using the property panels
3. Simulating mouse movements and clicks to perform drawing actions

#### Pixel Drawing

To draw a single pixel:

1. The pencil tool is selected with the minimum thickness (1px)
2. If a specific color is specified, that color is set as the active color
3. A single mouse click is performed at the exact coordinates
4. For better precision, the view is temporarily zoomed in if needed
5. Care is taken to ensure the click doesn't cause any dragging motion

### Mouse Event Simulation

Mouse events will be simulated using:

- `SendInput` with `INPUT` structures for mouse movement
- `MOUSEINPUT` structures for button clicks
- Coordinate translation from client to screen coordinates using `ClientToScreen`
- Normalized coordinates for accurate positioning across different screen resolutions

### Drawing Tools Selection

Windows 11 Paint has a modern toolbar with these tool positions:

| Tool | Position in Windows 11 UI |
|------|---------------------------|
| Pencil | Left side of toolbar |
| Brush | Next to pencil tool |
| Fill | Color fill tool in toolbar |
| Text | Text tool in toolbar |
| Eraser | Eraser tool in toolbar |
| Select | Selection tool in toolbar |
| Shapes | Shape tool with additional dropdown |

### Color Selection

Colors in Windows 11 Paint will be selected by:

1. Clicking the color button in the toolbar to open the color panel
2. Selecting from the color grid in the panel
3. For custom colors, using the expanded color picker when needed

### Thickness Selection

Line thickness in Windows 11 Paint is set via:

1. Accessing the properties panel on the right side
2. Clicking the thickness/size button
3. Selecting from available thickness options

### Brush Size Configuration

Windows 11 Paint provides more granular control over brush sizes:

1. The MCP selects the appropriate tool (pencil or brush)
2. Accesses the properties panel on the right side
3. Locates the size slider control in the panel
4. Sets the precise pixel size by:
   - For predefined sizes (small/medium/large): Clicking the appropriate preset
   - For custom sizes: Using the slider control to set exact pixel values
5. For pixel-precise work (1px), the pencil tool with minimum size is used
6. Different tools support different size ranges:
   - Pencil: 1-8px
   - Brush: 2-30px
   - Specialized brushes: Various ranges

The implementation maps the requested size to the closest available option in Windows 11 Paint's UI.

### Enhanced Text Support

Windows 11 Paint's text tool provides rich formatting options:

1. Text is added by:
   - Selecting the text tool from the toolbar
   - Clicking at the desired position to create a text box
   - Typing the desired text content
   - Clicking elsewhere or pressing Enter to finalize

2. Font settings are configured through:
   - Opening the text format dialog (typically Ctrl+F or via the text properties panel)
   - Setting font name from the dropdown list of available system fonts
   - Setting font size from the size options
   - Selecting style options (regular, bold, italic, bold italic)
   - Selecting text color from the color picker
   - Confirming settings by clicking OK

3. Text box handling requires precise interaction:
   - The MCP ensures proper timing between text tool selection and click
   - For multi-line text, newline characters are converted to appropriate key events
   - The implementation handles text completion to prevent text remaining in edit mode

### Image Transformations

Windows 11 Paint provides various image transformation capabilities:

1. Rotation is implemented by:
   - Selecting the entire canvas (Ctrl+A)
   - Accessing the Image or Rotate menu
   - Selecting the appropriate rotation option (90° clockwise/counterclockwise or 180°)
   - Waiting for the operation to complete

2. Flipping is implemented by:
   - Selecting the entire canvas (Ctrl+A)
   - Accessing the Image or Flip menu
   - Selecting horizontal or vertical flip option
   - Waiting for the operation to complete

3. Scaling/resizing is implemented by:
   - Accessing the Resize dialog through the Image menu or keyboard shortcut
   - Setting the desired dimensions or percentage
   - Setting or clearing the "Maintain aspect ratio" checkbox as needed
   - Confirming the resize operation
   - The implementation handles both pixel-based and percentage-based scaling

4. Cropping is implemented by:
   - Selecting the selection tool
   - Drawing a selection rectangle around the desired area
   - Triggering the crop command from the Image menu
   - Waiting for the operation to complete

### Canvas Management

Managing the canvas in Windows 11 Paint involves:

1. Creating a new canvas by:
   - Sending Ctrl+N or accessing the New command from the menu
   - Setting dimensions in the new canvas dialog
   - Confirming the creation
   - If a background color other than white is specified:
     - Setting the active color to the desired background color
     - Selecting the fill tool
     - Clicking anywhere on the canvas to fill it

2. Clearing the canvas by:
   - Selecting all (Ctrl+A)
   - Pressing Delete or accessing the Clear command
   - This creates a blank white canvas

### File Operations

#### Saving Files

Paint files are saved by:

1. Sending keyboard shortcuts (Ctrl+S) or using automation to trigger the save dialog
2. Entering the file path in the save dialog
3. Selecting the appropriate file format from the dropdown
4. Confirming the save operation

#### Fetching Images

Saved images are retrieved using the following procedure:

1. The MCP server verifies the file exists at the specified path
2. The file is read using secure file I/O operations
3. For PNG files, the image is loaded and validated as a valid PNG
4. The image data is encoded as base64 for transfer via JSON
5. Optional metadata (dimensions, color depth) is extracted from the image header

## Technical Aspects of Windows 11 Integration

### Modern UI Layout

Windows 11 Paint has a completely redesigned interface with:

1. A horizontal toolbar at the top of the window
2. Property panels that appear on the right side
3. Enhanced shape tools with fill/outline options
4. A cleaner canvas area with adjusted margins
5. Text formatting toolbar that appears when text tool is active
6. Image transformation options in the Image menu

### Handling High-DPI Displays

Windows 11 has better support for high-DPI displays. The implementation:

1. Uses normalized mouse coordinates (0-65535 range)
2. Properly accounts for scaling factors
3. Adjusts click positions based on actual screen dimensions

### Improved Window Activation

Windows 11 has stricter window activation policies. Our implementation:

1. Uses advanced activation techniques including Alt key simulation
2. Verifies active window status
3. Includes retry mechanisms with timeout

### Keyboard Simulation for UI Navigation

Some operations require keyboard navigation:

1. Key combinations are sent using `SendInput` with `KEYEVENTF_SCANCODE` flags
2. Modifier keys (Ctrl, Alt, Shift) are properly handled using key down/up pairs
3. Special characters and menu navigation uses appropriate virtual key codes
4. Dialog interaction uses Tab, Space, and Enter for navigation and confirmation

## Dialog Interaction

Dialog handling is critical for many operations:

1. Font selection dialog:
   - The dialog is identified by window class and title
   - Fields are accessed in sequence: font, style, size
   - Keyboard navigation moves between controls
   - Enter key confirms, or OK button is clicked

2. Resize canvas dialog:
   - Width and height fields are populated using keyboard input
   - Maintain aspect ratio checkbox is toggled if needed
   - Percentage vs. pixels mode is selected as appropriate
   - OK button is clicked or Enter key confirms

## Limitations

1. Operations requiring dialog interaction (like open/save) may be less reliable
2. Color matching is approximate as Paint has a predefined palette
3. Drawing complex shapes with precision may be challenging
4. Windows 11 security settings may prevent some automated interactions
5. Font availability depends on what's installed on the system
6. Some transformations may have limitations based on canvas size or available memory
7. Text alignment options may be limited by Paint's capabilities

## Future Enhancements

1. Direct bitmap manipulation for faster and more precise drawing
2. Support for Windows 11 Paint's enhanced features as they become available
3. Adaptation to Paint updates in future Windows 11 releases
4. Improved dialog handling for more robust interaction
5. Enhanced text formatting options including alignment and spacing
6. More advanced image transformations like skew and perspective 