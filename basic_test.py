#!/usr/bin/env python
import json
import subprocess
import sys
import time
import os

def main():
    # Kill any existing Paint processes
    os.system('taskkill /f /im mspaint.exe 2>nul')
    time.sleep(1)
    
    # Launch Paint
    subprocess.run(["start", "mspaint.exe"], shell=True)
    print("Launched MS Paint")
    time.sleep(3)  # Wait for Paint to start
    
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
        # Wait for server to start
        time.sleep(3)
        
        # Connect request
        print("Sending connect request...")
        connect_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "connect",
            "params": {
                "client_id": "basic-test",
                "client_name": "Basic Test Client"
            }
        }
        
        # Send the request
        request_json = json.dumps(connect_request) + "\n"
        server_process.stdin.write(request_json)
        server_process.stdin.flush()
        
        # Read response with timeout
        response = ""
        start_time = time.time()
        while time.time() - start_time < 10:  # 10 second timeout
            try:
                line = server_process.stdout.readline().strip()
                if line:
                    print(f"Received: {line}")
                    try:
                        response = json.loads(line)
                        break
                    except json.JSONDecodeError:
                        print(f"Not valid JSON: {line}")
            except Exception as e:
                print(f"Error reading response: {e}")
            time.sleep(0.1)
        
        if not response:
            print("No response received")
            stderr = server_process.stderr.read()
            if stderr:
                print(f"Server stderr: {stderr}")
            return
            
        print(f"Connect response: {response}")
        
        # Activate window request
        print("Sending activate_window request...")
        activate_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "activate_window",
            "params": {}
        }
        
        request_json = json.dumps(activate_request) + "\n"
        server_process.stdin.write(request_json)
        server_process.stdin.flush()
        
        # Read response
        response = ""
        start_time = time.time()
        while time.time() - start_time < 5:  # 5 second timeout
            line = server_process.stdout.readline().strip()
            if line:
                print(f"Received: {line}")
                try:
                    response = json.loads(line)
                    break
                except json.JSONDecodeError:
                    pass
            time.sleep(0.1)
        
        if not response:
            print("No response received for activate_window")
            return
            
        print(f"Activate response: {response}")
        
        # Draw a simple line
        print("Drawing a horizontal line...")
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
        server_process.stdin.write(request_json)
        server_process.stdin.flush()
        
        # Read response
        response = ""
        start_time = time.time()
        while time.time() - start_time < 10:  # 10 second timeout for drawing
            line = server_process.stdout.readline().strip()
            if line:
                print(f"Received: {line}")
                try:
                    response = json.loads(line)
                    break
                except json.JSONDecodeError:
                    pass
            time.sleep(0.1)
        
        if not response:
            print("No response received for draw_line")
            return
            
        print(f"Draw line response: {response}")
        
        # Wait to observe the result
        print("Test completed. Wait 5 seconds before cleanup...")
        time.sleep(5)
        
    except Exception as e:
        print(f"Error during test: {e}")
    finally:
        server_process.terminate()
        print("Server terminated")

if __name__ == "__main__":
    main() 