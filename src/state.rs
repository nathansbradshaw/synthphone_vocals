//! State management for autotune processing

use crate::config::AutotuneConfig;

// Only include Vec-based state when not in embedded-only mode
#[cfg(any(feature = "std", not(feature = "embedded")))]
use alloc::vec;
#[cfg(any(feature = "std", not(feature = "embedded")))]
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Musical settings for autotune processing
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MusicalSettings {
    /// Musical key (0-23, see keys module for mapping)
    pub key: i32,
    /// Specific note (0 = auto mode, 1-9 = specific note in scale)
    pub note: i32,
    /// Octave setting
    pub octave: i32,
    /// Formant shift mode (0 = none, 1 = lower, 2 = higher)
    pub formant: i32,
}

impl Default for MusicalSettings {
    fn default() -> Self {
        Self {
            key: 0,  // C Major
            note: 0, // Auto mode
            octave: 2,
            formant: 0, // No formant shift
        }
    }
}

// Only compile the Vec-based AutotuneState when not in embedded-only mode
#[cfg(any(feature = "std", not(feature = "embedded")))]
/// State that needs to be maintained between processing calls
pub struct AutotuneState {
    /// Phase tracking for input analysis
    pub last_input_phases: Vec<f32>,
    /// Phase tracking for output synthesis
    pub last_output_phases: Vec<f32>,
    /// Magnitude data for synthesis
    pub synthesis_magnitudes: Vec<f32>,
    /// Frequency data for synthesis
    pub synthesis_frequencies: Vec<f32>,
    /// Previous pitch shift ratio for smoothing
    pub previous_pitch_shift_ratio: f32,
    /// Configuration
    config: AutotuneConfig,
}

#[cfg(any(feature = "std", not(feature = "embedded")))]
impl AutotuneState {
    /// Create a new state with the given configuration
    pub fn new(config: AutotuneConfig) -> Self {
        let spectrum_size = config.spectrum_size();
        Self {
            last_input_phases: vec![0.0; spectrum_size],
            last_output_phases: vec![0.0; spectrum_size],
            synthesis_magnitudes: vec![0.0; spectrum_size],
            synthesis_frequencies: vec![0.0; spectrum_size],
            previous_pitch_shift_ratio: 1.0,
            config,
        }
    }

    /// Create a new state with validation for microfft compatibility
    pub fn new_validated(config: AutotuneConfig) -> Result<Self, crate::error::AutotuneError> {
        // Validate that the configuration is compatible with microfft
        if config.fft_size != 1024 {
            return Err(crate::error::AutotuneError::UnsupportedFftSize);
        }

        Ok(Self::new(config))
    }

    /// Get the configuration
    pub fn config(&self) -> &AutotuneConfig {
        &self.config
    }

    /// Get the FFT size
    pub fn fft_size(&self) -> usize {
        self.config.fft_size
    }

    /// Get the hop size
    pub fn hop_size(&self) -> usize {
        self.config.hop_size
    }

    /// Get the spectrum size (FFT size / 2)
    pub fn spectrum_size(&self) -> usize {
        self.config.spectrum_size()
    }

    /// Reset the state (clear all history)
    pub fn reset(&mut self) {
        self.last_input_phases.fill(0.0);
        self.last_output_phases.fill(0.0);
        self.synthesis_magnitudes.fill(0.0);
        self.synthesis_frequencies.fill(0.0);
        self.previous_pitch_shift_ratio = 1.0;
    }

    /// Resize the state vectors if configuration changes
    pub fn resize_for_config(
        &mut self,
        new_config: AutotuneConfig,
    ) -> Result<(), crate::error::AutotuneError> {
        // Validate new configuration
        if new_config.fft_size != 1024 {
            return Err(crate::error::AutotuneError::UnsupportedFftSize);
        }

        let new_spectrum_size = new_config.spectrum_size();

        // Only resize if the spectrum size actually changed
        if new_spectrum_size != self.spectrum_size() {
            self.last_input_phases.resize(new_spectrum_size, 0.0);
            self.last_output_phases.resize(new_spectrum_size, 0.0);
            self.synthesis_magnitudes.resize(new_spectrum_size, 0.0);
            self.synthesis_frequencies.resize(new_spectrum_size, 0.0);
        }

        self.config = new_config;
        self.reset(); // Clear any stale state

        Ok(())
    }

