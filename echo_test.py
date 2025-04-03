#!/usr/bin/env python
import subprocess
import threading
import sys
import time

def main():
    # Start the server
    print("Starting MCP server...")
    server_process = subprocess.Popen(
        ["cargo", "run"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1
    )
    
    # Thread for reading stdout
    def read_stdout():
        while True:
            line = server_process.stdout.readline()
            if not line:
                break
            print(f"SERVER STDOUT: {line.strip()}")
    
    # Thread for reading stderr
    def read_stderr():
        while True:
            line = server_process.stderr.readline()
            if not line:
                break
            print(f"SERVER STDERR: {line.strip()}")
    
    # Start stdout and stderr reader threads
    stdout_thread = threading.Thread(target=read_stdout, daemon=True)
    stderr_thread = threading.Thread(target=read_stderr, daemon=True)
    stdout_thread.start()
    stderr_thread.start()
    
    # Wait for server to start
    print("Waiting for server to start...")
    time.sleep(3)
    
    print("\n===== MCP SERVER ECHO TEST =====")
    print("Type JSON-RPC requests to send to the server.")
    print("Each line will be sent as a single request.")
    print("Type 'exit' to quit.")
    print("==================================\n")
    
    try:
        while True:
            try:
                user_input = input("> ")
                if user_input.lower() == 'exit':
                    break
                
                # Add newline to user input and send to server
                server_process.stdin.write(user_input + "\n")
                server_process.stdin.flush()
                print(f"Sent: {user_input}")
                
            except KeyboardInterrupt:
                break
    finally:
        print("Terminating server...")
        server_process.terminate()
        print("Server terminated.")

if __name__ == "__main__":
    main() 