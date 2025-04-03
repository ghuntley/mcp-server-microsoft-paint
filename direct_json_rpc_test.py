#!/usr/bin/env python
import json
import subprocess
import time
import sys

def main():
    # First manually launch Paint
    print("Launching MS Paint directly...")
    paint_process = subprocess.Popen(["mspaint.exe"])
    time.sleep(3)  # Give Paint time to start
    
    # Now start the MCP server
    print("Starting MCP server...")
    server_process = subprocess.Popen(
        ["cargo", "run", "--release"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=open("server_stderr.log", "w"),  # Capture stderr to file
        text=True,
        bufsize=1
    )
    
    try:
        # Wait for server to start up
        time.sleep(3)
        
        # Send connect request
        print("Sending connect request...")
        connect_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "connect",
            "params": {
                "client_id": "direct-test",
                "client_name": "Direct JSON-RPC Test"
            }
        }
        
        request_json = json.dumps(connect_request) + "\n"
        print(f"Request: {request_json.strip()}")
        server_process.stdin.write(request_json)
        server_process.stdin.flush()
        
        # Read response with timeout
        response = ""
        start_time = time.time()
        while time.time() - start_time < 5:  # 5 second timeout
            line = server_process.stdout.readline().strip()
            if line:
                print(f"Response: {line}")
                try:
                    response = json.loads(line)
                    break
                except json.JSONDecodeError:
                    print(f"Not valid JSON: {line}")
            time.sleep(0.1)
        
        if not response:
            print("No response received")
            return
            
        # Send a draw_line command
        print("Sending draw_line request...")
        line_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "draw_line",
            "params": {
                "start_x": 100,
                "start_y": 100,
                "end_x": 300,
                "end_y": 100,
                "color": "#FF0000",
                "thickness": 3
            }
        }
        
        request_json = json.dumps(line_request) + "\n"
        print(f"Request: {request_json.strip()}")
        server_process.stdin.write(request_json)
        server_process.stdin.flush()
        
        # Read response
        response = ""
        start_time = time.time()
        while time.time() - start_time < 10:  # 10 second timeout for drawing
            line = server_process.stdout.readline().strip()
            if line:
                print(f"Response: {line}")
                try:
                    response = json.loads(line)
                    break
                except json.JSONDecodeError:
                    print(f"Not valid JSON: {line}")
            time.sleep(0.1)
        
        if not response:
            print("No response received for draw_line")
            return
            
        print("Test completed. Wait 10 seconds to observe the result...")
        time.sleep(10)
        
    except Exception as e:
        print(f"Error during test: {e}")
    finally:
        server_process.terminate()
        paint_process.terminate()
        print("Tests terminated and MS Paint closed")

if __name__ == "__main__":
    main() 