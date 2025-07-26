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
        // const_assert!(HALF_FFT_SIZE == FFT_SIZE / 2);

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

/// Embedded-specific processing function with fixed-size arrays
pub fn process_autotune_embedded<const FFT_SIZE: usize, const HALF_FFT_SIZE: usize>(
    input_buffer: &[f32; FFT_SIZE],
    output_buffer: &mut [f32; FFT_SIZE],
    state: &mut EmbeddedAutotuneState<FFT_SIZE, HALF_FFT_SIZE>,
    settings: &MusicalSettings,
) -> Result<(), AutotuneError> {
    // Implementation would mirror core::process_autotune but use fixed arrays
    // This avoids heap allocation for embedded systems

    if FFT_SIZE != 1024 {
        return Err(AutotuneError::UnsupportedFftSize);
    }

    // For now, delegate to the main implementation with conversion
    // In a full implementation, this would be optimized for embedded use
    let mut dynamic_state = crate::AutotuneState::new(state.config);

    // Copy fixed arrays to dynamic arrays
    dynamic_state
        .last_input_phases
        .copy_from_slice(&state.last_input_phases);
    dynamic_state
        .last_output_phases
        .copy_from_slice(&state.last_output_phases);
    dynamic_state
        .synthesis_magnitudes
        .copy_from_slice(&state.synthesis_magnitudes);
    dynamic_state
        .synthesis_frequencies
        .copy_from_slice(&state.synthesis_frequencies);
    dynamic_state.previous_pitch_shift_ratio = state.previous_pitch_shift_ratio;

    // Process
    let result =
        crate::core::process_autotune(input_buffer, output_buffer, &mut dynamic_state, settings);

    // Copy back
    state
        .last_input_phases
        .copy_from_slice(&dynamic_state.last_input_phases);
    state
        .last_output_phases
        .copy_from_slice(&dynamic_state.last_output_phases);
    state
        .synthesis_magnitudes
        .copy_from_slice(&dynamic_state.synthesis_magnitudes);
    state
        .synthesis_frequencies
        .copy_from_slice(&dynamic_state.synthesis_frequencies);
    state.previous_pitch_shift_ratio = dynamic_state.previous_pitch_shift_ratio;

    result
}

/// Convenience type for 1024-point FFT embedded state
pub type EmbeddedAutotuneState1024 = EmbeddedAutotuneState<1024, 512>;
