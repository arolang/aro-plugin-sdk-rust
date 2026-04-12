//! Error types for ARO plugins.

use std::fmt;

/// Numeric error codes understood by the ARO runtime.
///
/// The runtime converts these codes into human-readable error messages that
/// are surfaced as "Code is the error message" diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PluginErrorCode {
    /// Generic / unclassified error.
    Unknown = 0,
    /// A required input field is missing.
    MissingInput = 1,
    /// An input field has an unexpected type.
    InvalidType = 2,
    /// A value is outside the permitted range.
    OutOfRange = 3,
    /// An external I/O operation failed.
    IoError = 4,
    /// A network operation failed.
    NetworkError = 5,
    /// Serialisation or deserialisation failed.
    SerializationError = 6,
    /// The requested resource was not found.
    NotFound = 7,
    /// The caller is not authorised to perform this operation.
    Unauthorized = 8,
    /// The operation exceeded its time budget.
    Timeout = 9,
    /// An internal plugin error (bug in plugin code).
    InternalError = 10,
}

impl fmt::Display for PluginErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Unknown => "Unknown",
            Self::MissingInput => "MissingInput",
            Self::InvalidType => "InvalidType",
            Self::OutOfRange => "OutOfRange",
            Self::IoError => "IoError",
            Self::NetworkError => "NetworkError",
            Self::SerializationError => "SerializationError",
            Self::NotFound => "NotFound",
            Self::Unauthorized => "Unauthorized",
            Self::Timeout => "Timeout",
            Self::InternalError => "InternalError",
        };
        write!(f, "{name}")
    }
}

/// The error type returned by plugin action and qualifier functions.
#[derive(Debug)]
pub struct PluginError {
    /// Machine-readable error code.
    pub code: PluginErrorCode,
    /// Human-readable description.
    pub message: String,
}

impl PluginError {
    /// Create a new `PluginError` with the given code and message.
    pub fn new(code: PluginErrorCode, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }

    /// Convenience constructor for [`PluginErrorCode::MissingInput`].
    pub fn missing(field: impl fmt::Display) -> Self {
        Self::new(
            PluginErrorCode::MissingInput,
            format!("Missing required field: {field}"),
        )
    }

    /// Convenience constructor for [`PluginErrorCode::InvalidType`].
    pub fn invalid_type(field: impl fmt::Display, expected: impl fmt::Display) -> Self {
        Self::new(
            PluginErrorCode::InvalidType,
            format!("Field '{field}' must be {expected}"),
        )
    }

    /// Convenience constructor for [`PluginErrorCode::NotFound`].
    pub fn not_found(resource: impl fmt::Display) -> Self {
        Self::new(PluginErrorCode::NotFound, format!("Not found: {resource}"))
    }

    /// Convenience constructor for [`PluginErrorCode::InternalError`].
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(PluginErrorCode::InternalError, message)
    }

    /// Return the numeric code value.
    pub fn code_value(&self) -> u8 {
        self.code as u8
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for PluginError {}

// Allow any `Display` error to be converted into a generic PluginError.
impl From<String> for PluginError {
    fn from(s: String) -> Self {
        Self::new(PluginErrorCode::Unknown, s)
    }
}

impl From<&str> for PluginError {
    fn from(s: &str) -> Self {
        Self::new(PluginErrorCode::Unknown, s)
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(e: serde_json::Error) -> Self {
        Self::new(PluginErrorCode::SerializationError, e.to_string())
    }
}

/// Convenience `Result` alias used throughout the SDK.
pub type PluginResult<T> = std::result::Result<T, PluginError>;
