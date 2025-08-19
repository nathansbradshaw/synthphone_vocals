//! Real-time embedded autotune implementation with proper overlap-add processing
//!
//! This module provides a true real-time implementation suitable for audio interrupts
//! on embedded systems like the Daisy Seed. It uses circular buffers and sliding
//! window processing to avoid the buffer management issues in the basic embedded module.

use crate::frequencies::find_nearest_note_in_key;
use crate::hann_window::HANN_WINDOW_1024;
use crate::keys::{get_frequency, get_scale_by_key};
use crate::process_frequencies::{find_fundamental_frequency, wrap_phase};
use crate::{AutotuneConfig, AutotuneError, MusicalSettings};
use libm::{atan2f, cosf, sinf, sqrtf};
use microfft;

const PI: f32 = 3.14159265358979323846264338327950288f32;
const FFT_SIZE: usize = 1024;
const SPECTRUM_SIZE: usize = 512;

/// Real-time embedded autotune state with circular buffer management
pub struct RealtimeAutotuneState {
    /// Input circular buffer for overlap-add processing
    pub input_buffer: [f32; FFT_SIZE],
    /// Output circular buffer for overlap-add reconstruction
    pub output_buffer: [f32; FFT_SIZE],
    /// Write position in circular buffers
    pub write_pos: usize,
    /// Read position in output buffer
    pub read_pos: usize,
    /// Samples accumulated since last processing
    pub samples_since_process: usize,
    /// Phase tracking for input analysis
    pub last_input_phases: [f32; SPECTRUM_SIZE],
    /// Phase tracking for output synthesis
    pub last_output_phases: [f32; SPECTRUM_SIZE],
    /// Magnitude data for synthesis
    pub synthesis_magnitudes: [f32; SPECTRUM_SIZE],
    /// Frequency data for synthesis
    pub synthesis_frequencies: [f32; SPECTRUM_SIZE],
    /// Previous pitch shift ratio for smoothing
    pub previous_pitch_shift_ratio: f32,
    /// Processing configuration
    pub config: AutotuneConfig,
    /// Flag indicating if we have valid output ready
    pub output_ready: bool,
}

impl RealtimeAutotuneState {
    /// Create a new real-time autotune state
    pub fn new(config: AutotuneConfig) -> Self {
        Self {
            input_buffer: [0.0; FFT_SIZE],
            output_buffer: [0.0; FFT_SIZE],
            write_pos: 0,
            read_pos: 0,
            samples_since_process: 0,
            last_input_phases: [0.0; SPECTRUM_SIZE],
            last_output_phases: [0.0; SPECTRUM_SIZE],
            synthesis_magnitudes: [0.0; SPECTRUM_SIZE],
            synthesis_frequencies: [0.0; SPECTRUM_SIZE],
            previous_pitch_shift_ratio: 1.0,
            config,
            output_ready: false,
        }
    }

    /// Reset the state
    pub fn reset(&mut self) {
        self.input_buffer = [0.0; FFT_SIZE];
        self.output_buffer = [0.0; FFT_SIZE];
        self.write_pos = 0;
        self.read_pos = 0;
        self.samples_since_process = 0;
        self.last_input_phases = [0.0; SPECTRUM_SIZE];
        self.last_output_phases = [0.0; SPECTRUM_SIZE];
        self.synthesis_magnitudes = [0.0; SPECTRUM_SIZE];
        self.synthesis_frequencies = [0.0; SPECTRUM_SIZE];
        self.previous_pitch_shift_ratio = 1.0;
        self.output_ready = false;
    }
}

/// Process a single sample through the real-time autotune
///
/// This function should be called from the audio interrupt for each input sample.
/// It manages the circular buffers and triggers FFT processing when enough samples
/// have been accumulated.
pub fn process_sample_realtime(
    input_sample: f32,
    state: &mut RealtimeAutotuneState,
    settings: &MusicalSettings,
) -> Result<f32, AutotuneError> {
    // Add input sample to circular buffer
    state.input_buffer[state.write_pos] = input_sample;

    // Advance write position
    state.write_pos = (state.write_pos + 1) % FFT_SIZE;
    state.samples_since_process += 1;

    // Process when we have accumulated hop_size samples
    if state.samples_since_process >= state.config.hop_size {
        // Trigger FFT processing
        process_fft_frame(state, settings)?;
        state.samples_since_process = 0;
        state.output_ready = true;
    }

    // Get output sample from circular buffer
    let output_sample = if state.output_ready {
        let sample = state.output_buffer[state.read_pos];
        state.read_pos = (state.read_pos + 1) % FFT_SIZE;
        sample
    } else {
        // During initial buffering, pass through input with reduced gain to avoid clicks
        input_sample * 0.1
    };

    Ok(output_sample)
}

