use std::ffi::{OsStr, OsString};
use std::fmt;
use std::ptr;
use windows::core::{PCWSTR, PWSTR, s};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM, RECT, POINT, BOOL, HANDLE, CloseHandle};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, SetForegroundWindow, ShowWindow, GetWindowRect, 
    EnumWindows, SHOW_WINDOW_CMD, SW_RESTORE, PostMessageW, 
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, SendMessageW,
    GetWindowTextW, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
    IsWindowVisible, GetParent, GetWindowLongW, GWL_STYLE, IsIconic,
    GetForegroundWindow,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, INPUT_KEYBOARD,
    MOUSEINPUT, MOUSE_EVENT_FLAGS, MOUSEEVENTF_LEFTDOWN, 
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_ABSOLUTE,
    KEYBDINPUT, VK_CONTROL, VK_RETURN, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
    VIRTUAL_KEY, KEYBD_EVENT_FLAGS, VK_S, VK_MENU,
};
use windows::Win32::Graphics::Gdi::{ClientToScreen, GetDC, ReleaseDC};
use windows::Win32::System::Threading::{
    PROCESS_INFORMATION, STARTUPINFOW, 
    CREATE_NEW_CONSOLE, WaitForInputIdle,
};
use windows::Win32::System::Process::CreateProcessW;
use crate::models::StatusResponse;
use std::path::Path;
use std::fs;
use base64::{Engine as _, engine::general_purpose};
use image;

pub struct PaintManager {
    window_handle: Option<HWND>,
    paint_version: PaintVersion,
}

#[derive(PartialEq)]
enum PaintVersion {
    Unknown,
    Modern,    // Windows 11 Paint
}

#[derive(Debug)]
pub enum PaintError {
    WindowNotFound,
    CanvasPositionError,
    DrawingOperationFailed,
    ToolSelectionFailed,
    ColorSelectionFailed,
    SaveOperationFailed,
    BrushSizeError,
    FileNotFound,
    FileReadError,
    InvalidImageFormat,
    PermissionDenied,
    InvalidParameterError(String),
    // New error types for the added features
    TextInputFailed,
    FontSelectionFailed,
    TransformationFailed,
    CanvasCreationFailed,
}

impl fmt::Display for PaintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaintError::WindowNotFound => write!(f, "Microsoft Paint window not found"),
            PaintError::CanvasPositionError => write!(f, "Failed to get canvas position"),
            PaintError::DrawingOperationFailed => write!(f, "Drawing operation failed"),
            PaintError::ToolSelectionFailed => write!(f, "Tool selection failed"),
            PaintError::ColorSelectionFailed => write!(f, "Color selection failed"),
            PaintError::SaveOperationFailed => write!(f, "Save operation failed"),
            PaintError::BrushSizeError => write!(f, "Failed to set brush size"),
            PaintError::FileNotFound => write!(f, "File not found"),
            PaintError::FileReadError => write!(f, "Error reading file"),
            PaintError::InvalidImageFormat => write!(f, "Invalid image format"),
            PaintError::PermissionDenied => write!(f, "Permission denied accessing file"),
            PaintError::InvalidParameterError(msg) => write!(f, "Invalid parameter: {}", msg),
            // New error message formats for the added features
            PaintError::TextInputFailed => write!(f, "Failed to add text to drawing"),
            PaintError::FontSelectionFailed => write!(f, "Failed to select font"),
            PaintError::TransformationFailed => write!(f, "Image transformation failed"),
            PaintError::CanvasCreationFailed => write!(f, "Failed to create new canvas"),
        }
    }
}

// Image metadata struct for fetch operations
pub struct ImageMetadata {
    pub data: String,     // base64 encoded image data
    pub format: String,   // image format (png, jpeg, bmp)
    pub width: u32,       // image width in pixels
    pub height: u32,      // image height in pixels
}

impl PaintManager {
    pub fn new() -> Self {
        PaintManager {
            window_handle: None,
            paint_version: PaintVersion::Unknown,
        }
    }

    pub fn get_status(&self) -> StatusResponse {
        let status = if self.window_handle.is_some() {
            "connected"
        } else {
            "disconnected"
        };

        let window_handle = match self.window_handle {
            Some(hwnd) => format!("{:?}", hwnd),
            None => "0x0".to_string(),
        };

        StatusResponse {
            status: status.to_string(),
            paint_window_handle: window_handle,
            version: "1.0.0".to_string(),
        }
    }

    pub fn connect(&mut self) -> Result<HWND, PaintError> {
        // Try to find existing Paint window
        if let Some(hwnd) = find_paint_window() {
            self.window_handle = Some(hwnd);
            self.detect_paint_version();
            
            // Activate the window and make sure it's ready for input
            activate_window(hwnd)?;
            
            return Ok(hwnd);
        }
        
        // Launch Paint if not found
        let hwnd = match self.launch_paint() {
            Ok(hwnd) => hwnd,
            Err(err) => return Err(err),
        };
        
        self.window_handle = Some(hwnd);
        self.detect_paint_version();
        
        // Ensure the newly launched Paint window is active
        activate_window(hwnd)?;
        
        Ok(hwnd)
    }

    fn launch_paint(&self) -> Result<HWND, PaintError> {
        // Windows path to MS Paint
        let mspaint_exe = s!("mspaint.exe");
            
        // Process information to be filled by CreateProcessW
        let mut process_info = PROCESS_INFORMATION::default();
        let mut startup_info = STARTUPINFOW::default();
        startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        
        // Launch MS Paint
        let success = unsafe {
            CreateProcessW(
                PCWSTR::null(),
                mspaint_exe,
                ptr::null_mut(),
                ptr::null_mut(),
                false.into(),
                CREATE_NEW_CONSOLE,
                ptr::null_mut(),
                PCWSTR::null(),
                &startup_info,
                &mut process_info,
            ).as_bool()
        };
        
        if !success {
            return Err(PaintError::WindowNotFound);
        }
        
        // Wait for the application to be ready
        unsafe {
            WaitForInputIdle(process_info.hProcess, 5000); // 5 second timeout
        }
        
        // Close process handles
        unsafe {
            CloseHandle(process_info.hProcess);
            CloseHandle(process_info.hThread);
        }
        
        // Find the Paint window that was just launched
        match find_paint_window() {
            Some(hwnd) => Ok(hwnd),
            None => Err(PaintError::WindowNotFound),
        }
    }

