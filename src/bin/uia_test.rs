// Sample file to test uiautomation crate
use uiautomation::{UIAutomation, types::TreeScope};
use uiautomation::controls::{Control, ButtonControl};
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
            // Create a condition for button type
            let true_condition = automation.create_true_condition()?;
            
            // Find all elements
            let all_elements = paint_window.find_all(TreeScope::Subtree, &true_condition)?;
            
            // Filter for button elements
            let buttons: Vec<_> = all_elements.into_iter()
                .filter(|el| {
                    if let Ok(control_type) = el.get_control_type() {
                        control_type == ButtonControl::TYPE
                    } else {
                        false
                    }
                })
                .collect();
                
            println!("Found {} buttons in Paint", buttons.len());
            
            // List button names
            for button in &buttons {
                if let Ok(name) = button.get_name() {
                    println!("Button: {}", name);
                }
            }
            
            // Close Paint
            if let Ok(control) = paint_window.get_pattern::<uiautomation::patterns::UIWindowPattern>() {
                control.close()?;
            }
        },
        Err(err) => {
            println!("Could not find Paint window: {}", err);
        }
    }
    
    Ok(())
} 