//! Core autotune processing engine
//!
//! This module contains the main autotune algorithm implementation using a phase vocoder
//! approach for real-time pitch correction.

use alloc::vec;
use libm::{atan2f, cosf, expf, floorf, logf, sinf, sqrtf};
use microfft;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::config::AutotuneConfig;
use crate::error::AutotuneError;
use crate::frequencies::{find_nearest_note_frequency, find_nearest_note_in_key};
use crate::hann_window::HANN_WINDOW;
use crate::keys::{get_frequency, get_scale_by_key};
use crate::process_frequencies::{collect_harmonics, find_fundamental_frequency, wrap_phase};
use crate::state::{AutotuneState, MusicalSettings};

const PI: f32 = 3.14159265358979323846264338327950288f32;

/// Helper function to safely convert a slice to a fixed-size array
fn slice_to_array_1024_f32(slice: &mut [f32]) -> Result<&mut [f32; 1024], AutotuneError> {
    slice
        .try_into()
        .map_err(|_| AutotuneError::BufferSizeMismatch)
}

/// Helper function to safely convert a slice to a fixed-size complex array
fn slice_to_array_1024_complex(
    slice: &mut [microfft::Complex32],
) -> Result<&mut [microfft::Complex32; 1024], AutotuneError> {
    slice
        .try_into()
        .map_err(|_| AutotuneError::BufferSizeMismatch)
}

/// Helper function for FFT processing with dynamic allocation
fn perform_fft_1024(input: &[f32]) -> Result<Vec<microfft::Complex32>, AutotuneError> {
    let mut windowed_input = input.to_vec();
    let windowed_array = slice_to_array_1024_f32(&mut windowed_input)?;
    Ok(microfft::real::rfft_1024(windowed_array).to_vec())
}

/// Helper function for inverse FFT processing
fn perform_ifft_1024(
    spectrum: &mut [microfft::Complex32],
) -> Result<Vec<microfft::Complex32>, AutotuneError> {
    let spectrum_array = slice_to_array_1024_complex(spectrum)?;
    Ok(microfft::inverse::ifft_1024(spectrum_array).to_vec())
}