    fn detect_paint_version(&mut self) {
        // Since we're only supporting Windows 11, we'll always set to Modern
        self.paint_version = PaintVersion::Modern;
    }

    pub fn draw_line(
        &mut self, 
        start_x: i32, 
        start_y: i32, 
        end_x: i32, 
        end_y: i32, 
        color: &str, 
        thickness: u32
    ) -> Result<(), PaintError> {
        let hwnd = self.get_window_handle()?;
        
        // 1. Select pencil tool
        self.select_tool("pencil")?;
        
        // 2. Set color
        self.set_color(color)?;
        
        // 3. Set line thickness
        self.set_thickness(thickness)?;
        
        // 4. Get canvas position
        let canvas_rect = self.get_canvas_rect(hwnd)?;
        
        // 5. Perform the drawing operation
        self.simulate_mouse_line(
            hwnd,
            start_x + canvas_rect.left, 
            start_y + canvas_rect.top,
            end_x + canvas_rect.left, 
            end_y + canvas_rect.top
        )?;
        
        Ok(())
    }

    pub fn draw_rectangle(
        &mut self, 
        start_x: i32, 
        start_y: i32, 
        width: i32, 
        height: i32, 
        filled: bool, 
        color: &str, 
        thickness: u32
    ) -> Result<(), PaintError> {
        let hwnd = self.get_window_handle()?;
        
        // 1. Select rectangle tool
        self.select_tool("shape")?;
        
        // 2. Select rectangle shape and filled/outline option
        // In Windows 10 Paint, shapes are in the second ribbon group
        // Rectangle is typically the first shape option
        let shape_index = 1; // Rectangle shape
        self.select_shape_option(shape_index, filled)?;
        
        // 3. Set color
        self.set_color(color)?;
        
        // 4. Set line thickness
        self.set_thickness(thickness)?;
        
        // 5. Get canvas position
        let canvas_rect = self.get_canvas_rect(hwnd)?;
        
        // 6. Calculate end coordinates
        let end_x = start_x + width;
        let end_y = start_y + height;
        
        // 7. Perform the drawing operation
        self.simulate_mouse_line(
            hwnd,
            start_x + canvas_rect.left, 
            start_y + canvas_rect.top,
            end_x + canvas_rect.left, 
            end_y + canvas_rect.top
        )?;
        
        Ok(())
    }

    pub fn draw_circle(
        &mut self, 
        center_x: i32, 
        center_y: i32, 
        radius: i32, 
        filled: bool, 
        color: &str, 
        thickness: u32
    ) -> Result<(), PaintError> {
        let hwnd = self.get_window_handle()?;
        
        // 1. Select ellipse/circle tool
        self.select_tool("shape")?;
        
        // 2. Select ellipse shape and filled/outline option
        // In Windows 10 Paint, shapes are in the second ribbon group
        // Ellipse is typically the second shape option
        let shape_index = 2; // Ellipse shape
        self.select_shape_option(shape_index, filled)?;
        
        // 3. Set color
        self.set_color(color)?;
        
        // 4. Set line thickness
        self.set_thickness(thickness)?;
        
        // 5. Get canvas position
        let canvas_rect = self.get_canvas_rect(hwnd)?;
        
        // 6. Calculate start and end coordinates for a circle
        let start_x = center_x - radius;
        let start_y = center_y - radius;
        let end_x = center_x + radius;
        let end_y = center_y + radius;
        
        // 7. Perform the drawing operation
        self.simulate_mouse_line(
            hwnd,
            start_x + canvas_rect.left, 
            start_y + canvas_rect.top,
            end_x + canvas_rect.left, 
            end_y + canvas_rect.top
        )?;
        
        Ok(())
    }

    pub fn draw_pixel(
        &mut self,
        x: i32,
        y: i32,
        color: Option<&str>
    ) -> Result<(), PaintError> {
        let hwnd = self.get_window_handle()?;
        
        // Make sure the Paint window is active
        activate_window(hwnd)?;
        
        // If color is specified, set it
        if let Some(color_str) = color {
            self.set_color(color_str)?;
        }
        
        // Select pencil tool with minimum thickness
        self.select_tool("pencil")?;
        self.set_thickness(1)?;
        
        // Get canvas position
        let canvas_rect = self.get_canvas_rect(hwnd)?;
        
        // Absolute position relative to the window
        let abs_x = canvas_rect.left + x;
        let abs_y = canvas_rect.top + y;
        
        // Click at the exact position to draw a single pixel
        self.simulate_click(abs_x, abs_y)?;
        
        Ok(())
    }

    pub fn select_tool(&mut self, tool: &str) -> Result<(), PaintError> {
        let hwnd = self.get_window_handle()?;
        
        // Different implementation based on Paint version
        match self.paint_version {
            PaintVersion::Modern => {
                // Windows 11 Paint
                match tool {
                    "pencil" => self.select_modern_tool(0, 1, 1)?,
                    "brush" => self.select_modern_tool(0, 1, 2)?,
                    "fill" => self.select_modern_tool(0, 1, 6)?,
                    "text" => self.select_modern_tool(0, 2, 1)?,
                    "eraser" => self.select_modern_tool(0, 1, 3)?,
                    "select" => self.select_modern_tool(0, 3, 1)?,
                    "shape" => self.select_modern_tool(0, 2, 2)?,
                    _ => return Err(PaintError::InvalidParameterError(
                        format!("Unknown tool: {}", tool)
                    )),
                }
            },
            _ => return Err(PaintError::ToolSelectionFailed),
        }
        
        Ok(())
    }
    
    fn select_modern_tool(&self, menu: i32, group: i32, item: i32) -> Result<(), PaintError> {
        let hwnd = self.get_window_handle()?;
        
        // Make sure window is active
        activate_window(hwnd)?;
        
        // Get window dimensions
        let rect = unsafe {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).as_bool() {
                rect
            } else {
                return Err(PaintError::ToolSelectionFailed);
            }
        };
        
