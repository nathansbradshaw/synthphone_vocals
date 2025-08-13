//! Configuration types for the autotune library

/// Configuration for the autotune processor
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutotuneConfig {
    /// FFT size (must be power of 2, currently only 1024 supported)
    pub fft_size: usize,
    /// Hop size for overlap-add processing
    pub hop_size: usize,
    /// Sample rate in Hz
    pub sample_rate: f32,
    /// Speed of pitch correction transition (0.0 to 1.0)
    pub transition_speed: f32,
    /// Strength of pitch correction (0.0 to 1.0, closer to 1.0 = stronger)
    pub pitch_correction_strength: f32,
    /// Minimum frequency to process (Hz)
    pub min_frequency: f32,
    /// Maximum frequency to process (Hz)
    pub max_frequency: f32,
}

impl Default for AutotuneConfig {
    fn default() -> Self {
        Self {
            fft_size: 1024,
            hop_size: 128,
            sample_rate: 48000.0,
            transition_speed: 0.1,
            pitch_correction_strength: 0.999,
            min_frequency: 50.0,
            max_frequency: 4000.0,
        }
    }
}

impl AutotuneConfig {
    /// Create a new configuration with validation
    pub fn new(
        fft_size: usize,
        hop_size: usize,
        sample_rate: f32,
    ) -> Result<Self, crate::AutotuneError> {
        if !fft_size.is_power_of_two() {
            return Err(crate::AutotuneError::InvalidConfiguration);
        }
        if hop_size >= fft_size {
            return Err(crate::AutotuneError::InvalidConfiguration);
        }
        if sample_rate <= 0.0 {
            return Err(crate::AutotuneError::InvalidConfiguration);
        }

        Ok(Self { fft_size, hop_size, sample_rate, ..Default::default() })
    }

    /// Get the bin width in Hz
    pub fn bin_width(&self) -> f32 {
        self.sample_rate / self.fft_size as f32
    }

    /// Get the spectrum size (FFT size / 2)
    pub fn spectrum_size(&self) -> usize {
        self.fft_size / 2
    }
}
