#!/usr/bin/env python
import json
import subprocess
import sys
import time
import logging
import os
import threading
import re

def setup_logging():
    log_formatter = logging.Formatter("%(asctime)s [%(levelname)-5.5s] %(message)s")
    root_logger = logging.getLogger()
    root_logger.setLevel(logging.DEBUG)

    # Console Handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setFormatter(log_formatter)
    root_logger.addHandler(console_handler)
    logging.info("Logging initialized for test_client.py")

def send_request(server_process, request):
    """Send a JSON-RPC request to the server and return the response."""
    request_id = request.get("id", "N/A")
    method_name = request.get("method", "Unknown")
    
    # --- Experimental Change: Add a top-level "type" field --- 
    request_to_send = request.copy()
    if "jsonrpc" in request_to_send and "method" in request_to_send:
        request_to_send["type"] = "request" # Add the dummy type field
    # --- End Experimental Change ---
    
    logging.debug(f"Sending request (ID: {request_id}, Method: {method_name}): {request_to_send}") # Log the modified request
    request_json = json.dumps(request_to_send) + "\n"
    try:
        # Send as string in text mode
        server_process.stdin.write(request_json) 
        server_process.stdin.flush()
        logging.debug("Request sent successfully (text mode).")
    except Exception as e:
        logging.error(f"Failed to send request to server: {e}")
        return None
    
    # Read response (should already handle text mode correctly)
    response_line = None
    response_json = None
    start_time = time.time()
    timeout = 20 # Increased timeout for potentially slower operations
    logging.debug(f"Waiting for response (timeout: {timeout}s)...")
    
    # This regex pattern helps identify JSON-RPC responses vs log lines
    json_pattern = r'^\s*\{\s*"jsonrpc"\s*:\s*"2\.0"'
    
    while time.time() - start_time < timeout:
         # Check if server has exited
        if server_process.poll() is not None:
            logging.warning(f"Server exited prematurely while waiting for response (code: {server_process.poll()})")
            return None
            
        try:
            # Read line as string in text mode
            line = server_process.stdout.readline()
            if not line:
                time.sleep(0.1) # Avoid busy-waiting if no data
                continue
                
            line = line.strip()
            
            # Skip empty lines
            if not line:
                continue
                
            # If the line looks like a log message (starts with timestamp), just log it
            if line.startswith("20") and "[" in line[:30]:
                logging.debug(f"Server log message: {line}")
                continue
                
            # Check if line looks like JSON
            if line.startswith("{") and "jsonrpc" in line:
                logging.debug(f"Found JSON response: {line}")
                try:
                    response_json = json.loads(line)
                    # Validate this is the response to our request
                    if "id" in response_json and response_json["id"] == request_id:
                        logging.debug("Parsed matching JSON response successfully.")
                        break
                    else:
                        logging.debug(f"Found JSON but ID {response_json.get('id')} doesn't match expected {request_id}")
                except json.JSONDecodeError:
                    logging.error(f"Error decoding what looked like JSON: {line}")
            else:
                logging.debug(f"Skipping non-JSON line: {line}")
                
        except json.JSONDecodeError:
            logging.error(f"Error decoding JSON response: {line}")
        except Exception as e:
            logging.error(f"Error reading response from server: {e}")
            return None
            
    if response_json is None:
         logging.warning(f"No valid JSON response received for request ID {request_id} within timeout.")
         
    return response_json

def send_notification(server_process, notification):
    """Sends a JSON-RPC notification without waiting for a response."""
    method_name = notification.get("method", "Unknown")
    logging.debug(f"Sending notification (Method: {method_name}): {notification}")
    # Add the experimental "type" field if needed (keeping consistent for now)
    notification_to_send = notification.copy()
    if "jsonrpc" in notification_to_send and "method" in notification_to_send:
        notification_to_send["type"] = "notification" # Technically more accurate type
        
    notification_json = json.dumps(notification_to_send) + "\n"
    try:
        # Send as string in text mode
        server_process.stdin.write(notification_json) 
        server_process.stdin.flush()
        logging.debug("Notification sent successfully (text mode).")
    except Exception as e:
        logging.error(f"Failed to send notification to server: {e}")

