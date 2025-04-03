#!/usr/bin/env python
import time
import sys
import os
import subprocess
import win32gui
import win32process
import win32con
import win32api
import psutil

def main():
    # Kill any existing Paint processes
    os.system('taskkill /f /im mspaint.exe 2>nul')
    time.sleep(1)
    
    # Launch Paint
    subprocess.run(["start", "mspaint.exe"], shell=True)
    print("Launched MS Paint")
    time.sleep(3)  # Wait for Paint to start
    
    # Find the Paint window
    hwnd = find_paint_window()
    if not hwnd:
        print("Could not find Paint window")
        return
    
    print(f"Found Paint window: {hwnd}")
    
    # Make sure Paint is visible
    win32gui.ShowWindow(hwnd, win32con.SW_MAXIMIZE)
    time.sleep(1)
    
    # Get window dimensions
    rect = win32gui.GetWindowRect(hwnd)
    window_x, window_y, window_right, window_bottom = rect
    print(f"Paint window rectangle: {rect}")
    
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
    
    print(f"Drawing horizontal line from ({start_x}, {start_y}) to ({end_x}, {end_y})")
    draw_line(start_x, start_y, end_x, end_y)
    
    # Wait a moment
    time.sleep(1)
    
    # Draw a vertical line intersecting the horizontal line
    start_x = center_x
    start_y = center_y + drawing_y_offset - 100
    end_x = center_x
    end_y = center_y + drawing_y_offset + 100
    
    print(f"Drawing vertical line from ({start_x}, {start_y}) to ({end_x}, {end_y})")
    draw_line(start_x, start_y, end_x, end_y)
    
    print("Drawing completed")
    
    # Keep the window open - user can close it manually
    input("Press Enter to exit the script...")

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
                    print(f"Error getting process info: {e}")
        return True
    
    results = []
    win32gui.EnumWindows(callback, results)
    
    if results:
        hwnd = results[0]
        return hwnd
    else:
        return None

def draw_line(start_x, start_y, end_x, end_y):
    """Draw a line using simulated mouse events at absolute screen coordinates"""
    # Move mouse to start position
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

if __name__ == "__main__":
    main() 