/// Process multiple samples through the real-time autotune
///
/// This is a convenience function for processing audio blocks
pub fn process_block_realtime(
    input_block: &[f32],
    output_block: &mut [f32],
    state: &mut RealtimeAutotuneState,
    settings: &MusicalSettings,
) -> Result<(), AutotuneError> {
    if input_block.len() != output_block.len() {
        return Err(AutotuneError::BufferSizeMismatch);
    }

    for (input_sample, output_sample) in input_block.iter().zip(output_block.iter_mut()) {
        *output_sample = process_sample_realtime(*input_sample, state, settings)?;
    }

    Ok(())
}

/// Internal function to process a complete FFT frame
fn process_fft_frame(
    state: &mut RealtimeAutotuneState,
    settings: &MusicalSettings,
) -> Result<(), AutotuneError> {
    let sample_rate = state.config.sample_rate;
    let hop_size = state.config.hop_size;
    let pitch_correction_strength = state.config.pitch_correction_strength;

    // Create windowed input from circular buffer
    let mut windowed_input = [0.0f32; FFT_SIZE];
    let window = &HANN_WINDOW_1024;

    // Extract samples from circular buffer starting from current write position
    for i in 0..FFT_SIZE {
        let buffer_idx = (state.write_pos + FFT_SIZE - FFT_SIZE + i) % FFT_SIZE;
        windowed_input[i] = state.input_buffer[buffer_idx] * window[i];
    }

    // Forward FFT
    let fft_result = microfft::real::rfft_1024(&mut windowed_input);

    // Analysis phase
    let mut analysis_magnitudes = [0.0f32; SPECTRUM_SIZE];
    let mut analysis_frequencies = [0.0f32; SPECTRUM_SIZE];

    let freq_per_bin = sample_rate / FFT_SIZE as f32;
    let phase_scale = 2.0 * PI * hop_size as f32 / FFT_SIZE as f32;

    for i in 0..SPECTRUM_SIZE {
        let real = fft_result[i].re;
        let imag = fft_result[i].im;

        // Magnitude
        analysis_magnitudes[i] = sqrtf(real * real + imag * imag);

        // Phase analysis with improved unwrapping
        let phase = atan2f(imag, real);
        let phase_diff = wrap_phase(phase - state.last_input_phases[i]);
        state.last_input_phases[i] = phase;

        // More accurate frequency estimation
        let expected_phase_advance = phase_scale * i as f32;
        let deviation = phase_diff - expected_phase_advance;
        analysis_frequencies[i] = (i as f32 + deviation / phase_scale) * freq_per_bin;
    }

    // Find fundamental frequency with improved detection
    let fundamental_bin = find_fundamental_frequency(&analysis_magnitudes);
    let fundamental_freq = fundamental_bin as f32 * freq_per_bin;

    // Calculate target frequency
    let target_freq = calculate_target_frequency(fundamental_freq, settings);

    // Calculate pitch shift ratio with better smoothing
    let current_pitch_shift_ratio = if fundamental_freq > 80.0 && fundamental_freq < 2000.0 {
        target_freq / fundamental_freq
    } else {
        1.0 // No correction for very low or very high frequencies
    };

    // Exponential smoothing for pitch shift
    let transition_speed = state.config.transition_speed;
    let alpha = 1.0 - libm::expf(-transition_speed * 10.0); // Better smoothing curve
    state.previous_pitch_shift_ratio =
        state.previous_pitch_shift_ratio * (1.0 - alpha) + current_pitch_shift_ratio * alpha;

    // Apply pitch correction to synthesis data
    state.synthesis_magnitudes.copy_from_slice(&analysis_magnitudes);

    for i in 0..SPECTRUM_SIZE {
        let original_freq = analysis_frequencies[i];
        let shifted_freq = original_freq * state.previous_pitch_shift_ratio;

        // Apply correction strength with better curve
        let correction_factor = pitch_correction_strength;
        state.synthesis_frequencies[i] =
            original_freq * (1.0 - correction_factor) + shifted_freq * correction_factor;
    }

    // Synthesis phase with improved reconstruction
    synthesize_output(state)?;

    Ok(())
}

