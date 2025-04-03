#!/usr/bin/env python
import time
import sys
import logging
import os
import subprocess
import win32gui
import win32process
import win32con
import win32api
import psutil
import ctypes
from ctypes import wintypes

def setup_logging():
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(levelname)-5.5s] %(message)s",
        handlers=[logging.StreamHandler(sys.stdout)]
    )
    logging.info("Logging initialized for direct_paint_test.py")

def launch_paint():
    """Launch Paint and wait for it to start"""
    try:
        # Try to close any existing Paint instances first
        os.system('taskkill /f /im mspaint.exe 2>nul')
        time.sleep(1)
        
        # Launch Paint maximized
        subprocess.run(["start", "/max", "mspaint.exe"], shell=True)
        logging.info("Started MS Paint")
        
        # Wait for Paint to initialize
        time.sleep(3)  # Longer wait to ensure Paint is ready
        return True
    except Exception as e:
        logging.error(f"Error launching MS Paint: {e}")
        return False

def find_paint_window():
    """Find the Paint window handle"""
    def callback(hwnd, results):
        if win32gui.IsWindowVisible(hwnd):
            title = win32gui.GetWindowText(hwnd)
            if "Paint" in title and "mcp-server" not in title:
                try:
                    _, pid = win32process.GetWindowThreadProcessId(hwnd)
                    proc = psutil.Process(pid)
                    if "mspaint" in proc.name().lower():
                        results.append(hwnd)
                except Exception as e:
                    logging.error(f"Error getting process info: {e}")
        return True
    
    results = []
    win32gui.EnumWindows(callback, results)
    
    if results:
        hwnd = results[0]
        logging.info(f"Found Paint window with handle: {hwnd}, title: {win32gui.GetWindowText(hwnd)}")
        return hwnd
    else:
        logging.error("No Paint window found")
        return None

def simple_draw_line(start_x, start_y, end_x, end_y):
    """Draw a line using simulated mouse events at absolute screen coordinates"""
    try:
        # Move mouse to start position (absolute screen coordinates)
        win32api.SetCursorPos((start_x, start_y))
        time.sleep(0.5)
        
        # Press left button
        win32api.mouse_event(win32con.MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0)
        time.sleep(0.5)
        
        # Move to end position in small steps
        steps = 10
        dx = (end_x - start_x) / steps
        dy = (end_y - start_y) / steps
        
        for i in range(1, steps + 1):
            x = int(start_x + (dx * i))
            y = int(start_y + (dy * i))
            win32api.SetCursorPos((x, y))
            time.sleep(0.05)
        
        # Ensure we're at the end position
        win32api.SetCursorPos((end_x, end_y))
        time.sleep(0.5)
        
        # Release left button
        win32api.mouse_event(win32con.MOUSEEVENTF_LEFTUP, 0, 0, 0, 0)
        time.sleep(0.5)
        
        logging.info(f"Drew line from ({start_x}, {start_y}) to ({end_x}, {end_y})")
        return True
    except Exception as e:
        logging.error(f"Error drawing line: {e}")
        return False

def main():
    setup_logging()
    
    # Launch Paint
    if not launch_paint():
        return
    
    # Find Paint window
    hwnd = find_paint_window()
    if not hwnd:
        return
    
    try:
        # Make sure Paint is visible (don't worry about activation)
        win32gui.ShowWindow(hwnd, win32con.SW_MAXIMIZE)
        time.sleep(1)
        
        # Get some coordinates to use
        rect = win32gui.GetWindowRect(hwnd)
        window_x, window_y, window_right, window_bottom = rect
        logging.info(f"Paint window rectangle: {rect}")
        
        # Calculate center points for drawing
        center_x = (window_x + window_right) // 2
        center_y = (window_y + window_bottom) // 2
        
        # Adjust for drawing area (skip the ribbon)
        drawing_y_offset = 150  # Approximate height of ribbon
        
        # Draw a horizontal line in the center
        start_x = center_x - 100
        start_y = center_y + drawing_y_offset
        end_x = center_x + 100
        end_y = center_y + drawing_y_offset
        
        logging.info(f"Drawing horizontal line from ({start_x}, {start_y}) to ({end_x}, {end_y})")
        simple_draw_line(start_x, start_y, end_x, end_y)
        
        # Wait a moment
        time.sleep(1)
        
        # Draw a vertical line intersecting the horizontal line
        start_x = center_x
        start_y = center_y + drawing_y_offset - 100
        end_x = center_x
        end_y = center_y + drawing_y_offset + 100
        
        logging.info(f"Drawing vertical line from ({start_x}, {start_y}) to ({end_x}, {end_y})")
        simple_draw_line(start_x, start_y, end_x, end_y)
        
        logging.info("Drawing operations completed")
        
    except Exception as e:
        logging.error(f"Error in main: {e}")
    
if __name__ == "__main__":
    main() 