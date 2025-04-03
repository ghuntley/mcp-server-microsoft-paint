use thiserror::Error;

#[derive(Error, Debug)]
pub enum MspMcpError {
    #[error("General MCP Server Error: {0}")]
    General(String),

    #[error("Paint window not found")]
    WindowNotFound, // 1001

    #[error("Operation timed out: {0}")]
    OperationTimeout(String), // 1002

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String), // 1003

    #[error("Invalid color format: {0}")]
    InvalidColorFormat(String), // 1004

    #[error("Invalid tool specified: {0}")]
    InvalidTool(String), // 1005

    #[error("Invalid shape specified: {0}")]
    InvalidShape(String), // 1006

    #[error("Window activation failed: {0}")]
    WindowActivationFailed(String), // 1007

    #[error("Operation not supported in Windows 11 Paint: {0}")]
    OperationNotSupported(String), // 1008

    #[error("File not found: {0}")]
    FileNotFound(String), // 1009

    #[error("Permission denied accessing file: {0}")]
    FilePermissionDenied(String), // 1010

    #[error("Invalid image format: {0}")]
    InvalidImageFormat(String), // 1011

    #[error("Text input failed: {0}")]
    TextInputFailed(String), // 1012

    #[error("Font selection failed: {0}")]
    FontSelectionFailed(String), // 1013

    #[error("Image transformation failed: {0}")]
    ImageTransformationFailed(String), // 1014

    #[error("Canvas creation failed: {0}")]
    CanvasCreationFailed(String), // 1015

    #[error("Windows API error: {0}")]
    WindowsApiError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Base64 decoding error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),

    // Add more specific errors as needed
}

// Optional: Map errors to MCP error codes
impl MspMcpError {
    pub fn code(&self) -> i32 {
        match self {
            MspMcpError::General(_) => 1000,
            MspMcpError::WindowNotFound => 1001,
            MspMcpError::OperationTimeout(_) => 1002,
            MspMcpError::InvalidParameters(_) => 1003,
            MspMcpError::InvalidColorFormat(_) => 1004,
            MspMcpError::InvalidTool(_) => 1005,
            MspMcpError::InvalidShape(_) => 1006,
            MspMcpError::WindowActivationFailed(_) => 1007,
            MspMcpError::OperationNotSupported(_) => 1008,
            MspMcpError::FileNotFound(_) => 1009,
            MspMcpError::FilePermissionDenied(_) => 1010,
            MspMcpError::InvalidImageFormat(_) => 1011,
            MspMcpError::TextInputFailed(_) => 1012,
            MspMcpError::FontSelectionFailed(_) => 1013,
            MspMcpError::ImageTransformationFailed(_) => 1014,
            MspMcpError::CanvasCreationFailed(_) => 1015,
            // Internal errors might map to a general code or have specific ones if needed
            MspMcpError::WindowsApiError(_) => 1000,
            MspMcpError::IoError(_) => 1000,
            MspMcpError::JsonError(_) => 1000,
            MspMcpError::Base64DecodeError(_) => 1003, // Map to invalid params maybe?
        }
    }
}

pub type Result<T> = std::result::Result<T, MspMcpError>; 