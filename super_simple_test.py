#!/usr/bin/env python
import json
import subprocess
import time
import sys
import os

def main():
    # First, make sure Paint is not running
    os.system('taskkill /f /im mspaint.exe 2>nul')
    time.sleep(1)
    
    # Launch Paint
    print("Launching MS Paint...")
    subprocess.Popen(["mspaint.exe"])
    time.sleep(3)  # Wait for Paint to start
    
    # Start the server in release mode
    print("Starting MCP server...")
    server_process = subprocess.Popen(
        ["cargo", "run", "--release"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=open("latest_server_log.txt", "w"),
        text=True,
        bufsize=1
    )
    
    try:
        # Step 1: Initialize
        print("Step 1: Sending initialize request...")
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }
        
        send_request(server_process, init_request)
        
        # Step 2: Connect
        print("Step 2: Sending connect request...")
        connect_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "connect",
            "params": {
                "client_id": "super_simple_test",
                "client_name": "Super Simple Test"
            }
        }
        
        send_request(server_process, connect_request)
        
        # Step 3: Draw a line
        print("Step 3: Sending draw_line request...")
        line_request = {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "draw_line",
            "params": {
                "start_x": 100,
                "start_y": 200,
                "end_x": 400, 
                "end_y": 200,
                "color": "#FF0000",  # Red color
                "thickness": 5       # Thick line
            }
        }
        
        send_request(server_process, line_request)
        
        print("Test completed! Keeping Paint open to observe results...")
        print("Press Enter to close the test...")
        input()
        
    except Exception as e:
        print(f"Test failed with error: {e}")
    finally:
        server_process.terminate()
        print("Server process terminated")
        
def send_request(process, request):
    """Send a request to the server and print the response."""
    request_str = json.dumps(request) + "\n"
    print(f"Sending: {request_str.strip()}")
    
    process.stdin.write(request_str)
    process.stdin.flush()
    
    # Read response with timeout
    start_time = time.time()
    timeout = 10  # seconds
    
    while time.time() - start_time < timeout:
        line = process.stdout.readline().strip()
        if line:
            print(f"Response: {line}")
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                print(f"Warning: Received non-JSON response: {line}")
        time.sleep(0.1)
    
    print("Warning: No response received within timeout")
    return None

if __name__ == "__main__":
    main() 