//! Error types for the autotune library

/// Errors that can occur during autotune processing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutotuneError {
    /// Input/output buffer size doesn't match expected size
    BufferSizeMismatch,
    /// FFT size is not supported
    UnsupportedFftSize,
    /// Configuration parameters are invalid
    InvalidConfiguration,
    /// Processing failed due to invalid input
    ProcessingFailed,
}

#[cfg(feature = "std")]
impl std::fmt::Display for AutotuneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutotuneError::BufferSizeMismatch => {
                write!(f, "Input/output buffer size mismatch")
            }
            AutotuneError::UnsupportedFftSize => {
                write!(f, "Unsupported FFT size")
            }
            AutotuneError::InvalidConfiguration => {
                write!(f, "Invalid autotune configuration")
            }
            AutotuneError::ProcessingFailed => {
                write!(f, "Autotune processing failed")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AutotuneError {}
