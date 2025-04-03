# MCP Server for Microsoft Paint

A JSON-RPC 2.0 compatible server for controlling Microsoft Paint through the Microsoft Commandline Protocol (MCP).

## Features

- Launch and connect to Microsoft Paint
- Draw lines, shapes, and pixels
- Set colors and tool properties
- Control the Paint window

## Requirements

- Windows 10/11 with Microsoft Paint installed
- Rust (for building the server)
- Python (for the test client examples)

## Building and Running

To build the server:

```
cargo build --release
```

To run the server:

```
cargo run --release
```

The server accepts JSON-RPC 2.0 requests via stdin and responds via stdout.

## JSON-RPC Methods

### `initialize`

Finds or launches Microsoft Paint.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {}
}
```

### `connect`

Connects to an already running Paint window.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "connect",
  "params": {
    "client_id": "your-client-id",
    "client_name": "Your Client Name"
  }
}
```

### `draw_line`

Draws a line from one point to another.

**Request:**
```json
{
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
```

### Other Methods

- `activate_window` - Brings the Paint window to the foreground
- `get_canvas_dimensions` - Returns the current canvas size
- `draw_pixel` - Draws a single pixel
- `draw_shape` - Draws a shape (rectangle, ellipse, etc.)
- `select_tool` - Selects a drawing tool
- `set_color` - Sets the current color
- And more...

## Example Test Client

A simple test client is provided in `final_test.py` to demonstrate how to use the server:

```bash
python final_test.py
```

## Troubleshooting

If you encounter issues with the server connecting to Paint:

1. Make sure Microsoft Paint is installed and accessible
2. Try manually launching Paint before starting the server
3. Check the server logs for detailed error messages

## License

This project is available under the MIT License. 