#!/usr/bin/env python
import json
import subprocess
import sys
import time
import logging
import os

def setup_logging():
    logging.basicConfig(
        level=logging.DEBUG,
        format="%(asctime)s [%(levelname)-5.5s] %(message)s",
        handlers=[logging.StreamHandler(sys.stdout)]
    )
    logging.info("Logging initialized for simple_test_client.py")

def send_request(server_process, request):
    """Send a JSON-RPC request to the server and return the response."""
    request_json = json.dumps(request) + "\n"
    try:
        server_process.stdin.write(request_json) 
        server_process.stdin.flush()
        logging.info(f"Sent request: {request['method']}")
    except Exception as e:
        logging.error(f"Failed to send request: {e}")
        return None
    
    # Read response
    start_time = time.time()
    timeout = 30
    
    # Capture log lines for debugging
    log_lines = []
    
    while time.time() - start_time < timeout:
        try:
            # Check if there's anything to read from stdout
            line = server_process.stdout.readline().strip()
            if line:
                logging.debug(f"Received raw line: {line}")
                
                # Skip lines that are clearly log output
                if any(marker in line for marker in ["[INFO]", "[DEBUG]", "[ERROR]", "[WARN]"]) or line.startswith("20"):
                    log_lines.append(line)
                    continue
                    
                # Try to parse line as JSON
                try:
                    if line.startswith('{') or line.startswith('['):
                        response = json.loads(line)
                        logging.info(f"Received response: {response}")
                        return response
                except json.JSONDecodeError as e:
                    logging.error(f"Failed to parse JSON: {line}, Error: {e}")
            else:
                # No data available, short sleep to prevent CPU spinning
                time.sleep(0.1)
                
            # Check if the process is still running
            if server_process.poll() is not None:
                logging.error(f"Server process terminated with exit code: {server_process.poll()}")
                return None
                
        except Exception as e:
            logging.error(f"Error reading response: {e}")
            time.sleep(0.1)
            
    # If we get here, we timed out
    logging.error(f"No response received within {timeout} seconds")
    if log_lines:
        logging.error("Last 10 log lines from server:")
        for log in log_lines[-10:]:
            logging.error(f"  {log}")
            
    # Try to read any error output
    stderr_output = server_process.stderr.read()
    if stderr_output:
        logging.error(f"Server stderr output: {stderr_output}")
            
    return None

