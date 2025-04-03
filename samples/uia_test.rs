// Sample file to test uiautomation crate
use uiautomation::UIAutomation;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing uiautomation with MS Paint");
    
    // Launch MS Paint
    let proc = std::process::Command::new("mspaint.exe").spawn()?;
    
    // Wait for Paint to start
    sleep(Duration::from_secs(2));
    
    // Initialize UI Automation
    let automation = UIAutomation::new()?;
    let root = automation.get_root_element()?;
    
    // Find Paint window
    let matcher = automation.create_matcher()
        .from(root)
        .timeout(10000)
        .classname("MSPaintApp");
    
    match matcher.find_first() {
        Ok(paint_window) => {
            println!("Found Paint window: {} - {}", 
                     paint_window.get_name()?, 
                     paint_window.get_classname()?);
            
            // Try to find toolbar elements
            if let Ok(buttons) = paint_window.find_all_by_control_type(&automation, uiautomation::ElementType::Button) {
                println!("Found {} buttons in Paint", buttons.len());
                
                // List button names
                for button in buttons {
                    if let Ok(name) = button.get_name() {
                        println!("Button: {}", name);
                    }
                }
            }
            
            // Close Paint
            if let Ok(control) = paint_window.get_pattern::<uiautomation::patterns::WindowPattern>(&automation) {
                control.close()?;
            }
        },
        Err(err) => {
            println!("Could not find Paint window: {}", err);
        }
    }
    
    Ok(())
} 