#!/usr/bin/env python
import subprocess
import time
import win32gui
import win32con
import win32api
import ctypes
import sys

def main():
    # Launch MS Paint
    print("Launching MS Paint...")
    subprocess.Popen(["mspaint.exe"])
    time.sleep(3)  # Wait for Paint to start

    # Find MS Paint window
    paint_hwnd = win32gui.FindWindow(None, "Untitled - Paint")
    if not paint_hwnd:
        print("Could not find MS Paint window.")
        return

    print(f"Found MS Paint window: {paint_hwnd}")

    # Activate MS Paint window
    win32gui.SetForegroundWindow(paint_hwnd)
    time.sleep(1)

    # Get window dimensions
    left, top, right, bottom = win32gui.GetClientRect(paint_hwnd)
    
    # Adjust for window borders and toolbars (approximate)
    toolbar_height = 150  # Adjust as needed for your Paint version
    drawing_area_top = top + toolbar_height
    
    # Calculate coordinates for a horizontal line
    start_x = 100
    start_y = 200
    end_x = 400
    end_y = 200
    
    # Convert client coordinates to screen coordinates
    start_screen_x, start_screen_y = win32gui.ClientToScreen(paint_hwnd, (start_x, start_y + toolbar_height))
    end_screen_x, end_screen_y = win32gui.ClientToScreen(paint_hwnd, (end_x, end_y + toolbar_height))
    
    print(f"Drawing line from ({start_screen_x}, {start_screen_y}) to ({end_screen_x}, {end_screen_y})")
    
    # Move mouse to starting position
    win32api.SetCursorPos((start_screen_x, start_screen_y))
    time.sleep(0.5)
    
    # Press left mouse button
    win32api.mouse_event(win32con.MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0)
    time.sleep(0.2)
    
    # Move to ending position (draw the line)
    # Do this in small steps for smoother line
    steps = 10
    for i in range(1, steps + 1):
        x = start_screen_x + (end_screen_x - start_screen_x) * i // steps
        y = start_screen_y + (end_screen_y - start_screen_y) * i // steps
        win32api.SetCursorPos((x, y))
        time.sleep(0.02)
    
    # Release left mouse button
    win32api.mouse_event(win32con.MOUSEEVENTF_LEFTUP, 0, 0, 0, 0)
    
    print("Line drawing completed. Paint window will remain open.")
    print("Press Enter to close this script (Paint will stay open)...")
    input()

if __name__ == "__main__":
    main() 