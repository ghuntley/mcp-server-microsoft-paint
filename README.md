# MCP Server for Windows 11 Paint

This project implements a Media Control Protocol (MCP) server for Windows 11 Paint, allowing
remote clients to control Paint through a JSON-RPC interface. It focuses on automating drawing
operations, window management, and image manipulation through Windows API calls.

## Features

- **Connection Management**: Connect/disconnect clients with the Paint application
- **Window Management**: Activate the Paint window and get canvas dimensions
- **Drawing Operations**:
  - Tool selection (pencil, brush, fill, text, eraser, select, shape)
  - Color and thickness configuration
  - Draw pixel, line, shape, and polyline operations
  - Fill type selection
- **Selection Operations**: Select regions, copy, and paste
- **Canvas Management**: Clear canvas, create new canvas
- **Image Transformations**: Rotate, flip, scale, and crop

## Implementation Status

See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for a detailed breakdown of implemented
features and work in progress items.

## Getting Started

### Prerequisites

- Windows 11 with MS Paint installed
- Rust (latest stable version)

### Building

```bash
cargo build --release
```

### Running

```bash
cargo run --release
```

The server accepts JSON-RPC commands over standard input/output.

## Command Examples

Connect to Paint:
```json
{"jsonrpc": "2.0", "id": 1, "method": "connect", "params": {"client_id": "test-client", "client_name": "Test Client"}}
```

Draw a line:
```json
{"jsonrpc": "2.0", "id": 2, "method": "draw_line", "params": {"start_x": 100, "start_y": 100, "end_x": 300, "end_y": 300, "color": "#FF0000", "thickness": 2}}
```

Draw a polyline:
```json
{"jsonrpc": "2.0", "id": 3, "method": "draw_polyline", "params": {"points": [{"x": 10, "y": 20}, {"x": 30, "y": 40}, {"x": 50, "y": 60}], "color": "#0000FF", "thickness": 2, "tool": "pencil"}}
```

Select a region:
```json
{"jsonrpc": "2.0", "id": 4, "method": "select_region", "params": {"start_x": 50, "start_y": 50, "end_x": 200, "end_y": 200}}
```

## Architecture

- `src/core.rs`: Command handlers for each MCP method
- `src/windows.rs`: Windows API integration for interacting with Paint
- `src/protocol.rs`: Protocol definitions for request/response payloads
- `src/error.rs`: Error type definitions and handling
- `src/lib.rs`: Server entry point and lifecycle management

## Testing

```bash
cargo test
```

## License

[MIT License](LICENSE) 