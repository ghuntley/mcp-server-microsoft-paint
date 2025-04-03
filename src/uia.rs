use crate::error::{MspMcpError, Result};
use log::{debug, info, warn, error};
use std::collections::HashMap;
use std::time::Duration;
use uiautomation::{
    UIAutomation,
    UIElement,
    patterns::UIInvokePattern,
    types::TreeScope,
    controls::{PaneControl, ToolBarControl, ButtonControl, Control},
};
use windows_sys::Win32::Foundation::HWND;
use crate::windows;

// Cached mapping of tool names to their UI Automation elements
static mut TOOL_BUTTON_CACHE: Option<HashMap<String, String>> = None;

/// Initialize UI Automation - must be called before using any UIA functions
pub fn initialize_uia() -> Result<UIAutomation> {
    match UIAutomation::new() {
        Ok(client) => {
            info!("UI Automation initialized successfully");
            Ok(client)
        },
        Err(err) => {
            error!("Failed to initialize UI Automation: {}", err);
            Err(MspMcpError::WindowsApiError(format!(
                "Failed to initialize UI Automation: {}", err
            )))
        }
    }
}

/// Get Paint's ribbon UI element given a window handle
pub fn get_paint_ribbon(automation: &UIAutomation, hwnd: HWND) -> Result<UIElement> {
    let window = match automation.element_from_handle((hwnd as isize).into()) {
        Ok(window) => window,
        Err(err) => {
            error!("Failed to get Paint window element: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to get Paint window element: {}", err
            )));
        }
    };

    // First, find the ribbon element which is typically a pane
    let matcher = automation.create_matcher()
        .from(window)
        .control_type(PaneControl::TYPE) // Pane control type
        .timeout(2000);
        
    match matcher.find_first() {
        Ok(ribbon) => {
            // Try to confirm this is the ribbon by checking for expected child elements
            if let Ok(id) = ribbon.get_automation_id() {
                debug!("Found potential ribbon with AutomationId: {}", id);
            }
            Ok(ribbon)
        },
        Err(err) => {
            warn!("Could not find Paint ribbon UI element: {}", err);
            Err(MspMcpError::ElementNotFound("Paint ribbon".to_string()))
        }
    }
}

/// Get toolbar buttons container
pub fn get_tools_container(automation: &UIAutomation, hwnd: HWND) -> Result<UIElement> {
    let ribbon = get_paint_ribbon(automation, hwnd)?;
    
    // Look for the tools container (toolbar or another container with tool buttons)
    let toolbar_matcher = automation.create_matcher()
        .from(ribbon.clone())
        .control_type(ToolBarControl::TYPE) // ToolBar control type
        .timeout(2000);
        
    match toolbar_matcher.find_first() {
        Ok(toolbar) => Ok(toolbar),
        Err(_) => {
            // If we can't find a toolbar specifically, try finding a pane that might contain the tools
            let pane_matcher = automation.create_matcher()
                .from(ribbon)
                .control_type(PaneControl::TYPE) // Pane control type
                .timeout(2000);
                
            match pane_matcher.find_first() {
                Ok(pane) => Ok(pane),
                Err(err) => {
                    warn!("Could not find Paint tools container: {}", err);
                    Err(MspMcpError::ElementNotFound("Paint tools container".to_string()))
                }
            }
        }
    }
}

