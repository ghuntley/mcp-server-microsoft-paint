import subprocess
import time
import os
import sys
import json
import logging

def setup_logging():
    log_formatter = logging.Formatter("%(asctime)s [%(levelname)-5.5s] %(message)s")
    root_logger = logging.getLogger()
    root_logger.setLevel(logging.INFO) # Default to INFO for this utility

    # Console Handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setFormatter(log_formatter)
    root_logger.addHandler(console_handler)
    logging.info("Logging initialized for debug_mcp_server.py")

def print_divider(title=""):
    """Print a section divider using logging"""
    logging.info("\n" + "=" * 60)
    if title:
        logging.info(title.center(60))
        logging.info("=" * 60)

def build_server():
    """Build the MCP server using cargo"""
    print_divider("Building MCP Server")
    try:
        logging.debug("Running 'cargo build --release'")
        result = subprocess.run(['cargo', 'build', '--release'], 
                               capture_output=True,
                               text=True)
        if result.returncode == 0:
            logging.info("Build successful")
            if result.stdout:
                 logging.debug(f"Build STDOUT:\n{result.stdout}")
        else:
            logging.error(f"Build failed with return code {result.returncode}")
            if result.stdout:
                 logging.error(f"Build STDOUT:\n{result.stdout}")
            if result.stderr:
                 logging.error(f"Build STDERR:\n{result.stderr}")
        return result.returncode == 0
    except FileNotFoundError:
        logging.error("'cargo' command not found. Make sure Rust and Cargo are installed and in your PATH.")
        return False
    except Exception as e:
        logging.error(f"Exception during build: {e}")
        return False

def launch_paint():
    """Attempt to launch MS Paint directly"""
    print_divider("Launching MS Paint directly")
    try:
        # Try multiple possible locations for mspaint.exe
        locations = [
            "mspaint.exe",  # Try PATH first
            "C:\\Windows\\System32\\mspaint.exe",
            "C:\\Windows\\mspaint.exe"
        ]
        
        for location in locations:
            logging.debug(f"Trying to launch from: {location}")
            try:
                process = subprocess.Popen([location], 
                                         stdout=subprocess.PIPE, 
                                         stderr=subprocess.PIPE,
                                         text=True)
                logging.debug(f"Launched process with PID: {process.pid}")
                time.sleep(2)  # Wait for process to potentially start/exit
                
                # Check if it's still running
                exit_code = process.poll()
                if exit_code is None:
                    logging.info(f"Successfully launched MS Paint from {location} (PID: {process.pid}). Terminating...")
                    process.terminate()
                    try:
                        process.wait(timeout=3)
                        logging.debug("Paint process terminated.")
                    except subprocess.TimeoutExpired:
                        logging.warning(f"Paint process {process.pid} did not terminate gracefully, killing.")
                        process.kill()
                        process.wait()
                        logging.debug("Paint process killed.")
                    return True
                else:
                    logging.warning(f"Process from {location} exited quickly with code: {exit_code}")
                    stdout, stderr = process.communicate()
                    if stdout:
                        logging.debug(f"STDOUT: {stdout}")
                    if stderr:
                        logging.warning(f"STDERR: {stderr}")
            except FileNotFoundError:
                 logging.warning(f"Executable not found at: {location}")
            except Exception as e:
                logging.error(f"Error launching from {location}: {e}")
        
        logging.error("Failed to launch Paint from any known location.")
        return False
    except Exception as e:
        logging.error(f"Exception during Paint launch test: {e}")
        return False

