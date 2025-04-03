#!/usr/bin/env python
import json
import subprocess
import time
import sys

def main():
    # First launch MS Paint
    print("Launching MS Paint...")
    subprocess.Popen(["mspaint.exe"])
    time.sleep(3)  # Give Paint time to start
    
    # Start the MCP server with our custom implementation
    print("Starting MCP server...")
    server_process = subprocess.Popen(
        ["cargo", "run", "--release"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )
    
    try:
        # Wait for server to start
        time.sleep(3)
        
        # First initialize and find Paint
        print("Sending initialize request...")
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }
        
        request_json = json.dumps(init_request) + "\n"
        print(f"Request: {request_json.strip()}")
        server_process.stdin.write(request_json)
        server_process.stdin.flush()
        
        # Read response
        response = server_process.stdout.readline().strip()
        print(f"Response: {response}")
        
        if not response:
            print("No response received from initialize")
            stderr = server_process.stderr.read()
            print(f"Server stderr: {stderr}")
            return
        
        # Send connect request
        print("Sending connect request...")
        connect_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "connect",
            "params": {
                "client_id": "final-test",
                "client_name": "Final JSON-RPC Test"
            }
        }
        
        request_json = json.dumps(connect_request) + "\n"
        print(f"Request: {request_json.strip()}")
        server_process.stdin.write(request_json)
        server_process.stdin.flush()
        
        # Read response
        response = server_process.stdout.readline().strip()
        print(f"Response: {response}")
        
        if not response:
            print("No response received from connect")
            stderr = server_process.stderr.read()
            print(f"Server stderr: {stderr}")
            return
        
        # Send draw_line request
        print("Sending draw_line request...")
        line_request = {
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
        
        request_json = json.dumps(line_request) + "\n"
        print(f"Request: {request_json.strip()}")
        server_process.stdin.write(request_json)
        server_process.stdin.flush()
        
        # Read response
        response = server_process.stdout.readline().strip()
        print(f"Response: {response}")
        
        if not response:
            print("No response received from draw_line")
            stderr = server_process.stderr.read()
            print(f"Server stderr: {stderr}")
            return
        
        # Wait a moment to see if the drawing succeeded
        print("Test completed. Keeping Paint open for 10 seconds to observe results...")
        time.sleep(10)
        
    except Exception as e:
        print(f"Error during test: {e}")
        import traceback
        traceback.print_exc()
    finally:
        # Clean up
        server_process.terminate()
        print("Server terminated")
        # Keep Paint open to see the results

if __name__ == "__main__":
    main() 