/// Build a mapping of tool names to their automation names/IDs for faster lookup
fn build_tool_mapping(automation: &UIAutomation, hwnd: HWND) -> Result<HashMap<String, String>> {
    let mut tool_map = HashMap::new();
    
    // Common tool names in Paint and their possible UIA names/identifiers
    let tool_mappings = [
        ("pencil", vec!["Pencil", "PencilTool", "Crayon"]),
        ("brush", vec!["Brush", "BrushTool", "Paintbrush"]),
        ("fill", vec!["Fill", "FillTool", "Paint Bucket", "Bucket"]),
        ("text", vec!["Text", "TextTool", "A"]),
        ("eraser", vec!["Eraser", "EraserTool", "Rubber"]),
        ("color_picker", vec!["Color Picker", "Eyedropper", "Pick Color", "ColorPickerTool"]),
        ("magnifier", vec!["Magnifier", "ZoomTool", "Zoom"]),
        ("select", vec!["Select", "Selection", "SelectionTool"]),
        ("free_select", vec!["Free-Form Select", "Free Select", "FreeSelectTool"]),
        ("rectangle", vec!["Rectangle", "RectangleTool", "Square"]),
        ("ellipse", vec!["Ellipse", "EllipseTool", "Circle", "Oval"]),
        ("line", vec!["Line", "LineTool", "Straight Line"]),
        ("curve", vec!["Curve", "CurveTool", "Curved Line"]),
        ("polygon", vec!["Polygon", "PolygonTool"]),
        ("rounded_rect", vec!["Rounded Rectangle", "RoundedRectTool"]),
    ];
    
    // Try to get the tools container
    let tools_container = match get_tools_container(automation, hwnd) {
        Ok(container) => container,
        Err(_) => {
            warn!("Couldn't find tools container, falling back to searching entire window");
            // Fall back to the main window if we can't find the container
            match automation.element_from_handle((hwnd as isize).into()) {
                Ok(window) => window,
                Err(err) => {
                    error!("Failed to get Paint window element: {}", err);
                    return Err(MspMcpError::WindowsApiError(format!(
                        "Failed to get Paint window element for tool mapping: {}", err
                    )));
                }
            }
        }
    };

    // Create a true condition
    let true_condition = match automation.create_true_condition() {
        Ok(condition) => condition,
        Err(err) => {
            error!("Failed to create true condition: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to create UICondition: {}", err
            )));
        }
    };

    // Find all button elements that might be tools
    let all_elements = match tools_container.find_all(TreeScope::Subtree, &true_condition) {
        Ok(elements) => elements,
        Err(err) => {
            error!("Error finding elements: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Error finding elements: {}", err
            )));
        }
    };
    
    // Filter for button elements
    let buttons: Vec<UIElement> = all_elements.into_iter()
        .filter(|el| {
            if let Ok(control_type) = el.get_control_type() {
                control_type == ButtonControl::TYPE // Button control type
            } else {
                false
            }
        })
        .collect();
    
    info!("Found {} potential tool buttons", buttons.len());
    
    // Check each button and try to identify it as a known tool
    for button in buttons {
        if let Ok(name) = button.get_name() {
            debug!("Found button with name: {}", name);
            
            // Check if this name matches any of our known tools
            for (tool_id, possible_names) in &tool_mappings {
                let name_lower = name.to_lowercase();
                if possible_names.iter().any(|n| name_lower.contains(&n.to_lowercase())) {
                    debug!("Identified tool '{}' as '{}'", tool_id, name);
                    tool_map.insert(tool_id.to_string(), name.to_string());
                    break;
                }
            }
        }
        
        // If no name match, try automation ID
        if let Ok(id) = button.get_automation_id() {
            if !id.is_empty() {
                debug!("Button has AutomationId: {}", id);
                
                // Check if this ID matches any of our known tools
                for (tool_id, possible_names) in &tool_mappings {
                    let id_lower = id.to_lowercase();
                    if possible_names.iter().any(|n| id_lower.contains(&n.to_lowercase())) || 
                       id_lower.contains(&tool_id.to_lowercase()) {
                        debug!("Identified tool '{}' via AutomationId '{}'", tool_id, id);
                        tool_map.insert(tool_id.to_string(), id.to_string());
                        break;
                    }
                }
            }
        }
    }
    
    info!("Built tool mapping with {} identified tools", tool_map.len());
    Ok(tool_map)
}

/// Get cached or build a new mapping of tool names to their UIA identifiers
fn get_tool_mapping(automation: &UIAutomation, hwnd: HWND) -> Result<HashMap<String, String>> {
    unsafe {
        if let Some(ref cache) = TOOL_BUTTON_CACHE {
            if !cache.is_empty() {
                debug!("Using cached tool mapping with {} entries", cache.len());
                return Ok(cache.clone());
            }
        }
        
        // If no cache or empty cache, build a new mapping
        let mapping = build_tool_mapping(automation, hwnd)?;
        TOOL_BUTTON_CACHE = Some(mapping.clone());
        Ok(mapping)
    }
}