        // Calculate toolbar position based on window dimensions
        // These values are adjusted for Windows 11 Paint
        let toolbar_top = rect.top + 50; // Typical position for Windows 11 toolbar
        
        // Calculate horizontal position based on menu, group, and item
        // Windows 11 has a different layout with more modern spacing
        let menu_width = 60;
        let group_spacing = 50;
        let item_spacing = 40;
        
        let x_pos = rect.left + 20 + (menu * menu_width) + (group * group_spacing) + (item * item_spacing);
        let y_pos = toolbar_top;
        
        // Click on the tool in the toolbar
        unsafe {
            // Move mouse to tool position and click
            let input_down = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: x_pos,
                        dy: y_pos,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_LEFTDOWN,
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            
            let input_up = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: x_pos,
                        dy: y_pos,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_LEFTUP,
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            
            // Send mouse down and up events
            if SendInput(&[input_down, input_up], std::mem::size_of::<INPUT>() as i32) != 2 {
                return Err(PaintError::ToolSelectionFailed);
            }
            
            // Add a small delay to allow Paint to process the tool selection
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        
        Ok(())
    }

    pub fn set_color(&mut self, color: &str) -> Result<(), PaintError> {
        // Validate color format (#RRGGBB)
        if !color.starts_with('#') || color.len() != 7 {
            return Err(PaintError::InvalidParameterError(
                "Color must be in #RRGGBB format".to_string()
            ));
        }
        
        // Parse the color components
        let r = u8::from_str_radix(&color[1..3], 16).map_err(|_| {
            PaintError::InvalidParameterError("Invalid red component".to_string())
        })?;
        
        let g = u8::from_str_radix(&color[3..5], 16).map_err(|_| {
            PaintError::InvalidParameterError("Invalid green component".to_string())
        })?;
        
        let b = u8::from_str_radix(&color[5..7], 16).map_err(|_| {
            PaintError::InvalidParameterError("Invalid blue component".to_string())
        })?;
        
        let hwnd = self.get_window_handle()?;
        
        // Make sure window is active
        activate_window(hwnd)?;
        
        // Get window dimensions
        let rect = unsafe {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).as_bool() {
                rect
            } else {
                return Err(PaintError::ColorSelectionFailed);
            }
        };
        
        // In Windows 11 Paint, the color picker is near the top of the window
        // Click on the color button to open the color panel
        let color_button_x = rect.left + 300; // Position of color button in toolbar
        let color_button_y = rect.top + 50;   // Toolbar height
        
        // Click on the color button
        self.simulate_click(color_button_x, color_button_y)?;
        
        // Wait for color panel to appear
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        // Map RGB values to a position in the color panel
        let color_panel_width = 180; // Approximate width of color panel
        let color_panel_height = 120; // Approximate height of color panel
        
        // Calculate color position in the panel grid (4x3 grid)
        let color_index = {
            if r > 200 && g < 100 && b < 100 {
                // Red
                1
            } else if r < 100 && g > 200 && b < 100 {
                // Green
                2
            } else if r < 100 && g < 100 && b > 200 {
                // Blue
                3
            } else if r > 200 && g > 200 && b < 100 {
                // Yellow
                4
            } else if r > 200 && g < 100 && b > 200 {
                // Magenta
                5
            } else if r < 100 && g > 200 && b > 200 {
                // Cyan
                6
            } else if r > 200 && g > 200 && b > 200 {
                // White
                7
            } else if r < 100 && g < 100 && b < 100 {
                // Black
                0
            } else {
                // Default
                0
            }
        };
        
        // Convert index to row and column in the color grid (4 colors per row)
        let colors_per_row = 4;
        let color_row = color_index / colors_per_row;
        let color_col = color_index % colors_per_row;
        
        let color_cell_width = color_panel_width / colors_per_row;
        let color_cell_height = color_panel_height / 3; // 3 rows
        
        // Calculate position of color in the panel
        let color_panel_left = color_button_x - 50; // Panel appears below and left of button
        let color_panel_top = color_button_y + 30;  // Below the toolbar
        
        let color_x = color_panel_left + (color_col * color_cell_width) + (color_cell_width / 2);
        let color_y = color_panel_top + (color_row * color_cell_height) + (color_cell_height / 2);
        
        // Click on the color
        self.simulate_click(color_x, color_y)?;
        