/// Main autotune processing function
///
/// This function implements a phase vocoder-based pitch correction algorithm:
/// 1. Windowing and FFT analysis of input signal
/// 2. Phase unwrapping for precise frequency estimation
/// 3. Pitch detection and target frequency calculation
/// 4. Optional formant envelope extraction
/// 5. Pitch shifting via bin reallocation
/// 6. Phase synthesis and inverse FFT
///
/// # Arguments
/// * `input_buffer` - Time domain input samples (length = fft_size)
/// * `output_buffer` - Time domain output samples (length = fft_size)
/// * `state` - Persistent state between processing calls
/// * `settings` - Musical settings (key, note, etc.)
///
/// # Returns
/// * `Ok(())` on successful processing
/// * `Err(AutotuneError)` on processing error
pub fn process_autotune(
    input_buffer: &[f32],
    output_buffer: &mut [f32],
    state: &mut AutotuneState,
    settings: &MusicalSettings,
) -> Result<(), AutotuneError> {
    let config = *state.config(); // Copy the config to avoid borrowing issues
    let fft_size = config.fft_size;
    let spectrum_size = config.spectrum_size();
    let hop_size = config.hop_size;

    // Validate buffer sizes
    if input_buffer.len() != fft_size || output_buffer.len() != fft_size {
        return Err(AutotuneError::BufferSizeMismatch);
    }

    // Currently only 1024-point FFT is supported by microfft
    if fft_size != 1024 {
        return Err(AutotuneError::UnsupportedFftSize);
    }

    // Step 1: Apply window function and prepare for FFT
    let mut windowed_input = vec![0.0f32; fft_size];
    let window = &HANN_WINDOW;

    for i in 0..fft_size {
        windowed_input[i] = input_buffer[i] * window[i];
    }

    // Step 2: Forward FFT for frequency domain analysis
    let fft_result = perform_fft_1024(&windowed_input)?;

    // Step 3: Analysis phase - extract magnitude and precise frequency for each bin
    let mut analysis_magnitudes = vec![0.0f32; spectrum_size];
    let mut analysis_frequencies = vec![0.0f32; spectrum_size];

    for i in 0..spectrum_size {
        // Calculate magnitude and phase from complex FFT result
        let amplitude =
            sqrtf(fft_result[i].re * fft_result[i].re + fft_result[i].im * fft_result[i].im);
        let phase = atan2f(fft_result[i].im, fft_result[i].re);

        // Phase unwrapping for precise frequency estimation
        // This accounts for the phase change between hops to get exact frequency
        let phase_diff = phase - state.last_input_phases[i];
        let bin_centre_frequency = 2.0 * PI * i as f32 / fft_size as f32;
        let expected_phase_advance = bin_centre_frequency * hop_size as f32;
        let wrapped_phase_diff = wrap_phase(phase_diff - expected_phase_advance);

        // Convert phase deviation back to frequency deviation
        let bin_deviation = wrapped_phase_diff * fft_size as f32 / hop_size as f32 / (2.0 * PI);

        // Store precise frequency and magnitude
        analysis_frequencies[i] = i as f32 + bin_deviation;
        analysis_magnitudes[i] = amplitude;

        // Update phase history for next hop
        state.last_input_phases[i] = phase;
    }

    // Step 4: Find fundamental frequency and determine pitch correction target
    let fundamental_index = find_fundamental_frequency(&analysis_magnitudes);
    let exact_frequency = analysis_frequencies[fundamental_index] * config.bin_width();

    // Reset synthesis arrays for new processing cycle
    state.synthesis_magnitudes.fill(0.0);
    state.synthesis_frequencies.fill(0.0);

    // Skip processing if frequency is outside our range of interest
    if exact_frequency < config.min_frequency || exact_frequency > config.max_frequency {
        // Pass through without modification
        output_buffer.copy_from_slice(input_buffer);
        return Ok(());
    }

    // Step 5: Determine target frequency based on musical settings
    let target_frequency = calculate_target_frequency(exact_frequency, settings)?;
    let current_pitch_shift_ratio = target_frequency / exact_frequency;

    // Smooth the pitch shift ratio to avoid artifacts
    let pitch_shift_ratio = config.pitch_correction_strength * current_pitch_shift_ratio
        + (1.0 - config.pitch_correction_strength) * state.previous_pitch_shift_ratio;

    // Step 6: Optional formant envelope extraction
    #[cfg(feature = "formant-shifting")]
    let formant_envelope = if settings.formant != 0 {
        Some(extract_formant_envelope(&analysis_magnitudes, fft_size)?)
    } else {
        None
    };

    // Step 7: Pitch shifting by redistributing spectral content
    let octave_factor = calculate_octave_factor(settings.octave);

    for i in 0..spectrum_size {
        // Calculate new bin position after pitch shift
        let shifted_bin_float = i as f32 * pitch_shift_ratio * octave_factor;
        let new_bin = floorf(shifted_bin_float + 0.5) as usize;

        if new_bin < spectrum_size {
            // Apply formant correction if enabled
            #[cfg(feature = "formant-shifting")]
            let corrected_magnitude = if let Some(ref envelope) = formant_envelope {
                apply_formant_correction(
                    analysis_magnitudes[i],
                    envelope,
                    i,
                    new_bin,
                    settings.formant,
                    spectrum_size,
                )
            } else {
                analysis_magnitudes[i]
            };

            #[cfg(not(feature = "formant-shifting"))]
            let corrected_magnitude = analysis_magnitudes[i];

            // Accumulate magnitude (multiple source bins may map to same target)
            state.synthesis_magnitudes[new_bin] += corrected_magnitude;
            state.synthesis_frequencies[new_bin] =
                analysis_frequencies[i] * pitch_shift_ratio * octave_factor;
        }
    }

    // Update pitch shift history
    state.previous_pitch_shift_ratio = pitch_shift_ratio;

    // Step 8: Synthesis phase - convert back to complex spectrum
    let mut full_spectrum = vec![microfft::Complex32 { re: 0.0, im: 0.0 }; fft_size];

    for i in 0..spectrum_size {
        let amplitude = state.synthesis_magnitudes[i];

        // Calculate precise phase for synthesis
        let bin_deviation = state.synthesis_frequencies[i] - i as f32;
        let mut phase_advance = bin_deviation * 2.0 * PI * hop_size as f32 / fft_size as f32;
        let expected_phase_advance = 2.0 * PI * i as f32 / fft_size as f32 * hop_size as f32;
        phase_advance += expected_phase_advance;

        // Update output phase tracking
        let output_phase = wrap_phase(state.last_output_phases[i] + phase_advance);

        // Convert back to complex representation
        let complex_sample = microfft::Complex32 {
            re: amplitude * cosf(output_phase),
            im: amplitude * sinf(output_phase),
        };

        // Set both positive and negative frequency components (Hermitian symmetry)
        full_spectrum[i] = complex_sample;
        if i > 0 && i < spectrum_size {
            full_spectrum[fft_size - i] = complex_sample.conj();
        }

        // Update phase history
        state.last_output_phases[i] = output_phase;
    }

    // Step 9: Inverse FFT to get back to time domain
    let time_domain_result = perform_ifft_1024(&mut full_spectrum)?;

    // Step 10: Apply output window and write result
    for i in 0..fft_size {
        output_buffer[i] = time_domain_result[i].re * window[i];
    }

    Ok(())
}