/// Find a tool button element by its name
pub fn find_tool_button(automation: &UIAutomation, hwnd: HWND, tool_name: &str) -> Result<UIElement> {
    let tool_mapping = get_tool_mapping(automation, hwnd)?;
    
    // Check if we have this tool in our mapping
    let tool_uia_name = match tool_mapping.get(tool_name) {
        Some(name) => name.clone(),
        None => {
            // If we don't have this exact tool name, try a fuzzy match
            let closest_match = tool_mapping.keys()
                .find(|k| k.contains(tool_name) || tool_name.contains(k.as_str()));
            
            match closest_match {
                Some(key) => tool_mapping[key].clone(),
                None => {
                    // If still not found, just use the tool name as is
                    warn!("Tool '{}' not found in mapping, using name directly", tool_name);
                    tool_name.to_string()
                }
            }
        }
    };
    
    info!("Looking for tool '{}' using UIA name '{}'", tool_name, tool_uia_name);
    
    // Try to get the tools container first
    let container = match get_tools_container(automation, hwnd) {
        Ok(container) => container,
        Err(_) => {
            warn!("Couldn't find tools container, searching entire window");
            match automation.element_from_handle((hwnd as isize).into()) {
                Ok(window) => window,
                Err(err) => {
                    error!("Failed to get Paint window element: {}", err);
                    return Err(MspMcpError::WindowsApiError(format!(
                        "Failed to get Paint window element: {}", err
                    )));
                }
            }
        }
    };
    
    // Create a true condition
    let true_condition = match automation.create_true_condition() {
        Ok(condition) => condition,
        Err(err) => {
            error!("Failed to create true condition: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to create UICondition: {}", err
            )));
        }
    };
    
    // Find all elements
    let all_elements = match container.find_all(TreeScope::Subtree, &true_condition) {
        Ok(elements) => elements,
        Err(err) => {
            error!("Error finding elements: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Error finding elements: {}", err
            )));
        }
    };
    
    // Filter for button elements
    let buttons: Vec<UIElement> = all_elements.into_iter()
        .filter(|el| {
            if let Ok(control_type) = el.get_control_type() {
                control_type == ButtonControl::TYPE // Button control type
            } else {
                false
            }
        })
        .collect();
    
    // Search through the buttons for our tool
    for button in buttons {
        // Check name property
        if let Ok(name) = button.get_name() {
            let name_lower = name.to_lowercase();
            let tool_lower = tool_uia_name.to_lowercase();
            
            if name_lower.contains(&tool_lower) || tool_lower.contains(&name_lower) {
                info!("Found tool button '{}' with name '{}'", tool_name, name);
                return Ok(button);
            }
        }
        
        // Check automation ID as fallback
        if let Ok(id) = button.get_automation_id() {
            if !id.is_empty() {
                let id_lower = id.to_lowercase();
                let tool_lower = tool_name.to_lowercase();
                
                if id_lower.contains(&tool_lower) || tool_lower.contains(&id_lower) {
                    info!("Found tool button '{}' with automation ID '{}'", tool_name, id);
                    return Ok(button);
                }
            }
        }
    }
    
    // If we get here, we couldn't find the tool
    warn!("Could not find tool button '{}' after searching all buttons", tool_name);
    Err(MspMcpError::ElementNotFound(format!("Tool button '{}'", tool_name)))
}

/// Select a tool in Paint using UI Automation
pub fn select_tool_uia(hwnd: HWND, tool_name: &str) -> Result<()> {
    info!("Selecting tool '{}' using UI Automation", tool_name);
    
    // Initialize UIA if needed
    let automation = initialize_uia()?;
    
    // Find the tool button
    let button = find_tool_button(&automation, hwnd, tool_name)?;
    
    // Click the button using the Invoke pattern
    match button.get_pattern::<UIInvokePattern>() {
        Ok(invoke_pattern) => {
            match invoke_pattern.invoke() {
                Ok(_) => {
                    info!("Successfully selected tool '{}' using UIA", tool_name);
                    Ok(())
                },
                Err(err) => {
                    error!("Error invoking tool button '{}': {}", tool_name, err);
                    Err(MspMcpError::WindowsApiError(format!(
                        "Error invoking tool button '{}': {}", tool_name, err
                    )))
                }
            }
        },
        Err(_) => {
            warn!("Tool button doesn't support Invoke pattern, falling back to sending space key");
            // Fall back to sending a space key which should activate the button
            match button.send_keys(" ", 10) {
                Ok(_) => {
                    info!("Sent space key to tool '{}' as fallback method", tool_name);
                    Ok(())
                },
                Err(err) => {
                    error!("Error sending keys to tool button '{}': {}", tool_name, err);
                    Err(MspMcpError::WindowsApiError(format!(
                        "Failed to activate tool button '{}': {}", tool_name, err
                    )))
                }
            }
        }
    }
}