/// Synthesize output from frequency domain data
fn synthesize_output(state: &mut RealtimeAutotuneState) -> Result<(), AutotuneError> {
    let hop_size = state.config.hop_size;
    let sample_rate = state.config.sample_rate;

    // Create complex spectrum for IFFT
    let mut full_spectrum = [microfft::Complex32 { re: 0.0, im: 0.0 }; FFT_SIZE];

    let freq_per_bin = sample_rate / FFT_SIZE as f32;
    let phase_scale = 2.0 * PI * hop_size as f32 / FFT_SIZE as f32;

    // Fill positive frequencies
    for i in 0..SPECTRUM_SIZE {
        let magnitude = state.synthesis_magnitudes[i];
        let target_freq = state.synthesis_frequencies[i];

        if magnitude > 1e-8 {
            // Only process significant magnitudes
            // Calculate phase advance
            let target_bin = target_freq / freq_per_bin;
            let phase_advance = phase_scale * target_bin;

            state.last_output_phases[i] = wrap_phase(state.last_output_phases[i] + phase_advance);
            let phase = state.last_output_phases[i];

            // Create complex value
            full_spectrum[i] =
                microfft::Complex32 { re: magnitude * cosf(phase), im: magnitude * sinf(phase) };
        }
    }

    // Ensure Hermitian symmetry for real IFFT
    for i in 1..(SPECTRUM_SIZE - 1) {
        let conjugate_idx = FFT_SIZE - i;
        full_spectrum[conjugate_idx] =
            microfft::Complex32 { re: full_spectrum[i].re, im: -full_spectrum[i].im };
    }

    // DC and Nyquist bins should be real
    full_spectrum[0].im = 0.0;
    full_spectrum[SPECTRUM_SIZE].im = 0.0;

    // Inverse FFT
    let ifft_result = microfft::inverse::ifft_1024(&mut full_spectrum);

    // Apply window and add to output buffer with proper overlap-add
    let window = &HANN_WINDOW_1024;
    let normalization = 0.5 / (FFT_SIZE as f32); // Improved normalization

    for i in 0..FFT_SIZE {
        let windowed_sample = ifft_result[i].re * window[i] * normalization;
        let output_idx = (state.write_pos + i) % FFT_SIZE;

        // Overlap-add: accumulate with existing output
        state.output_buffer[output_idx] += windowed_sample;
    }

    // Clear the portion of output buffer we just read to prevent accumulation
    for i in 0..hop_size {
        let clear_idx = (state.read_pos + i) % FFT_SIZE;
        state.output_buffer[clear_idx] *= 0.8; // Gradual decay instead of hard clear
    }

    Ok(())
}

/// Calculate target frequency based on musical settings
fn calculate_target_frequency(fundamental_freq: f32, settings: &MusicalSettings) -> f32 {
    if fundamental_freq <= 80.0 || fundamental_freq >= 2000.0 {
        return fundamental_freq; // Don't correct very low or very high frequencies
    }

    let scale = get_scale_by_key(settings.key);

    if settings.note == 0 {
        // Auto mode - find nearest note in key
        find_nearest_note_in_key(fundamental_freq, scale)
    } else {
        // Specific note mode
        get_frequency(settings.key, settings.note, settings.octave, false)
    }
}

/// Configuration presets for common scenarios
impl AutotuneConfig {
    /// Create a configuration optimized for real-time processing
    /// - Balanced latency and CPU usage
    /// - Good for most embedded applications
    pub fn realtime() -> Self {
        Self {
            fft_size: 1024,
            hop_size: 256,
            sample_rate: 48000.0,
            transition_speed: 0.2,
            pitch_correction_strength: 0.8,
            min_frequency: 80.0,
            max_frequency: 2000.0,
        }
    }

    /// Create a configuration for low-latency processing
    /// - Higher CPU usage but lower latency
    /// - Good for interactive applications
    pub fn low_latency() -> Self {
        Self {
            fft_size: 1024,
            hop_size: 128,
            sample_rate: 48000.0,
            transition_speed: 0.3,
            pitch_correction_strength: 0.7,
            min_frequency: 80.0,
            max_frequency: 2000.0,
        }
    }

    /// Create a configuration for high-quality processing
    /// - Lower CPU usage, higher latency
    /// - Good for non-interactive or battery-powered applications
    pub fn high_quality() -> Self {
        Self {
            fft_size: 1024,
            hop_size: 512,
            sample_rate: 48000.0,
            transition_speed: 0.15,
            pitch_correction_strength: 0.85,
            min_frequency: 60.0,
            max_frequency: 4000.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realtime_state_creation() {
        let config = AutotuneConfig::realtime();
        let state = RealtimeAutotuneState::new(config);

        assert_eq!(state.write_pos, 0);
        assert_eq!(state.read_pos, 0);
        assert!(!state.output_ready);
        assert_eq!(state.previous_pitch_shift_ratio, 1.0);
    }

    #[test]
    fn test_sample_processing() {
        let config = AutotuneConfig::realtime();
        let mut state = RealtimeAutotuneState::new(config);
        let settings = MusicalSettings::default();

        // Process some samples
        for i in 0..100 {
            let input = (i as f32 * 0.01).sin();
            let result = process_sample_realtime(input, &mut state, &settings);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_block_processing() {
        let config = AutotuneConfig::realtime();
        let mut state = RealtimeAutotuneState::new(config);
        let settings = MusicalSettings::default();

        let input_block = [0.1, 0.2, 0.3, 0.4];
        let mut output_block = [0.0; 4];

        let result = process_block_realtime(&input_block, &mut output_block, &mut state, &settings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_state_reset() {
        let config = AutotuneConfig::realtime();
        let mut state = RealtimeAutotuneState::new(config);

        // Modify state
        state.write_pos = 100;
        state.output_ready = true;
        state.previous_pitch_shift_ratio = 2.0;

        // Reset and verify
        state.reset();
        assert_eq!(state.write_pos, 0);
        assert!(!state.output_ready);
        assert_eq!(state.previous_pitch_shift_ratio, 1.0);
    }
}
