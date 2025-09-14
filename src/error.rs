//! Error types for the vocal effects library

/// Errors that can occur during vocal effects processing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VocalEffectsError {
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
impl std::fmt::Display for VocalEffectsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VocalEffectsError::BufferSizeMismatch => {
                write!(f, "Input/output buffer size mismatch")
            }
            VocalEffectsError::UnsupportedFftSize => {
                write!(f, "Unsupported FFT size")
            }
            VocalEffectsError::InvalidConfiguration => {
                write!(f, "Invalid vocal effects configuration")
            }
            VocalEffectsError::ProcessingFailed => {
                write!(f, "Vocal effects processing failed")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for VocalEffectsError {}