/// Set color in Paint using UI Automation
pub fn set_color_uia(hwnd: HWND, color_hex: &str) -> Result<()> {
    info!("Setting color to '{}' using UI Automation", color_hex);
    
    // Initialize UIA
    let automation = initialize_uia()?;
    
    // Get the Paint window element
    let window = match automation.element_from_handle((hwnd as isize).into()) {
        Ok(window) => window,
        Err(err) => {
            error!("Failed to get Paint window element: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to get Paint window element: {}", err
            )));
        }
    };
    
    // Try to find the color picker section
    let matcher = automation.create_matcher()
        .from(window.clone())
        .contains_name("Colors")
        .timeout(2000);
    
    let color_section = match matcher.find_first() {
        Ok(section) => section,
        Err(_) => {
            // Try by automation ID
            let id_matcher = automation.create_matcher()
                .from(window)
                .classname("ColorPicker")
                .timeout(2000);
                
            match id_matcher.find_first() {
                Ok(section) => section,
                Err(err) => {
                    warn!("Could not find color picker UI element: {}", err);
                    return Err(MspMcpError::ElementNotFound("Color picker section".to_string()));
                }
            }
        }
    };
    
    // Create a true condition
    let true_condition = match automation.create_true_condition() {
        Ok(condition) => condition,
        Err(err) => {
            error!("Failed to create true condition: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to create UICondition: {}", err
            )));
        }
    };
    
    // Find all elements in the color section
    let all_elements = match color_section.find_all(TreeScope::Subtree, &true_condition) {
        Ok(elements) => elements,
        Err(err) => {
            error!("Error finding elements: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Error finding elements: {}", err
            )));
        }
    };
    
    // Filter for button elements
    let buttons: Vec<UIElement> = all_elements.into_iter()
        .filter(|el| {
            if let Ok(control_type) = el.get_control_type() {
                control_type == ButtonControl::TYPE // Button control type
            } else {
                false
            }
        })
        .collect();
    
    // Find the more colors button
    let more_colors_button = buttons.iter().find(|button| {
        if let Ok(name) = button.get_name() {
            let name_lower = name.to_lowercase();
            name_lower.contains("more color") || name_lower.contains("edit color")
        } else {
            false
        }
    });
    
    // Check if we found the button
    let more_colors_button = match more_colors_button {
        Some(button) => button,
        None => {
            warn!("Could not find 'More colors' button");
            return Err(MspMcpError::ElementNotFound("More colors button".to_string()));
        }
    };
    
    // Click the "More colors" button
    match more_colors_button.get_pattern::<UIInvokePattern>() {
        Ok(invoke_pattern) => {
            match invoke_pattern.invoke() {
                Ok(_) => {
                    info!("Clicked 'More colors' button");
                },
                Err(err) => {
                    error!("Error invoking 'More colors' button: {}", err);
                    return Err(MspMcpError::WindowsApiError(format!(
                        "Error invoking 'More colors' button: {}", err
                    )));
                }
            }
        },
        Err(_) => {
            // Try sending space key as a fallback
            match more_colors_button.send_keys(" ", 10) {
                Ok(_) => {
                    info!("Sent space key to 'More colors' button as fallback method");
                },
                Err(err) => {
                    error!("Error sending keys to 'More colors' button: {}", err);
                    return Err(MspMcpError::WindowsApiError(format!(
                        "Failed to activate 'More colors' button: {}", err
                    )));
                }
            }
        }
    };
    
    // Wait for the color dialog to appear
    std::thread::sleep(Duration::from_millis(500));
    
    // TODO: Implement the actual color selection using the hex value
    // This would involve finding and interacting with the RGB input fields
    
    info!("Successfully opened color dialog, but color selection not yet implemented");
    warn!("Full color selection via UI Automation not implemented yet");
    
    // Close the dialog by sending Escape key
    let window_element = automation.element_from_handle((hwnd as isize).into())
        .map_err(|e| MspMcpError::WindowsApiError(format!("Failed to get window element: {}", e)))?;
    
    // Send Escape key to close dialog
    window_element.send_keys("{ESC}", 10)
        .map_err(|e| MspMcpError::WindowsApiError(format!("Failed to send Escape key: {}", e)))?;
    
    // For now, return an "not fully implemented" error
    Err(MspMcpError::OperationNotSupported(
        "Full color selection via UI Automation not implemented yet".to_string()
    ))
}

