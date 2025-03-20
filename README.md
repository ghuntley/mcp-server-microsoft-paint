# Paint MCP - Model Context Protocol for Microsoft Paint

A Rust implementation of a Model Context Protocol (MCP) server that allows programmatic interaction with Microsoft Paint on Windows systems.

## Overview

Paint MCP provides a HTTP API that allows applications to interact with Microsoft Paint through a series of endpoints. It uses undocumented Windows APIs to find, control, and manipulate the Paint interface programmatically.

## Features

- Connect to an existing Paint instance or launch a new one
- Draw basic shapes (lines, rectangles, circles)
- Select different drawing tools
- Set colors
- Save drawings

## System Requirements

- Windows OS (tested on Windows 10/11)
- Microsoft Paint installed
- Rust toolchain

## Installation

```bash
# Clone the repository
git clone https://github.com/ghuntley/mcp-server-microsoft-paint.git
cd mcp-server-microsoft-paint

# Build the project
cargo build --release
```

## Usage

### Start the MCP Server

```bash
cargo run --release
```

The server will start on `http://localhost:3000`.

### API Endpoints

See the [API documentation](specs/mcp_protocol.md) for detailed information about the available endpoints.

### Example: Drawing a Red Line

```bash
curl -X POST http://localhost:3000/draw/line \
  -H "Content-Type: application/json" \
  -d '{
    "start_x": 100,
    "start_y": 100,
    "end_x": 300,
    "end_y": 200,
    "color": "#FF0000",
    "thickness": 2
  }'
```

## Technical Details

This project uses undocumented Windows APIs to interact with Microsoft Paint. It simulates mouse and keyboard events to operate the Paint interface, which means:

1. It's sensitive to Paint's UI layout, which can change between Windows versions
2. It may break with Windows updates
3. Operations are performed in real-time, visible to the user

For more technical details, see the [Windows Integration Specification](specs/windows_integration.md).

## Project Structure

- `src/` - Rust source code
  - `main.rs` - API server implementation
  - `models.rs` - Data structures for API requests/responses
  - `paint_integration.rs` - Windows API integration
- `specs/` - Protocol and technical specifications

## Limitations

As this project uses undocumented APIs and simulates user inputs, it has several limitations:

1. It's not suitable for high-volume or production use
2. It depends on specific UI layouts that may change
3. Drawing operations must be sequential
4. Success of operations depends on Paint being visible and in the foreground

## License

MIT

## Disclaimer

This project uses undocumented and unsupported APIs. It is provided for educational purposes only and should not be used in production environments. Microsoft Paint's UI and behavior may change with Windows updates, potentially breaking this integration. 