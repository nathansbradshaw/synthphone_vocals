//! Embedded-specific implementations using fixed-size arrays

use crate::{AutotuneConfig, AutotuneError, MusicalSettings};

/// Fixed-size autotune state for embedded systems
pub struct EmbeddedAutotuneState<const FFT_SIZE: usize, const HALF_FFT_SIZE: usize> {
    /// Phase tracking for input analysis
    pub last_input_phases: [f32; HALF_FFT_SIZE],
    /// Phase tracking for output synthesis
    pub last_output_phases: [f32; HALF_FFT_SIZE],
    /// Magnitude data for synthesis
    pub synthesis_magnitudes: [f32; HALF_FFT_SIZE],
    /// Frequency data for synthesis
    pub synthesis_frequencies: [f32; HALF_FFT_SIZE],
    /// Previous pitch shift ratio for smoothing
    pub previous_pitch_shift_ratio: f32,
    /// Configuration
    pub config: AutotuneConfig,
}

impl<const FFT_SIZE: usize, const HALF_FFT_SIZE: usize>
    EmbeddedAutotuneState<FFT_SIZE, HALF_FFT_SIZE>
{
    /// Create a new embedded state
    pub fn new(config: AutotuneConfig) -> Self {
        Self {
            last_input_phases: [0.0; HALF_FFT_SIZE],
            last_output_phases: [0.0; HALF_FFT_SIZE],
            synthesis_magnitudes: [0.0; HALF_FFT_SIZE],
            synthesis_frequencies: [0.0; HALF_FFT_SIZE],
            previous_pitch_shift_ratio: 1.0,
            config,
        }
    }

    /// Reset the state
    pub fn reset(&mut self) {
        self.last_input_phases = [0.0; HALF_FFT_SIZE];
        self.last_output_phases = [0.0; HALF_FFT_SIZE];
        self.synthesis_magnitudes = [0.0; HALF_FFT_SIZE];
        self.synthesis_frequencies = [0.0; HALF_FFT_SIZE];
        self.previous_pitch_shift_ratio = 1.0;
    }
}

/// Truly no_std embedded processing function with fixed-size arrays
pub fn process_autotune_embedded<const FFT_SIZE: usize, const HALF_FFT_SIZE: usize>(
    input_buffer: &[f32; FFT_SIZE],
    output_buffer: &mut [f32; FFT_SIZE],
    state: &mut EmbeddedAutotuneState<FFT_SIZE, HALF_FFT_SIZE>,
    settings: &MusicalSettings,
) -> Result<(), AutotuneError> {
    crate::embedded_core::process_autotune_embedded_core(
        input_buffer,
        output_buffer,
        &mut state.last_input_phases,
        &mut state.last_output_phases,
        &mut state.synthesis_magnitudes,
        &mut state.synthesis_frequencies,
        &mut state.previous_pitch_shift_ratio,
        &state.config,
        settings,
    )
}

/// Convenience type for 1024-point FFT embedded state
pub type EmbeddedAutotuneState1024 = EmbeddedAutotuneState<1024, 512>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_state_creation() {
        let config = AutotuneConfig {
            fft_size: 1024,
            hop_size: 256,
            sample_rate: 48000.0,
            pitch_correction_strength: 0.8,
            transition_speed: 0.1,
            ..Default::default()
        };

        let state = EmbeddedAutotuneState1024::new(config);

        // Verify initial state
        assert_eq!(state.previous_pitch_shift_ratio, 1.0);
        assert_eq!(state.config.fft_size, 1024);
        assert_eq!(state.config.sample_rate, 48000.0);

        // Verify arrays are properly initialized
        assert_eq!(state.last_input_phases.len(), 512);
        assert_eq!(state.last_output_phases.len(), 512);
        assert!(state.last_input_phases.iter().all(|&x| x == 0.0));
        assert!(state.last_output_phases.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_embedded_state_reset() {
        let config = AutotuneConfig::default();
        let mut state = EmbeddedAutotuneState1024::new(config);

        // Modify some values
        state.last_input_phases[0] = 3.14;
        state.previous_pitch_shift_ratio = 2.0;

        // Reset and verify
        state.reset();
        assert_eq!(state.last_input_phases[0], 0.0);
        assert_eq!(state.previous_pitch_shift_ratio, 1.0);
    }

    #[test]
    fn test_embedded_autotune_processing() {
        let config = AutotuneConfig {
            fft_size: 1024,
            hop_size: 256,
            sample_rate: 48000.0,
            pitch_correction_strength: 0.0, // No correction for test
            ..Default::default()
        };

        let mut state = EmbeddedAutotuneState1024::new(config);
        let settings = MusicalSettings::default();

        // Test with silence
        let input = [0.0f32; 1024];
        let mut output = [0.0f32; 1024];

        let result = process_autotune_embedded(&input, &mut output, &mut state, &settings);
        assert!(result.is_ok());

        // With no correction, output should be close to input
        for (i, &sample) in output.iter().enumerate() {
            assert!(sample.abs() < 0.1, "Sample {} too large: {}", i, sample);
        }
    }

    #[test]
    fn test_no_std_compatibility() {
        // This test verifies that we can use the embedded types
        // without any std-only features
        let config = AutotuneConfig::default();
        let state = EmbeddedAutotuneState1024::new(config);

        // These operations should all work in no_std
        assert_eq!(core::mem::size_of_val(&state.last_input_phases), 512 * 4);
        assert_eq!(core::mem::size_of_val(&state.last_output_phases), 512 * 4);
        assert_eq!(core::mem::size_of_val(&state.synthesis_magnitudes), 512 * 4);
        assert_eq!(core::mem::size_of_val(&state.synthesis_frequencies), 512 * 4);

        // Total state size should be predictable
        let total_size = core::mem::size_of::<EmbeddedAutotuneState1024>();
        assert!(total_size >= 8192); // At least 8KB for the arrays
        assert!(total_size < 10240); // Less than 10KB total
    }
}