def launch_paint():
    """Launch MS Paint"""
    try:
        subprocess.run(["taskkill", "/f", "/im", "mspaint.exe"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        time.sleep(1)
        subprocess.run(["start", "mspaint.exe"], shell=True)
        logging.info("Launched MS Paint")
        time.sleep(3)  # Wait for Paint to start
        return True
    except Exception as e:
        logging.error(f"Failed to launch Paint: {e}")
        return False

def main():
    setup_logging()
    
    # Launch Paint first
    launch_paint()
    
    # Start MCP server with stderr redirected to a file for debugging
    log_file_path = "server_stderr.log"
    with open(log_file_path, "w") as stderr_file:
        logging.info("Starting MCP server...")
        server_process = subprocess.Popen(
            ["cargo", "run", "--release"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=stderr_file,
            text=True,
            bufsize=1,
            env=dict(os.environ, RUST_LOG="info,debug")
        )
    
    try:
        # Wait for server to start
        logging.info("Waiting for server to initialize...")
        time.sleep(3)
        
        # Connect to server
        connect_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "connect",
            "params": {
                "client_id": "simple-test-client",
                "client_name": "Simple Test Client"
            }
        }
        
        connect_response = send_request(server_process, connect_request)
        if not connect_response:
            logging.error("No response received from connect request")
            with open(log_file_path, "r") as f:
                logging.error(f"Server stderr log:\n{f.read()}")
            return
            
        if connect_response.get("status") != "success":
            logging.error(f"Failed to connect to server: {connect_response}")
            return
            
        logging.info(f"Connected to server. Response: {connect_response}")
        canvas_width = connect_response.get('canvas_width', 800)
        canvas_height = connect_response.get('canvas_height', 600)
        
        # Activate Paint window
        logging.info("Sending activate_window request...")
        activate_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "activate_window",
            "params": {}
        }
        
        activate_response = send_request(server_process, activate_request)
        if not activate_response:
            logging.error("No response received from activate_window request")
            return
            
        if activate_response.get("status") != "success":
            logging.warning(f"Failed to activate window, but continuing anyway: {activate_response}")
        else:
            logging.info(f"Window activated successfully: {activate_response}")
        
        # Get canvas dimensions (alternative method)
        logging.info("Sending get_canvas_dimensions request...")
        dimensions_request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "get_canvas_dimensions",
            "params": {}
        }
        
        dimensions_response = send_request(server_process, dimensions_request)
        if not dimensions_response:
            logging.error("No response received from get_canvas_dimensions request")
            with open(log_file_path, "r") as f:
                logging.error(f"Server stderr log:\n{f.read()}")
            return
            
        if dimensions_response.get("status") != "success":
            logging.warning(f"Failed to get canvas dimensions: {dimensions_response}")
            canvas_width = 800
            canvas_height = 600
            logging.warning(f"Using default canvas dimensions: {canvas_width}x{canvas_height}")
        else:
            canvas_width = dimensions_response.get("width", 800)
            canvas_height = dimensions_response.get("height", 600)
            logging.info(f"Canvas dimensions: {canvas_width}x{canvas_height}")
        
        # Calculate center points for drawing
        center_x = canvas_width // 2
        center_y = canvas_height // 2
        
        # Adjust for drawing area (skip the ribbon)
        drawing_y_offset = 0  # No needed for MCP since we're using canvas coordinates
        
        # Draw a horizontal line in the center
        start_x = center_x - 100
        start_y = center_y + drawing_y_offset
        end_x = center_x + 100
        end_y = center_y + drawing_y_offset
        
        logging.info(f"Drawing horizontal line from ({start_x}, {start_y}) to ({end_x}, {end_y})")
        draw_line_request = {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "draw_line",
            "params": {
                "start_x": start_x,
                "start_y": start_y,
                "end_x": end_x,
                "end_y": end_y,
                "color": "#FF0000",  # Red color
                "thickness": 3
            }
        }
        
        logging.info("Sending draw_line request for horizontal line...")
        line_response = send_request(server_process, draw_line_request)
        if not line_response:
            logging.error("No response received from horizontal line draw_line request")
            with open(log_file_path, "r") as f:
                logging.error(f"Server stderr log:\n{f.read()}")
            return
            
        if line_response.get("status") != "success":
            logging.error(f"Failed to draw horizontal line: {line_response}")
            return
            
        logging.info("Successfully drew horizontal line")
        
        # Wait a moment between drawing operations (like in direct_paint_test)
        time.sleep(1)
            
        # Draw a vertical line intersecting the horizontal line
        start_x = center_x
        start_y = center_y + drawing_y_offset - 100
        end_x = center_x
        end_y = center_y + drawing_y_offset + 100
        
        logging.info(f"Drawing vertical line from ({start_x}, {start_y}) to ({end_x}, {end_y})")
        draw_vert_request = {
            "jsonrpc": "2.0",
            "id": 5,
            "method": "draw_line",
            "params": {
                "start_x": start_x,
                "start_y": start_y,
                "end_x": end_x,
                "end_y": end_y,
                "color": "#0000FF",  # Blue color
                "thickness": 3
            }
        }
        
        logging.info("Sending draw_line request for vertical line...")    
        vert_response = send_request(server_process, draw_vert_request)
        if not vert_response:
            logging.error("No response received from vertical line draw_line request")
            with open(log_file_path, "r") as f:
                logging.error(f"Server stderr log:\n{f.read()}")
            return
            
        if vert_response.get("status") != "success":
            logging.error(f"Failed to draw vertical line: {vert_response}")
            return
            
        logging.info("Successfully drew vertical line")
        
        # Test is complete
        logging.info("Test completed successfully")
        
    except Exception as e:
        logging.error(f"Error during test: {e}", exc_info=True)
    finally:
        # Terminate the server process
        try:
            server_process.terminate()
            server_process.wait(timeout=5)
            logging.info("Server terminated")
        except:
            logging.warning("Could not cleanly terminate the server process")
        
        # Output server logs for review
        logging.info(f"Server stderr log available at: {log_file_path}")
        with open(log_file_path, "r") as f:
            server_logs = f.read()
            if len(server_logs) > 0:
                logging.info(f"Last 200 characters of server logs: {server_logs[-200:]}")

if __name__ == "__main__":
    main() 