def main():
    setup_logging()
    server_process = None
    try:
        logging.info("Launching MCP server in TEXT mode...")
        
        # Set environment variables to control Rust logging
        env = os.environ.copy()
        env["RUST_LOG"] = "debug" # Set to info to reduce debug spam
        
        # Launch the server with text=True and stderr piped again
        server_process = subprocess.Popen(
            ["target/release/mcp-server-microsoft-paint.exe"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE, # Pipe stderr again to avoid it mixing with stdout
            text=True, 
            encoding='utf-8',
            env=env # Pass custom environment
        )
        logging.info(f"Server process started with PID: {server_process.pid}")
        
        # Every 10 lines of stderr, print a summary to avoid overwhelming the console
        stderr_thread = threading.Thread(
            target=_log_stderr, 
            args=(server_process.stderr,),
            daemon=True
        )
        stderr_thread.start()
        
        # Give the server a moment to initialize
        logging.debug("Waiting for server process to start (2 seconds)...") 
        time.sleep(2)

        # --- Send Initialize Request (Standard MCP) ---
        logging.info("Sending 'initialize' request...")
        initialize_request = {
            "jsonrpc": "2.0",
            "id": 0, # Often use 0 or a distinct ID for initialize
            "method": "initialize",
            "params": {
                "processId": os.getpid() if hasattr(os, 'getpid') else None, # Optional client process ID
                "clientInfo": { # Optional client info
                    "name": "test_client.py",
                    "version": "0.1.0"
                },
                "capabilities": {}, # Placeholder for client capabilities
                # Provide the expected 'name' field within implementation
                "implementation": {"name": "DummyClientImplementation", "version": "0.0.1"} 
            }
        }
        initialize_response = send_request(server_process, initialize_request)
        logging.info(f"Initialize response: {initialize_response}")

        if not initialize_response or initialize_response.get("error"): 
            # Handle cases where the response is None or contains an error field
            error_details = initialize_response.get("error", "Unknown error") if initialize_response else "No response"
            logging.error(f"Initialize failed: {error_details}, aborting tests.")
            # Attempt to log server stderr before exiting
            if server_process:
                try:
                    # Try non-blocking read first
                    stderr_data = server_process.stderr.read() if server_process.stderr else b''
                    if not stderr_data:
                         # If nothing read, try communicate with timeout
                        _, stderr_data = server_process.communicate(timeout=1)
                    if stderr_data:
                        logging.error(f"Server stderr on initialize failure:\n{stderr_data.decode('utf-8', errors='replace')}")
                except Exception as e:
                    logging.error(f"Failed to get stderr after initialize failure: {e}")
            return
        
        # --- Add Delay After Initialize ---
        logging.debug("Waiting 1 second after initialize response...")
        time.sleep(1)
        # --- End Delay ---

        # --- Send Initialized Notification --- 
        logging.info("Sending 'initialized' notification...")
        initialized_notification = {
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {} 
            # No "id" field for notifications
        }
        # Use the new function for notifications
        send_notification(server_process, initialized_notification) 
        logging.debug("'initialized' notification sent.")
        # --- End Initialized Notification ---

        # Add a small delay after sending initialized notification
        time.sleep(0.5)

        # Connect to the Paint application (Now happens *after* initialized notification)
        logging.info("Sending 'connect' request...")
        connect_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "connect",
            "params": {
                "client_id": "test-client",
                "client_name": "Test Client"
            }
        })
        logging.info(f"Connect response: {connect_response}")
        # Check for success status within the 'result' field for our custom connect response
        if not connect_response or connect_response.get("result", {}).get("status") != "success":
            logging.error("Connect failed, aborting tests.")
            error_details = connect_response.get("error", "Unknown error") # Capture specific error if present
            if isinstance(error_details, dict):
                 error_details = f"Code: {error_details.get('code', 'N/A')}, Message: {error_details.get('message', 'N/A')}"
            elif not connect_response:
                 error_details = "No response received from server."
            else: # If connect_response exists but doesn't match expected success structure
                 error_details = f"Unexpected response format: {connect_response}"
            
            logging.error(f"Connect error details: {error_details}")
            # Attempt to log server stderr *at the point of connect failure*
            if server_process:
                try:
                    # Try non-blocking read first
                    stderr_data = server_process.stderr.read() if server_process.stderr else b''
                    if not stderr_data:
                         # If nothing read, try communicate with timeout
                        _, stderr_data = server_process.communicate(timeout=1)
                    if stderr_data:
                        logging.error(f"Server stderr on connect failure:\n{stderr_data.decode('utf-8', errors='replace')}")
                    else:
                        logging.warning("No stderr output captured from server on connect failure.")
                except Exception as e:
                    logging.error(f"Failed to get stderr after connect failure: {e}")
            return
        
        # Activate the Paint window
        logging.info("Sending 'activate_window' request...")
        activate_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "activate_window",
            "params": {}
        })
        logging.info(f"Activate window response: {activate_response}")
        time.sleep(1) # Wait a bit after activation
        
        # Get canvas dimensions
        logging.info("Sending 'get_canvas_dimensions' request...")
        dimensions_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "get_canvas_dimensions",
            "params": {}
        })
        logging.info(f"Canvas dimensions response: {dimensions_response}")
        
        canvas_width = 0
        canvas_height = 0
        if dimensions_response and dimensions_response.get("result", {}).get("status") == "success":
            canvas_width = dimensions_response["result"].get("width", 0)
            canvas_height = dimensions_response["result"].get("height", 0)
            logging.info(f"Canvas dimensions retrieved: {canvas_width}x{canvas_height}")
        else:
            logging.error("Failed to get canvas dimensions. Cannot draw pixel in center.")
            # Continue with other tests, but skip the center pixel draw

        # Draw a pixel in the center if dimensions were retrieved
        if canvas_width > 0 and canvas_height > 0:
            center_x = canvas_width // 2
            center_y = canvas_height // 2
            logging.info(f"Sending 'draw_pixel' request at center ({center_x}, {center_y})...")
            draw_pixel_response = send_request(server_process, {
                "jsonrpc": "2.0",
                "id": 314, # New ID for this request
                "method": "draw_pixel",
                "params": {
                    "x": center_x,
                    "y": center_y,
                    "color": "#000000" # Draw a black pixel
                }
            })
            logging.info(f"Draw pixel response: {draw_pixel_response}")
            time.sleep(1) # Wait a bit after drawing
        
        # --- Continue with existing tests --- 

        # Select a tool (pencil)
        logging.info("Sending 'select_tool' (pencil) request...")
        select_tool_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "select_tool",
            "params": {
                "tool": "pencil"
            }
        })
        logging.info(f"Select tool response: {select_tool_response}")
        
        # Draw a line
        logging.info("Sending 'draw_line' request...")
        draw_line_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 5,
            "method": "draw_line",
            "params": {
                "start_x": 100,
                "start_y": 100,
                "end_x": 300,
                "end_y": 300,
                "color": "#FF0000",
                "thickness": 2
            }
        })
        logging.info(f"Draw line response: {draw_line_response}")
        
        # Draw a polyline
        logging.info("Sending 'draw_polyline' request...")
        draw_polyline_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 6,
            "method": "draw_polyline",
            "params": {
                "points": [
                    {"x": 400, "y": 100},
                    {"x": 450, "y": 150},
                    {"x": 500, "y": 100},
                    {"x": 550, "y": 150}
                ],
                "color": "#0000FF",
                "thickness": 3,
                "tool": "brush"
            }
        })
        logging.info(f"Draw polyline response: {draw_polyline_response}")
        
        # Wait to see the results
        logging.info("Drawing operations completed. Press Enter to continue...")
        input()
        
        # Select a region
        logging.info("Sending 'select_region' request...")
        select_region_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 7,
            "method": "select_region",
            "params": {
                "start_x": 50,
                "start_y": 50,
                "end_x": 200,
                "end_y": 200
            }
        })
        logging.info(f"Select region response: {select_region_response}")
        
        # Copy selection
        logging.info("Sending 'copy_selection' request...")
        copy_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 8,
            "method": "copy_selection",
            "params": {}
        })
        logging.info(f"Copy selection response: {copy_response}")
        
        # Paste at different location
        logging.info("Sending 'paste' request...")
        paste_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 9,
            "method": "paste",
            "params": {
                "x": 400,
                "y": 300
            }
        })
        logging.info(f"Paste response: {paste_response}")
        
        # Add text
        logging.info("Sending 'add_text' request...")
        add_text_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 10,
            "method": "add_text",
            "params": {
                "x": 200,
                "y": 400,
                "text": "Hello Paint!",
                "color": "#0000FF",
                "font_size": 24
            }
        })
        logging.info(f"Add text response: {add_text_response}")
        
        # Create new canvas
        logging.info("Sending 'create_canvas' request...")
        create_canvas_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 11,
            "method": "create_canvas",
            "params": {
                "width": 800,
                "height": 600,
                "background_color": "#FFFFFF"
            }
        })
        logging.info(f"Create canvas response: {create_canvas_response}")
        
        # Clear canvas
        logging.info("Sending 'clear_canvas' request...")
        clear_canvas_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 12,
            "method": "clear_canvas",
            "params": {}
        })
        logging.info(f"Clear canvas response: {clear_canvas_response}")
        
        # Wait to see the results
        logging.info("All operations completed. Press Enter to exit...")
        input()
        
        # Disconnect
        logging.info("Sending 'disconnect' request...")
        disconnect_response = send_request(server_process, {
            "jsonrpc": "2.0",
            "id": 13,
            "method": "disconnect",
            "params": {}
        })
        logging.info(f"Disconnect response: {disconnect_response}")
        
    except KeyboardInterrupt:
        logging.info("User interrupted the test client.")
    except Exception as e:
        logging.error(f"An unexpected error occurred: {e}")
        import traceback
        logging.error(traceback.format_exc())
    finally:
        # Terminate the server
        if server_process:
            logging.info("Terminating server process...")
            try:
                server_process.terminate()
                outs, errs = server_process.communicate(timeout=3)
                logging.debug(f"Server terminated with code: {server_process.returncode}")
                if outs:
                    logging.debug(f"Final server stdout:\n{outs.decode('utf-8', errors='replace')}")
                if errs:
                    logging.warning(f"Final server stderr:\n{errs.decode('utf-8', errors='replace')}")
            except subprocess.TimeoutExpired:
                logging.warning("Server did not terminate gracefully, killing...")
                server_process.kill()
                outs, errs = server_process.communicate()
                logging.debug("Server killed.")
                if outs:
                    logging.debug(f"Final server stdout:\n{outs.decode('utf-8', errors='replace')}")
                if errs:
                    logging.warning(f"Final server stderr:\n{errs.decode('utf-8', errors='replace')}")
            except Exception as e:
                logging.error(f"Error terminating server: {e}")
        logging.info("Test client finished.")

# Add a helper function to log stderr in a background thread
def _log_stderr(stderr):
    """Log stderr from server in background thread"""
    line_buffer = []
    
    for line in stderr:
        line = line.strip()
        if line:
            line_buffer.append(line)
            
            # Every 10 lines or if line contains ERROR/WARN, print a summary
            if len(line_buffer) >= 10 or "ERROR" in line or "WARN" in line:
                if "ERROR" in line or "WARN" in line:
                    logging.warning(f"Server log: {line}")
                else:
                    logging.debug(f"Server wrote {len(line_buffer)} line(s) to stderr. Last: {line}")
                line_buffer = []

if __name__ == "__main__":
    main() 