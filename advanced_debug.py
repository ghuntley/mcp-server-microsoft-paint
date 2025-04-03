import subprocess
import time
import json
import sys
import os
import datetime
import platform
import tempfile
import logging  # Import logging

# Enable detailed logging (controlled by logger level now)
# DEBUG_MODE = True # Removed
LOG_FILE = os.path.join(tempfile.gettempdir(), "mcp_advanced_debug.log")

# Removed custom log_debug function
# def log_debug(message):
#     """Log debug message to console and file"""
#     timestamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")
#     log_line = f"[DEBUG] {timestamp} - {message}"
#     
#     if DEBUG_MODE:
#         print(log_line)
#     
#     with open(LOG_FILE, "a", encoding="utf-8") as f:
#         f.write(log_line + "\n")

def get_windows_process_tree():
    """Capture the Windows process tree using wmic"""
    logging.debug("Capturing process tree...") # Use logging
    processes = []
    
    # First try the more detailed approach with wmic
    try:
        # Get process info: ID, Parent ID, Name, Command Line
        output = subprocess.check_output(
            ["wmic", "process", "get", "ProcessID,ParentProcessID,Name,CommandLine", "/format:list"], 
            text=True, 
            stderr=subprocess.STDOUT
        )
        processes.append("=== WMIC PROCESS LIST ===")
        processes.append(output)
    except Exception as e:
        logging.debug(f"Error getting wmic process info: {e}") # Use logging
    
    # Also try tasklist for a different view
    try:
        output = subprocess.check_output(
            ["tasklist", "/v"], 
            text=True,
            stderr=subprocess.STDOUT
        )
        processes.append("\n=== TASKLIST OUTPUT ===")
        processes.append(output)
    except Exception as e:
        logging.debug(f"Error getting tasklist info: {e}") # Use logging
    
    # Specifically check for mspaint.exe processes
    try:
        output = subprocess.check_output(
            ["tasklist", "/FI", "IMAGENAME eq mspaint.exe", "/v"], 
            text=True,
            stderr=subprocess.STDOUT
        )
        processes.append("\n=== MSPAINT PROCESSES ===")
        processes.append(output)
    except Exception as e:
        logging.debug(f"Error getting mspaint process info: {e}") # Use logging
    
    return "\n".join(processes)

def dump_process_tree(label=""):
    """Dump process tree information to the log"""
    if platform.system() == "Windows":
        process_info = get_windows_process_tree()
        logging.debug(f"Process Tree {label}:\n{process_info}") # Use logging
    else:
        logging.debug(f"Process tree capture not implemented for this platform: {platform.system()}") # Use logging

def collect_system_info():
    """Collect system information for debugging"""
    info = {}
    
    # Basic system info
    info["platform"] = platform.platform()
    info["python_version"] = sys.version
    info["cwd"] = os.getcwd()
    
    # Environment variables that might be relevant
    env_vars = ["PATH", "TEMP", "TMP", "SYSTEMROOT", "WINDIR"]
    info["environment"] = {var: os.environ.get(var, "Not set") for var in env_vars}
    
    # Check if mspaint.exe exists in common locations
    paint_paths = [
        os.path.join(os.environ.get("SYSTEMROOT", "C:\\Windows"), "system32", "mspaint.exe"),
        os.path.join(os.environ.get("WINDIR", "C:\\Windows"), "mspaint.exe")
    ]
    info["mspaint_paths"] = {path: os.path.exists(path) for path in paint_paths}
    
    # Check if the server executable exists
    server_path = os.path.join("target", "release", "mcp-server-microsoft-paint.exe")
    info["server_executable"] = os.path.exists(server_path)
    
    return info