/// Set thickness in Paint using UI Automation
pub fn set_thickness_uia(hwnd: HWND, level: u32) -> Result<()> {
    info!("Setting thickness to level {} using UI Automation", level);
    
    // Initialize UIA
    let automation = initialize_uia()?;
    
    // Get the Paint window element
    let window = match automation.element_from_handle((hwnd as isize).into()) {
        Ok(window) => window,
        Err(err) => {
            error!("Failed to get Paint window element: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to get Paint window element: {}", err
            )));
        }
    };
    
    // Try to find the thickness/size section
    let size_matcher = automation.create_matcher()
        .from(window.clone())
        .contains_name("Size")
        .timeout(2000);
    
    let thickness_section = match size_matcher.find_first() {
        Ok(section) => section,
        Err(_) => {
            // Try by automation ID
            let id_matcher = automation.create_matcher()
                .from(window)
                .classname("SizePanel")
                .timeout(2000);
                
            match id_matcher.find_first() {
                Ok(section) => section,
                Err(err) => {
                    warn!("Could not find thickness/size UI element: {}", err);
                    return Err(MspMcpError::ElementNotFound("Thickness/size section".to_string()));
                }
            }
        }
    };
    
    // Create a true condition
    let true_condition = match automation.create_true_condition() {
        Ok(condition) => condition,
        Err(err) => {
            error!("Failed to create true condition: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to create UICondition: {}", err
            )));
        }
    };
    
    // Find all elements in the thickness section
    let all_elements = match thickness_section.find_all(TreeScope::Subtree, &true_condition) {
        Ok(elements) => elements,
        Err(err) => {
            error!("Error finding elements: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Error finding elements: {}", err
            )));
        }
    };
    
    // Filter for button elements
    let buttons: Vec<UIElement> = all_elements.into_iter()
        .filter(|el| {
            if let Ok(control_type) = el.get_control_type() {
                control_type == ButtonControl::TYPE // Button control type
            } else {
                false
            }
        })
        .collect();
    
    // Parse level to index (0-based)
    let level_index = (level - 1) as usize;
    
    // Check if we have enough buttons
    if buttons.len() <= level_index {
        warn!("Not enough thickness buttons found (found {}, needed at least {})", 
              buttons.len(), level_index + 1);
        return Err(MspMcpError::ElementNotFound(format!(
            "Thickness button for level {}", level
        )));
    }
    
    // Click the appropriate button
    let button = &buttons[level_index];
    match button.get_pattern::<UIInvokePattern>() {
        Ok(invoke_pattern) => {
            match invoke_pattern.invoke() {
                Ok(_) => {
                    info!("Successfully set thickness to level {} using UIA", level);
                    Ok(())
                },
                Err(err) => {
                    error!("Error invoking thickness button: {}", err);
                    Err(MspMcpError::WindowsApiError(format!(
                        "Error invoking thickness button: {}", err
                    )))
                }
            }
        },
        Err(_) => {
            // Try sending space key as fallback
            match button.send_keys(" ", 10) {
                Ok(_) => {
                    info!("Successfully set thickness to level {} by sending space key", level);
                    Ok(())
                },
                Err(err) => {
                    error!("Error sending keys to thickness button: {}", err);
                    Err(MspMcpError::WindowsApiError(format!(
                        "Failed to activate thickness button: {}", err
                    )))
                }
            }
        }
    }
}