def test_manual_server_launch():
    """Test launching the MCP server manually and passing a simple command"""
    print_divider("Testing Manual Server Launch")
    server_process = None
    try:
        server_path = os.path.join("target", "release", "mcp-server-microsoft-paint.exe")
        if not os.path.exists(server_path):
            logging.error(f"Server executable not found at {server_path}")
            return False
        
        logging.info(f"Launching server from {server_path}")
        
        # Launch the server process
        server_process = subprocess.Popen(
            [server_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True, # Use text mode for direct interaction in this test
            encoding='utf-8' # Specify encoding
        )
        logging.debug(f"Server process launched with PID: {server_process.pid}")
        
        # Wait for server to initialize
        logging.debug("Waiting for server initialization (1 second)...")
        time.sleep(1)
        
        # Send a get_version command
        request = json.dumps({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "get_version",
            "params": {}
        }) + "\n"
        
        logging.debug(f"Sending request: {request.strip()}")
        server_process.stdin.write(request)
        server_process.stdin.flush()
        
        # Read response with timeout
        output_lines = []
        stderr_lines = []
        
        # Non-blocking read with timeout
        start_time = time.time()
        timeout = 5  # seconds
        logging.debug(f"Waiting for response (timeout: {timeout}s)")
        
        response_received = False
        while time.time() - start_time < timeout:
            # Check if process is still running
            if server_process.poll() is not None:
                logging.warning(f"Server process exited prematurely with code: {server_process.poll()}")
                break
            
            # Check for output
            stdout_line = server_process.stdout.readline().strip()
            if stdout_line:
                logging.debug(f"Received stdout line: {stdout_line}")
                output_lines.append(stdout_line)
                # Assuming the first valid JSON is the response
                try:
                    json.loads(stdout_line) 
                    response_received = True
                    break # Got the response
                except json.JSONDecodeError:
                    logging.warning(f"Received non-JSON line on stdout: {stdout_line}")
                    # Continue reading, might be debug info
            
            # Check for stderr output (non-blocking)
            # This part is tricky with readline, might need select or threading for robust non-blocking stderr
            # For simplicity, we'll read stderr at the end
            
            # Brief pause to avoid CPU spin
            time.sleep(0.1)
        
        if not response_received and server_process.poll() is None:
             logging.warning("Timeout waiting for response.")
             
        # Clean up
        if server_process.poll() is None:
            logging.debug("Terminating server process...")
            server_process.terminate()
            try:
                stdout_rem, stderr_rem = server_process.communicate(timeout=3)
                logging.debug(f"Server terminated with code: {server_process.returncode}")
                if stdout_rem:
                    logging.debug(f"Remaining stdout:\n{stdout_rem}")
                    output_lines.append(stdout_rem)
                if stderr_rem:
                    logging.warning(f"Remaining stderr:\n{stderr_rem}")
                    stderr_lines.append(stderr_rem)
            except subprocess.TimeoutExpired:
                logging.warning("Server did not terminate gracefully, killing...")
                server_process.kill()
                stdout_rem, stderr_rem = server_process.communicate()
                logging.debug("Server killed.")
                if stdout_rem:
                    logging.debug(f"Remaining stdout:\n{stdout_rem}")
                    output_lines.append(stdout_rem)
                if stderr_rem:
                    logging.warning(f"Remaining stderr:\n{stderr_rem}")
                    stderr_lines.append(stderr_rem)
        else: # Server already exited
            logging.debug("Server already exited. Reading remaining output.")
            stdout_rem, stderr_rem = server_process.communicate()
            if stdout_rem:
                logging.debug(f"Remaining stdout:\n{stdout_rem}")
                output_lines.append(stdout_rem)
            if stderr_rem:
                logging.warning(f"Remaining stderr:\n{stderr_rem}")
                stderr_lines.append(stderr_rem)
        
        # Print the results
        if response_received:
            logging.info("Response received:")
            for line in output_lines:
                logging.info(line) # Log each line of output
            if stderr_lines: # Also log stderr if any was captured
                 logging.warning("Stderr received:")
                 for line in stderr_lines:
                     logging.warning(line)
            return True
        else:
            logging.error("No valid JSON response received from server")
            if output_lines:
                logging.error("Raw stdout received:")
                for line in output_lines:
                    logging.error(line)
            if stderr_lines:
                logging.error("Stderr received:")
                for line in stderr_lines:
                     logging.error(line)
            return False
    
    except Exception as e:
        logging.error(f"Exception during server test: {e}")
        import traceback
        logging.error(traceback.format_exc())
        # Ensure process termination if started
        if server_process and server_process.poll() is None:
             logging.warning("Terminating server process due to exception...")
             server_process.kill()
             server_process.wait()
        return False

def main():
    setup_logging()
    logging.info("MCP Server for Windows 11 Paint Debug Utility")
    logging.info(f"Python version: {sys.version}")
    logging.info(f"Current directory: {os.getcwd()}")
    
    # Run the tests
    build_ok = build_server()
    paint_ok = launch_paint()
    server_ok = test_manual_server_launch()
    
    # Summary
    print_divider("Summary")
    logging.info(f"Build successful: {build_ok}")
    logging.info(f"MS Paint launch successful: {paint_ok}")
    logging.info(f"Server test successful: {server_ok}")
    
    if not paint_ok:
        logging.warning("\nPossible issues with MS Paint:")
        logging.warning("1. MS Paint may not be installed on this system")
        logging.warning("2. MS Paint executable might be in a non-standard location")
        logging.warning("3. There might be permission issues launching MS Paint")
    
    if not server_ok:
        logging.warning("\nPossible issues with MCP Server:")
        logging.warning("1. Server might be failing to initialize")
        logging.warning("2. JSON-RPC handling might be incorrect")
        logging.warning("3. Server might be failing to find or launch MS Paint")
        logging.warning("4. Server output format might not match what we're expecting")
        logging.warning("Check server logs (stderr) for more details if possible.")

if __name__ == "__main__":
    main() 