#!/usr/bin/env python
import json
import subprocess
import sys
import time
import logging
import os

def setup_logging():
    logging.basicConfig(
        level=logging.INFO,
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
    
    while time.time() - start_time < timeout:
        line = server_process.stdout.readline().strip()
        if not line:
            time.sleep(0.1)
            continue
            
        try:
            response = json.loads(line)
            logging.info(f"Received response: {response}")
            return response
        except json.JSONDecodeError:
            pass
            
    logging.error(f"No response received within {timeout} seconds")
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
    
    # Start MCP server
    logging.info("Starting MCP server...")
    server_process = subprocess.Popen(
        ["cargo", "run", "--release"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
        env=dict(os.environ, RUST_LOG="info,debug")
    )
    
    try:
        # Wait for server to start
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
        if not connect_response or connect_response.get("status") != "success":
            logging.error("Failed to connect to server")
            return
            
        logging.info(f"Connected to server. Canvas size: {connect_response.get('canvas_width')}x{connect_response.get('canvas_height')}")
        
        # Activate Paint window
        activate_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "activate_window",
            "params": {}
        }
        
        activate_response = send_request(server_process, activate_request)
        if not activate_response or activate_response.get("status") != "success":
            logging.warning("Failed to activate window, but continuing anyway")
            
        # Draw a horizontal line
        logging.info("Drawing horizontal line...")
        draw_line_request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "draw_line",
            "params": {
                "start_x": 100,
                "start_y": 100,
                "end_x": 400,
                "end_y": 100,
                "color": "#FF0000",
                "thickness": 3
            }
        }
        
        line_response = send_request(server_process, draw_line_request)
        if not line_response or line_response.get("status") != "success":
            logging.error(f"Failed to draw horizontal line: {line_response}")
        else:
            logging.info("Successfully drew horizontal line")
            
            # Draw a vertical line
            logging.info("Drawing vertical line...")
            draw_vert_request = {
                "jsonrpc": "2.0",
                "id": 4,
                "method": "draw_line",
                "params": {
                    "start_x": 250,
                    "start_y": 50,
                    "end_x": 250,
                    "end_y": 150,
                    "color": "#0000FF",
                    "thickness": 3
                }
            }
            
            vert_response = send_request(server_process, draw_vert_request)
            if not vert_response or vert_response.get("status") != "success":
                logging.error(f"Failed to draw vertical line: {vert_response}")
            else:
                logging.info("Successfully drew vertical line")
                
        logging.info("Test completed")
        
    except Exception as e:
        logging.error(f"Error during test: {e}")
    finally:
        server_process.terminate()
        logging.info("Server terminated")

if __name__ == "__main__":
    main() 