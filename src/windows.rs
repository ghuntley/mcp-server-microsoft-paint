// Placeholder for Windows API interaction logic 

use crate::error::{MspMcpError, Result};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, TRUE, FALSE, POINT};
use windows_sys::Win32::System::Threading::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindowTextW, IsWindowVisible, GetWindowRect,
    SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOWMAXIMIZED,
    GetWindowLongW, SetWindowPos, GWL_STYLE, WS_MAXIMIZE, HWND_TOP, SWP_SHOWWINDOW,
    GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
};
// Input-related imports from correct modules
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT_MOUSE, MOUSEEVENTF_MOVE, MOUSEEVENTF_ABSOLUTE, 
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    // Keyboard related imports
    INPUT_KEYBOARD, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, VK_CONTROL, VK_SHIFT, VK_MENU,
    VK_RETURN, VK_TAB, VK_ESCAPE, VK_DELETE, VK_BACK, VK_SPACE, VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN,
};
// INPUT struct and MOUSEINPUT
use windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::MOUSEINPUT;
// ClientToScreen is in Win32::UI::Input::KeyboardAndMouse
use windows_sys::Win32::Graphics::Gdi::ClientToScreen;

use log::{debug, info, warn, error};

const PAINT_CLASS_NAME: &str = "MSPaintApp";
const PAINT_WINDOW_TITLE_SUBSTRING: &str = "Paint";
const MSPAINT_EXECUTABLE: &str = "mspaint.exe";

// Structure to hold data passed to the EnumWindows callback
struct EnumWindowData {
    hwnd: Option<HWND>,
    target_class: Vec<u16>,
    target_title_substring: Vec<u16>,
}

// Callback function for EnumWindows
unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // Check visibility first
    if IsWindowVisible(hwnd) == FALSE {
        // Optional: Log skipped invisible windows at debug level if needed
        // debug!("Skipping invisible window: HWND={}", hwnd);
        return TRUE; // Continue enumeration
    }

    let data = &mut *(lparam as *mut EnumWindowData);
    let mut found = false;

    // Always get class name and title for logging, even if not searching by them
    let mut current_class_name: [u16; 128] = [0; 128];
    let class_name_len = GetClassNameW(hwnd, current_class_name.as_mut_ptr(), current_class_name.len() as i32);
    let class_name_str = if class_name_len > 0 { 
        String::from_utf16_lossy(&current_class_name[..class_name_len as usize])
    } else { 
        "<Unknown Class>".to_string() 
    };

    let mut current_window_title: [u16; 256] = [0; 256];
    let window_title_len = GetWindowTextW(hwnd, current_window_title.as_mut_ptr(), current_window_title.len() as i32);
    let window_title_str = if window_title_len > 0 { 
        String::from_utf16_lossy(&current_window_title[..window_title_len as usize]) 
    } else { 
        "<Unknown Title>".to_string() 
    };

    // IMPORTANT: Skip the MCP server itself
    if window_title_str.contains("mcp-server-microsoft-paint") {
        debug!("Skipping MCP server window: HWND={}, Class='{}', Title='{}'", 
               hwnd, class_name_str, window_title_str);
        return TRUE; // Continue enumeration, ignore this window
    }

    // Log every visible window encountered at debug level
    debug!("EnumWindows Checking: HWND={}, Class='{}', Title='{}'", hwnd, class_name_str, window_title_str);

    // Check class name if specified in search criteria
    if !data.target_class.is_empty() {
        let target_class_str = String::from_utf16_lossy(&data.target_class[..data.target_class.len() - 1]); // Remove null term
        if class_name_str.contains(&target_class_str) {
            info!("Found window matching class '{}': HWND={}, Class='{}', Title='{}'", 
                  target_class_str, hwnd, class_name_str, window_title_str);
            data.hwnd = Some(hwnd);
            found = true;
        }
    }

    // Check window title if specified in search criteria (and not already found by class)
    if !found && !data.target_title_substring.is_empty() {
        let target_title_str = String::from_utf16_lossy(&data.target_title_substring);
        if window_title_str.to_lowercase().contains(&target_title_str.to_lowercase()) {
            info!("Found window matching title '{}': HWND={}, Class='{}', Title='{}'", 
                  target_title_str, hwnd, class_name_str, window_title_str);
            data.hwnd = Some(hwnd);
            found = true;
        }
    }

    if found {
        FALSE // Stop enumeration
    } else {
        TRUE // Continue enumeration
    }
}

/// Log all visible windows - useful for diagnostics
pub fn log_all_visible_windows() -> Result<()> {
    info!("==== LOGGING ALL VISIBLE WINDOWS ====");
    
    unsafe {
        let enum_data = &mut EnumWindowData {
            hwnd: None,
            target_class: Vec::new(),
            target_title_substring: Vec::new(),
        };
        let lparam = enum_data as *mut _ as LPARAM;
        EnumWindows(Some(enum_diagnostic_window_proc), lparam);
    }
    
    info!("==== END WINDOW ENUMERATION ====");
    Ok(())
}

// A special callback for diagnostic logging of ALL windows
unsafe extern "system" fn enum_diagnostic_window_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    // Skip invisible windows
    if IsWindowVisible(hwnd) == FALSE {
        return TRUE; // Continue enumeration
    }

    // Get class name
    let mut class_name: [u16; 128] = [0; 128];
    let class_len = GetClassNameW(hwnd, class_name.as_mut_ptr(), class_name.len() as i32);
    let class_str = if class_len > 0 { 
        String::from_utf16_lossy(&class_name[..class_len as usize])
    } else { 
        "<Unknown Class>".to_string() 
    };

    // Get window title
    let mut title: [u16; 256] = [0; 256];
    let title_len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
    let title_str = if title_len > 0 { 
        String::from_utf16_lossy(&title[..title_len as usize]) 
    } else { 
        "<No Title>".to_string() 
    };

    // Log every visible window
    info!("HWND={}, Class='{}', Title='{}'", hwnd, class_str, title_str);
    
    // Continue enumeration
    TRUE
}