def run_advanced_test():
    """Run a test of the MCP server with enhanced debugging"""
    # Configure logging
    log_formatter = logging.Formatter("%(asctime)s [%(levelname)-5.5s] %(message)s")
    root_logger = logging.getLogger()
    
    # Remove existing handlers if any (useful for rerunning in interactive sessions)
    for handler in root_logger.handlers[:]:
        root_logger.removeHandler(handler)
        handler.close()

    root_logger.setLevel(logging.DEBUG)

    # File Handler
    file_handler = logging.FileHandler(LOG_FILE, mode='w', encoding='utf-8')
    file_handler.setFormatter(log_formatter)
    root_logger.addHandler(file_handler)

    # Console Handler
    console_handler = logging.StreamHandler(sys.stdout) # Use stdout to match previous print behavior
    console_handler.setFormatter(log_formatter)
    root_logger.addHandler(console_handler)

    logging.info(f"=== MCP SERVER ADVANCED DEBUG LOG - {datetime.datetime.now()} ===")
    logging.info("Logging initialized. Outputting to console and %s", LOG_FILE)

    # Start new log file (handled by FileHandler mode='w')
    # with open(LOG_FILE, "w", encoding="utf-8") as f:
    #     f.write(f"=== MCP SERVER ADVANCED DEBUG LOG - {datetime.datetime.now()} ===\n\n")
    
    logging.debug("Starting advanced Paint MCP server debugging session...") # Use logging
    
    # Collect and log system information
    system_info = collect_system_info()
    logging.debug(f"System Information:\n{json.dumps(system_info, indent=2)}") # Use logging
    
    # Log process tree before starting the server
    dump_process_tree("BEFORE SERVER START")
    
    # Launch the server with binary mode for pipes
    server_process = None
    try:
        logging.debug("Launching MCP server process...") # Use logging
        server_path = os.path.join("target", "release", "mcp-server-microsoft-paint.exe")
        
        # Add environment variables for more verbose Rust logging
        env = os.environ.copy()
        env["RUST_LOG"] = "debug,mcp_server_microsoft_paint=debug,windows=debug"
        
        server_process = subprocess.Popen(
            [server_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=False,  # Use binary mode
            env=env,
            creationflags=subprocess.CREATE_NEW_PROCESS_GROUP  # Windows-specific
        )
        
        logging.debug(f"Server process launched with PID: {server_process.pid}") # Use logging
        
        # Wait for server to initialize
        logging.debug("Waiting for server initialization (3 seconds)...") # Use logging
        time.sleep(3)
        
        # Log process tree after server start
        dump_process_tree("AFTER SERVER START")
        
        # Define test sequence with longer waits between steps
        tests = [
            {
                "name": "Get Version",
                "request": {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "get_version",
                    "params": {}
                },
                "wait_after": 2  # seconds to wait after this test
            },
            {
                "name": "Connect",
                "request": {
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "connect",
                    "params": {
                        "client_id": "advanced-debug-client",
                        "client_name": "Advanced Debug Client"
                    }
                },
                "wait_after": 5  # longer wait after connect to let Paint launch
            },
            {
                "name": "Activate Window",
                "request": {
                    "jsonrpc": "2.0",
                    "id": 3,
                    "method": "activate_window",
                    "params": {}
                },
                "wait_after": 3
            },
            {
                "name": "Get Canvas Dimensions",
                "request": {
                    "jsonrpc": "2.0",
                    "id": 4,
                    "method": "get_canvas_dimensions",
                    "params": {}
                },
                "wait_after": 2
            }
        ]
        
        # Run each test
        for test_index, test in enumerate(tests):
            logging.debug(f"\n[TEST {test_index+1}/{len(tests)}] Running test: {test['name']}") # Use logging
            request = test["request"]
            request_json = json.dumps(request)
            logging.debug(f"Sending request: {request_json}") # Use logging
            
            try:
                # Send request to server as bytes
                request_bytes = (request_json + "\n").encode('utf-8')
                server_process.stdin.write(request_bytes)
                server_process.stdin.flush()
                logging.debug(f"Request sent to server, {len(request_bytes)} bytes") # Use logging
                
                # Read response (with timeout)
                start_time = time.time()
                timeout = 20  # longer timeout for debugging
                response = None
                
                logging.debug(f"Waiting for response (timeout: {timeout}s)...") # Use logging
                
                # Buffer for partial output
                output_buffer = b""
                
                while time.time() - start_time < timeout:
                    # Check if server is still running
                    if server_process.poll() is not None:
                        logging.debug(f"Server process has exited with code: {server_process.poll()}") # Use logging
                        break
                    
                    # Try to read a line
                    output_line = server_process.stdout.readline()
                    
                    if output_line:
                        try:
                            # Try to decode the response
                            line_text = output_line.decode('utf-8').strip()
                            logging.debug(f"Received raw output: {line_text}") # Use logging
                            
                            response = json.loads(line_text)
                            logging.debug(f"Parsed JSON response successfully") # Use logging
                            break
                        except json.JSONDecodeError as je:
                            logging.debug(f"Invalid JSON response: {je}") # Use logging
                            output_buffer += output_line
                    else:
                        # No data available, sleep briefly to avoid CPU spin
                        time.sleep(0.1)
                
                if response:
                    logging.debug(f"Response: {json.dumps(response, indent=2)}") # Use logging
                else:
                    logging.debug(f"No valid response received (timeout or error)") # Use logging
                    logging.debug(f"Accumulated output buffer: {output_buffer}") # Use logging
                    if output_buffer:
                        try:
                            logging.debug(f"Trying to decode buffer: {output_buffer.decode('utf-8', errors='replace')}") # Use logging
                        except Exception as decode_err:
                            logging.error(f"Error decoding buffer: {decode_err}") # Use logging
                            logging.error(f"Raw buffer content (repr): {repr(output_buffer)}") # Use logging

            except Exception as e:
                logging.error(f"Error during test '{test['name']}': {e}") # Use logging
                # Log server stderr if available
                try:
                    stderr_output = server_process.stderr.read().decode('utf-8', errors='replace')
                    if stderr_output:
                        logging.error(f"Server stderr output:\n{stderr_output}") # Use logging
                except Exception as stderr_e:
                    logging.error(f"Could not read server stderr: {stderr_e}") # Use logging
                break # Stop tests on error
            
            # Wait after test completion
            wait_duration = test.get("wait_after", 1)
            logging.debug(f"Waiting {wait_duration} seconds...") # Use logging
            time.sleep(wait_duration)
        
        # Final process tree check
        dump_process_tree("AFTER TESTS")
        
        # Check if Paint is still running
        paint_running = False
        try:
            output = subprocess.check_output(["tasklist", "/FI", "IMAGENAME eq mspaint.exe"], text=True)
            if "mspaint.exe" in output:
                paint_running = True
        except Exception:
            pass # tasklist might fail if no process found
        logging.debug(f"Is Paint process running after tests? {paint_running}") # Use logging
        
    except Exception as e:
        logging.error(f"An error occurred during the test run: {e}") # Use logging
        import traceback
        logging.error(traceback.format_exc()) # Log full traceback
        
    finally:
        # Log final server process status
        if server_process:
            server_status = server_process.poll()
            if server_status is None:
                logging.debug("Terminating server process...") # Use logging
                try:
                    # Attempt graceful termination first
                    server_process.terminate()
                    try:
                        server_process.wait(timeout=5) # Wait for termination
                        logging.debug(f"Server terminated with code: {server_process.returncode}") # Use logging
                    except subprocess.TimeoutExpired:
                        logging.warning("Server did not terminate gracefully, killing...") # Use logging
                        server_process.kill()
                        server_process.wait()
                        logging.debug("Server killed.") # Use logging
                except Exception as term_err:
                    logging.error(f"Error terminating server: {term_err}") # Use logging
                    try:
                        logging.warning("Attempting to kill server process forcefully...") # Use logging
                        server_process.kill()
                        server_process.wait()
                        logging.debug("Server killed.") # Use logging
                    except Exception as kill_err:
                        logging.error(f"Error killing server: {kill_err}") # Use logging
            else:
                logging.debug(f"Server process already exited with code: {server_status}") # Use logging
            
            # Log any remaining stderr/stdout
            try:
                stdout, stderr = server_process.communicate()
                if stdout:
                    logging.debug(f"Final server stdout:\n{stdout.decode('utf-8', errors='replace')}") # Use logging
                if stderr:
                    logging.error(f"Final server stderr:\n{stderr.decode('utf-8', errors='replace')}") # Use logging
            except Exception as comm_err:
                logging.error(f"Error getting final server output: {comm_err}") # Use logging
        else:
            logging.debug("Server process was not successfully started.") # Use logging
        
        # Final process tree check after termination
        dump_process_tree("AFTER SERVER TERMINATION")
        logging.info("Advanced debugging session finished.") # Use logging
        logging.info(f"Full log available at: {LOG_FILE}") # Use logging

if __name__ == "__main__":
    run_advanced_test() 