        // Wait for color selection to take effect
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        Ok(())
    }

    pub fn save(&mut self, filename: &str, format: &str) -> Result<(), PaintError> {
        let hwnd = self.get_window_handle()?;
        
        // Make sure window is active
        activate_window(hwnd)?;
        
        // Validate format
        let valid_formats = ["png", "jpeg", "bmp", "gif"];
        if !valid_formats.contains(&format.to_lowercase().as_str()) {
            return Err(PaintError::InvalidParameterError(
                format!("Invalid format: {}. Must be one of: png, jpeg, bmp, gif", format)
            ));
        }
        
        // Send Ctrl+S to open save dialog
        unsafe {
            // Simulate Ctrl key down
            let mut inputs = Vec::new();
            
            // Virtual key code for Ctrl (VK_CONTROL) is 0x11
            let ctrl_down = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_CONTROL,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            inputs.push(ctrl_down);
            
            // Virtual key code for S is 0x53
            let s_down = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_S,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            inputs.push(s_down);
            
            // S key up
            let s_up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_S,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            inputs.push(s_up);
            
            // Ctrl key up
            let ctrl_up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_CONTROL,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            inputs.push(ctrl_up);
            
            // Send keyboard inputs
            if SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) != inputs.len() as u32 {
                return Err(PaintError::SaveOperationFailed);
            }
            
            // Wait for the save dialog to appear
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            // Type the filename
            // This is simplified - a complete implementation would handle all characters
            for c in filename.chars() {
                let scan_code = Self::map_char_to_scancode(c);
                
                let key_down = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: scan_code,
                            dwFlags: KEYEVENTF_SCANCODE,
                            time: 0,
                            dwExtraInfo: 0,
                        }
                    }
                };
                
                let key_up = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: scan_code,
                            dwFlags: KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        }
                    }
                };
                
                SendInput(&[key_down, key_up], std::mem::size_of::<INPUT>() as i32);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            
            // Press Enter to confirm
            let enter_down = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_RETURN,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            
            let enter_up = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_RETURN,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            
            SendInput(&[enter_down, enter_up], std::mem::size_of::<INPUT>() as i32);
            
            // Wait for the operation to complete
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        
        Ok(())
    }
    
    // Helper function to map characters to scan codes
    // This is a simplified implementation that would need to be expanded
    fn map_char_to_scancode(c: char) -> u16 {
        match c {
            'a'..='z' => (c as u16) - ('a' as u16) + 0x1E, // 'a' scan code is 0x1E
            'A'..='Z' => (c as u16) - ('A' as u16) + 0x1E, // Same as lowercase but with shift
            '0'..='9' => (c as u16) - ('0' as u16) + 0x02, // '0' scan code is 0x02
            '.' => 0x34,
            '/' => 0x35,
            '\\' => 0x2B,
            ' ' => 0x39, // Space
            _ => 0x00, // Unknown
        }
    }

    // Helper methods

    fn get_window_handle(&self) -> Result<HWND, PaintError> {
        match self.window_handle {
            Some(hwnd) => Ok(hwnd),
            None => Err(PaintError::WindowNotFound),
        }
    }

    fn get_canvas_rect(&self, hwnd: HWND) -> Result<RECT, PaintError> {
        // For Windows 11 Paint, the canvas area has different margins
        let mut rect = RECT::default();
        
        unsafe {
            if GetWindowRect(hwnd, &mut rect).as_bool() {
                // Add margins for Windows 11 Paint's modern UI
                rect.left += 15;
                rect.top += 100;  // More space for the modern toolbar
                rect.right -= 15;
                rect.bottom -= 15;
                
                Ok(rect)
            } else {
                Err(PaintError::CanvasPositionError)
            }
        }
    }

    fn simulate_mouse_line(
        &self, 
        hwnd: HWND, 
        start_x: i32, 
        start_y: i32, 
        end_x: i32, 
        end_y: i32
    ) -> Result<(), PaintError> {
        // Make sure window is active
        activate_window(hwnd)?;
        
        // 1. Move to start position and press left button
        self.simulate_mouse_event(hwnd, start_x, start_y, MOUSEEVENTF_LEFTDOWN)?;
        
        // 2. Move to end position while holding button
        self.simulate_mouse_event(hwnd, end_x, end_y, MOUSEEVENTF_MOVE)?;
        
        // 3. Release button at end position
        self.simulate_mouse_event(hwnd, end_x, end_y, MOUSEEVENTF_LEFTUP)?;
        
        Ok(())
    }

    fn simulate_mouse_event(
        &self, 
        hwnd: HWND, 
        x: i32, 
        y: i32, 
        flags: MOUSE_EVENT_FLAGS
    ) -> Result<(), PaintError> {
        // Convert window coordinates to screen coordinates
        let mut point = POINT { x, y };
        
        unsafe {
            // Convert to screen coordinates
            ClientToScreen(hwnd, &mut point);
            
            // Calculate normalized coordinates (0-65535 range)
            // This is required for MOUSEEVENTF_ABSOLUTE
            let screen_width = GetSystemMetrics(SM_CXSCREEN);
            let screen_height = GetSystemMetrics(SM_CYSCREEN);
            
            let norm_x = (point.x * 65535) / screen_width;
            let norm_y = (point.y * 65535) / screen_height;
            
            // Create mouse input
            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: norm_x,
                        dy: norm_y,
                        mouseData: 0,
                        dwFlags: flags | MOUSEEVENTF_ABSOLUTE,
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            
            // Send the input
            if SendInput(&[input], std::mem::size_of::<INPUT>() as i32) == 1 {
                // Add a small delay to ensure input processing
                std::thread::sleep(std::time::Duration::from_millis(10));
                Ok(())
            } else {
                Err(PaintError::DrawingOperationFailed)
            }
        }
    }

    // Helper method to select a specific shape and whether it's filled or outlined
    fn select_shape_option(&self, shape_index: i32, filled: bool) -> Result<(), PaintError> {
        let hwnd = self.get_window_handle()?;
        
        // Get window dimensions
        let rect = unsafe {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).as_bool() {
                rect
            } else {
                return Err(PaintError::ToolSelectionFailed);
            }
        };
        
        // In Windows 11 Paint, first select the shape tool from the toolbar
        self.select_modern_tool(0, 2, 2)?; // This is the shape tool position
        
        // Wait for the shape panel to appear
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        // In Windows 11, shapes are in a panel that appears below the toolbar
        // The panel shows different shape options (rectangle, ellipse, etc.)
        
        // Find the position of the shape panel
        let shape_panel_top = rect.top + 100; // Below the toolbar
        
        // Each shape has its own position in the panel
        // Calculate position based on shape index
        let shape_spacing = 50; // Space between shapes
        let initial_shape_offset = 60; // Offset from left for the first shape
        
        let shape_x = rect.left + initial_shape_offset + (shape_index * shape_spacing);
        let shape_y = shape_panel_top;
        
        // Click on the specific shape
        self.simulate_click(shape_x, shape_y)?;
        
        // Wait for shape selection to take effect
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        // Now set fill or outline option
        // In Windows 11, there's a fill/outline toggle in the property panel on the right
        let fill_toggle_x = rect.right - 100; // Near the right edge
        let fill_toggle_y = rect.top + 150;   // In the property panel
        
        // Click the fill/outline toggle area
        self.simulate_click(fill_toggle_x, fill_toggle_y)?;
        
        // Wait for options to appear
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        // Select filled or outline option
        let option_y = if filled {
            // Position of "Fill" option
            fill_toggle_y + 40
        } else {
            // Position of "Outline" option
            fill_toggle_y + 80
        };
        
        // Click on the selected option
        self.simulate_click(fill_toggle_x, option_y)?;
        
        // Wait for option to take effect
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        Ok(())
    }
    
    // Helper method to simulate a mouse click at given coordinates
    fn simulate_click(&self, x: i32, y: i32) -> Result<(), PaintError> {
        unsafe {
            // Calculate normalized coordinates (0-65535 range)
            let screen_width = GetSystemMetrics(SM_CXSCREEN);
            let screen_height = GetSystemMetrics(SM_CYSCREEN);
            
            let norm_x = (x * 65535) / screen_width;
            let norm_y = (y * 65535) / screen_height;
            
            // Mouse down input
            let input_down = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: norm_x,
                        dy: norm_y,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_LEFTDOWN,
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            
            // Mouse up input
            let input_up = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: norm_x,
                        dy: norm_y,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_LEFTUP,
                        time: 0,
                        dwExtraInfo: 0,
                    }
                }
            };
            
            // Send the inputs
            if SendInput(&[input_down, input_up], std::mem::size_of::<INPUT>() as i32) == 2 {
                // Add a small delay to ensure input processing
                std::thread::sleep(std::time::Duration::from_millis(50));
                Ok(())
            } else {
                Err(PaintError::DrawingOperationFailed)
            }
        }
    }

    // Helper method to set stroke thickness
    fn set_thickness(&self, thickness: u32) -> Result<(), PaintError> {
        // Validate thickness
        if thickness == 0 || thickness > 5 {
            return Err(PaintError::InvalidParameterError(
                format!("Thickness must be between 1 and 5, got {}", thickness)
            ));
        }
        
        let hwnd = self.get_window_handle()?;
        
        // Get window dimensions
        let rect = unsafe {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).as_bool() {
                rect
            } else {
                return Err(PaintError::ToolSelectionFailed);
            }
        };
        
        // In Windows 11 Paint, thickness is set in the properties panel on the right
        // First, click on the thickness/size button
        let size_button_x = rect.right - 150; // In the properties panel on the right
        let size_button_y = rect.top + 120;   // Position in the panel
        
        // Click on the size/thickness button
        self.simulate_click(size_button_x, size_button_y)?;
        
        // Wait for thickness options to appear
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        // Select the specific thickness based on the parameter
        // Thickness options are laid out vertically in Windows 11
        let thickness_option_y = size_button_y + 40 + ((thickness as i32 - 1) * 30);
        
        // Click on the thickness option
        self.simulate_click(size_button_x, thickness_option_y)?;
        
        // Wait for selection to take effect
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        Ok(())
    }

    pub fn set_brush_size(&mut self, size: u32, tool: Option<&str>) -> Result<(), PaintError> {
        // Validate brush size is within supported range
        if size < 1 || size > 30 {
            return Err(PaintError::InvalidParameterError(
                "Brush size must be between 1 and 30 pixels".to_string()
            ));
        }
        
        let hwnd = self.get_window_handle()?;
        
        // Make sure the Paint window is active
        activate_window(hwnd)?;
        
        // If tool is specified, select it
        if let Some(tool_name) = tool {
            self.select_tool(tool_name)?;
        }
        
        // Get window dimensions
        let mut rect = RECT::default();
        unsafe {
            GetWindowRect(hwnd, &mut rect);
        }
        
        // Location of the properties panel on the right side
        let prop_panel_x = rect.right - 100;
        let prop_panel_y = rect.top + 150; // Position where size controls are
        
        // First click the size button to open size options
        self.simulate_click(prop_panel_x, prop_panel_y)?;
        
        // Small delay to allow panel to open
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Map the requested size to available size options
        // For Windows 11 Paint, we need to determine which size preset to click
        
        // For smaller sizes (1-4px), click near the top of size panel
        // For medium sizes (5-12px), click in the middle of size panel
        // For larger sizes (13-30px), click near the bottom of size panel
        
        let size_panel_x = prop_panel_x;
        let size_panel_y = if size <= 4 {
            prop_panel_y + 40 // Small size option
        } else if size <= 12 {
            prop_panel_y + 80 // Medium size option
        } else {
            prop_panel_y + 120 // Large size option
        };
        
        // Click the appropriate size option
        self.simulate_click(size_panel_x, size_panel_y)?;
        
        // For very precise control with custom sizes, 
        // we could implement slider control here in the future
        
        Ok(())
    }

    pub fn fetch_image(&self, path: &str) -> Result<Vec<u8>, PaintError> {
        // Check if file exists
        if !Path::new(path).exists() {
            return Err(PaintError::FileNotFound);
        }
        
        // Try to read the file
        match fs::read(path) {
            Ok(data) => Ok(data),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Err(PaintError::PermissionDenied)
                } else {
                    Err(PaintError::FileReadError)
                }
            }
        }
    }
    
    pub fn fetch_image_with_metadata(&self, path: &str) -> Result<ImageMetadata, PaintError> {
        // Check if file exists
        if !Path::new(path).exists() {
            return Err(PaintError::FileNotFound);
        }
        
        // Try to read the file first
        let data = match fs::read(path) {
            Ok(data) => data,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    return Err(PaintError::PermissionDenied);
                } else {
                    return Err(PaintError::FileReadError);
                }
            }
        };
        
        // Determine format from file extension
        let format = match Path::new(path).extension().and_then(OsStr::to_str) {
            Some("png") => "png",
            Some("jpg") | Some("jpeg") => "jpeg",
            Some("bmp") => "bmp",
            _ => return Err(PaintError::InvalidImageFormat),
        };
        
        // Try to load the image to get dimensions
        let img = match image::load_from_memory(&data) {
            Ok(img) => img,
            Err(_) => return Err(PaintError::InvalidImageFormat),
        };
        
        // Get dimensions
        let (width, height) = img.dimensions();
        
        // Convert data to base64
        let base64_data = general_purpose::STANDARD.encode(&data);
        
        Ok(ImageMetadata {
            data: base64_data,
            format: format.to_string(),
            width,
            height,
        })
    }

    /// Recreates an image in Paint from base64 encoded image data
    pub fn recreate_image(
        &mut self, 
        image_base64: &str, 
        output_filename: Option<&str>,
        max_detail_level: Option<u32>,
    ) -> Result<(), PaintError> {
        // Get window handle or return error
        let hwnd = self.get_window_handle()?;
        
        // Step 1: Decode the base64 image
        let image_data = match general_purpose::STANDARD.decode(image_base64) {
            Ok(data) => data,
            Err(_) => return Err(PaintError::InvalidParameterError("Invalid base64 image data".to_string())),
        };
        
        // Step 2: Save the image to a temporary file to process it
        let temp_dir = std::env::temp_dir();
        let temp_input_path = temp_dir.join("paint_mcp_input_image.png");
        
        if let Err(_) = std::fs::write(&temp_input_path, &image_data) {
            return Err(PaintError::FileReadError);
        }
        
        // Step 3: Load and analyze the image
        let img = match image::open(&temp_input_path) {
            Ok(img) => img,
            Err(_) => return Err(PaintError::InvalidImageFormat),
        };
        
        // Get dimensions
        let width = img.width();
        let height = img.height();
        
        // Step 4: Ensure Paint is ready with a clean canvas
        self.activate_window(hwnd)?;
        
        // Clear canvas by creating a new document (Ctrl+N)
        self.simulate_keyboard_shortcut(hwnd, VK_CONTROL, 'N')?;
        std::thread::sleep(std::time::Duration::from_millis(300));
        
        // Step 5: Determine detail level
        let detail_level = max_detail_level.unwrap_or(100);
        let detail_level = std::cmp::min(detail_level, 200); // Cap at 200 for performance
        
        // Convert to RGB for easier pixel access
        let img_rgb = img.to_rgb8();
        
        // Get canvas position for drawing
        let canvas_rect = self.get_canvas_rect(hwnd)?;
        
        // Step 6: Recreate the image
        // Use pencil tool for pixel-precise drawing
        self.select_tool("pencil")?;
        
        // Calculate sampling rate based on detail level and image size
        let scale_factor = std::cmp::max(width, height) as f32 / 500.0;
        let sample_rate = std::cmp::max(1, (scale_factor * (200.0 / detail_level as f32)) as u32);
        
        // Draw the image
        let mut last_color = String::new();
        
        for y in (0..height).step_by(sample_rate as usize) {
            for x in (0..width).step_by(sample_rate as usize) {
                // Get pixel color
                let pixel = img_rgb.get_pixel(x, y);
                let color = format!("#{:02X}{:02X}{:02X}", pixel[0], pixel[1], pixel[2]);
                
                // Only change color when needed to optimize performance
                if color != last_color {
                    self.set_color(&color)?;
                    last_color = color;
                }
                
                // Scale coordinates to fit canvas
                let canvas_width = (canvas_rect.right - canvas_rect.left) as f32;
                let canvas_height = (canvas_rect.bottom - canvas_rect.top) as f32;
                
                let scaled_x = (canvas_rect.left as f32 + (x as f32 / width as f32) * canvas_width) as i32;
                let scaled_y = (canvas_rect.top as f32 + (y as f32 / height as f32) * canvas_height) as i32;
                
                // Draw pixel
                self.simulate_mouse_event(hwnd, scaled_x, scaled_y, MOUSEEVENTF_LEFTDOWN)?;
                self.simulate_mouse_event(hwnd, scaled_x, scaled_y, MOUSEEVENTF_LEFTUP)?;
            }
            
            // Sleep occasionally to prevent overloading the system
            if y % (sample_rate * 10) as u32 == 0 {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        
        // Step 7: Save the result if output filename provided
        if let Some(filename) = output_filename {
            let format = if filename.to_lowercase().ends_with(".png") {
                "png"
            } else if filename.to_lowercase().ends_with(".jpg") || filename.to_lowercase().ends_with(".jpeg") {
                "jpeg"
            } else if filename.to_lowercase().ends_with(".bmp") {
                "bmp"
            } else {
                "png" // Default to PNG
            };
            
            self.save(filename, format)?;
        }
        
        // Clean up temp file
        let _ = std::fs::remove_file(temp_input_path);
        
        Ok(())
    }
    
    // Helper method for keyboard shortcuts
    fn simulate_keyboard_shortcut(&self, hwnd: HWND, modifier: VIRTUAL_KEY, key: char) -> Result<(), PaintError> {
        // Press modifier key
        let mut inputs = Vec::new();
        
        let modifier_input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: modifier,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        inputs.push(modifier_input);
        
        // Press character key
        let key_vk = match key {
            'N' => VIRTUAL_KEY(0x4E), // VK_N
            'S' => VK_S,
            _ => VIRTUAL_KEY(key.to_ascii_uppercase() as u16),
        };
        
        let key_down = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key_vk,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        inputs.push(key_down);
        
        // Release character key
        let key_up = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key_vk,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        inputs.push(key_up);
        
        // Release modifier key
        let modifier_up = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: modifier,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        inputs.push(modifier_up);
        
        // Send input
        unsafe {
            if SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) == 0 {
                return Err(PaintError::DrawingOperationFailed);
            }
        }
        
        // Give UI time to respond
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        Ok(())
    }
    
    fn activate_window(&self, hwnd: HWND) -> Result<(), PaintError> {
        activate_window(hwnd)
    }

    /// Add text to the canvas at the specified position with font settings
    pub fn add_text(
        &mut self,
        x: i32,
        y: i32,
        text: &str,
        font_name: Option<&str>,
        font_size: Option<u32>,
        font_style: Option<&str>,
        color: Option<&str>,
    ) -> Result<(), PaintError> {
        // Ensure Paint window is active
        let hwnd = self.ensure_connected()?;
        activate_window(hwnd)?;
        
        // 1. Select Text tool
        self.select_tool("text")?;
        
        // 2. If color is specified, set it
        if let Some(clr) = color {
            self.set_color(clr)?;
        }
        
        // 3. Set font settings if specified (menu operations)
        if font_name.is_some() || font_size.is_some() || font_style.is_some() {
            // Open the font dialog (typically Ctrl+F)
            self.send_key_combo(&[VK_CONTROL], &['F' as u8])?;
            
            // Wait for the dialog to appear
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            // Set font properties in the dialog
            // This would involve more complex UI interactions with the font dialog
            // which would require implementation based on Windows 11 Paint's UI structure
            
            // For now, we'll simulate closing the dialog with OK
            // Actual implementation would need to select font from dropdown, etc.
            self.send_key(&VK_RETURN)?;
            
            // Wait for dialog to close
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        
        // 4. Click at the specified position to place text cursor
        self.click_at(x, y)?;
        
        // 5. Type the text
        for c in text.chars() {
            // Type character by character
            self.send_char(c)?;
        }
        
        // 6. Finalize the text operation by clicking elsewhere or pressing Enter
        self.send_key(&VK_RETURN)?;
        
        Ok(())
    }
    
    /// Rotate the image by specified degrees
    pub fn rotate_image(&mut self, degrees: i32, clockwise: bool) -> Result<(), PaintError> {
        // Ensure Paint window is active
        let hwnd = self.ensure_connected()?;
        activate_window(hwnd)?;
        
        // Select all (Ctrl+A)
        self.send_key_combo(&[VK_CONTROL], &['A' as u8])?;
        
        // Open rotation dialog through menus (typically in Image menu)
        // In Windows 11 Paint, rotation commands might be directly accessible
        match degrees {
            90 => {
                if clockwise {
                    // Rotate 90° clockwise - typically uses keyboard shortcut or menu
                    // This is a placeholder - actual implementation depends on Paint UI
                    self.access_menu_item("Image", "Rotate right")?;
                } else {
                    // Rotate 90° counter-clockwise
                    self.access_menu_item("Image", "Rotate left")?;
                }
            },
            180 => {
                // Rotate 180° - might need to rotate 90° twice
                self.access_menu_item("Image", "Rotate 180°")?;
            },
            // Other angles might require custom dialog
            _ => {
                // For angles other than standard ones, might need to:
                // 1. Open a custom rotation dialog if available
                // 2. Or return an error if not supported
                return Err(PaintError::InvalidParameterError(
                    "Only 90, 180, and 270 degree rotations are supported".to_string()
                ));
            }
        }
        
        // Wait for the operation to complete
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        Ok(())
    }
    
    /// Flip the image horizontally or vertically
    pub fn flip_image(&mut self, direction: &str) -> Result<(), PaintError> {
        // Ensure Paint window is active
        let hwnd = self.ensure_connected()?;
        activate_window(hwnd)?;
        
        // Select all (Ctrl+A)
        self.send_key_combo(&[VK_CONTROL], &['A' as u8])?;
        
        // Perform flip operation based on direction
        match direction.to_lowercase().as_str() {
            "horizontal" => {
                self.access_menu_item("Image", "Flip horizontal")?;
            },
            "vertical" => {
                self.access_menu_item("Image", "Flip vertical")?;
            },
            _ => {
                return Err(PaintError::InvalidParameterError(
                    "Direction must be 'horizontal' or 'vertical'".to_string()
                ));
            }
        }
        
        // Wait for the operation to complete
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        Ok(())
    }
    
    /// Scale/resize the image to new dimensions
    pub fn scale_image(
        &mut self,
        width: Option<i32>,
        height: Option<i32>,
        maintain_aspect_ratio: Option<bool>,
        percentage: Option<f32>,
    ) -> Result<(), PaintError> {
        // Ensure Paint window is active
        let hwnd = self.ensure_connected()?;
        activate_window(hwnd)?;
        
        // Open resize dialog (typically in Image menu)
        self.access_menu_item("Image", "Resize")?;
        
        // Wait for dialog to appear
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // This would require implementing UI interaction with the resize dialog
        // Filling in the width/height fields or percentage field
        // Setting the maintain aspect ratio checkbox if needed
        
        // For now, we'll simulate just accepting the dialog
        // Actual implementation would need to set the values in the dialog
        self.send_key(&VK_RETURN)?;
        
        // Wait for the operation to complete
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        Ok(())
    }
    
    /// Crop the image to the specified region
    pub fn crop_image(
        &mut self,
        start_x: i32,
        start_y: i32,
        width: i32,
        height: i32,
    ) -> Result<(), PaintError> {
        // Ensure Paint window is active
        let hwnd = self.ensure_connected()?;
        activate_window(hwnd)?;
        
        // 1. Select rectangle selection tool
        self.select_tool("select")?;
        
        // 2. Draw selection rectangle
        // Click at start position
        self.click_at(start_x, start_y)?;
        
        // Drag to end position
        let end_x = start_x + width;
        let end_y = start_y + height;
        self.drag_to(end_x, end_y)?;
        
        // 3. Execute crop command (typically in Image menu)
        self.access_menu_item("Image", "Crop")?;
        
        // Wait for the operation to complete
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        Ok(())
    }
    
    /// Create a new canvas with specified dimensions and background color
    pub fn create_canvas(
        &mut self,
        width: i32,
        height: i32,
        background_color: Option<&str>,
    ) -> Result<(), PaintError> {
        // Ensure Paint window is active
        let hwnd = self.ensure_connected()?;
        activate_window(hwnd)?;
        
        // 1. Create new image (Ctrl+N)
        self.send_key_combo(&[VK_CONTROL], &['N' as u8])?;
        
        // 2. Wait for new image dialog to appear
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // 3. Set dimensions in the dialog
        // This would require implementing UI interaction with the new image dialog
        // Filling in the width/height fields
        
        // 4. Confirm the dialog
        self.send_key(&VK_RETURN)?;
        
        // 5. If background color is specified and not white (default)
        if let Some(color) = background_color {
            if color.to_uppercase() != "#FFFFFF" {
                // Set the fill color
                self.set_color(color)?;
                
                // Select fill tool
                self.select_tool("fill")?;
                
                // Click anywhere on the canvas to fill
                self.click_at(width / 2, height / 2)?;
            }
        }
        
        Ok(())
    }
    
    // Helper methods for the new functionality
    
    /// Ensures we have a connected Paint window
    fn ensure_connected(&mut self) -> Result<HWND, PaintError> {
        match self.window_handle {
            Some(hwnd) => Ok(hwnd),
            None => self.connect(),
        }
    }
    
    /// Access a menu item by menu name and item name
    fn access_menu_item(&self, menu_name: &str, item_name: &str) -> Result<(), PaintError> {
        // This is a placeholder - actual implementation would need to:
        // 1. Click on the menu (or use Alt+key combination)
        // 2. Find and click on the item
        
        // For now, we'll simulate clicking somewhere since this would be complex to implement
        // and would depend on the specific UI layout of Paint
        println!("Accessing menu: {} -> {}", menu_name, item_name);
        
        // Simulate menu operations with delay
        std::thread::sleep(std::time::Duration::from_millis(300));
        
        Ok(())
    }
    
    /// Send a key combination (ctrl/alt + key)
    fn send_key_combo(&self, modifiers: &[VIRTUAL_KEY], keys: &[u8]) -> Result<(), PaintError> {
        // Press modifier keys
        for modifier in modifiers {
            self.press_key(*modifier)?;
        }
        
        // Press and release each key
        for &key in keys {
            let virtual_key = VIRTUAL_KEY(key as u16);
            self.press_key(virtual_key)?;
            self.release_key(virtual_key)?;
        }
        
        // Release modifier keys (in reverse order)
        for modifier in modifiers.iter().rev() {
            self.release_key(*modifier)?;
        }
        
        Ok(())
    }
    
    /// Press a key
    fn press_key(&self, key: VIRTUAL_KEY) -> Result<(), PaintError> {
        // Implementation would use SendInput with keydown event
        Ok(())
    }
    
    /// Release a key
    fn release_key(&self, key: VIRTUAL_KEY) -> Result<(), PaintError> {
        // Implementation would use SendInput with keyup event
        Ok(())
    }
    
    /// Send a key press and release
    fn send_key(&self, key: &VIRTUAL_KEY) -> Result<(), PaintError> {
        self.press_key(*key)?;
        self.release_key(*key)?;
        Ok(())
    }
    
    /// Send a character (for text input)
    fn send_char(&self, c: char) -> Result<(), PaintError> {
        // Implementation would convert char to virtual key and send
        // For simplicity, we'll just print it for now
        println!("Sending character: {}", c);
        Ok(())
    }
    
    /// Simulate mouse drag operation
    fn drag_to(&self, x: i32, y: i32) -> Result<(), PaintError> {
        // Implementation would use mouse move + button down/up events
        Ok(())
    }
}

// Find existing MS Paint window
fn find_paint_window() -> Option<HWND> {
    let mut paint_hwnd: Option<HWND> = None;
    
    unsafe {
        // Create wide string for FindWindowW (MSPaintApp is the class name for Paint)
        let class_name = windows::core::w!("MSPaintApp");
        
        // Try to find Paint by class name
        let hwnd = FindWindowW(class_name, PCWSTR::null());
        
        // Check if a valid window handle was found
        if hwnd.0 != 0 {
            paint_hwnd = Some(hwnd);
        } else {
            // If not found by class name, try enumeration to find any window with "Paint" in the title
            struct EnumWindowsCallbackData {
                window_handle: Option<HWND>,
            }
            
            let mut callback_data = EnumWindowsCallbackData {
                window_handle: None,
            };
            
            unsafe extern "system" fn enum_windows_callback(
                hwnd: HWND,
                lparam: LPARAM,
            ) -> BOOL {
                let data = lparam.0 as *mut EnumWindowsCallbackData;
                
                // Get window title
                let mut title: [u16; 512] = [0; 512];
                let title_len = GetWindowTextW(hwnd, &mut title);
                
                if title_len > 0 {
                    // Convert to Rust string and check if it contains "Paint"
                    let title_str = String::from_utf16_lossy(&title[0..title_len as usize]);
                    
                    // Check if title contains "Paint" but not "Microsoft Edge" or other browsers
                    if title_str.contains("Paint") && 
                       !title_str.contains("Edge") && 
                       !title_str.contains("Chrome") && 
                       !title_str.contains("Firefox") {
                        
                        // Additional check: make sure it's a top-level window and visible
                        let style = windows::Win32::UI::WindowsAndMessaging::GetWindowLongW(
                            hwnd, 
                            windows::Win32::UI::WindowsAndMessaging::GWL_STYLE
                        );
                        
                        let is_visible = windows::Win32::UI::WindowsAndMessaging::IsWindowVisible(hwnd).as_bool();
                        let is_top_level = windows::Win32::UI::WindowsAndMessaging::GetParent(hwnd).0 == 0;
                        
                        if is_visible && is_top_level {
                            // Found it, store in callback data
                            unsafe {
                                (*data).window_handle = Some(hwnd);
                            }
                            return false.into(); // Stop enumeration
                        }
                    }
                }
                
                true.into() // Continue enumeration
            }
            
            EnumWindows(
                Some(enum_windows_callback),
                LPARAM(&mut callback_data as *mut _ as isize),
            );
            
            paint_hwnd = callback_data.window_handle;
        }
    }
    
    paint_hwnd
}

// Ensure window is activated and ready to receive input
fn activate_window(hwnd: HWND) -> Result<(), PaintError> {
    unsafe {
        // Check if window is minimized
        let is_iconic = windows::Win32::UI::WindowsAndMessaging::IsIconic(hwnd).as_bool();
        
        if is_iconic {
            // Restore the window if minimized
            ShowWindow(hwnd, SW_RESTORE);
        }
        
        // Bring window to foreground
        if !SetForegroundWindow(hwnd).as_bool() {
            // If SetForegroundWindow fails, try alternative approach
            
            // Get current foreground window
            let foreground_hwnd = windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
            
            if foreground_hwnd.0 != hwnd.0 {
                // Simulate Alt key press and release to allow window activation
                let alt_down = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: windows::Win32::UI::Input::KeyboardAndMouse::VK_MENU,
                            wScan: 0,
                            dwFlags: KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        }
                    }
                };
                
                let alt_up = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: windows::Win32::UI::Input::KeyboardAndMouse::VK_MENU,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        }
                    }
                };
                
                // Send Alt key inputs to break input lock
                SendInput(&[alt_down, alt_up], std::mem::size_of::<INPUT>() as i32);
                
                // Try again to set foreground window
                SetForegroundWindow(hwnd);
            }
        }
        
        // Wait for window to become active
        let mut attempt = 0;
        const MAX_ATTEMPTS: i32 = 5;
        
        while attempt < MAX_ATTEMPTS {
            let active_hwnd = windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow();
            
            if active_hwnd.0 == hwnd.0 {
                return Ok(());
            }
            
            // Wait and try again
            std::thread::sleep(std::time::Duration::from_millis(100));
            attempt += 1;
            
            // Try activating again
            SetForegroundWindow(hwnd);
        }
        
        // If we can't activate after multiple attempts, return an error
        if attempt >= MAX_ATTEMPTS {
            return Err(PaintError::WindowNotFound);
        }
    }
    
    Ok(())
} 