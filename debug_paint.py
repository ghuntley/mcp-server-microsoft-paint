import subprocess
import time
import os
import sys
import logging

def setup_logging():
    log_formatter = logging.Formatter("%(asctime)s [%(levelname)-5.5s] %(message)s")
    root_logger = logging.getLogger()
    root_logger.setLevel(logging.INFO) # Default to INFO for this utility

    # Console Handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setFormatter(log_formatter)
    root_logger.addHandler(console_handler)
    logging.info("Logging initialized for debug_paint.py")

def is_paint_running():
    """Check if MS Paint is running using tasklist"""
    try:
        result = subprocess.run(['tasklist', '/FI', 'IMAGENAME eq mspaint.exe'], 
                              capture_output=True, text=True, check=False)
        is_running = 'mspaint.exe' in result.stdout
        logging.debug(f"tasklist check for mspaint.exe: stdout='{result.stdout[:50]}...', running={is_running}")
        return is_running
    except FileNotFoundError:
        logging.error("'tasklist' command not found. Cannot check if Paint is running.")
        return False
    except Exception as e:
        logging.error(f"Error checking if Paint is running: {e}")
        return False

def main():
    setup_logging()
    logging.info("Debugging MS Paint launch...")
    
    # Check if Paint is already running
    if is_paint_running():
        logging.info("MS Paint is already running")
    else:
        logging.info("MS Paint is not running, attempting to launch...")
        
        process = None
        try:
            # Try to launch MS Paint
            logging.debug("Launching 'mspaint.exe'...")
            process = subprocess.Popen(['mspaint.exe'], 
                                      stdout=subprocess.PIPE, 
                                      stderr=subprocess.PIPE)
            logging.debug(f"Paint process started with PID: {process.pid}")
            
            # Wait a bit for Paint to start
            logging.debug("Waiting 2 seconds for Paint to initialize...")
            time.sleep(2)
            
            # Check if it's running now
            if is_paint_running():
                logging.info("Successfully launched MS Paint")
            else:
                logging.error("Failed to launch MS Paint or it exited quickly!")
                
            # Get the exit code (should be None if still running)
            exit_code = process.poll()
            logging.info(f"Paint process exit code after 2s wait: {exit_code}")
            
        except FileNotFoundError:
            logging.error("'mspaint.exe' not found in PATH. Cannot launch Paint.")
        except Exception as e:
            logging.error(f"Error launching MS Paint: {e}")
        finally:
            # Try to terminate the process if it was started and might still be running
            if process and process.poll() is None:
                logging.debug("Terminating MS Paint process...")
                process.terminate()
                try:
                    outs, errs = process.communicate(timeout=3)
                    logging.debug(f"Paint process terminated with code: {process.returncode}")
                    if outs:
                         logging.debug(f"Paint stdout:\n{outs.decode('utf-8', errors='replace')}")
                    if errs:
                         logging.warning(f"Paint stderr:\n{errs.decode('utf-8', errors='replace')}")
                except subprocess.TimeoutExpired:
                     logging.warning("Paint did not terminate gracefully, killing...")
                     process.kill()
                     outs, errs = process.communicate()
                     logging.debug("Paint killed.")
                     if outs:
                          logging.debug(f"Paint stdout:\n{outs.decode('utf-8', errors='replace')}")
                     if errs:
                          logging.warning(f"Paint stderr:\n{errs.decode('utf-8', errors='replace')}")
                except Exception as term_err:
                     logging.error(f"Error terminating Paint process: {term_err}")
            elif process:
                 logging.debug(f"Paint process already exited with code: {process.poll()}")
                 # Read remaining streams
                 try:
                    outs, errs = process.communicate()
                    if outs:
                         logging.debug(f"Paint stdout:\n{outs.decode('utf-8', errors='replace')}")
                    if errs:
                         logging.warning(f"Paint stderr:\n{errs.decode('utf-8', errors='replace')}")
                 except Exception as comm_err:
                      logging.error(f"Error reading Paint streams after exit: {comm_err}")
    
    logging.info("\nSystem info:")
    logging.info(f"Python version: {sys.version}")
    logging.info(f"Current directory: {os.getcwd()}")
    
    # List mspaint.exe in Windows directory
    logging.info("\nChecking for mspaint.exe in common Windows directories:")
    windows_dirs = [
        os.environ.get('WINDIR', 'C:\\Windows'),
        os.path.join(os.environ.get('WINDIR', 'C:\\Windows'), 'System32'),
        # os.path.join(os.environ.get('PROGRAMFILES', 'C:\\Program Files'), 'WindowsApps') # Usually restricted access
        os.environ.get('SystemRoot', 'C:\\Windows') # Often same as WINDIR
    ]
    
    # Deduplicate paths
    checked_paths = set()
    for directory in windows_dirs:
        if not directory or not os.path.isdir(directory):
             logging.debug(f"Skipping invalid or inaccessible directory: {directory}")
             continue
             
        mspaint_path = os.path.join(directory, 'mspaint.exe')
        if mspaint_path in checked_paths:
             continue
        checked_paths.add(mspaint_path)
        
        if os.path.exists(mspaint_path):
            logging.info(f"Found mspaint.exe at: {mspaint_path}")
        else:
            logging.info(f"mspaint.exe not found at: {mspaint_path}")
            
    # Also check PATH
    logging.info("\nChecking if 'mspaint.exe' is in system PATH...")
    try:
        result = subprocess.run(['where', 'mspaint.exe'], capture_output=True, text=True, check=False)
        if result.returncode == 0 and result.stdout:
            logging.info(f"Found mspaint.exe via PATH: {result.stdout.strip()}")
        else:
            logging.warning("'mspaint.exe' not found in PATH.")
    except FileNotFoundError:
        logging.warning("'where' command not found. Cannot check PATH.")
    except Exception as e:
        logging.error(f"Error checking PATH for mspaint.exe: {e}")
        
    logging.info("\nPaint debug check finished.")

if __name__ == "__main__":
    main() 