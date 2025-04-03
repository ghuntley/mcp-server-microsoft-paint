#!/usr/bin/env python
import json
import subprocess
import time
import sys
import os
import threading
import traceback

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
        ["cargo", "run", "--bin", "mcp-server-microsoft-paint"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )
    
    # Keep track of if the server is still running
    server_alive = True
    
    # Create a thread to continuously read and print stderr
    def read_stderr():
        nonlocal server_alive
        while server_alive:
            try:
                line = server_process.stderr.readline()
                if not line:
                    print("SERVER STDERR: End of stderr stream")
                    break
                print(f"SERVER STDERR: {line.strip()}")
            except Exception as e:
                print(f"Error reading stderr: {e}")
                break
            
    stderr_thread = threading.Thread(target=read_stderr, daemon=True)
    stderr_thread.start()
    
    # Create a thread to continuously check if the server is running
    def check_server_alive():
        nonlocal server_alive
        while server_alive:
            if server_process.poll() is not None:
                server_alive = False
                print(f"SERVER PROCESS TERMINATED with return code {server_process.returncode}")
                break
            time.sleep(0.5)
            
    alive_thread = threading.Thread(target=check_server_alive, daemon=True)
    alive_thread.start()
    
    try:
        # Step 1: Initialize
        print("Step 1: Sending initialize request...")
        init_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }
        
        response = send_request(server_process, init_request, server_alive)
        if not response:
            print("ERROR: No response received for initialize request")
            if not server_alive:
                print("SERVER TERMINATED during or after initialize request")
                return
        else:
            print(f"Initialize response received: {response}")
        
        # Verify server still running
        if not server_alive or server_process.poll() is not None:
            print("SERVER TERMINATED after initialize request")
            return
            
        # Step 2: Connect
        print("Step 2: Sending connect request...")
        connect_request = {
            "jsonrpc": "2.0",
            "id": 2,
            "method": "connect",
            "params": {
                "client_id": "rectangle_test",
                "client_name": "Rectangle Test Client"
            }
        }
        
        try:
            response = send_request(server_process, connect_request, server_alive)
            if not response:
                print("ERROR: No response received for connect request")
            else:
                print(f"Connect response received: {response}")
                
                # Step 3: Draw a rectangle directly
                print("Step 3: Sending draw_shape request (rectangle)...")
                rectangle_request = {
                    "jsonrpc": "2.0", 
                    "id": 4,
                    "method": "draw_shape",
                    "params": {
                        "shape_type": "rectangle",
                        "start_x": 100,
                        "start_y": 100,
                        "end_x": 500,
                        "end_y": 400,
                        "thickness": 3,
                        "fill_type": "outline"  # Just the outline, not filled
                    }
                }
                
                response = send_request(server_process, rectangle_request, server_alive)
                if not response:
                    print("ERROR: No response received for draw_shape request")
        except BrokenPipeError:
            print("ERROR: Server pipe closed before or during connect request")
        except OSError as e:
            print(f"ERROR: OSError during connect request: {e}")
            traceback.print_exc()
        
        print("Test completed! Check Paint to see if a rectangle was drawn.")
        print("Press Enter to close the test and kill Paint...")
        input()
        
    except Exception as e:
        print(f"Test failed with error: {type(e).__name__}: {e}")
        traceback.print_exc()
    finally:
        server_alive = False
        if server_process.poll() is None:
            print("Terminating server process...")
            server_process.terminate()
        print("Server process terminated")
        
        if paint_process.poll() is None:
            print("Terminating Paint process...")
            paint_process.terminate()
        print("Paint process terminated")
        
def send_request(process, request, server_alive):
    """Send a request to the server and print the response."""
    if not server_alive or process.poll() is not None:
        print("Cannot send request - server process is not running")
        return None
        
    request_str = json.dumps(request) + "\n"
    print(f"SENDING: {request_str.strip()}")
    
    try:
        process.stdin.write(request_str)
        process.stdin.flush()
        print("Request sent and flushed")
    except BrokenPipeError:
        print("ERROR: Broken pipe when trying to send request")
        raise
    except OSError as e:
        print(f"ERROR: OSError when trying to send request: {e}")
        raise
    
    # Read response with timeout
    start_time = time.time()
    timeout = 10  # seconds
    
    while time.time() - start_time < timeout and server_alive and process.poll() is None:
        try:
            print("Waiting for response...")
            line = process.stdout.readline().strip()
            if line:
                print(f"RESPONSE: {line}")
                try:
                    return json.loads(line)
                except json.JSONDecodeError:
                    print(f"WARNING: Received non-JSON response: {line}")
        except Exception as e:
            print(f"Error reading response: {e}")
            return None
        time.sleep(0.5)
    
    if not server_alive or process.poll() is not None:
        print("WARNING: Server terminated while waiting for response")
    else:
        print("WARNING: No response received within timeout")
    return None

if __name__ == "__main__":
    main() 