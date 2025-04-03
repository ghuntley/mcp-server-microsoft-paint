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
    
    # Start the server in debug mode (not release) for more verbose output
    print("Starting MCP server...")
    server_process = subprocess.Popen(
        ["cargo", "run"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )
    
    # Create a thread to continuously read and print stderr
    def read_stderr():
        while True:
            line = server_process.stderr.readline()
            if not line:
                break
            print(f"SERVER STDERR: {line.strip()}")
            
    stderr_thread = threading.Thread(target=read_stderr, daemon=True)
    stderr_thread.start()
    
    try:
        # Step 1: Initialize
        print("Step 1: Sending initialize request...")
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }
        
        response = send_request(server_process, init_request)
        if not response:
            print("ERROR: No response received for initialize request")
        
        # Step 2: Connect
        print("Step 2: Sending connect request...")
        connect_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "connect",
            "params": {
                "client_id": "debug_test",
                "client_name": "Debug Test Client"
            }
        }
        
        response = send_request(server_process, connect_request)
        if not response:
            print("ERROR: No response received for connect request")
        
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
        
        response = send_request(server_process, line_request)
        if not response:
            print("ERROR: No response received for draw_line request")
        
        print("Test completed! Check Paint to see if anything was drawn.")
        print("Press Enter to close the test and kill Paint...")
        input()
        
    except Exception as e:
        print(f"Test failed with error: {e}")
    finally:
        server_process.terminate()
        print("Server process terminated")
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
    timeout = 10  # seconds
    
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