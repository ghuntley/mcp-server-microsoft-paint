use mcp_server_microsoft_paint::PaintServerState;
use mcp_rust_sdk::server::ServerHandler;
use mcp_rust_sdk::transport::stdio::StdioTransport;
use std::process;
use log::{info, error, debug};
use simplelog::{CombinedLogger, Config, ConfigBuilder, TermLogger, WriteLogger, TerminalMode, ColorChoice, LevelFilter};
use std::fs::File;
use std::sync::Once;
use std::path::PathBuf;
use std::env;
use std::io;
use serde_json;

// Use a Once to ensure we only initialize the logger once
static LOGGER_INIT: Once = Once::new();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    init_logger();
    
    info!("Starting MCP Server for Windows 11 Paint...");
    
    // Print version information
    let version = env!("CARGO_PKG_VERSION");
    info!("MCP Server version: {}", version);
    
    // Run the JSON-RPC server
    run_server_async().await?;
    
    info!("MCP Server shutting down");
    Ok(())
}

// The main run loop for the server
async fn run_server_async() -> Result<(), Box<dyn std::error::Error>> {
    info!("MCP Server starting run loop...");

    // Create the Paint server state
    let paint_server = PaintServerState::new();

    let mut buffer = String::new();
    
    loop {
        // Reset the buffer for the next request
        buffer.clear();
        
        // Read a line from stdin
        match io::stdin().read_line(&mut buffer) {
            Ok(0) => {
                // End of input (Ctrl+D or stream closed)
                info!("End of input - server shutting down");
                break;
            }
            Ok(_) => {
                // Process the received JSON-RPC request
                if let Some(parsed_request) = parse_json_rpc_request(&buffer) {
                    // If parsing successful, handle the request
                    info!("Received request: {}", parsed_request.trim());
                    
                    // Extract method and params
                    match extract_method_and_params(&parsed_request) {
                        Ok((method, params, id)) => {
                            // Handle the method call
                            debug!("Handling method: {}, params: {:?}", method, params);
                            
                            let result = paint_server.clone().handle_method(&method, params).await;
                            
                            // Send the result back as a JSON-RPC response
                            match result {
                                Ok(response) => {
                                    // Make sure the response has the correct ID
                                    let mut response_obj = response.as_object().unwrap_or(&serde_json::Map::new()).clone();
                                    response_obj.insert("id".to_string(), id);
                                    
                                    if !response_obj.contains_key("jsonrpc") {
                                        response_obj.insert("jsonrpc".to_string(), serde_json::Value::String("2.0".to_string()));
                                    }
                                    
                                    let response_json = serde_json::to_string(&response_obj)?;
                                    println!("{}", response_json);
                                }
                                Err(e) => {
                                    let error_response = serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "id": id,
                                        "error": {
                                            "code": -32603, // Internal error
                                            "message": e.to_string()
                                        }
                                    });
                                    println!("{}", serde_json::to_string(&error_response)?);
                                }
                            }
                        }
                        Err(e) => {
                            let error_response = serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": null,
                                "error": {
                                    "code": -32600, // Invalid request
                                    "message": e
                                }
                            });
                            println!("{}", serde_json::to_string(&error_response)?);
                        }
                    }
                }
            }
            Err(e) => {
                // Handle read errors
                error!("Error reading from stdin: {}", e);
                break;
            }
        }
    }

    Ok(())
}

// Parse a string as a JSON-RPC request
fn parse_json_rpc_request(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    
    match serde_json::from_str::<serde_json::Value>(trimmed) {
        Ok(json) => {
            // Just verify this is an object - more detailed checking
            // happens in extract_method_and_params
            if json.is_object() {
                Some(trimmed.to_string())
            } else {
                error!("Invalid JSON-RPC request: Not an object");
                None
            }
        }
        Err(e) => {
            error!("Failed to parse JSON-RPC request: {}", e);
            None
        }
    }
}

// Extract method and params from JSON-RPC request
fn extract_method_and_params(request_str: &str) -> Result<(String, Option<serde_json::Value>, serde_json::Value), String> {
    // Parse the request
    let request: serde_json::Value = serde_json::from_str(request_str)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    // Check this is a JSON-RPC 2.0 request object
    let obj = request.as_object()
        .ok_or_else(|| "Request must be a JSON object".to_string())?;
    
    // Extract the JSON-RPC version (optional check)
    if let Some(version) = obj.get("jsonrpc") {
        if version != "2.0" {
            return Err("Only JSON-RPC 2.0 is supported".to_string());
        }
    }
    
    // Extract the method
    let method = obj.get("method")
        .ok_or_else(|| "Missing 'method' field".to_string())?
        .as_str()
        .ok_or_else(|| "'method' must be a string".to_string())?
        .to_string();
    
    // Extract the params (optional)
    let params = obj.get("params").cloned();
    
    // Extract the id (or use default)
    let id = obj.get("id").unwrap_or(&serde_json::Value::Null).clone();
    
    Ok((method, params, id))
}

// Initialize the logger
fn init_logger() {
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
            
            // Log to stderr instead of stdout to avoid interfering with JSON-RPC
            let term_logger = TermLogger::new(
                LevelFilter::Info, 
                config, 
                TerminalMode::Stderr, // Use Stderr instead of Mixed mode
                ColorChoice::Auto
            );

            if let Err(e) = CombinedLogger::init(vec![term_logger, write_logger]) {
                eprintln!("Failed to initialize combined logger: {}", e); // Fallback
            }
            
            info!("Logging initialized. Debug logs writing to: {:?}", log_file_path);

        } else {
            eprintln!("Failed to create log file at {:?}, logging to stderr only.", log_file_path);
            if let Err(e) = TermLogger::init(
                log_level, 
                ConfigBuilder::new().build(), 
                TerminalMode::Stderr, // Use Stderr instead of Mixed mode
                ColorChoice::Auto
            ) {
                 eprintln!("Failed to initialize terminal logger: {}", e); // Fallback
            }
        }
    });
} 