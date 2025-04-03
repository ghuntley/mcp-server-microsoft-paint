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
    timeout = 30 # Increased timeout for potentially slower operations
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
            if line.startswith("{"):
                logging.debug(f"Found JSON-like response: {line}")
                try:
                    response_json = json.loads(line)
                    # Validate this is a proper response
                    if "status" in response_json:
                        logging.debug("Found server response with status field")
                        break
                    # Also check for id field matching our request
                    elif "id" in response_json and response_json["id"] == request_id:
                        logging.debug("Parsed matching JSON response (by id) successfully.")
                        break
                    else:
                        logging.debug(f"Found JSON but it doesn't match our request - continuing to wait")
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

def verify_paint_running():
    """Verify if Paint is running using tasklist"""
    try:
        # Check if Paint is already running
        result = subprocess.run(
            ["tasklist", "/FI", "IMAGENAME eq mspaint.exe"],
            capture_output=True,
            text=True
        )
        
        if "mspaint.exe" in result.stdout:
            logging.info("MS Paint is already running")
            return True
        else:
            logging.info("MS Paint is not running")
            return False
    except Exception as e:
        logging.error(f"Error checking if Paint is running: {e}")
        return False

def launch_paint():
    """Manually launch Paint"""
    try:
        subprocess.run(["start", "mspaint.exe"], shell=True)
        logging.info("Started MS Paint")
        time.sleep(2)  # Give Paint time to start
        return True
    except Exception as e:
        logging.error(f"Error launching MS Paint: {e}")
        return False

def main():
    setup_logging()
    server_process = None
    try:
        # First check if Paint is running, if not launch it
        if not verify_paint_running():
            if not launch_paint():
                logging.error("Failed to launch Paint. Continuing anyway but issues may occur.")

        logging.info("Launching MCP server in TEXT mode...")
        
        # Set environment variables to control Rust logging
        env = os.environ.copy()
        env['RUST_LOG'] = 'info,debug' # Set Rust logging level
        
        # Use subprocess.Popen to capture stdin/stdout/stderr
        server_process = subprocess.Popen(
            ["cargo", "run", "--release"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,  # Use text mode
            bufsize=1,  # Line buffered
            env=env,
        )
        
        logging.info(f"Server process started with PID: {server_process.pid}")
        
        # Wait a bit for the server to initialize
        logging.debug("Waiting for server process to start (2 seconds)...")
        time.sleep(2)
        
        # Check again if Paint is running
        verify_paint_running()
        
        # Send initialize request with client info
        logging.info("Sending 'initialize' request...")
        initialize_request = {
            "jsonrpc": "2.0",
            "id": 0,
            "method": "initialize",
            "params": {
                "processId": os.getpid(),
                "clientInfo": {
                    "name": "test_client.py",
                    "version": "0.1.0"
                },
                "capabilities": {},
                "implementation": {
                    "name": "DummyClientImplementation",
                    "version": "0.0.1"
                }
            }
        }
        
        initialize_response = send_request(server_process, initialize_request)
        
        if initialize_response and initialize_response.get("status") == "success":
            logging.info("Initialize successful. Server capabilities: " + str(initialize_response.get("capabilities", {})))
            
            # Send initialized notification
            logging.info("Sending 'initialized' notification...")
            initialized_notification = {
                "jsonrpc": "2.0",
                "method": "initialized",
                "params": {}
            }
            send_notification(server_process, initialized_notification)
            
            # Wait a moment for the server to process the notification
            time.sleep(0.5)
            
            # Now send connect request
            logging.info("Sending 'connect' request...")
            connect_request = {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "connect",
                "params": {
                    "client_id": "test-client-123",
                    "client_name": "Python Test Client"
                }
            }
            
            connect_response = send_request(server_process, connect_request)
            
            if connect_response:
                logging.info("Connect response: " + str(connect_response))
                
                # If connected successfully, try a simple operation
                if connect_response.get("status") == "success":
                    logging.info("Connection successful. Canvas dimensions: " + 
                                str(connect_response.get("canvas_width", "unknown")) + "x" + 
                                str(connect_response.get("canvas_height", "unknown")))
                    
                    # Activate the window to make sure it's in the foreground
                    logging.info("Sending 'activate_window' request...")
                    activate_request = {
                        "jsonrpc": "2.0",
                        "id": 2,
                        "method": "activate_window",
                        "params": {}
                    }
                    
                    activate_response = send_request(server_process, activate_request)
                    if activate_response:
                        logging.info("Activate window response: " + str(activate_response))
                    
                    # Try to draw a pixel
                    logging.info("Sending 'draw_pixel' request...")
                    draw_pixel_request = {
                        "jsonrpc": "2.0",
                        "id": 3,
                        "method": "draw_pixel",
                        "params": {
                            "x": 100,
                            "y": 100,
                            "color": "#FF0000"  # Optional, red color
                        }
                    }
                    
                    draw_pixel_response = send_request(server_process, draw_pixel_request)
                    
                    if draw_pixel_response:
                        logging.info("Draw pixel response: " + str(draw_pixel_response))
                        
                        if draw_pixel_response.get("status") == "success":
                            logging.info("Successfully drew a pixel! MCP server is working correctly.")
                        else:
                            logging.error("Failed to draw pixel: " + str(draw_pixel_response))
                    else:
                        logging.error("Failed to get response for draw_pixel request")
                else:
                    logging.error("Connect failed: " + str(connect_response))
            else:
                logging.error("Failed to get response for connect request")
        else:
            logging.error("Initialize failed or no response received")
                    
    except KeyboardInterrupt:
        logging.info("User interrupted the test client.")
    except Exception as e:
        logging.error(f"Error in test client: {e}")
        import traceback
        logging.error(traceback.format_exc())
    finally:
        if server_process:
            logging.info("Terminating server process...")
            try:
                server_process.terminate()
                logging.debug(f"Server terminated with code: {server_process.wait()}")
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