    /// Get mutable access to input phases for FFT processing
    pub fn input_phases_mut(&mut self) -> &mut [f32] {
        &mut self.last_input_phases
    }

    /// Get mutable access to output phases for synthesis
    pub fn output_phases_mut(&mut self) -> &mut [f32] {
        &mut self.last_output_phases
    }

    /// Get mutable access to synthesis magnitudes
    pub fn synthesis_magnitudes_mut(&mut self) -> &mut [f32] {
        &mut self.synthesis_magnitudes
    }

    /// Get mutable access to synthesis frequencies
    pub fn synthesis_frequencies_mut(&mut self) -> &mut [f32] {
        &mut self.synthesis_frequencies
    }

    /// Get read-only access to input phases
    pub fn input_phases(&self) -> &[f32] {
        &self.last_input_phases
    }

    /// Get read-only access to output phases
    pub fn output_phases(&self) -> &[f32] {
        &self.last_output_phases
    }

    /// Get read-only access to synthesis magnitudes
    pub fn synthesis_magnitudes(&self) -> &[f32] {
        &self.synthesis_magnitudes
    }

    /// Get read-only access to synthesis frequencies
    pub fn synthesis_frequencies(&self) -> &[f32] {
        &self.synthesis_frequencies
    }

    /// Validate that the state is compatible with the current configuration
    pub fn validate(&self) -> Result<(), crate::error::AutotuneError> {
        let expected_spectrum_size = self.config.spectrum_size();

        if self.last_input_phases.len() != expected_spectrum_size
            || self.last_output_phases.len() != expected_spectrum_size
            || self.synthesis_magnitudes.len() != expected_spectrum_size
            || self.synthesis_frequencies.len() != expected_spectrum_size
        {
            return Err(crate::error::AutotuneError::BufferSizeMismatch);
        }

        if self.config.fft_size != 1024 {
            return Err(crate::error::AutotuneError::UnsupportedFftSize);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AutotuneConfig;

    #[cfg(any(feature = "std", not(feature = "embedded")))]
    #[test]
    fn test_state_creation() {
        let config = AutotuneConfig::default();
        let state = AutotuneState::new(config);

        assert_eq!(state.fft_size(), 1024);
        assert_eq!(state.spectrum_size(), 512);
        assert_eq!(state.previous_pitch_shift_ratio, 1.0);
    }

    #[cfg(any(feature = "std", not(feature = "embedded")))]
    #[test]
    fn test_state_validation() {
        let config = AutotuneConfig::default();
        let state = AutotuneState::new_validated(config).unwrap();

        assert!(state.validate().is_ok());
    }

    #[cfg(any(feature = "std", not(feature = "embedded")))]
    #[test]
    fn test_state_reset() {
        let config = AutotuneConfig::default();
        let mut state = AutotuneState::new(config);

        // Modify some state
        state.last_input_phases[0] = 1.0;
        state.previous_pitch_shift_ratio = 2.0;

        // Reset and verify
        state.reset();
        assert_eq!(state.last_input_phases[0], 0.0);
        assert_eq!(state.previous_pitch_shift_ratio, 1.0);
    }

    #[cfg(any(feature = "std", not(feature = "embedded")))]
    #[test]
    fn test_slice_access() {
        let config = AutotuneConfig::default();
        let mut state = AutotuneState::new(config);

        // Test mutable access
        let phases = state.input_phases_mut();
        phases[0] = 3.14;

        // Test read access
        assert_eq!(state.input_phases()[0], 3.14);
    }

    #[test]
    fn test_musical_settings_default() {
        let settings = MusicalSettings::default();
        assert_eq!(settings.key, 0);
        assert_eq!(settings.note, 0);
        assert_eq!(settings.octave, 2);
        assert_eq!(settings.formant, 0);
    }
}
