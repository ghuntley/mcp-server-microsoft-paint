#!/usr/bin/env python
import json
import subprocess
import sys
import time
import logging
import os
import threading
import re
from ctypes import windll

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

def check_paint_window_state():
    """Check if the Paint window is visible and capture its state"""
    try:
        import win32gui
        import win32process
        import psutil
        
        def callback(hwnd, results):
            if win32gui.IsWindowVisible(hwnd):
                title = win32gui.GetWindowText(hwnd)
                if "Paint" in title and not "mcp-server" in title:
                    try:
                        _, pid = win32process.GetWindowThreadProcessId(hwnd)
                        proc = psutil.Process(pid)
                        results.append({
                            "hwnd": hwnd,
                            "title": title,
                            "pid": pid,
                            "status": "Running" if proc.status() == psutil.STATUS_RUNNING else proc.status(),
                            "cpu_percent": proc.cpu_percent(),
                            "memory_mb": proc.memory_info().rss / (1024 * 1024),
                            "window_rect": win32gui.GetWindowRect(hwnd),
                            "is_minimized": win32gui.IsIconic(hwnd),
                            "is_visible": win32gui.IsWindowVisible(hwnd),
                        })
                    except Exception as e:
                        logging.error(f"Error getting process info for Paint window {hwnd}: {e}")
            return True
            
        results = []
        win32gui.EnumWindows(callback, results)
        
        if results:
            for win in results:
                logging.info(f"Found Paint window: {win}")
            return results
        else:
            logging.warning("No visible Paint windows found")
            return []
            
    except ImportError:
        logging.warning("pywin32/psutil not installed, cannot check Paint window state")
        return []
    except Exception as e:
        logging.error(f"Error checking Paint window state: {e}")
        return []

def main():
    setup_logging()
    server_process = None
    try:
        # First check if Paint is running, if not launch it
        if not verify_paint_running():
            if not launch_paint():
                logging.error("Failed to launch Paint. Continuing anyway but issues may occur.")
        else:
            logging.info("MS Paint is already running. Attempting to use existing instance.")
            
        # Check Paint window state
        check_paint_window_state()

        # Log the current display settings to help with debugging
        try:
            dc = windll.user32.GetDC(0)
            width = windll.gdi32.GetDeviceCaps(dc, 8)  # HORZRES
            height = windll.gdi32.GetDeviceCaps(dc, 10)  # VERTRES
            dpi_x = windll.gdi32.GetDeviceCaps(dc, 88)  # LOGPIXELSX
            dpi_y = windll.gdi32.GetDeviceCaps(dc, 90)  # LOGPIXELSY
            windll.user32.ReleaseDC(0, dc)
            logging.info(f"Display settings: {width}x{height} resolution, {dpi_x}x{dpi_y} DPI")
        except Exception as e:
            logging.error(f"Failed to get display settings: {e}")

        logging.info("Launching MCP server in TEXT mode...")
        
        # Set environment variables to control Rust logging
        env = os.environ.copy()
        env['RUST_LOG'] = 'info,trace,debug' # Set Rust logging level for more details
        
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
                    
                    # Check Paint window state after activation
                    logging.info("Checking Paint window state after activation:")
                    check_paint_window_state()
                    
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
                            
                            # ---- Draw a very visible line with explicit tool selection first ----
                            
                            # First, select the pencil tool
                            logging.info("Selecting pencil tool...")
                            select_tool_request = {
                                "jsonrpc": "2.0",
                                "id": 4,
                                "method": "select_tool",
                                "params": {
                                    "tool": "pencil"
                                }
                            }
                            
                            select_tool_response = send_request(server_process, select_tool_request)
                            if not select_tool_response or select_tool_response.get("status") != "success":
                                logging.error(f"Failed to select pencil tool: {select_tool_response}")
                                return
                            
                            logging.info("Successfully selected pencil tool")
                            
                            # Set color to bright red
                            logging.info("Setting color to bright red...")
                            set_color_request = {
                                "jsonrpc": "2.0",
                                "id": 5,
                                "method": "set_color",
                                "params": {
                                    "color": "#FF0000"  # Pure red
                                }
                            }
                            
                            set_color_response = send_request(server_process, set_color_request)
                            if not set_color_response or set_color_response.get("status") != "success":
                                logging.error(f"Failed to set color: {set_color_response}")
                                # Try again with the draw_line command which includes the color
                                logging.info("Will include color directly in the draw_line command")
                            else:
                                logging.info("Successfully set color to red")
                            
                            # Draw a simple horizontal line in the upper portion of the canvas
                            logging.info("Drawing a horizontal red line...")
                            draw_line_request = {
                                "jsonrpc": "2.0",
                                "id": 6,
                                "method": "draw_line",
                                "params": {
                                    "start_x": 100,
                                    "start_y": 100,
                                    "end_x": 300,
                                    "end_y": 100,
                                    "color": "#FF0000",  # Explicitly include color in case set_color failed
                                    "thickness": 3  # Make it thicker for visibility
                                }
                            }
                            
                            draw_line_response = send_request(server_process, draw_line_request)
                            
                            if not draw_line_response:
                                logging.error("No response received for draw_line request")
                            elif draw_line_response.get("status") != "success":
                                logging.error(f"Failed to draw line: {draw_line_response}")
                            else:
                                logging.info("Successfully drew a red horizontal line!")
                                
                                # Now try a vertical line in blue
                                logging.info("Setting color to blue...")
                                set_color_request = {
                                    "jsonrpc": "2.0",
                                    "id": 7,
                                    "method": "set_color",
                                    "params": {
                                        "color": "#0000FF"
                                    }
                                }
                                
                                set_color_response = send_request(server_process, set_color_request)
                                if not set_color_response or set_color_response.get("status") != "success":
                                    logging.error(f"Failed to set color to blue: {set_color_response}")
                                else:
                                    logging.info("Drawing a vertical blue line...")
                                    draw_line_request = {
                                        "jsonrpc": "2.0",
                                        "id": 8,
                                        "method": "draw_line",
                                        "params": {
                                            "start_x": 200,
                                            "start_y": 50,
                                            "end_x": 200,
                                            "end_y": 150
                                        }
                                    }
                                    
                                    draw_line_response = send_request(server_process, draw_line_request)
                                    
                                    if not draw_line_response:
                                        logging.error("No response received for vertical line request")
                                    elif draw_line_response.get("status") != "success":
                                        logging.error(f"Failed to draw vertical line: {draw_line_response}")
                                    else:
                                        logging.info("Successfully drew a blue vertical line!")
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