/// Finds the HWND of a visible Windows 11 Paint window.
/// Searches first by class name "MSPaintApp", then by title containing "Paint".
pub fn find_paint_window() -> Result<HWND> {
    info!("Attempting to find Paint window...");
    
    // First, let's log all visible windows to help diagnose the issue
    log_all_visible_windows()?;
    
    // Add more possible class and title names for Paint
    const POSSIBLE_CLASS_NAMES: [&str; 4] = [
        "MSPaintApp",      // Windows 11/10 Paint class
        "Afx:1000000:8",   // Older Paint class name
        "ApplicationFrameWindow", // Modern UWP container class
        "MSPaintDesktop::CMainWindow", // Another possible Paint class
    ];
    
    const POSSIBLE_TITLE_SUBSTRINGS: [&str; 8] = [
        "Paint",         // English
        "paint",         // Case insensitive match
        "Untitled - Paint", // Common title
        "Bez tytułu - Paint", // Other variations
        "Paint 3D",      // Paint 3D (as fallback)
        "Malování",      // Other localized names
        "Drawing",       // Alternative name
        ""               // Empty string to catch all windows (will check class afterwards)
    ];

    // Add a prohibited strings list to explicitly filter out the MCP server
    const PROHIBITED_SUBSTRINGS: [&str; 1] = [
        "mcp-server-microsoft-paint"
    ];
    
    // First try window that might be from manually started Paint
    // Look for any window with "Paint" in title and manually verify it's not the server
    unsafe {
        // Create specialized search data
        let mut search_data = EnumWindowData {
            hwnd: None,
            target_class: Vec::new(),
            target_title_substring: OsStr::new("paint").encode_wide().collect(),
        };
        let lparam = &mut search_data as *mut EnumWindowData as LPARAM;
        EnumWindows(Some(enum_window_proc), lparam);
        
        if let Some(hwnd) = search_data.hwnd {
            let mut window_title: [u16; 256] = [0; 256];
            let title_len = GetWindowTextW(hwnd, window_title.as_mut_ptr(), window_title.len() as i32);
            
            if title_len > 0 {
                let title_str = String::from_utf16_lossy(&window_title[..title_len as usize]);
                
                if !PROHIBITED_SUBSTRINGS.iter().any(|s| title_str.contains(s)) {
                    info!("Found manually started Paint window: HWND={}, Title='{}'", hwnd, title_str);
                    return Ok(hwnd);
                }
            }
        }
    }
    
    // Search by class name
    for class_name in &POSSIBLE_CLASS_NAMES {
        let target_class_u16: Vec<u16> = OsStr::new(class_name).encode_wide().chain(Some(0)).collect();
        
        let mut data = EnumWindowData {
            hwnd: None,
            target_class: target_class_u16,
            target_title_substring: Vec::new(), // Not used for class search
        };
        
        unsafe {
            let lparam = &mut data as *mut EnumWindowData as LPARAM;
            EnumWindows(Some(enum_window_proc), lparam);
            
            if let Some(hwnd) = data.hwnd {
                // Additional check to make sure it's not the MCP server
                let mut window_title: [u16; 256] = [0; 256];
                let title_len = GetWindowTextW(hwnd, window_title.as_mut_ptr(), window_title.len() as i32);
                if title_len > 0 {
                    let title_str = String::from_utf16_lossy(&window_title[..title_len as usize]);
                    
                    // Check if this is a prohibited window (MCP server)
                    let is_prohibited = PROHIBITED_SUBSTRINGS.iter()
                        .any(|prohibited| title_str.contains(prohibited));
                        
                    if is_prohibited {
                        debug!("Skipping prohibited window: {}", title_str);
                        continue; // Skip this window, try next class
                    }
                }
                
                info!("Found Paint window via class name '{}': HWND={}", class_name, hwnd);
                return Ok(hwnd);
            }
        }
    }
    
    // If class name search failed, try by title substring
    for title in &POSSIBLE_TITLE_SUBSTRINGS {
        let target_title_u16: Vec<u16> = OsStr::new(title).encode_wide().collect();
        
        let mut data = EnumWindowData {
            hwnd: None,
            target_class: Vec::new(), // Not used for title search 
            target_title_substring: target_title_u16,
        };
        
        unsafe {
            let lparam = &mut data as *mut EnumWindowData as LPARAM;
            EnumWindows(Some(enum_window_proc), lparam);
            
            if let Some(hwnd) = data.hwnd {
                // Additional check to make sure it's not the MCP server
                let mut window_title: [u16; 256] = [0; 256];
                let title_len = GetWindowTextW(hwnd, window_title.as_mut_ptr(), window_title.len() as i32);
                if title_len > 0 {
                    let title_str = String::from_utf16_lossy(&window_title[..title_len as usize]);
                    
                    // Check if this is a prohibited window (MCP server)
                    let is_prohibited = PROHIBITED_SUBSTRINGS.iter()
                        .any(|prohibited| title_str.contains(prohibited));
                        
                    if is_prohibited {
                        debug!("Skipping prohibited window: {}", title_str);
                        continue; // Skip this window, try next title
                    }
                    
                    // Additional class check when using empty title
                    if title.is_empty() {
                        // Get the class name and check if it's a paint class
                        let mut class_name: [u16; 128] = [0; 128];
                        let class_len = GetClassNameW(hwnd, class_name.as_mut_ptr(), class_name.len() as i32);
                        
                        if class_len > 0 {
                            let class_str = String::from_utf16_lossy(&class_name[..class_len as usize]);
                            
                            let is_paint_class = POSSIBLE_CLASS_NAMES.iter()
                                .any(|paint_class| class_str.contains(paint_class));
                                
                            if !is_paint_class {
                                debug!("Skipping non-Paint class window: {}", class_str);
                                continue;
                            }
                        }
                    }
                }
                
                info!("Found Paint window via title substring '{}': HWND={}", title, hwnd);
                return Ok(hwnd);
            }
        }
    }
    
    // As a last resort, try to find any window with "paint" in its executable path
    unsafe {
        // First log process IDs to help with debugging
        let _ = std::process::Command::new("wmic")
            .args(["process", "where", "name='mspaint.exe'", "get", "processid,commandline", "/format:list"])
            .status();
            
        // This is a lot more complex in reality, but left as a future enhancement
    }
    
    warn!("Paint window not found via EnumWindows.");
    Err(MspMcpError::WindowNotFound)
}

/// Launches the mspaint.exe process.
pub fn launch_paint() -> Result<()> {
    info!("Launching mspaint.exe using ShellExecuteW...");
    
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_NORMAL;
    use std::ptr::null;
    
    let operation: Vec<u16> = OsStr::new("open").encode_wide().chain(Some(0)).collect();
    let file: Vec<u16> = OsStr::new(MSPAINT_EXECUTABLE).encode_wide().chain(Some(0)).collect();
    
    let result = unsafe {
        ShellExecuteW(
            0,                      // hwnd (NULL for no parent)
            operation.as_ptr(),     // lpOperation ("open")
            file.as_ptr(),          // lpFile ("mspaint.exe")
            null(),                 // lpParameters (NULL for no parameters)
            null(),                 // lpDirectory (NULL for current directory)
            SW_NORMAL               // nShowCmd (normal window)
        )
    };
    
    // ShellExecuteW returns an HINSTANCE, which is interpreted differently than a BOOL
    // A value > 32 indicates success
    if result <= 32 {
        let error_code = result;
        error!("Failed to launch mspaint.exe with ShellExecuteW. Error code: {}", error_code);
        return Err(MspMcpError::WindowsApiError(format!("ShellExecuteW failed for mspaint.exe with error code {}", error_code)));
    }

    // Increase initial delay after launch
    info!("Waiting 3 seconds after launch attempt...");
    std::thread::sleep(std::time::Duration::from_millis(3000));

    info!("mspaint.exe launch attempt finished."); 
    Ok(())
}

