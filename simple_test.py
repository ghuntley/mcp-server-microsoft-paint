import subprocess
import time
import json
import sys
import logging

def setup_logging():
    log_formatter = logging.Formatter("%(asctime)s [%(levelname)-5.5s] %(message)s")
    root_logger = logging.getLogger()
    root_logger.setLevel(logging.DEBUG)

    # Console Handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setFormatter(log_formatter)
    root_logger.addHandler(console_handler)
    logging.info("Logging initialized for simple_test.py")

def run_minimal_test():
    setup_logging()
    logging.info("Starting minimal Paint MCP server test...")
    
    # Launch the server with binary mode for pipes
    server_process = None
    try:
        logging.debug("Launching MCP server process...")
        server_process = subprocess.Popen(
            ["target/release/mcp-server-microsoft-paint.exe"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=False  # Use binary mode
        )
        logging.debug(f"Server process launched with PID: {server_process.pid}")
    except Exception as e:
        logging.error(f"Failed to launch server process: {e}")
        return
    
    # Wait for server to initialize
    logging.debug("Waiting for server initialization (2 seconds)...")
    time.sleep(2)
    
    # Define a simple test with just connect and get_version
    tests = [
        {
            "name": "Get Version",
            "request": {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "get_version",
                "params": {}
            }
        },
        {
            "name": "Connect",
            "request": {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "connect",
                "params": {
                    "client_id": "test-client",
                    "client_name": "Simple Test"
                }
            }
        },
        {
            "name": "Activate Window",
            "request": {
                "jsonrpc": "2.0",
                "id": 3,
                "method": "activate_window",
                "params": {}
            }
        }
    ]
    
    try:
        # Run each test
        for test in tests:
            logging.info(f"\nRunning test: {test['name']}")
            request = test["request"]
            request_json = json.dumps(request)
            logging.debug(f"Sending: {request_json}")
            
            try:
                # Send request to server as bytes
                request_bytes = (request_json + "\n").encode('utf-8')
                server_process.stdin.write(request_bytes)
                server_process.stdin.flush()
                logging.debug(f"Request sent ({len(request_bytes)} bytes)")
                
                # Read response (with timeout)
                start_time = time.time()
                timeout = 10  # seconds
                response = None
                logging.debug(f"Waiting for response (timeout: {timeout}s)...")
                
                while time.time() - start_time < timeout:
                    # Check if server has exited
                    if server_process.poll() is not None:
                        logging.warning(f"Server exited prematurely with code: {server_process.poll()}")
                        break
                        
                    # Read line as bytes and decode
                    line_bytes = server_process.stdout.readline()
                    if not line_bytes:
                        time.sleep(0.1) # Avoid busy-waiting
                        continue
                        
                    line = line_bytes.decode('utf-8').strip()
                    logging.debug(f"Received raw line: {line}")
                    if line:
                        try:
                            response = json.loads(line)
                            logging.debug("Parsed JSON response successfully")
                            break
                        except json.JSONDecodeError as je:
                            logging.error(f"Invalid JSON response: {line}, Error: {je}")
                            # Potentially continue reading if it's just debug output?
                            # For now, we assume one line per response.
                            # If the server mixes debug and JSON, this needs adjustment.
                    
                if response:
                    logging.info(f"Response: {json.dumps(response, indent=2)}")
                else:
                    logging.warning("No valid response received (timeout or server exit)")
                    # Attempt to read stderr after timeout/exit
                    break
                    
            except Exception as e:
                logging.error(f"Error during test '{test['name']}': {e}")
                import traceback
                logging.error(traceback.format_exc())
                break
        
        # Wait for a moment to see if Paint stays open
        logging.info("\nTests completed. Waiting 5 seconds to see if Paint stays open...")
        time.sleep(5)
            
    finally:
        # Clean up the server process
        if server_process and server_process.poll() is None:
            logging.info("\nTerminating server process...")
            try:
                server_process.terminate()
                outs, errs = server_process.communicate(timeout=3)
                logging.debug(f"Server terminated with code: {server_process.returncode}")
                if outs:
                    logging.debug(f"Final server stdout:\n{outs.decode('utf-8', errors='replace')}")
                if errs:
                    logging.warning(f"Final server stderr:\n{errs.decode('utf-8', errors='replace')}")
            except subprocess.TimeoutExpired:
                logging.warning("Server did not terminate gracefully, killing...")
                server_process.kill()
                outs, errs = server_process.communicate()
                logging.debug("Server killed.")
                if outs:
                    logging.debug(f"Final server stdout:\n{outs.decode('utf-8', errors='replace')}")
                if errs:
                    logging.warning(f"Final server stderr:\n{errs.decode('utf-8', errors='replace')}")
            except Exception as e:
                logging.error(f"Error during termination: {e}")
        elif server_process:
             logging.info(f"Server process already exited with code: {server_process.poll()}")
             # Read remaining output/error
             try:
                 outs, errs = server_process.communicate()
                 if outs:
                    logging.debug(f"Remaining server stdout:\n{outs.decode('utf-8', errors='replace')}")
                 if errs:
                    logging.warning(f"Remaining server stderr:\n{errs.decode('utf-8', errors='replace')}")
             except Exception as e:
                 logging.error(f"Error reading remaining server output: {e}")
        else:
            logging.info("Server process was not running or failed to start.")

        logging.info("Minimal test finished.")

if __name__ == "__main__":
    run_minimal_test() 