/// Calculate the target frequency based on musical settings
fn calculate_target_frequency(
    exact_frequency: f32,
    settings: &MusicalSettings,
) -> Result<f32, AutotuneError> {
    let target_frequency = if settings.note == 0 {
        // Auto mode - snap to nearest note in the current key
        let scale_frequencies = get_scale_by_key(settings.key);
        find_nearest_note_in_key(exact_frequency, scale_frequencies)
    } else {
        // Manual mode - snap to specific note in key
        get_frequency(settings.key, settings.note, settings.octave, false)
    };

    if target_frequency <= 0.0 {
        return Err(AutotuneError::ProcessingFailed);
    }

    Ok(target_frequency)
}

/// Calculate octave scaling factor
fn calculate_octave_factor(octave: i32) -> f32 {
    match octave {
        1 => 0.5, // Down one octave
        2 => 1.0, // Normal
        4 => 2.0, // Up one octave
        _ => 1.0, // Default to normal
    }
}

/// Extract formant envelope using cepstral analysis (optional feature)
#[cfg(feature = "formant-shifting")]
fn extract_formant_envelope(
    analysis_magnitudes: &[f32],
    fft_size: usize,
) -> Result<Vec<f32>, AutotuneError> {
    const LIFTER_CUTOFF: usize = 64; // Low quefrency cutoff for formant extraction
    let spectrum_size = fft_size / 2;

    // Prepare full spectrum for cepstral analysis
    let mut log_spectrum = vec![microfft::Complex32 { re: 0.0, im: 0.0 }; fft_size];

    // Create log magnitude spectrum with Hermitian symmetry
    for i in 0..spectrum_size {
        let magnitude = analysis_magnitudes[i].max(1e-6); // Avoid log(0)
        let log_magnitude = logf(magnitude);

        log_spectrum[i] = microfft::Complex32 {
            re: log_magnitude,
            im: 0.0,
        };

        // Mirror for negative frequencies (except DC and Nyquist)
        if i > 0 && i < spectrum_size {
            log_spectrum[fft_size - i] = microfft::Complex32 {
                re: log_magnitude,
                im: 0.0,
            };
        }
    }

    // Compute cepstrum via inverse FFT of log spectrum
    let cepstrum_result = perform_ifft_1024(&mut log_spectrum)?;

    // Apply liftering - keep only low quefrency components (formants)
    let mut liftered_cepstrum = vec![0.0f32; fft_size];

    // Keep low quefrency components
    for i in 0..LIFTER_CUTOFF.min(fft_size / 2) {
        liftered_cepstrum[i] = cepstrum_result[i].re;
    }

    // Keep high quefrency components (symmetric)
    for i in ((fft_size - LIFTER_CUTOFF).max(fft_size / 2))..fft_size {
        liftered_cepstrum[i] = cepstrum_result[i].re;
    }

    // Transform back to frequency domain to get smoothed envelope
    let envelope_spectrum = perform_fft_1024(&liftered_cepstrum)?;

    // Convert back to magnitude domain
    let mut formant_envelope = vec![0.0f32; spectrum_size];
    for i in 0..spectrum_size {
        formant_envelope[i] = expf(envelope_spectrum[i].re);
    }

    Ok(formant_envelope)
}

/// Apply formant correction during pitch shifting
#[cfg(feature = "formant-shifting")]
fn apply_formant_correction(
    original_magnitude: f32,
    formant_envelope: &[f32],
    source_bin: usize,
    target_bin: usize,
    formant_mode: i32,
    spectrum_size: usize,
) -> f32 {
    // Formant shift ratios
    let formant_shift_ratio = match formant_mode {
        1 => 0.5, // Lower formants (deeper voice)
        2 => 2.0, // Higher formants (higher voice)
        _ => 1.0, // No formant shift
    };

    // Extract residual (harmonic content) by dividing out the formant envelope
    let source_envelope = formant_envelope[source_bin].max(1e-6);
    let residual = original_magnitude / source_envelope;

    // Find the formant envelope value at the shifted position
    let formant_lookup_pos =
        (target_bin as f32 / formant_shift_ratio).clamp(0.0, (spectrum_size - 1) as f32);

    // Linear interpolation for non-integer positions
    let lookup_index = formant_lookup_pos as usize;
    let lookup_fraction = formant_lookup_pos - lookup_index as f32;

    let target_envelope = if lookup_index < spectrum_size - 1 {
        formant_envelope[lookup_index] * (1.0 - lookup_fraction)
            + formant_envelope[lookup_index + 1] * lookup_fraction
    } else {
        formant_envelope[lookup_index]
    };

    // Apply the shifted formant envelope to the residual
    residual * target_envelope
}