/// Set fill type in Paint using UI Automation
pub fn set_fill_uia(hwnd: HWND, fill_type: &str) -> Result<()> {
    info!("Setting fill type to '{}' using UI Automation", fill_type);
    
    // Initialize UIA
    let automation = initialize_uia()?;
    
    // Get the Paint window element
    let window = match automation.element_from_handle((hwnd as isize).into()) {
        Ok(window) => window,
        Err(err) => {
            error!("Failed to get Paint window element: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to get Paint window element: {}", err
            )));
        }
    };
    
    // Try to find the fill section
    let fill_matcher = automation.create_matcher()
        .from(window.clone())
        .contains_name("Fill")
        .timeout(2000);
    
    let fill_section = match fill_matcher.find_first() {
        Ok(section) => section,
        Err(_) => {
            // Try by automation ID
            let id_matcher = automation.create_matcher()
                .from(window)
                .classname("FillPanel")
                .timeout(2000);
                
            match id_matcher.find_first() {
                Ok(section) => section,
                Err(err) => {
                    warn!("Could not find fill UI element: {}", err);
                    return Err(MspMcpError::ElementNotFound("Fill section".to_string()));
                }
            }
        }
    };
    
    // Create a true condition
    let true_condition = match automation.create_true_condition() {
        Ok(condition) => condition,
        Err(err) => {
            error!("Failed to create true condition: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to create UICondition: {}", err
            )));
        }
    };
    
    // Find all elements in the fill section
    let all_elements = match fill_section.find_all(TreeScope::Subtree, &true_condition) {
        Ok(elements) => elements,
        Err(err) => {
            error!("Error finding elements: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Error finding elements: {}", err
            )));
        }
    };
    
    // Filter for button elements
    let buttons: Vec<UIElement> = all_elements.into_iter()
        .filter(|el| {
            if let Ok(control_type) = el.get_control_type() {
                control_type == ButtonControl::TYPE // Button control type
            } else {
                false
            }
        })
        .collect();
    
    // Map fill type to expected button names or descriptions
    let button_name = match fill_type.to_lowercase().as_str() {
        "none" => "No fill",
        "solid" => "Solid color",
        "outline" => "Outline",
        _ => return Err(MspMcpError::InvalidParameters(format!(
            "Invalid fill type: '{}'. Must be 'none', 'solid', or 'outline'", fill_type
        ))),
    };
    
    // Find the appropriate button by name or ID
    let target_button = buttons.iter().find(|button| {
        // Check name
        if let Ok(name) = button.get_name() {
            let name_lower = name.to_lowercase();
            let button_name_lower = button_name.to_lowercase();
            if name_lower.contains(&button_name_lower) {
                return true;
            }
        }
        
        // Check automation ID
        if let Ok(id) = button.get_automation_id() {
            if !id.is_empty() {
                let id_lower = id.to_lowercase();
                let fill_type_lower = fill_type.to_lowercase();
                if id_lower.contains(&fill_type_lower) {
                    return true;
                }
            }
        }
        
        false
    });
    
    // Check if we found a button
    match target_button {
        Some(button) => {
            // Click the button
            match button.get_pattern::<UIInvokePattern>() {
                Ok(invoke_pattern) => {
                    match invoke_pattern.invoke() {
                        Ok(_) => {
                            info!("Successfully set fill type to '{}' using UIA", fill_type);
                            Ok(())
                        },
                        Err(err) => {
                            error!("Error invoking fill button: {}", err);
                            Err(MspMcpError::WindowsApiError(format!(
                                "Error invoking fill button: {}", err
                            )))
                        }
                    }
                },
                Err(_) => {
                    // Try sending space key as fallback
                    match button.send_keys(" ", 10) {
                        Ok(_) => {
                            info!("Successfully set fill type to '{}' by sending space key", fill_type);
                            Ok(())
                        },
                        Err(err) => {
                            error!("Error sending keys to fill button: {}", err);
                            Err(MspMcpError::WindowsApiError(format!(
                                "Failed to activate fill button: {}", err
                            )))
                        }
                    }
                }
            }
        },
        None => {
            warn!("Could not find button for fill type '{}'", fill_type);
            Err(MspMcpError::ElementNotFound(format!(
                "Button for fill type '{}'", fill_type
            )))
        }
    }
}

