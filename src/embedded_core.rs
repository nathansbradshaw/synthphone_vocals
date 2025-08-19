//! Truly no_std core implementation for embedded systems
//! This avoids all heap allocation by using fixed-size arrays

use crate::frequencies::find_nearest_note_in_key;
use crate::hann_window::HANN_WINDOW_1024;
use crate::keys::{get_frequency, get_scale_by_key};
use crate::process_frequencies::{find_fundamental_frequency, wrap_phase};
use crate::{AutotuneConfig, AutotuneError, MusicalSettings};
use libm::{atan2f, cosf, sinf, sqrtf};
use microfft;

const PI: f32 = 3.14159265358979323846264338327950288f32;

/// Embedded autotune processing with fixed-size arrays only
pub fn process_autotune_embedded_core<const FFT_SIZE: usize, const HALF_FFT_SIZE: usize>(
    input_buffer: &[f32; FFT_SIZE],
    output_buffer: &mut [f32; FFT_SIZE],
    last_input_phases: &mut [f32; HALF_FFT_SIZE],
    last_output_phases: &mut [f32; HALF_FFT_SIZE],
    synthesis_magnitudes: &mut [f32; HALF_FFT_SIZE],
    synthesis_frequencies: &mut [f32; HALF_FFT_SIZE],
    previous_pitch_shift_ratio: &mut f32,
    config: &AutotuneConfig,
    settings: &MusicalSettings,
) -> Result<(), AutotuneError> {
    // Only support 1024 FFT for now (can be extended)
    if FFT_SIZE != 1024 || HALF_FFT_SIZE != 512 {
        return Err(AutotuneError::UnsupportedFftSize);
    }

    let fft_size = 1024;
    let spectrum_size = 512;
    let hop_size = config.hop_size;
    let sample_rate = config.sample_rate;
    let pitch_correction_strength = config.pitch_correction_strength;

    // Step 1: Apply window function - use stack array
    let mut windowed_input = [0.0f32; 1024];
    let window = &HANN_WINDOW_1024;

    for i in 0..fft_size {
        windowed_input[i] = input_buffer[i] * window[i];
    }

    // Step 2: Forward FFT using microfft with fixed array
    let fft_result = microfft::real::rfft_1024(&mut windowed_input);

    // Step 3: Analysis phase - extract magnitude and frequency
    let mut analysis_magnitudes = [0.0f32; 512];
    let mut analysis_frequencies = [0.0f32; 512];

    let freq_per_bin = sample_rate / fft_size as f32;
    let phase_scale = 2.0 * PI * hop_size as f32 / fft_size as f32;

    for i in 0..spectrum_size {
        let real = fft_result[i].re;
        let imag = fft_result[i].im;

        // Magnitude
        analysis_magnitudes[i] = sqrtf(real * real + imag * imag);

        // Phase and frequency analysis
        let phase = atan2f(imag, real);
        let phase_diff = phase - last_input_phases[i];
        last_input_phases[i] = phase;

        // Unwrap phase difference
        let mut unwrapped_diff = phase_diff;
        unwrapped_diff = wrap_phase(unwrapped_diff);

        // Calculate precise frequency
        let expected_phase_advance = phase_scale * i as f32;
        let deviation = unwrapped_diff - expected_phase_advance;
        analysis_frequencies[i] = (i as f32 + deviation / phase_scale) * freq_per_bin;
    }

    // Step 4: Find fundamental frequency (convert bin to frequency)
    let fundamental_bin = find_fundamental_frequency(&analysis_magnitudes);
    let fundamental_freq = fundamental_bin as f32 * freq_per_bin;

    // Step 5: Calculate target frequency based on musical settings
    let target_freq = calculate_target_frequency(fundamental_freq, settings);

    // Step 6: Calculate pitch shift ratio with smoothing
    let current_pitch_shift_ratio = if fundamental_freq > 80.0 {
        target_freq / fundamental_freq
    } else {
        1.0 // No correction for very low frequencies
    };

    // Smooth the pitch shift to avoid artifacts
    let transition_speed = config.transition_speed;
    let pitch_shift_ratio = *previous_pitch_shift_ratio * (1.0 - transition_speed)
        + current_pitch_shift_ratio * transition_speed;
    *previous_pitch_shift_ratio = pitch_shift_ratio;

    // Step 7: Apply pitch correction
    synthesis_magnitudes.copy_from_slice(&analysis_magnitudes);

    // Apply pitch shift to frequencies
    for i in 0..spectrum_size {
        let shifted_freq = analysis_frequencies[i] * pitch_shift_ratio;
        synthesis_frequencies[i] = shifted_freq;

        // Apply correction strength
        let original_freq = analysis_frequencies[i];
        synthesis_frequencies[i] = original_freq * (1.0 - pitch_correction_strength)
            + shifted_freq * pitch_correction_strength;
    }

    // Step 8: Synthesis phase - convert back to complex spectrum
    let mut full_spectrum = [microfft::Complex32 { re: 0.0, im: 0.0 }; 1024];

    for i in 0..spectrum_size {
        let magnitude = synthesis_magnitudes[i];
        let target_freq = synthesis_frequencies[i];

        // Calculate new phase
        let target_bin = target_freq / freq_per_bin;
        let phase_advance = phase_scale * target_bin;

        last_output_phases[i] = wrap_phase(last_output_phases[i] + phase_advance);
        let phase = last_output_phases[i];

        // Create complex value
        full_spectrum[i] =
            microfft::Complex32 { re: magnitude * cosf(phase), im: magnitude * sinf(phase) };
    }

    // Ensure Hermitian symmetry for real IFFT
    for i in 1..(spectrum_size - 1) {
        let conjugate_idx = fft_size - i;
        if conjugate_idx < fft_size {
            full_spectrum[conjugate_idx] =
                microfft::Complex32 { re: full_spectrum[i].re, im: -full_spectrum[i].im };
        }
    }

    // Step 9: Inverse FFT
    let ifft_result = microfft::inverse::ifft_1024(&mut full_spectrum);

    // Step 10: Apply window and overlap-add
    for i in 0..fft_size {
        let windowed_sample = ifft_result[i].re * window[i] / fft_size as f32;
        output_buffer[i] = windowed_sample;
    }

    Ok(())
}

/// Calculate target frequency based on musical settings
fn calculate_target_frequency(fundamental_freq: f32, settings: &MusicalSettings) -> f32 {
    if fundamental_freq <= 80.0 {
        return fundamental_freq; // Don't correct very low frequencies
    }

    let scale = get_scale_by_key(settings.key);

    if settings.note == 0 {
        // Auto mode - find nearest note in key
        find_nearest_note_in_key(fundamental_freq, scale)
    } else {
        // Specific note mode - use the main get_frequency function
        get_frequency(settings.key, settings.note, settings.octave, false)
    }
}