/// Overlap-add processing helper for streaming audio
///
/// This function helps manage the overlap-add process needed for phase vocoder processing
/// when dealing with streaming audio that comes in smaller chunks than the FFT size.
pub fn process_streaming_autotune(
    input_samples: &[f32],
    output_samples: &mut [f32],
    state: &mut AutotuneState,
    settings: &MusicalSettings,
    input_buffer: &mut Vec<f32>,
    output_buffer: &mut Vec<f32>,
    samples_processed: &mut usize,
) -> Result<(), AutotuneError> {
    let config = state.config();
    let fft_size = config.fft_size;
    let hop_size = config.hop_size;

    // Initialize buffers if needed
    if input_buffer.len() != fft_size {
        input_buffer.resize(fft_size, 0.0);
    }
    if output_buffer.len() != fft_size {
        output_buffer.resize(fft_size, 0.0);
    }

    let mut output_written = 0;

    for &input_sample in input_samples {
        // Add new input sample to buffer
        input_buffer.rotate_left(1);
        input_buffer[fft_size - 1] = input_sample;

        *samples_processed += 1;

        // Process when we have enough samples for a new hop
        if *samples_processed % hop_size == 0 {
            let mut temp_output = vec![0.0f32; fft_size];

            // Process the current window
            process_autotune(input_buffer, &mut temp_output, state, settings)?;

            // Overlap-add the result
            for i in 0..fft_size {
                output_buffer[i] += temp_output[i];
            }
        }

        // Output the oldest sample and shift buffer
        if output_written < output_samples.len() {
            output_samples[output_written] = output_buffer[0];
            output_written += 1;
        }

        output_buffer.rotate_left(1);
        output_buffer[fft_size - 1] = 0.0;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AutotuneConfig, AutotuneState, MusicalSettings};

    #[test]
    fn test_process_autotune_silence() {
        let config = AutotuneConfig::default();
        let mut state = AutotuneState::new(config);
        let settings = MusicalSettings::default();

        let input = vec![0.0f32; 1024];
        let mut output = vec![0.0f32; 1024];

        assert!(process_autotune(&input, &mut output, &mut state, &settings).is_ok());

        // Output should be close to silent for silent input
        let max_output = output.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
        assert!(max_output < 0.01);
    }

    #[test]
    fn test_buffer_size_validation() {
        let config = AutotuneConfig::default();
        let mut state = AutotuneState::new(config);
        let settings = MusicalSettings::default();

        let input = vec![0.0f32; 512]; // Wrong size
        let mut output = vec![0.0f32; 1024];

        assert_eq!(
            process_autotune(&input, &mut output, &mut state, &settings),
            Err(AutotuneError::BufferSizeMismatch)
        );
    }

    #[test]
    fn test_target_frequency_calculation() {
        let settings = MusicalSettings {
            key: 0,  // C Major
            note: 0, // Auto mode
            octave: 2,
            formant: 0,
        };

        // 440 Hz should snap to A (440 Hz)
        let target = calculate_target_frequency(440.0, &settings).unwrap();
        assert!((target - 440.0).abs() < 1.0);

        // 445 Hz should also snap to A (440 Hz)
        let target = calculate_target_frequency(445.0, &settings).unwrap();
        assert!((target - 440.0).abs() < 10.0);
    }

    #[test]
    fn test_slice_conversion_helpers() {
        let mut test_slice = vec![1.0f32; 1024];
        assert!(slice_to_array_1024_f32(&mut test_slice).is_ok());

        let mut wrong_size = vec![1.0f32; 512];
        assert!(slice_to_array_1024_f32(&mut wrong_size).is_err());
    }

    #[test]
    fn test_fft_helpers() {
        let input = vec![0.0f32; 1024];
        assert!(perform_fft_1024(&input).is_ok());

        let mut spectrum = vec![microfft::Complex32 { re: 0.0, im: 0.0 }; 1024];
        assert!(perform_ifft_1024(&mut spectrum).is_ok());
    }
}