/// Draw a shape in Paint using UI Automation
pub fn draw_shape_uia(hwnd: HWND, shape_type: &str, start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> Result<()> {
    info!("Drawing shape '{}' from ({},{}) to ({},{}) using UI Automation", shape_type, start_x, start_y, end_x, end_y);
    
    // Initialize UIA
    let automation = initialize_uia()?;
    
    // Get the Paint window element
    let window = match automation.element_from_handle((hwnd as isize).into()) {
        Ok(window) => window,
        Err(err) => {
            error!("Failed to get Paint window element: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to get Paint window element: {}", err
            )));
        }
    };
    
    // Validate shape type
    let valid_shapes = ["rectangle", "ellipse", "line", "arrow", "triangle", "pentagon", "hexagon"];
    if !valid_shapes.contains(&shape_type.to_lowercase().as_str()) {
        return Err(MspMcpError::InvalidParameters(
            format!("Invalid shape type: {}. Must be one of: rectangle, ellipse, line, arrow, triangle, pentagon, hexagon", 
                    shape_type)));
    }
    
    // First, select the shape tool
    // Try using the direct approach to click the Shapes button in the ribbon
    info!("Selecting shape tool from ribbon");
    
    // First activate the Paint window
    windows::activate_paint_window(hwnd)?;
    
    // Find the "Home" tab or main ribbon area
    let ribbon_matcher = automation.create_matcher()
        .from(window.clone())
        .control_type(PaneControl::TYPE)
        .timeout(2000);
        
    let ribbon = match ribbon_matcher.find_first() {
        Ok(ribbon) => ribbon,
        Err(err) => {
            warn!("Could not find ribbon: {}", err);
            return Err(MspMcpError::ElementNotFound("Ribbon section".to_string()));
        }
    };
    
    // Create a true condition
    let true_condition = match automation.create_true_condition() {
        Ok(condition) => condition,
        Err(err) => {
            error!("Failed to create true condition: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to create UICondition: {}", err
            )));
        }
    };
    
    // Find all buttons in the ribbon
    let all_elements = match ribbon.find_all(TreeScope::Subtree, &true_condition) {
        Ok(elements) => elements,
        Err(err) => {
            error!("Error finding elements: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Error finding elements: {}", err
            )));
        }
    };
    
    // Look for the "Shapes" button 
    info!("Searching for Shapes button among {} elements", all_elements.len());
    let shapes_button = all_elements.into_iter()
        .filter(|el| {
            if let Ok(control_type) = el.get_control_type() {
                if control_type != ButtonControl::TYPE {
                    return false;
                }
                
                // Check for "Shapes" in name or automation ID
                if let Ok(name) = el.get_name() {
                    let name_lower = name.to_lowercase();
                    if name_lower.contains("shape") {
                        info!("Found button named: {}", name);
                        return true;
                    }
                }
                
                if let Ok(id) = el.get_automation_id() {
                    let id_lower = id.to_lowercase();
                    if id_lower.contains("shape") {
                        info!("Found button with ID: {}", id);
                        return true;
                    }
                }
            }
            false
        })
        .next();
    
    // Click the shapes button if found
    if let Some(button) = shapes_button {
        match button.get_pattern::<UIInvokePattern>() {
            Ok(invoke_pattern) => {
                match invoke_pattern.invoke() {
                    Ok(_) => {
                        info!("Clicked Shapes button successfully");
                    },
                    Err(err) => {
                        error!("Error invoking Shapes button: {}", err);
                        return Err(MspMcpError::WindowsApiError(format!(
                            "Error invoking Shapes button: {}", err
                        )));
                    }
                }
            },
            Err(_) => {
                // Try sending space key as fallback
                match button.send_keys(" ", 10) {
                    Ok(_) => {
                        info!("Activated Shapes button with space key");
                    },
                    Err(err) => {
                        error!("Error sending keys to Shapes button: {}", err);
                        return Err(MspMcpError::WindowsApiError(format!(
                            "Failed to activate Shapes button: {}", err
                        )));
                    }
                }
            }
        }
        
        // Wait for the shapes dropdown to appear
        std::thread::sleep(Duration::from_millis(500));
    } else {
        // If we couldn't find the Shapes button, try using keyboard shortcuts
        info!("Shapes button not found, using fallback keyboard method");
        
        // First, activate the Paint window (again to be sure)
        windows::activate_paint_window(hwnd)?;
        
        // Alt+H to access Home tab, then S for Shapes, then Down Arrow
        window.send_keys("%h", 100)?; // Alt+H
        std::thread::sleep(Duration::from_millis(300));
        window.send_keys("s", 100)?; // S for Shapes
        std::thread::sleep(Duration::from_millis(300));
    }
    
    // Now the shapes dropdown should be open - select rectangle or specific shape
    // First, use arrow keys to navigate to the right shape
    let shape_index = match shape_type.to_lowercase().as_str() {
        "rectangle" => 0, // First shape
        "ellipse" => 1,   // Second shape
        "line" => 7,      // Eighth shape
        "arrow" => 9,     // Tenth shape
        _ => 0,           // Default to rectangle
    };
    
    // Press down arrow key shape_index times
    info!("Selecting shape {} using keyboard navigation", shape_type);
    for _ in 0..shape_index {
        window.send_keys("{DOWN}", 50)?;
        std::thread::sleep(Duration::from_millis(100));
    }
    
    // Enter to select the shape
    window.send_keys("{ENTER}", 100)?;
    std::thread::sleep(Duration::from_millis(300));
    
    // Now draw the shape by finding the canvas and performing mouse actions
    info!("Finding canvas element to draw shape");
    
    // Get the canvas element
    let canvas_matcher = automation.create_matcher()
        .from(window.clone())
        .timeout(3000);
        
    let elements = match canvas_matcher.find_all() {
        Ok(elements) => elements,
        Err(err) => {
            error!("Failed to find elements: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to find elements: {}", err
            )));
        }
    };
    
    // Find the canvas - it's typically the largest pane element
    let canvas = elements.into_iter()
        .filter(|el| {
            if let Ok(control_type) = el.get_control_type() {
                return control_type == PaneControl::TYPE;
            }
            false
        })
        .filter(|el| {
            // Filter by name or role if possible
            if let Ok(name) = el.get_name() {
                return name.contains("Canvas") || name.contains("Drawing");
            }
            true
        })
        .max_by_key(|el| {
            // Get the element with the largest area (likely the canvas)
            if let Ok(rect) = el.get_bounding_rectangle() {
                let width = rect.get_right() - rect.get_left();
                let height = rect.get_bottom() - rect.get_top();
                width * height
            } else {
                0
            }
        });
    
    // Fallback to the main window if we can't find the canvas
    let canvas = match canvas {
        Some(canvas) => canvas,
        None => {
            warn!("Could not find canvas element, using main window");
            window
        }
    };
    
    // Get canvas bounds
    let bounds = match canvas.get_bounding_rectangle() {
        Ok(bounds) => bounds,
        Err(err) => {
            error!("Failed to get canvas bounds: {}", err);
            return Err(MspMcpError::WindowsApiError(format!(
                "Failed to get canvas bounds: {}", err
            )));
        }
    };
    
    info!("Canvas bounds: left={}, top={}, right={}, bottom={}", 
          bounds.get_left(), bounds.get_top(), bounds.get_right(), bounds.get_bottom());
    
    // Convert our coordinates to be relative to the canvas
    let canvas_x = bounds.get_left();
    let canvas_y = bounds.get_top();
    
    // Adjust coordinates to be within canvas bounds
    let adjusted_start_x = canvas_x + start_x;
    let adjusted_start_y = canvas_y + start_y;
    let adjusted_end_x = canvas_x + end_x;
    let adjusted_end_y = canvas_y + end_y;
    
    info!("Drawing from ({},{}) to ({},{}) in screen coordinates", 
          adjusted_start_x, adjusted_start_y, adjusted_end_x, adjusted_end_y);
    
    // Now use the windows API to directly manipulate the mouse
    // This is more reliable than sending keyboard events for exact positioning
    windows::move_mouse_to(adjusted_start_x, adjusted_start_y)?;
    std::thread::sleep(Duration::from_millis(300));
    
    // Mouse down
    windows::send_mouse_down()?;
    std::thread::sleep(Duration::from_millis(300));
    
    // Move to end position
    windows::move_mouse_to(adjusted_end_x, adjusted_end_y)?;
    std::thread::sleep(Duration::from_millis(300));
    
    // Mouse up
    windows::send_mouse_up()?;
    
    info!("Successfully drew shape '{}' from ({},{}) to ({},{}) using UIA", 
          shape_type, start_x, start_y, end_x, end_y);
    Ok(())
} 