use mcp_rust_sdk::{
    server::{ServerHandler, Server}, 
    transport::stdio::StdioTransport,
    types::{ClientCapabilities, ServerCapabilities, Implementation},
    Error as SdkError, error::ErrorCode
};
use log::{info, error, LevelFilter, debug, warn};
use tokio::runtime::Runtime;
use std::sync::Arc;
use std::sync::Mutex;
use windows_sys::Win32::Foundation::HWND;
use std::process::Command;
use std::io::{self, Write};

// Define modules
pub mod error;
pub mod protocol;
pub mod windows;
pub mod core;

use crate::error::{Result, MspMcpError};

// Helper function to log process tree (Windows specific for now)
fn log_process_tree(label: &str) {
    if cfg!(target_os = "windows") {
        debug!("Capturing process tree ({}) using tasklist...", label);
        match Command::new("tasklist").arg("/V").output() {
            Ok(output) => {
                if output.status.success() {
                    match String::from_utf8(output.stdout) {
                        Ok(stdout_str) => debug!("Process Tree ({}):\n{}", label, stdout_str),
                        Err(e) => warn!("Failed to decode tasklist stdout: {}", e),
                    }
                } else {
                    warn!("tasklist command failed with status: {}", output.status);
                    if let Ok(stderr_str) = String::from_utf8(output.stderr) {
                        warn!("tasklist stderr:\n{}", stderr_str);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to execute tasklist command: {}", e);
            }
        }
    } else {
        debug!("Process tree logging not implemented for this OS.");
    }
}

// Define a struct to hold our server state
#[derive(Clone)]
pub struct PaintServerState {
    paint_hwnd: Arc<Mutex<Option<HWND>>>, // Store HWND in Arc<Mutex>
}

// Implement the server handler trait from mcp_rust_sdk
#[async_trait::async_trait]
impl ServerHandler for PaintServerState {

    // Required method: initialize
    async fn initialize(&self, _implementation: Implementation, _client_capabilities: ClientCapabilities) -> std::result::Result<ServerCapabilities, SdkError> {
        info!("Server received initialize request. Finding/Launching Paint...");

        // --- Log process tree BEFORE attempting launch ---
        log_process_tree("Before Paint Find/Launch");
        // -----------------------------------------------

        // --- Start: Logic moved from handle_connect ---
        match crate::windows::get_paint_hwnd() {
            Ok(hwnd) => {
                 // Store the HWND in the shared state
                let mut hwnd_state = self.paint_hwnd.lock()
                    .map_err(|_| SdkError::protocol(ErrorCode::InternalError, "Failed to lock HWND state".to_string()))?;
                *hwnd_state = Some(hwnd);
                info!("Stored Paint HWND: {}", hwnd);

                // --- Log process tree AFTER successful find/launch ---
                log_process_tree("After Paint Find/Launch");
                // -----------------------------------------------------
            }
            Err(e) => {
                // Log process tree on failure too
                log_process_tree("After Paint Find/Launch Failure");

                // If we can't get the HWND during init, it's a fatal error for this server
                let error_msg = format!("Failed to find or launch Paint during initialization: {}", e);
                error!("{}", error_msg);
                // Convert our error to an SdkError for the initialize response
                return Err(SdkError::protocol(ErrorCode::InternalError, error_msg));
            }
        }
        // --- End: Logic moved from handle_connect ---
        
        // Return default capabilities (or customize later if needed)
        info!("Paint found/launched. Initialization successful.");
        Ok(ServerCapabilities::default())
    }

    // Required method: shutdown
    async fn shutdown(&self) -> std::result::Result<(), SdkError> {
        info!("Server received shutdown request.");
        // TODO: Perform cleanup if necessary
        Ok(())
    }

    // Required method: handle_method
    async fn handle_method(&self, method: &str, params: Option<serde_json::Value>) -> std::result::Result<serde_json::Value, SdkError> {
        info!("Handling method: {} with params: {:?}", method, params);

        // Route request to appropriate async handler in `core` module
        // Pass the cloned state to the handler
        let result: std::result::Result<serde_json::Value, MspMcpError> = match method {
            "connect" => {
                core::handle_connect(self.clone(), params).await
            }
            "disconnect" => {
                core::handle_disconnect(self.clone(), params).await
            }
            "get_version" => {
                core::handle_get_version(self.clone(), params).await
            }
            "activate_window" => {
                core::handle_activate_window(self.clone(), params).await
            }
            "get_canvas_dimensions" => {
                core::handle_get_canvas_dimensions(self.clone(), params).await
            }
            "draw_pixel" => {
                core::handle_draw_pixel(self.clone(), params).await
            }
            "draw_line" => {
                core::handle_draw_line(self.clone(), params).await
            }
            // Add other method handlers here, calling functions in core.rs
            _ => {
                Err(MspMcpError::OperationNotSupported(format!("Method '{}' not implemented", method)))
            }
        };

        // Convert our Result<Value, MspMcpError> to Result<Value, SdkError>
        match result {
            Ok(value) => Ok(value),
            Err(msp_error) => {
                let code = msp_error.code(); // Keep our internal code for logging
                let message = msp_error.to_string();
                error!("Error processing method '{}': Code {}, Message: {}", method, code, message);
                // Map our internal error to SDK's Protocol error using a standard JSON-RPC code
                Err(SdkError::Protocol {
                    code: ErrorCode::InternalError, // Use standard InternalError code (-32603)
                    message,
                    data: None,
                })
            }
        }
    }
}

// Main entry point function
pub fn run_server() -> Result<()> {
    // Remove the env_logger initialization since we're using simplelog now
    // env_logger::Builder::from_default_env()
    //    .filter_level(LevelFilter::Info)
    //    .try_init().map_err(|e| MspMcpError::General(format!("Failed to init logger: {}", e)))?;

    info!("Starting MCP Server for Windows 11 Paint (Async Version)...");

    let rt = Runtime::new().map_err(|e| MspMcpError::IoError(e))?;

    rt.block_on(async {
        let initial_state = PaintServerState {
            paint_hwnd: Arc::new(Mutex::new(None)),
        };
        let (transport, _handler_connection) = StdioTransport::new(); // handler_connection might not be needed here

        let handler = Arc::new(initial_state);
        let transport_arc = Arc::new(transport);

        // Correct Server::new call (takes transport and handler)
        let server = Server::new(transport_arc.clone(), handler.clone());

        info!("MCP Server starting run loop...");

        // Use server.start() to run the server loop
        if let Err(e) = server.start().await {
            error!("MCP Server run failed: {}", e);
            // Attempt to downcast the SDK error or format it
            let error_message = format!("Server run failed: {}", e);
            return Err(MspMcpError::General(error_message));
        }

        info!("MCP Server finished.");
        Ok(())
    })
}


#[cfg(test)]
mod tests {
    // use super::*; // Keep this if testing functions within lib.rs directly

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    // Tests for async functions require #[tokio::test]
    // Add tests for core handlers in core.rs
    // Add tests for protocol structs in protocol.rs
    // Add tests for windows functions in windows.rs
}
