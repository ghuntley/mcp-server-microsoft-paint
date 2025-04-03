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
    
    # Start the server with higher log level
    print("Starting MCP server...")
    server_process = subprocess.Popen(
        ["cargo", "run"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )
    
    # Create threads to continuously read and print output
    def read_stdout():
        while True:
            line = server_process.stdout.readline()
            if not line:
                break
            print(f"SERVER STDOUT: {line.strip()}")
            
    def read_stderr():
        while True:
            line = server_process.stderr.readline()
            if not line:
                break
            print(f"SERVER STDERR: {line.strip()}")
            
    stdout_thread = threading.Thread(target=read_stdout, daemon=True)
    stderr_thread = threading.Thread(target=read_stderr, daemon=True)
    
    stdout_thread.start()
    stderr_thread.start()
    
    try:
        # Step 1: Initialize
        print("\n=== STEP 1: Sending initialize request ===")
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }
        
        response = send_request(server_process, init_request)
        print(f"Initialize response: {json.dumps(response, indent=2)}")
        
        # Step 2: Connect
        print("\n=== STEP 2: Sending connect request ===")
        connect_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "connect",
            "params": {
                "client_id": "rectangle_test",
                "client_name": "Rectangle Test Client"
            }
        }
        
        response = send_request(server_process, connect_request)
        print(f"Connect response: {json.dumps(response, indent=2)}")
        
        # Make sure the window is properly maximized
        time.sleep(2)
        
        # Step 3: Draw a rectangle
        print("\n=== STEP 3: Drawing a rectangle ===")
        rectangle_request = {
            "jsonrpc": "2.0", 
            "id": 3,
            "method": "draw_shape",
            "params": {
                "shape_type": "rectangle",
                "start_x": 100,
                "start_y": 100,
                "end_x": 400,
                "end_y": 300
            }
        }
        
        response = send_request(server_process, rectangle_request)
        print(f"Rectangle response: {json.dumps(response, indent=2)}")
        
        print("\nTest completed! Check Paint to see if the rectangle was drawn correctly.")
        print("Press Enter to close the test and kill Paint...")
        input()
        
    except Exception as e:
        print(f"Test failed with error: {e}")
    finally:
        # Terminate the server process
        server_process.terminate()
        print("Server process terminated")
        
        # Allow time for process to terminate
        time.sleep(1)
        
        # Terminate Paint
        paint_process.terminate()
        print("Paint process terminated")
        
def send_request(process, request):
    """Send a request to the server and print the response."""
    request_str = json.dumps(request) + "\n"
    print(f"SENDING: {request_str.strip()}")
    
    process.stdin.write(request_str)
    process.stdin.flush()
    print("Request sent and flushed")
    
    # Read response with timeout
    start_time = time.time()
    timeout = 30  # seconds - increased for more debugging time
    
    while time.time() - start_time < timeout:
        print("Waiting for response...")
        line = process.stdout.readline().strip()
        if line:
            print(f"RESPONSE: {line}")
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                print(f"WARNING: Received non-JSON response: {line}")
        time.sleep(0.5)
    
    print("WARNING: No response received within timeout")
    return None

if __name__ == "__main__":
    main() 