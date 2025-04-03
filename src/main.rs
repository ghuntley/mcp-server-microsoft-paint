use mcp_server_microsoft_paint::run_server;
use std::process;
use std::sync::Once;
use log::{error, info, LevelFilter};
use simplelog::{CombinedLogger, WriteLogger, TermLogger, TerminalMode, ColorChoice, ConfigBuilder};
use std::fs::File;
use std::path::PathBuf;
use std::env;

// Use a Once to ensure we only initialize the logger once
static LOGGER_INIT: Once = Once::new();

fn main() {
    // Initialize logger exactly once
    LOGGER_INIT.call_once(|| {
        let log_level = LevelFilter::Debug; // Log debug level and above
        let log_file_path = env::temp_dir().join("mcp_server_debug.log");

        if let Ok(log_file) = File::create(&log_file_path) {
            let config = ConfigBuilder::new()
                .set_time_format_rfc3339()
                .build();

            let write_logger = WriteLogger::new(
                log_level, 
                config.clone(), 
                log_file
            );
            
            // Also log Info level to terminal
            let term_logger = TermLogger::new(
                LevelFilter::Info, 
                config, 
                TerminalMode::Mixed,
                ColorChoice::Auto
            );

            if let Err(e) = CombinedLogger::init(vec![term_logger, write_logger]) {
                eprintln!("Failed to initialize combined logger: {}", e); // Fallback
            }
            
            info!("Logging initialized. Debug logs writing to: {:?}", log_file_path);

        } else {
            eprintln!("Failed to create log file at {:?}, logging to terminal only.", log_file_path);
            if let Err(e) = TermLogger::init(log_level, ConfigBuilder::new().build(), TerminalMode::Mixed, ColorChoice::Auto) {
                 eprintln!("Failed to initialize terminal logger: {}", e); // Fallback
            }
        }
    });
    
    // Use log::info! for startup message
    info!("Starting MCP Server for Windows 11 Paint...");
    
    // Run the MCP server and handle any errors
    match run_server() {
        Ok(_) => {
            // Use log::info! for successful termination
            info!("MCP server terminated successfully");
            process::exit(0);
        }
        Err(e) => {
            // Use log::error! for general error
            error!("Error running MCP server: {}", e);
            
            // Use log::error! for specific error details
            if e.to_string().contains("WindowNotFound") {
                error!("Could not find or launch MS Paint application.");
                error!("Please make sure MS Paint is installed and accessible.");
            } else if e.to_string().contains("IoError") {
                error!("I/O error encountered, possibly with stdin/stdout communication.");
                error!("Make sure the server has access to standard input and output streams.");
            }
            process::exit(1);
        }
    }
} 