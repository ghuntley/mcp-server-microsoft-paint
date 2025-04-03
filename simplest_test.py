#!/usr/bin/env python
import json
import subprocess
import time
import sys
import os
import threading

def main():
    # First, make sure Paint is not running
    os.system('taskkill /f /im mspaint.exe 2>nul')
    time.sleep(1)
    
    # Launch Paint
    print("Launching MS Paint...")
    paint_process = subprocess.Popen(["mspaint.exe"])
    time.sleep(3)  # Wait for Paint to start
    
    # Start the server
    print("Starting MCP server...")
    server_process = subprocess.Popen(
        ["cargo", "run"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=None,  # Don't capture stderr so it appears in console directly
        text=True,
        bufsize=1
    )
    
    try:
        # Step 1: Initialize
        print("Sending initialize request...")
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }
        
        send_request_and_wait(server_process, init_request, 3)
        
        # Step 2: Connect
        print("Sending connect request...")
        connect_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "connect",
            "params": {
                "client_id": "simplest_test",
                "client_name": "Simplest Test Client"
            }
        }
        
        send_request_and_wait(server_process, connect_request, 3)
        
        # Step 3: Draw a large rectangle with lots of time for each step
        print("Drawing a large rectangle...")
        rectangle_request = {
            "jsonrpc": "2.0", 
            "id": 3,
            "method": "draw_shape",
            "params": {
                "shape_type": "rectangle",
                "start_x": 50,
                "start_y": 50,
                "end_x": 500,
                "end_y": 400
            }
        }
        
        # Send request and wait longer for rectangle drawing to complete
        send_request_and_wait(server_process, rectangle_request, 10)
        
        print("Test completed. Rectangle should be drawn.")
        print("Press Enter to close test and Paint...")
        input()
        
    except Exception as e:
        print(f"Test failed with error: {e}")
    finally:
        # Terminate the server process
        server_process.terminate()
        print("Server process terminated")
        
        # Terminate Paint
        paint_process.terminate()
        print("Paint process terminated")
        
def send_request_and_wait(process, request, wait_time=5):
    """Send a request to the server and wait a fixed time."""
    request_str = json.dumps(request) + "\n"
    print(f"SENDING: {request_str.strip()}")
    
    process.stdin.write(request_str)
    process.stdin.flush()
    
    # Wait for response - simple approach with fixed wait time
    print(f"Waiting {wait_time} seconds for operation to complete...")
    
    # Read response
    response_line = process.stdout.readline().strip()
    if response_line:
        print(f"RESPONSE: {response_line}")
    
    # Wait additional time after response to ensure operation completes
    time.sleep(wait_time)
    
    return response_line

if __name__ == "__main__":
    main() 