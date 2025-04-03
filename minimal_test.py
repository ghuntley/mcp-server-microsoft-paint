#!/usr/bin/env python
import json
import subprocess
import time
import sys

def main():
    # Start the MCP server
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
        # Wait for server to initialize
        time.sleep(3)
        
        # Send a simple initialize method (this should find or launch Paint)
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
        print("Reading response...")
        response = server_process.stdout.readline().strip()
        print(f"Response: {response}")
        
        if not response:
            print("No response received")
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
                "client_id": "minimal-test",
                "client_name": "Minimal Test Client"
            }
        }
        
        request_json = json.dumps(connect_request) + "\n"
        print(f"Request: {request_json.strip()}")
        server_process.stdin.write(request_json)
        server_process.stdin.flush()
        
        # Read response
        print("Reading response...")
        response = server_process.stdout.readline().strip()
        print(f"Response: {response}")
        
        if not response:
            print("No response received")
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
        print("Reading response...")
        response = server_process.stdout.readline().strip()
        print(f"Response: {response}")
        
        if not response:
            print("No response received")
            stderr = server_process.stderr.read()
            print(f"Server stderr: {stderr}")
            return
            
        print("Test completed successfully!")
            
    except Exception as e:
        print(f"Error during test: {e}")
    finally:
        # Give some time to observe the result
        time.sleep(5)
        server_process.terminate()
        print("Server terminated")

if __name__ == "__main__":
    main() 