/// Attempts to find an existing Paint window, or launches it if not found.
/// Retries finding the window briefly after launching.
/// Returns the HWND of the Paint window.
pub fn get_paint_hwnd() -> Result<HWND> {
    info!("Starting get_paint_hwnd - attempting to find or launch Paint");
    
    // Try direct command line to check if mspaint.exe exists on the system
    let check_command = std::process::Command::new("where")
        .arg("mspaint.exe")
        .output();
    
    match check_command {
        Ok(output) => {
            if output.status.success() {
                if let Ok(paths) = String::from_utf8(output.stdout) {
                    info!("mspaint.exe found at: {}", paths.trim());
                }
            } else {
                warn!("mspaint.exe not found in PATH");
            }
        }
        Err(e) => {
            warn!("Failed to run 'where mspaint.exe': {}", e);
        }
    }
    
    // Check if mspaint.exe is already running
    check_mspaint_running();
    
    // First check for any existing Paint windows using normal methods
    match find_paint_window() {
        Ok(hwnd) => {
            info!("Found existing Paint window: HWND={}", hwnd);
            return Ok(hwnd);
        }
        Err(MspMcpError::WindowNotFound) => {
            // If a manually started Paint is running but not detected,
            // we can try a more direct approach to find it
            info!("Regular Paint window detection failed, checking if Paint is manually running...");
            
            // Check if there's a running Paint process
            let mspaint_running = is_mspaint_running();
            
            if mspaint_running {
                info!("mspaint.exe is running according to task list, trying to force-capture it");
                
                // Special last-resort approach: Find ANY window that might be Paint
                // since we know Paint is running but our detection fails
                match find_any_paint_window() {
                    Ok(hwnd) => {
                        info!("Found potential Paint window with last-resort method: HWND={}", hwnd);
                        return Ok(hwnd);
                    }
                    Err(_) => {
                        warn!("Failed to find Paint window despite process running");
                        // Continue with normal launch procedure
                    }
                }
            }
            
            info!("Paint window not found, attempting to launch...");
            
            // First attempt - use ShellExecuteW to launch Paint
            match launch_paint() {
                Ok(_) => {
                    info!("Successfully launched Paint using ShellExecuteW");
                }
                Err(e) => {
                    // If ShellExecuteW failed, try an alternative approach
                    warn!("Primary launch method failed: {}. Trying alternative...", e);
                    
                    // Try using a more direct "start" command which has elevated privileges
                    match std::process::Command::new("cmd")
                        .args(["/C", "start", "mspaint.exe"])
                        .spawn() {
                        Ok(_) => {
                            info!("Successfully launched Paint using cmd start command");
                            // Give it time to start
                            std::thread::sleep(std::time::Duration::from_millis(3000));
                        }
                        Err(e) => {
                            // Try a third method - run Paint directly using Command
                            warn!("Second launch method failed: {}. Trying third method...", e);
                            match std::process::Command::new("mspaint.exe").spawn() {
                                Ok(_) => {
                                    info!("Successfully launched Paint using direct Command::new");
                                    std::thread::sleep(std::time::Duration::from_millis(3000));
                                }
                                Err(e) => {
                                    error!("All Paint launch methods failed. Last error: {}", e);
                                    return Err(MspMcpError::WindowsApiError(
                                        format!("Failed to launch Paint after multiple attempts: {}", e)
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            
            // After launch, check if mspaint.exe process is running
            check_mspaint_running();
            
            // Increase retry count and delay for more reliable window detection
            let max_retries = 20; // Significantly increased from 10
            let retry_delay = std::time::Duration::from_millis(1000);
            
            for attempt in 1..=max_retries {
                info!("Retrying find_paint_window (attempt {}/{}) after launch...", attempt, max_retries);
                
                // On certain attempts, force enumeration of ALL windows for debugging
                if attempt % 2 == 0 {
                    debug!("Diagnostic window enumeration on attempt {}:", attempt);
                    unsafe {
                        let enum_data = &mut EnumWindowData {
                            hwnd: None,
                            target_class: Vec::new(),
                            target_title_substring: Vec::new(),
                        };
                        let lparam = enum_data as *mut _ as LPARAM;
                        EnumWindows(Some(enum_window_proc), lparam);
                    }
                }
                
                std::thread::sleep(retry_delay);
                
                // On every 3rd attempt, try the last-resort method
                if attempt % 3 == 0 {
                    match find_any_paint_window() {
                        Ok(hwnd) => {
                            info!("Found Paint window with last-resort method: HWND={}", hwnd);
                            return Ok(hwnd);
                        }
                        Err(_) => {} // Ignore error from last-resort method
                    }
                }
                
                match find_paint_window() {
                    Ok(hwnd) => {
                        info!("Found Paint window after launch: HWND={}", hwnd);
                        
                        // Try to activate the window as a final check
                        match activate_paint_window(hwnd) {
                            Ok(_) => {
                                info!("Successfully activated Paint window");
                                return Ok(hwnd);
                            }
                            Err(e) => {
                                warn!("Found Paint window but failed to activate it: {}", e);
                                // Continue anyway - at least we found the window
                                return Ok(hwnd);
                            }
                        }
                    }
                    Err(MspMcpError::WindowNotFound) => {
                        // Try again
                        continue;
                    }
                    Err(e) => return Err(e), // Propagate other errors
                }
            }
            
            error!("Failed to find Paint window after {} retries", max_retries);
            Err(MspMcpError::WindowNotFound)
        }
        Err(e) => Err(e), // Propagate other errors
    }
}

/// Helper function to check if mspaint.exe is running using tasklist
fn check_mspaint_running() {
    match std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq mspaint.exe", "/FO", "LIST"])
        .output() {
        Ok(output) => {
            if let Ok(tasklist) = String::from_utf8(output.stdout) {
                // Only consider it running if the output contains both "mspaint.exe" AND "Image Name"
                let is_running = tasklist.contains("mspaint.exe") && tasklist.contains("Image Name");
                
                if is_running {
                    info!("Found mspaint.exe process running");
                    info!("Tasklist results for mspaint.exe:\n{}", tasklist);
                } else {
                    info!("No mspaint.exe process found in tasklist");
                }
            }
        }
        Err(e) => {
            warn!("Failed to check tasklist for mspaint.exe: {}", e);
        }
    }
}

/// Helper function that returns true if mspaint.exe is running
fn is_mspaint_running() -> bool {
    match std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq mspaint.exe", "/FO", "LIST"])
        .output() {
        Ok(output) => {
            if let Ok(tasklist) = String::from_utf8(output.stdout) {
                return tasklist.contains("mspaint.exe") && tasklist.contains("Image Name");
            }
        }
        Err(e) => {
            warn!("Failed to check tasklist for mspaint.exe: {}", e);
        }
    }
    false
}

/// Last-resort method to find any window that might be Paint
pub fn find_any_paint_window() -> Result<HWND> {
    // This is a more aggressive approach when we know Paint is running
    // but our normal detection methods fail
    
    info!("Attempting last-resort Paint window detection...");
    
    unsafe {
        // Look for any window that might be Paint with basic criteria
        let mut potential_hwnd = 0;
        
        // Try direct window captures based on common patterns
        let hwnd_result = std::process::Command::new("powershell")
            .args([
                "-Command", 
                r#"Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public class WindowFinder { [DllImport("user32.dll")] public static extern IntPtr FindWindow(string lpClassName, string lpWindowName); }'; [WindowFinder]::FindWindow($null, 'Untitled - Paint')"#
            ])
            .output();
            
        if let Ok(output) = hwnd_result {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                if let Ok(hwnd) = output_str.trim().parse::<i32>() {
                    if hwnd != 0 {
                        return Ok(hwnd as HWND);
                    }
                }
            }
        }
        
        // Try a general purpose enumeration looking for specific features
        let enum_data = &mut EnumWindowData {
            hwnd: None,
            target_class: Vec::new(),
            target_title_substring: Vec::new(),
        };
        let lparam = enum_data as *mut _ as LPARAM;
        
        // Custom callback for finding any window that might be Paint
        unsafe extern "system" fn find_any_paint_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            if IsWindowVisible(hwnd) == FALSE {
                return TRUE; // Continue enumeration
            }
            
            let data = &mut *(lparam as *mut EnumWindowData);
            
            // Get window title
            let mut window_title: [u16; 256] = [0; 256];
            let title_len = GetWindowTextW(hwnd, window_title.as_mut_ptr(), window_title.len() as i32);
            
            if title_len > 0 {
                let title_str = String::from_utf16_lossy(&window_title[..title_len as usize]);
                
                // Don't match our own application
                if title_str.contains("mcp-server-microsoft-paint") {
                    return TRUE; // Continue enumeration
                }
                
                // If it has "Paint" in the title, it's a strong candidate
                if title_str.to_lowercase().contains("paint") {
                    data.hwnd = Some(hwnd);
                    return FALSE; // Stop enumeration
                }
            }
            
            // Get class name
            let mut class_name: [u16; 128] = [0; 128];
            let class_len = GetClassNameW(hwnd, class_name.as_mut_ptr(), class_name.len() as i32);
            
            if class_len > 0 {
                let class_str = String::from_utf16_lossy(&class_name[..class_len as usize]);
                
                // Check for any class that might be Paint-related
                if class_str.contains("Paint") || class_str.contains("Afx") {
                    data.hwnd = Some(hwnd);
                    return FALSE; // Stop enumeration
                }
            }
            
            TRUE // Continue enumeration
        }
        
        EnumWindows(Some(find_any_paint_proc), lparam);
        
        if let Some(found_hwnd) = enum_data.hwnd {
            // Double-check this looks like a Paint window
            let mut window_title: [u16; 256] = [0; 256];
            let title_len = GetWindowTextW(found_hwnd, window_title.as_mut_ptr(), window_title.len() as i32);
            
            let mut class_name: [u16; 128] = [0; 128];
            let class_len = GetClassNameW(found_hwnd, class_name.as_mut_ptr(), class_name.len() as i32);
            
            let title_str = if title_len > 0 { 
                String::from_utf16_lossy(&window_title[..title_len as usize]) 
            } else { 
                "<No Title>".to_string() 
            };
            
            let class_str = if class_len > 0 { 
                String::from_utf16_lossy(&class_name[..class_len as usize])
            } else { 
                "<Unknown Class>".to_string() 
            };
            
            info!("Last-resort found potential Paint window: HWND={}, Class='{}', Title='{}'", 
                  found_hwnd, class_str, title_str);
            
            return Ok(found_hwnd);
        }
    }
    
    Err(MspMcpError::WindowNotFound)
}

/// Activates the Paint window, bringing it to the foreground.
/// Handles maximized state and ensures the window is not minimized.
pub fn activate_paint_window(hwnd: HWND) -> Result<()> {
    info!("Activating Paint window: HWND={}", hwnd);
    
    // Check if window is valid
    let is_visible = unsafe { IsWindowVisible(hwnd) };
    if is_visible == FALSE {
        return Err(MspMcpError::WindowNotFound);
    }
    
    // Wait a bit before activation attempts - helps with stability
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    let mut success = true;
    let mut activation_error = String::new();
    
    unsafe {
        // Determine if window is maximized
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        let is_maximized = (style & WS_MAXIMIZE as i32) != 0;
        
        // First ensure window is not minimized
        let show_cmd = if is_maximized { SW_SHOWMAXIMIZED } else { SW_RESTORE };
        if ShowWindow(hwnd, show_cmd) == FALSE {
            success = false;
            activation_error = "ShowWindow failed".to_string();
        }
        
        // Wait a bit after ShowWindow
        std::thread::sleep(std::time::Duration::from_millis(300));
        
        // Attempt to activate window (bring to foreground)
        if SetForegroundWindow(hwnd) == FALSE {
            // Enhanced activation attempts if normal method fails
            if success { // Only update error if we don't already have one
                success = false;
                activation_error = "SetForegroundWindow failed".to_string();
            }
            
            // Wait a bit before alternate attempt
            std::thread::sleep(std::time::Duration::from_millis(200));
            
            // Attempt alternative activation method
            // SetWindowPos can sometimes succeed when SetForegroundWindow fails
            if SetWindowPos(
                hwnd, 
                HWND_TOP,
                0, 0, 0, 0, // Don't change position or size
                SWP_SHOWWINDOW // Just show the window
            ) == FALSE {
                activation_error.push_str(", SetWindowPos also failed");
            } else {
                // SetWindowPos succeeded, consider it a partial success
                success = true;
                info!("Activated Paint window using SetWindowPos as fallback");
            }
        } else {
            info!("Activated Paint window successfully with SetForegroundWindow");
        }
    }
    
    if !success {
        error!("Failed to activate Paint window: {}", activation_error);
        return Err(MspMcpError::WindowActivationFailed(activation_error));
    }
    
    // Give the window more time to become fully active
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    Ok(())
}

/// Calculates the actual canvas dimensions within the Paint window.
/// This is a more accurate version of get_initial_canvas_dimensions.
/// TODO: Implement proper calculation based on Win11 Paint's UI layout.
pub fn get_canvas_dimensions(hwnd: HWND) -> Result<(u32, u32)> {
    // First ensure the window is activated, as dimensions might not be correct
    // if the window is minimized
    activate_paint_window(hwnd)?;
    
    // Get the window rectangle
    let mut rect: windows_sys::Win32::Foundation::RECT = unsafe { std::mem::zeroed() };
    unsafe {
        if GetWindowRect(hwnd, &mut rect) == FALSE {
            return Err(MspMcpError::WindowsApiError("GetWindowRect failed".to_string()));
        }
    }
    
    // Calculate window dimensions first
    let window_width = (rect.right - rect.left) as u32;
    let window_height = (rect.bottom - rect.top) as u32;
    
    // Approximate the canvas dimensions by subtracting typical UI elements sizes
    // These values are estimates and may need adjustment based on actual Win11 Paint UI
    const TITLE_BAR_HEIGHT: u32 = 32;
    const MENU_BAR_HEIGHT: u32 = 30; 
    const TOOLBAR_HEIGHT: u32 = 80;  // Combined height of ribbon/toolbar
    const STATUS_BAR_HEIGHT: u32 = 25;
    const LEFT_PANEL_WIDTH: u32 = 0;  // No left panel in modern Paint
    const RIGHT_PANEL_WIDTH: u32 = 270; // Right tools/properties panel
    
    // Calculate canvas dimensions by subtracting UI elements
    // Ensure we don't underflow if window is very small
    let canvas_width = window_width.saturating_sub(LEFT_PANEL_WIDTH + RIGHT_PANEL_WIDTH);
    let canvas_height = window_height.saturating_sub(
        TITLE_BAR_HEIGHT + MENU_BAR_HEIGHT + TOOLBAR_HEIGHT + STATUS_BAR_HEIGHT
    );
    
    info!("Calculated canvas dimensions: {}x{} (window: {}x{})", 
        canvas_width, canvas_height, window_width, window_height);
    
    Ok((canvas_width, canvas_height))
}

// TODO: Add tests (might require manual setup or #[ignore])

/// Gets the initial canvas dimensions (placeholder: uses window dimensions).
/// TODO: Implement accurate canvas area calculation for Win11 Paint.
pub fn get_initial_canvas_dimensions(hwnd: HWND) -> Result<(u32, u32)> {
    // Initialize RECT using zeroed
    let mut rect: windows_sys::Win32::Foundation::RECT = unsafe { std::mem::zeroed() };
    unsafe {
        if GetWindowRect(hwnd, &mut rect) == FALSE {
            return Err(MspMcpError::WindowsApiError("GetWindowRect failed".to_string()));
        }
    }
    // Placeholder: return full window dimensions
    let width = (rect.right - rect.left) as u32;
    let height = (rect.bottom - rect.top) as u32;
    info!("GetWindowRect returned dimensions: {}x{}", width, height);
    // TODO: Subtract toolbars, panels, etc., to get canvas size
    Ok((width, height))
}

/// Converts client coordinates to screen coordinates
/// Client coordinates are relative to the client area of the window,
/// while screen coordinates are absolute positions on the screen.
pub fn client_to_screen(hwnd: HWND, client_x: i32, client_y: i32) -> Result<(i32, i32)> {
    let mut point = POINT {
        x: client_x,
        y: client_y,
    };
    
    unsafe {
        if ClientToScreen(hwnd, &mut point) == FALSE {
            return Err(MspMcpError::WindowsApiError("ClientToScreen failed".to_string()));
        }
    }
    
    Ok((point.x, point.y))
}

/// Converts a screen coordinate to a normalized coordinate (0-65535 range)
/// Normalized coordinates are used by SendInput to ensure compatibility with multiple monitors
/// and different screen resolutions.
fn screen_to_normalized(x: i32, y: i32) -> (i32, i32) {
    let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    
    let normalized_x = (x * 65535) / screen_width;
    let normalized_y = (y * 65535) / screen_height;
    
    (normalized_x, normalized_y)
}

/// Simulates moving the mouse cursor to the specified screen coordinates.
/// Uses normalized absolute coordinates for reliable positioning.
pub fn move_mouse_to(screen_x: i32, screen_y: i32) -> Result<()> {
    let (normalized_x, normalized_y) = screen_to_normalized(screen_x, screen_y);
    
    debug!("Moving mouse to screen ({}, {}) -> normalized ({}, {})", 
           screen_x, screen_y, normalized_x, normalized_y);
    
    // Create the INPUT structure for mouse movement
    let mut input_struct: INPUT = unsafe { std::mem::zeroed() };
    input_struct.r#type = INPUT_MOUSE;
    
    // Using normalized absolute coordinates (0-65535 range)
    unsafe {
        // Access the union field correctly
        let mi = &mut input_struct.Anonymous.mi;
        mi.dx = normalized_x;
        mi.dy = normalized_y;
        mi.mouseData = 0;
        mi.dwFlags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        
        // Send the input
        let inputs_sent = SendInput(1, &mut input_struct, std::mem::size_of::<INPUT>() as i32);
        
        if inputs_sent != 1 {
            return Err(MspMcpError::WindowsApiError("Failed to send mouse movement input".to_string()));
        }
    }
    
    // Brief delay to allow the movement to register
    std::thread::sleep(std::time::Duration::from_millis(5));
    
    Ok(())
}

/// Simulates a left mouse button click at the current cursor position.
pub fn click_left_mouse_button() -> Result<()> {
    debug!("Simulating left mouse click...");
    // Create two INPUT structs: one for mouse down, one for mouse up
    let mut inputs: [INPUT; 2] = unsafe { std::mem::zeroed() };
    
    unsafe {
        // Set up mouse down input
        inputs[0].r#type = INPUT_MOUSE;
        let mi_down = &mut inputs[0].Anonymous.mi;
        mi_down.dx = 0;
        mi_down.dy = 0;
        mi_down.mouseData = 0;
        mi_down.dwFlags = MOUSEEVENTF_LEFTDOWN;
        mi_down.time = 0;
        mi_down.dwExtraInfo = 0;
        
        // Set up mouse up input
        inputs[1].r#type = INPUT_MOUSE;
        let mi_up = &mut inputs[1].Anonymous.mi;
        mi_up.dx = 0;
        mi_up.dy = 0;
        mi_up.mouseData = 0;
        mi_up.dwFlags = MOUSEEVENTF_LEFTUP;
        mi_up.time = 0;
        mi_up.dwExtraInfo = 0;
        
        // Send the inputs
        debug!("Sending MOUSEEVENTF_LEFTDOWN + MOUSEEVENTF_LEFTUP");
        let inputs_sent = SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
        
        if inputs_sent != 2 {
            error!("SendInput failed for left click (sent {} inputs)", inputs_sent);
            return Err(MspMcpError::WindowsApiError("Failed to send mouse click input".to_string()));
        } else {
            debug!("SendInput successful for left click.");
        }
    }
    
    // Brief delay to allow the click to register
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    Ok(())
}

/// Simulates a right mouse button click at the current cursor position.
pub fn click_right_mouse_button() -> Result<()> {
    debug!("Simulating right mouse click...");
    // Create two INPUT structs: one for mouse down, one for mouse up
    let mut inputs: [INPUT; 2] = unsafe { std::mem::zeroed() };
    
    unsafe {
        // Set up mouse down input
        inputs[0].r#type = INPUT_MOUSE;
        let mi_down = &mut inputs[0].Anonymous.mi;
        mi_down.dx = 0;
        mi_down.dy = 0;
        mi_down.mouseData = 0;
        mi_down.dwFlags = MOUSEEVENTF_RIGHTDOWN;
        mi_down.time = 0;
        mi_down.dwExtraInfo = 0;
        
        // Set up mouse up input
        inputs[1].r#type = INPUT_MOUSE;
        let mi_up = &mut inputs[1].Anonymous.mi;
        mi_up.dx = 0;
        mi_up.dy = 0;
        mi_up.mouseData = 0;
        mi_up.dwFlags = MOUSEEVENTF_RIGHTUP;
        mi_up.time = 0;
        mi_up.dwExtraInfo = 0;
        
        // Send the inputs
        debug!("Sending MOUSEEVENTF_RIGHTDOWN + MOUSEEVENTF_RIGHTUP");
        let inputs_sent = SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
        
        if inputs_sent != 2 {
            error!("SendInput failed for right click (sent {} inputs)", inputs_sent);
            return Err(MspMcpError::WindowsApiError("Failed to send mouse right-click input".to_string()));
        } else {
            debug!("SendInput successful for right click.");
        }
    }
    
    // Brief delay to allow the click to register
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    Ok(())
}

/// Simulates a mouse drag operation from one position to another.
/// This is useful for drawing lines and shapes.
pub fn drag_mouse(start_screen_x: i32, start_screen_y: i32, end_screen_x: i32, end_screen_y: i32) -> Result<()> {
    // Move to start position
    move_mouse_to(start_screen_x, start_screen_y)?;
    
    // Brief delay before clicking
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // Perform mouse down
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    input.r#type = INPUT_MOUSE;
    
    unsafe {
        // Mouse down
        let mi = &mut input.Anonymous.mi;
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = MOUSEEVENTF_LEFTDOWN;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        
        debug!("Sending MOUSEEVENTF_LEFTDOWN for drag start at ({}, {})", start_screen_x, start_screen_y);
        let inputs_sent = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        if inputs_sent != 1 {
            error!("SendInput failed for drag start (sent {} inputs)", inputs_sent);
            return Err(MspMcpError::WindowsApiError("Failed to send mouse down input".to_string()));
        } else {
            debug!("SendInput successful for drag start.");
        }
    }
    
    // Move to end position in small steps for smoother drawing
    let steps = 10; // Use 10 steps for smoother drawing
    let dx = (end_screen_x - start_screen_x) as f32 / steps as f32;
    let dy = (end_screen_y - start_screen_y) as f32 / steps as f32;
    
    for i in 1..=steps {
        let x = start_screen_x + (dx * i as f32) as i32;
        let y = start_screen_y + (dy * i as f32) as i32;
        
        // Move to intermediate position
        move_mouse_to(x, y)?;
        
        // Brief delay between steps
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    
    // Ensure we're at the end position
    move_mouse_to(end_screen_x, end_screen_y)?;
    
    // Brief delay before releasing
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // Perform mouse up
    unsafe {
        // Mouse up
        let mi = &mut input.Anonymous.mi;
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = MOUSEEVENTF_LEFTUP;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        
        debug!("Sending MOUSEEVENTF_LEFTUP for drag end at ({}, {})", end_screen_x, end_screen_y);
        let inputs_sent = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        if inputs_sent != 1 {
            error!("SendInput failed for drag end (sent {} inputs)", inputs_sent);
            return Err(MspMcpError::WindowsApiError("Failed to send mouse up input".to_string()));
        } else {
            debug!("SendInput successful for drag end.");
        }
    }
    
    Ok(())
}

/// Helper function to click at a specific position.
/// Moves the mouse to the screen coordinates and performs a left-click.
pub fn click_at_position(screen_x: i32, screen_y: i32) -> Result<()> {
    move_mouse_to(screen_x, screen_y)?;
    click_left_mouse_button()
}

/// Helper function to click at a position in client coordinates.
/// Converts client coordinates to screen coordinates first.
pub fn click_at_client_position(hwnd: HWND, client_x: i32, client_y: i32) -> Result<()> {
    let (screen_x, screen_y) = client_to_screen(hwnd, client_x, client_y)?;
    click_at_position(screen_x, screen_y)
}

/// Calculate the drawing area offset
/// This adds the extra vertical offset needed to account for toolbars in Paint
pub fn get_drawing_area_offset(hwnd: HWND) -> Result<(i32, i32)> {
    // The toolbar and ribbon height varies based on Paint version
    // Windows 11 Paint has a larger ribbon than Windows 10
    // These are approximations that should work in most cases
    let toolbar_height = 120;  // Combined height of title bar, ribbon, etc.
    let left_offset = 5;       // Small left margin
    
    Ok((left_offset, toolbar_height))
}

/// Draws a pixel at the specified coordinates.
pub fn draw_pixel_at(hwnd: HWND, canvas_x: i32, canvas_y: i32) -> Result<()> {
    // First make sure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // Select the pencil tool for reliable drawing
    select_tool(hwnd, "pencil")?;
    
    // Get drawing area offset
    let (offset_x, offset_y) = get_drawing_area_offset(hwnd)?;
    
    // Add offset to canvas coordinates to get client coordinates
    let client_x = canvas_x + offset_x;
    let client_y = canvas_y + offset_y;
    
    // Convert to screen coordinates
    let (screen_x, screen_y) = client_to_screen(hwnd, client_x, client_y)?;
    
    // Simple click to draw a pixel with the pencil tool
    click_at_position(screen_x, screen_y)
}

/// Simulates pressing a keyboard key (key down followed by key up).
/// This is useful for typing text and keyboard shortcuts.
pub fn press_key(key_code: u16) -> Result<()> {
    // Create two INPUT structs: one for key down, one for key up
    let mut inputs: [INPUT; 2] = unsafe { std::mem::zeroed() };
    
    unsafe {
        // Set up key down input
        inputs[0].r#type = INPUT_KEYBOARD;
        let ki_down = &mut inputs[0].Anonymous.ki;
        ki_down.wVk = key_code;
        ki_down.wScan = 0;
        ki_down.dwFlags = 0; // Key down has no special flags
        ki_down.time = 0;
        ki_down.dwExtraInfo = 0;
        
        // Set up key up input
        inputs[1].r#type = INPUT_KEYBOARD;
        let ki_up = &mut inputs[1].Anonymous.ki;
        ki_up.wVk = key_code;
        ki_up.wScan = 0;
        ki_up.dwFlags = KEYEVENTF_KEYUP;
        ki_up.time = 0;
        ki_up.dwExtraInfo = 0;
        
        // Send the inputs
        let inputs_sent = SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
        
        if inputs_sent != 2 {
            return Err(MspMcpError::WindowsApiError("Failed to send key press input".to_string()));
        }
    }
    
    // Brief delay to allow the key press to register
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    Ok(())
}

/// Simulates pressing a keyboard key using scan code instead of virtual key code.
/// Some applications work better with scan codes, especially for text entry.
pub fn press_key_scan(scan_code: u16) -> Result<()> {
    // Create two INPUT structs: one for key down, one for key up
    let mut inputs: [INPUT; 2] = unsafe { std::mem::zeroed() };
    
    unsafe {
        // Set up key down input with scan code
        inputs[0].r#type = INPUT_KEYBOARD;
        let ki_down = &mut inputs[0].Anonymous.ki;
        ki_down.wVk = 0; // 0 indicates we're using scan code
        ki_down.wScan = scan_code;
        ki_down.dwFlags = KEYEVENTF_SCANCODE; // Use scan code instead of virtual key
        ki_down.time = 0;
        ki_down.dwExtraInfo = 0;
        
        // Set up key up input with scan code
        inputs[1].r#type = INPUT_KEYBOARD;
        let ki_up = &mut inputs[1].Anonymous.ki;
        ki_up.wVk = 0;
        ki_up.wScan = scan_code;
        ki_up.dwFlags = KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP;
        ki_up.time = 0;
        ki_up.dwExtraInfo = 0;
        
        // Send the inputs
        let inputs_sent = SendInput(2, inputs.as_mut_ptr(), std::mem::size_of::<INPUT>() as i32);
        
        if inputs_sent != 2 {
            return Err(MspMcpError::WindowsApiError("Failed to send key press scan code input".to_string()));
        }
    }
    
    // Brief delay to allow the key press to register
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    Ok(())
}

/// Simulates pressing and holding a key down without releasing it.
/// Useful as part of keyboard shortcuts (e.g., Ctrl+C).
pub fn key_down(key_code: u16) -> Result<()> {
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    
    unsafe {
        input.r#type = INPUT_KEYBOARD;
        let ki = &mut input.Anonymous.ki;
        ki.wVk = key_code;
        ki.wScan = 0;
        ki.dwFlags = 0; // Key down
        ki.time = 0;
        ki.dwExtraInfo = 0;
        
        let inputs_sent = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        
        if inputs_sent != 1 {
            return Err(MspMcpError::WindowsApiError("Failed to send key down input".to_string()));
        }
    }
    
    // Brief delay to ensure the key press registers
    std::thread::sleep(std::time::Duration::from_millis(5));
    
    Ok(())
}

/// Simulates releasing a key that was previously held down.
/// Used together with key_down to create keyboard shortcuts.
pub fn key_up(key_code: u16) -> Result<()> {
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    
    unsafe {
        input.r#type = INPUT_KEYBOARD;
        let ki = &mut input.Anonymous.ki;
        ki.wVk = key_code;
        ki.wScan = 0;
        ki.dwFlags = KEYEVENTF_KEYUP;
        ki.time = 0;
        ki.dwExtraInfo = 0;
        
        let inputs_sent = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        
        if inputs_sent != 1 {
            return Err(MspMcpError::WindowsApiError("Failed to send key up input".to_string()));
        }
    }
    
    // Brief delay
    std::thread::sleep(std::time::Duration::from_millis(5));
    
    Ok(())
}

/// Simulates pressing Ctrl+A (Select All)
pub fn press_ctrl_a() -> Result<()> {
    key_down(VK_CONTROL)?;
    press_key('A' as u16)?;
    key_up(VK_CONTROL)
}

/// Simulates pressing Ctrl+C (Copy)
pub fn press_ctrl_c() -> Result<()> {
    key_down(VK_CONTROL)?;
    press_key('C' as u16)?;
    key_up(VK_CONTROL)
}

/// Simulates pressing Ctrl+V (Paste)
pub fn press_ctrl_v() -> Result<()> {
    key_down(VK_CONTROL)?;
    press_key('V' as u16)?;
    key_up(VK_CONTROL)
}

/// Simulates pressing Ctrl+N (New)
pub fn press_ctrl_n() -> Result<()> {
    key_down(VK_CONTROL)?;
    press_key('N' as u16)?;
    key_up(VK_CONTROL)
}

/// Simulates pressing Ctrl+S (Save)
pub fn press_ctrl_s() -> Result<()> {
    key_down(VK_CONTROL)?;
    press_key('S' as u16)?;
    key_up(VK_CONTROL)
}

/// Simulates pressing Delete key
pub fn press_delete() -> Result<()> {
    press_key(VK_DELETE)
}

/// Simulates pressing Enter key
pub fn press_enter() -> Result<()> {
    press_key(VK_RETURN)
}

/// Simulates pressing Tab key
pub fn press_tab() -> Result<()> {
    press_key(VK_TAB)
}

/// Simulates pressing Escape key
pub fn press_escape() -> Result<()> {
    press_key(VK_ESCAPE)
}

/// Simulates typing a simple text string.
/// Note: This function only supports basic ASCII characters.
/// For more complex text input, use a more sophisticated approach.
pub fn type_text(text: &str) -> Result<()> {
    for c in text.chars() {
        // Convert character to uppercase for virtual key code
        // (Windows virtual key codes use uppercase letters)
        let upper_c = c.to_uppercase().next().unwrap_or(c);
        
        // Handle special characters or use key codes for letters/numbers
        match upper_c {
            ' ' => press_key(VK_SPACE)?,
            '\t' => press_key(VK_TAB)?,
            '\n' | '\r' => press_key(VK_RETURN)?,
            // For letters and numbers, use their virtual key codes
            'A'..='Z' | '0'..='9' => {
                // Convert to virtual key code (which is just the ASCII value for letters/numbers)
                let key_code = upper_c as u16;
                
                // If original was lowercase and it's a letter, we need to type lowercase
                if c.is_lowercase() && c.is_alphabetic() {
                    // For lowercase, don't use Shift
                    press_key(key_code)?;
                } else if c.is_uppercase() && c.is_alphabetic() {
                    // For uppercase letters, use Shift
                    key_down(VK_SHIFT)?;
                    press_key(key_code)?;
                    key_up(VK_SHIFT)?;
                } else {
                    // For numbers and other characters
                    press_key(key_code)?;
                }
            }
            // Add more special cases as needed
            _ => {
                // Skip unsupported characters
                warn!("Unsupported character in type_text: '{}'", c);
            }
        }
        
        // Brief delay between key presses
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    
    Ok(())
}

/// Helper function to draw a line from (start_x, start_y) to (end_x, end_y).
/// Uses the mouse drag functionality to simulate drawing a line - similar to the direct_paint_test.py approach.
pub fn draw_line_at(hwnd: HWND, start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> Result<()> {
    // Make sure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // Select the pencil tool for reliable drawing
    select_tool(hwnd, "pencil")?;
    
    // Get drawing area offset
    let (offset_x, offset_y) = get_drawing_area_offset(hwnd)?;
    
    // Add offset to canvas coordinates to get client coordinates
    let client_start_x = start_x + offset_x;
    let client_start_y = start_y + offset_y;
    let client_end_x = end_x + offset_x;
    let client_end_y = end_y + offset_y;
    
    // Convert client coordinates to screen coordinates
    let (start_screen_x, start_screen_y) = client_to_screen(hwnd, client_start_x, client_start_y)?;
    let (end_screen_x, end_screen_y) = client_to_screen(hwnd, client_end_x, client_end_y)?;
    
    info!("Drawing line from ({},{}) to ({},{}) on screen: ({},{}) to ({},{})", 
          start_x, start_y, end_x, end_y,
          start_screen_x, start_screen_y, end_screen_x, end_screen_y);
    
    // First, move to the start position
    move_mouse_to(start_screen_x, start_screen_y)?;
    
    // Wait a moment to ensure position
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // Mouse down at start position
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    input.r#type = INPUT_MOUSE;
    
    unsafe {
        let mi = &mut input.Anonymous.mi;
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = MOUSEEVENTF_LEFTDOWN;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        
        let inputs_sent = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        if inputs_sent != 1 {
            return Err(MspMcpError::WindowsApiError("Failed to send mouse down input".to_string()));
        }
    }
    
    // Wait a moment
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // Move in small steps to the end position for smoother drawing
    let steps = 10;
    let dx = (end_screen_x - start_screen_x) as f32 / steps as f32;
    let dy = (end_screen_y - start_screen_y) as f32 / steps as f32;
    
    for i in 1..=steps {
        let x = start_screen_x + (dx * i as f32) as i32;
        let y = start_screen_y + (dy * i as f32) as i32;
        
        // Move to intermediate position
        move_mouse_to(x, y)?;
        
        // Brief delay between steps
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    
    // Ensure we're at the end position
    move_mouse_to(end_screen_x, end_screen_y)?;
    
    // Wait a moment before releasing
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // Mouse up at end position
    unsafe {
        let mi = &mut input.Anonymous.mi;
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = MOUSEEVENTF_LEFTUP;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        
        let inputs_sent = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        if inputs_sent != 1 {
            return Err(MspMcpError::WindowsApiError("Failed to send mouse up input".to_string()));
        }
    }
    
    // Wait a moment to ensure the drawing is complete
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    Ok(())
}

/// Selects a drawing tool in Paint by clicking its position in the toolbar.
/// The tool positions are based on Windows 11 Paint's modern UI layout.
pub fn select_tool(hwnd: HWND, tool: &str) -> Result<()> {
    // First ensure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // Get window dimensions to help with adaptive positioning
    let mut rect: windows_sys::Win32::Foundation::RECT = unsafe { std::mem::zeroed() };
    unsafe {
        if GetWindowRect(hwnd, &mut rect) == FALSE {
            return Err(MspMcpError::WindowsApiError("GetWindowRect failed".to_string()));
        }
    }
    
    let window_width = rect.right - rect.left;
    
    // Define tool positions based on the top toolbar (using percentages of window width)
    // These are approximate positions that should work across different window sizes
    let tool_positions = match tool.to_lowercase().as_str() {
        "pencil" => (window_width / 20, 60),       // Left toolbar area
        "brush" => (window_width / 10, 60),        // Brush tool
        "fill" => (window_width / 7, 60),          // Fill tool
        "text" => (window_width / 5, 60),          // Text tool
        "eraser" => (window_width / 4, 60),        // Eraser tool
        "select" => (window_width / 3, 60),        // Selection tool
        "shape" => (window_width / 2.5 as i32, 60),// Shape tool
        _ => return Err(MspMcpError::InvalidParameters(format!("Unsupported tool: {}", tool))),
    };
    
    info!("Selecting tool: {} at position ({}, {})", tool, tool_positions.0, tool_positions.1);
    
    // Convert toolbar coordinates to screen coordinates
    let (screen_x, screen_y) = client_to_screen(hwnd, tool_positions.0, tool_positions.1)?;
    
    // Click the tool position
    click_at_position(screen_x, screen_y)?;
    
    // Wait for tool selection to take effect
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    Ok(())
}

/// Sets the active color in Paint by selecting it from the color panel.
/// The color should be in "#RRGGBB" format.
pub fn set_color(hwnd: HWND, color: &str) -> Result<()> {
    // First ensure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // Parse the color string
    if !color.starts_with('#') || color.len() != 7 {
        return Err(MspMcpError::InvalidParameters("Color must be in #RRGGBB format".to_string()));
    }
    
    // For now, just log the color that would be selected
    // In a real implementation, we would interact with Paint's color picker
    info!("Would select color: {}", color);
    
    Ok(())
}

/// Sets the line thickness or brush size in Paint.
/// The level parameter should be between 1 and 5.
pub fn set_thickness(hwnd: HWND, level: u32) -> Result<()> {
    // Validate thickness level
    if level < 1 || level > 5 {
        return Err(MspMcpError::InvalidParameters("Thickness level must be between 1 and 5".to_string()));
    }
    
    // First ensure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // For now, just log the thickness that would be selected
    // In a real implementation, we would interact with Paint's thickness control
    info!("Would set thickness level: {}", level);
    
    Ok(())
}

/// Sets the brush size for a specific tool.
/// The size parameter should be between 1 and 30 pixels.
pub fn set_brush_size(hwnd: HWND, size: u32, tool: Option<&str>) -> Result<()> {
    // Validate brush size
    if size < 1 || size > 30 {
        return Err(MspMcpError::InvalidParameters("Brush size must be between 1 and 30".to_string()));
    }
    
    // First ensure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // If a specific tool is provided, select it first
    if let Some(tool_name) = tool {
        select_tool(hwnd, tool_name)?;
    }
    
    // For now, just log the size that would be selected
    info!("Would set brush size: {} for tool: {}", size, tool.unwrap_or("current"));
    
    Ok(())
}

/// Sets the fill type for shapes in Paint.
/// The fill_type parameter should be "none", "solid", or "outline".
pub fn set_fill(hwnd: HWND, fill_type: &str) -> Result<()> {
    // Validate fill type
    match fill_type.to_lowercase().as_str() {
        "none" | "solid" | "outline" => {},
        _ => return Err(MspMcpError::InvalidParameters(
            format!("Fill type must be 'none', 'solid', or 'outline', got '{}'", fill_type)))
    }
    
    // First ensure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // For now, just log the fill type that would be selected
    info!("Would set fill type: {}", fill_type);
    
    Ok(())
}

/// Draws a shape from (start_x, start_y) to (end_x, end_y).
/// Selects the appropriate shape tool and uses mouse drag to create the shape.
pub fn draw_shape(hwnd: HWND, shape_type: &str, start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> Result<()> {
    // First, try to use the UIA implementation
    if let Ok(()) = crate::uia::draw_shape_uia(hwnd, shape_type, start_x, start_y, end_x, end_y) {
        return Ok(());
    }
    
    // Fall back to the old implementation if UIA fails
    info!("Falling back to legacy draw_shape implementation");
    
    // Make sure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // First select the shape tool
    select_tool(hwnd, "shape")?;
    
    // Validate and select the specific shape type
    let valid_shapes = ["rectangle", "ellipse", "line", "arrow", "triangle", "pentagon", "hexagon"];
    if !valid_shapes.contains(&shape_type.to_lowercase().as_str()) {
        return Err(MspMcpError::InvalidParameters(
            format!("Invalid shape type: {}. Must be one of: rectangle, ellipse, line, arrow, triangle, pentagon, hexagon", 
                    shape_type)));
    }
    
    // Log what shape would be selected
    info!("Would select shape type: {}", shape_type);
    
    // Convert client coordinates to screen coordinates
    let (start_screen_x, start_screen_y) = client_to_screen(hwnd, start_x, start_y)?;
    let (end_screen_x, end_screen_y) = client_to_screen(hwnd, end_x, end_y)?;
    
    // Draw the shape with a mouse drag
    // Move to start position
    move_mouse_to(start_screen_x, start_screen_y)?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // Press mouse down
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    input.r#type = INPUT_MOUSE;
    
    unsafe {
        let mi = &mut input.Anonymous.mi;
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = MOUSEEVENTF_LEFTDOWN;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        
        let inputs_sent = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        if inputs_sent != 1 {
            return Err(MspMcpError::WindowsApiError("Failed to send mouse down input".to_string()));
        }
    }
    
    // Move to end position
    move_mouse_to(end_screen_x, end_screen_y)?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // Release mouse button
    unsafe {
        let mi = &mut input.Anonymous.mi;
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = MOUSEEVENTF_LEFTUP;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        
        let inputs_sent = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        if inputs_sent != 1 {
            return Err(MspMcpError::WindowsApiError("Failed to send mouse up input".to_string()));
        }
    }
    
    Ok(())
}

/// Draws a polyline (series of connected lines) by drawing line segments between consecutive points.
pub fn draw_polyline(hwnd: HWND, points: &[(i32, i32)]) -> Result<()> {
    // Validate input
    if points.len() < 2 {
        return Err(MspMcpError::InvalidParameters(
            "Polyline requires at least 2 points".to_string()));
    }
    
    // Make sure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // Select the pencil tool
    select_tool(hwnd, "pencil")?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // Convert first point to screen coordinates
    let (start_screen_x, start_screen_y) = client_to_screen(hwnd, points[0].0, points[0].1)?;
    
    // Move to start position
    move_mouse_to(start_screen_x, start_screen_y)?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // Press mouse down
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    input.r#type = INPUT_MOUSE;
    
    unsafe {
        let mi = &mut input.Anonymous.mi;
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = MOUSEEVENTF_LEFTDOWN;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        
        let inputs_sent = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        if inputs_sent != 1 {
            return Err(MspMcpError::WindowsApiError("Failed to send mouse down input".to_string()));
        }
    }
    
    // Move through each point
    for i in 1..points.len() {
        let (screen_x, screen_y) = client_to_screen(hwnd, points[i].0, points[i].1)?;
        move_mouse_to(screen_x, screen_y)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    
    // Release mouse button
    unsafe {
        let mi = &mut input.Anonymous.mi;
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = MOUSEEVENTF_LEFTUP;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        
        let inputs_sent = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        if inputs_sent != 1 {
            return Err(MspMcpError::WindowsApiError("Failed to send mouse up input".to_string()));
        }
    }
    
    Ok(())
}

/// Clears the canvas in Paint using Ctrl+A then Delete.
pub fn clear_canvas(hwnd: HWND) -> Result<()> {
    // Make sure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // Select all (Ctrl+A)
    press_ctrl_a()?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // Press Delete
    press_delete()?;
    
    Ok(())
}

/// Selects a region in Paint from start coordinates to end coordinates.
pub fn select_region(hwnd: HWND, start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> Result<()> {
    // Make sure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // Select the selection tool
    select_tool(hwnd, "select")?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // Convert client coordinates to screen coordinates
    let (start_screen_x, start_screen_y) = client_to_screen(hwnd, start_x, start_y)?;
    let (end_screen_x, end_screen_y) = client_to_screen(hwnd, end_x, end_y)?;
    
    // Draw the selection with a mouse drag
    // Move to start position
    move_mouse_to(start_screen_x, start_screen_y)?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // Press mouse down
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    input.r#type = INPUT_MOUSE;
    
    unsafe {
        let mi = &mut input.Anonymous.mi;
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = MOUSEEVENTF_LEFTDOWN;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        
        let inputs_sent = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        if inputs_sent != 1 {
            return Err(MspMcpError::WindowsApiError("Failed to send mouse down input".to_string()));
        }
    }
    
    // Move to end position
    move_mouse_to(end_screen_x, end_screen_y)?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // Release mouse button
    unsafe {
        let mi = &mut input.Anonymous.mi;
        mi.dx = 0;
        mi.dy = 0;
        mi.mouseData = 0;
        mi.dwFlags = MOUSEEVENTF_LEFTUP;
        mi.time = 0;
        mi.dwExtraInfo = 0;
        
        let inputs_sent = SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
        if inputs_sent != 1 {
            return Err(MspMcpError::WindowsApiError("Failed to send mouse up input".to_string()));
        }
    }
    
    Ok(())
}

/// Copies the current selection to the clipboard.
pub fn copy_selection(hwnd: HWND) -> Result<()> {
    // Make sure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // Press Ctrl+C
    press_ctrl_c()?;
    
    Ok(())
}

/// Pastes at the specified coordinates.
pub fn paste_at(hwnd: HWND, x: i32, y: i32) -> Result<()> {
    // Make sure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // Click at the paste location
    let (screen_x, screen_y) = client_to_screen(hwnd, x, y)?;
    click_at_position(screen_x, screen_y)?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // Press Ctrl+V
    press_ctrl_v()?;
    
    Ok(())
}

/// Adds text at the specified position.
pub fn add_text(
    hwnd: HWND, 
    x: i32, 
    y: i32, 
    text: &str, 
    color: Option<&str>,
    font_name: Option<&str>,
    font_size: Option<u32>,
    font_style: Option<&str>
) -> Result<()> {
    // Make sure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // Select the text tool
    select_tool(hwnd, "text")?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // If a color is specified, set it
    if let Some(color_str) = color {
        set_color(hwnd, color_str)?;
    }
    
    // Click at the text position
    let (screen_x, screen_y) = client_to_screen(hwnd, x, y)?;
    click_at_position(screen_x, screen_y)?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    
    // Type the text
    type_text(text)?;
    
    // Click somewhere else to finalize the text
    click_at_position(screen_x + 300, screen_y + 300)?;
    
    Ok(())
}

/// Creates a new canvas with the specified dimensions.
pub fn create_canvas(
    hwnd: HWND, 
    width: u32, 
    height: u32, 
    background_color: Option<&str>
) -> Result<()> {
    // Make sure the Paint window is active
    activate_paint_window(hwnd)?;
    
    // Press Ctrl+N for a new canvas
    press_ctrl_n()?;
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // For now, just log the action
    info!("Would create a {}x{} canvas with background: {}", 
          width, 
          height, 
          background_color.unwrap_or("default"));
    
    // Press Enter to accept
    press_enter()?;
    
    Ok(())
}

/// Alternative function to get the Paint window handle directly.
pub fn get_direct_paint_hwnd() -> Result<HWND> {
    // For now, just delegate to the regular function
    get_paint_hwnd()
}

// TODO: Add tests for tool selection and color management functions 