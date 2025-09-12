use core::f32::consts::PI;

use libm::{atan2f, cosf, expf, fabsf, floorf, logf, sinf, sqrtf};

use crate::{
    AutotuneConfig, MusicalSettings, fft_config, find_fundamental_frequency,
    find_nearest_note_in_key, frequencies::D_MAJOR_SCALE_FREQUENCIES, get_frequency, hann_window,
    process_frequencies::collect_harmonics, ring_buffer::RingBuffer, wrap_phase,
};

fft_config!(1024, 48_014.312);

pub fn autotune_audio(
    unwrapped_buffer: &mut [f32; FFT_SIZE],
    last_input_phases: &mut [f32; FFT_SIZE],
    last_output_phases: &mut [f32; FFT_SIZE],
    mut previous_pitch_shift_ratio: f32,
    config: &AutotuneConfig,
    settings: &MusicalSettings,
) -> [f32; FFT_SIZE] {
    let analysis_window_buffer: [f32; FFT_SIZE] = hann_window::HANN_WINDOW;
    let mut full_spectrum: [microfft::Complex32; FFT_SIZE] =
        [microfft::Complex32 { re: 0.0, im: 0.0 }; FFT_SIZE];
    let mut analysis_magnitudes = [0.0; FFT_SIZE / 2];
    let mut analysis_frequencies = [0.0; FFT_SIZE / 2];
    let mut synthesis_magnitudes: [f32; FFT_SIZE] = [0.0; FFT_SIZE];
    let mut synthesis_frequencies: [f32; FFT_SIZE] = [0.0; FFT_SIZE];
    let mut _synthesis_count = [0; FFT_SIZE / 2];

    let formant = settings.formant;
    let is_auto = settings.note == 0;
    let note = settings.note;

    perform_fft_analysis(
        unwrapped_buffer,
        &analysis_window_buffer,
        last_input_phases,
        &mut analysis_magnitudes,
        &mut analysis_frequencies,
    );

    // Extract spectral envelope for formant preservation
    let mut envelope = [1.0f32; FFT_SIZE / 2];
    extract_cepstral_envelope(
        &analysis_magnitudes,
        &mut envelope,
        &mut full_spectrum,
        formant != 0,
    );

    // Calculate pitch shift ratio based on detected vs target frequency
    if let Some(pitch_shift_ratio) = calculate_pitch_shift_ratio(
        &analysis_magnitudes,
        &analysis_frequencies,
        is_auto,
        settings.note,
        settings.key,    // key
        settings.octave, // octave
        previous_pitch_shift_ratio,
    ) {
        // Update stored ratio for next iteration
        previous_pitch_shift_ratio = pitch_shift_ratio;

        // Apply pitch shifting and formant processing to spectrum
        apply_spectral_shift(
            &analysis_magnitudes,
            &analysis_frequencies,
            &envelope,
            &mut synthesis_magnitudes,
            &mut synthesis_frequencies,
            pitch_shift_ratio,
            formant,
            1.0, // octave_factor
        );
    }
    // SYNTHESIS

    let mut synthesis_output = [0.0f32; FFT_SIZE];
    perform_synthesis(
        &synthesis_magnitudes,
        &synthesis_frequencies,
        last_output_phases,
        &mut full_spectrum,
        &analysis_window_buffer,
        &mut synthesis_output,
    );

    synthesis_output
}

/// Performs FFT analysis on the input buffer and extracts magnitude and frequency information.
///
/// This function takes a time-domain audio buffer, applies windowing, performs FFT analysis,
/// and converts the complex frequency-domain data into magnitude and frequency representations
/// suitable for pitch shifting algorithms.
///
/// # Parameters
///
/// * `input_buffer` - Time domain audio samples to analyze
/// * `window` - Analysis window (typically Hann window) for spectral analysis
/// * `last_input_phases` - Phase values from previous hop, updated during analysis
/// * `analysis_magnitudes` - Output array for magnitude values (half spectrum)
/// * `analysis_frequencies` - Output array for precise frequency values (half spectrum)
///
/// # Algorithm
///
/// 1. **Windowing**: Applies analysis window to reduce spectral leakage
/// 2. **FFT**: Converts time domain to frequency domain using real FFT
/// 3. **Magnitude calculation**: Computes amplitude from real/imaginary components
/// 4. **Phase unwrapping**: Tracks phase evolution between hops for precise frequency estimation
/// 5. **Frequency estimation**: Uses phase vocoder technique to determine exact frequencies
///
/// The phase vocoder technique allows us to determine frequencies more precisely than
/// just the bin center frequencies, enabling high-quality pitch shifting.
pub fn perform_fft_analysis(
    input_buffer: &mut [f32; FFT_SIZE],
    window: &[f32; FFT_SIZE],
    last_input_phases: &mut [f32; FFT_SIZE],
    analysis_magnitudes: &mut [f32; FFT_SIZE / 2],
    analysis_frequencies: &mut [f32; FFT_SIZE / 2],
) {
    // Apply windowing to reduce spectral leakage
    for i in 0..FFT_SIZE {
        input_buffer[i] *= window[i];
    }

    // Perform real FFT (returns FFT_SIZE/2 + 1 complex values)
    let fft_result = microfft::real::rfft_1024(input_buffer);

    // Process each frequency bin
    for i in 0..fft_result.len() {
        // Turn real and imaginary components into amplitude(magnitude) and phase (Polar form)
        // This gives us how loud a given frequency component is in the signal
        let amplitude =
            sqrtf(fft_result[i].re * fft_result[i].re + fft_result[i].im * fft_result[i].im);
        let phase = atan2f(fft_result[i].im, fft_result[i].re);

        // Calculate the phase difference in this bin between the last
        // hop and this one, which will indirectly give us the exact frequency
        let mut phase_diff = phase - last_input_phases[i];

        // Subtract the amount of phase increment we'd expect to see based
        // on the centre frequency of this bin (2*pi*n/gFftSize) for this
        // hop size, then wrap to the range -pi to pi
        // This calculates the theoretical center frequency, each bin represents a frequency range, not a single frequency.
        let bin_centre_frequency = 2.0 * PI * i as f32 / FFT_SIZE as f32;
        phase_diff = wrap_phase(phase_diff - bin_centre_frequency * HOP_SIZE as f32);

        // Find deviation from the centre frequency
        // This lets us know where the actual frequency is relative to the center frequency of the bin.
        let bin_deviation = phase_diff * FFT_SIZE as f32 / HOP_SIZE as f32 / (2.0 * PI);

        // Add the original bin number to get the fractional bin where this partial belongs
        analysis_frequencies[i] = i as f32 + bin_deviation;
        // Save the magnitude for later
        analysis_magnitudes[i] = amplitude;

        // Save the phase for next hop
        last_input_phases[i] = phase;
    }
}

/// Extracts the spectral envelope using cepstral analysis for formant preservation.
///
/// This function implements cepstral envelope extraction to separate the harmonic content
/// (pitch-related information) from the spectral envelope (formant structure). This allows
/// for independent manipulation of pitch and formants during processing.
///
/// # Parameters
///
/// * `analysis_magnitudes` - Input magnitude spectrum from FFT analysis
/// * `envelope` - Output array to store the extracted spectral envelope
/// * `full_spectrum_buffer` - Working buffer for FFT operations (will be modified)
/// * `perform_extraction` - Whether to actually perform extraction (false returns unity envelope)
///
/// # Algorithm
///
/// 1. **Log magnitude**: Convert magnitude spectrum to log domain
/// 2. **Mirror spectrum**: Create full symmetric spectrum for real IFFT
/// 3. **Cepstrum calculation**: Apply inverse FFT to get cepstrum (quefrency domain)
/// 4. **Liftering**: Apply low-pass filter in cepstrum domain to extract envelope
/// 5. **Envelope reconstruction**: Transform back to frequency domain via FFT
///
/// The cepstral technique separates slowly-varying envelope (formants) from
/// rapidly-varying harmonic structure, enabling formant-preserving pitch shifting.
///
/// # Lifter Cutoff
///
/// The lifter cutoff of 64 samples determines the smoothness of the extracted envelope.
/// Lower values = smoother envelope, higher values = more detailed envelope.
pub fn extract_cepstral_envelope(
    analysis_magnitudes: &[f32; FFT_SIZE / 2],
    envelope: &mut [f32; FFT_SIZE / 2],
    full_spectrum_buffer: &mut [microfft::Complex32; FFT_SIZE],
    perform_extraction: bool,
) {
    // Initialize envelope to unity (no modification)
    envelope.fill(1.0);

    if !perform_extraction {
        return;
    }

    // Cepstral envelope extraction parameters
    const LIFTER_CUTOFF: usize = 64; // Controls envelope smoothness
    let mut cepstrum_buffer = [0.0f32; FFT_SIZE];

    // Step 1: Create log magnitude spectrum with mirrored symmetry
    for i in 0..(FFT_SIZE / 2) {
        let mag = analysis_magnitudes[i].max(1e-6); // Avoid log(0)
        let log_mag = logf(mag);

        // Set positive frequencies
        full_spectrum_buffer[i] = microfft::Complex32 { re: log_mag, im: 0.0 };

        // Mirror for negative frequencies (except DC)
        if i != 0 {
            full_spectrum_buffer[FFT_SIZE - i] = microfft::Complex32 { re: log_mag, im: 0.0 };
        }
    }

    // Step 2: Compute cepstrum via inverse FFT
    let cepstrum = microfft::inverse::ifft_1024(full_spectrum_buffer);

    // Step 3: Apply lifter (low-pass filter in quefrency domain)
    // Clear buffer and copy only low quefrency components
    cepstrum_buffer.fill(0.0);

    // Copy low quefrency components (positive side)
    for i in 0..LIFTER_CUTOFF {
        cepstrum_buffer[i] = cepstrum[i].re;
    }

    // Copy low quefrency components (negative side)
    for i in (FFT_SIZE - LIFTER_CUTOFF)..FFT_SIZE {
        cepstrum_buffer[i] = cepstrum[i].re;
    }

    // Step 4: Transform back to get smoothed log magnitude spectrum
    let envelope_fft = microfft::real::rfft_1024(&mut cepstrum_buffer);

    // Step 5: Convert back to linear magnitude domain
    for i in 0..(FFT_SIZE / 2) {
        envelope[i] = expf(envelope_fft[i].re);
    }
}

/// Calculates the pitch shift ratio needed to correct the input frequency to the target note.
///
/// This function performs fundamental frequency detection, finds the target frequency based on
/// the musical scale and settings, then calculates and smooths the pitch shift ratio to minimize
/// artifacts during real-time processing.
///
/// # Parameters
///
/// * `analysis_magnitudes` - Magnitude spectrum from FFT analysis
/// * `analysis_frequencies` - Precise frequency values for each bin
/// * `is_auto` - Whether to use automatic note detection (true) or fixed note
/// * `note` - Target note index (used when is_auto is false)
/// * `key` - Musical key (used when is_auto is false)
/// * `octave` - Target octave (used when is_auto is false)
/// * `previous_ratio` - Previous pitch shift ratio for smoothing
///
/// # Returns
///
/// * `Option<f32>` - The calculated pitch shift ratio, or None if no valid pitch detected
///
/// # Algorithm
///
/// 1. **Fundamental detection**: Find the loudest frequency component
/// 2. **Target calculation**: Determine target frequency from scale/note settings
/// 3. **Ratio calculation**: Compute pitch shift ratio (target/detected)
/// 4. **Clamping**: Limit ratio to reasonable bounds (0.5 to 2.0)
/// 5. **Smoothing**: Apply temporal smoothing to reduce artifacts
pub fn calculate_pitch_shift_ratio(
    analysis_magnitudes: &[f32; FFT_SIZE / 2],
    analysis_frequencies: &[f32; FFT_SIZE / 2],
    is_auto: bool,
    note: i32,
    key: i32,
    octave: i32,
    previous_ratio: f32,
) -> Option<f32> {
    // Detect fundamental frequency (loudest component)
    let fundamental_index = find_fundamental_frequency(analysis_magnitudes);
    let _harmonics = collect_harmonics(fundamental_index);

    // Convert bin index to actual frequency
    let detected_frequency = analysis_frequencies[fundamental_index] * BIN_WIDTH;

    // Skip processing if frequency is too low (likely noise or silence)
    if detected_frequency <= 0.001 {
        return None;
    }

    // Determine target frequency based on musical settings
    let target_frequency = if is_auto {
        // Auto mode: snap to nearest note in the current scale
        let scale_frequencies = &D_MAJOR_SCALE_FREQUENCIES;
        find_nearest_note_in_key(detected_frequency, scale_frequencies)
    } else {
        // Manual mode: use specified note/key/octave
        get_frequency(key, note, octave, false)
    };

    // Calculate raw pitch shift ratio
    let raw_ratio = target_frequency / detected_frequency;

    // Clamp to reasonable bounds to prevent extreme artifacts
    let clamped_ratio = raw_ratio.clamp(0.5, 2.0);

    // Apply aggressive temporal smoothing to reduce pitch wobble
    const SMOOTHING_FACTOR: f32 = 0.99;
    let smoothed_pitch_shift_ratio =
        clamped_ratio * SMOOTHING_FACTOR + previous_ratio * (1.0 - SMOOTHING_FACTOR);

    Some(smoothed_pitch_shift_ratio)
}

/// Applies pitch shifting and formant processing to the frequency spectrum.
///
/// This function takes the analysis results and applies the calculated pitch shift ratio
/// while optionally preserving or modifying formant characteristics. It implements the
/// core spectral manipulation for the pitch correction effect.
///
/// # Parameters
///
/// * `analysis_magnitudes` - Input magnitude spectrum
/// * `analysis_frequencies` - Input frequency values for each bin
/// * `envelope` - Spectral envelope for formant processing
/// * `synthesis_magnitudes` - Output magnitude spectrum (will be filled)
/// * `synthesis_frequencies` - Output frequency spectrum (will be filled)
/// * `pitch_shift_ratio` - How much to shift the pitch (1.0 = no change)
/// * `formant_mode` - Formant processing mode (0=none, 1=lower, 2=raise)
/// * `octave_factor` - Additional octave shifting factor
///
/// # Algorithm
///
/// 1. **Residual extraction**: Separate harmonic content from formants
/// 2. **Frequency mapping**: Map each input bin to new output frequency
/// 3. **Formant shifting**: Apply independent formant manipulation
/// 4. **Envelope interpolation**: Reconstruct formant structure at new positions
/// 5. **Magnitude reconstruction**: Combine residual with shifted envelope
pub fn apply_spectral_shift(
    analysis_magnitudes: &[f32; FFT_SIZE / 2],
    analysis_frequencies: &[f32; FFT_SIZE / 2],
    envelope: &[f32; FFT_SIZE / 2],
    synthesis_magnitudes: &mut [f32; FFT_SIZE],
    synthesis_frequencies: &mut [f32; FFT_SIZE],
    pitch_shift_ratio: f32,
    formant_mode: i32,
    octave_factor: f32,
) {
    // Clear output arrays
    synthesis_magnitudes.fill(0.0);
    synthesis_frequencies.fill(0.0);

    // Determine formant shift ratio based on mode
    let formant_ratio = match formant_mode {
        1 => 0.5, // Lower formants (deeper voice)
        2 => 2.0, // Raise formants (higher voice)
        _ => 1.0, // No formant modification
    };

    let use_formants = formant_mode != 0;

    // Process each frequency bin
    for i in 0..(FFT_SIZE / 2) {
        // Skip bins with negligible energy
        if analysis_magnitudes[i] <= 1e-8 {
            continue;
        }

        // Extract residual (harmonic content separated from formant envelope)
        let residual = if use_formants {
            analysis_magnitudes[i] / envelope[i].max(1e-6)
        } else {
            analysis_magnitudes[i]
        };

        // Calculate target bin for pitch-shifted content
        let new_bin_f = i as f32 * pitch_shift_ratio * octave_factor;
        let new_bin = (floorf(new_bin_f + 0.5) as usize).min((FFT_SIZE / 2) - 1);

        // Skip if target bin is out of range
        if new_bin >= FFT_SIZE / 2 {
            continue;
        }

        // Get formant envelope at the shifted position
        let shifted_envelope = if use_formants {
            // Calculate position in envelope to sample from
            let env_pos = (i as f32 / formant_ratio).clamp(0.0, (FFT_SIZE / 2 - 1) as f32);
            let env_idx = env_pos as usize;
            let frac = env_pos - env_idx as f32;

            // Linear interpolation between envelope samples
            if env_idx < (FFT_SIZE / 2) - 1 {
                envelope[env_idx] * (1.0 - frac) + envelope[env_idx + 1] * frac
            } else {
                envelope[env_idx]
            }
        } else {
            1.0
        };

        // Reconstruct final magnitude: residual * shifted envelope
        let final_magnitude = residual * shifted_envelope;

        // Store results in synthesis arrays
        synthesis_magnitudes[new_bin] = final_magnitude;
        synthesis_frequencies[new_bin] =
            analysis_frequencies[i] * pitch_shift_ratio * octave_factor;
    }
}

/// Performs inverse FFT synthesis to convert processed frequency domain data back to audio.
///
/// This function implements the synthesis stage of a phase vocoder, converting the processed
/// magnitude and frequency information back into time-domain audio samples. It handles phase
/// coherence and prepares windowed samples for overlap-add reconstruction.
///
/// # Parameters
///
/// * `synthesis_magnitudes` - Processed magnitude spectrum to synthesize
/// * `synthesis_frequencies` - Processed frequency values for each bin
/// * `last_output_phases` - Phase accumulator from previous hop (updated during synthesis)
/// * `full_spectrum_buffer` - Working buffer for full complex spectrum
/// * `analysis_window` - Window function used for overlap-add (typically Hann)
/// * `output_samples` - Output buffer to receive the synthesized audio samples
///
/// # Algorithm
///
/// 1. **Phase calculation**: Update phase based on frequency deviation from bin centers
/// 2. **Complex spectrum construction**: Create full hermitian-symmetric spectrum for IFFT
/// 3. **Inverse FFT**: Convert frequency domain back to time domain
/// 4. **Windowing**: Apply window function for proper overlap-add reconstruction
/// 5. **Gain compensation**: Adjust for window overlap losses
/// 6. **Soft limiting**: Prevent clipping with gentle amplitude limiting
///
/// The phase vocoder synthesis ensures phase coherence between overlapping frames,
/// which is critical for maintaining audio quality during pitch shifting operations.
///
/// Note: This function does NOT write to the output ring buffer to avoid holding
/// locks during expensive computation. Use `write_synthesis_output` separately.
pub fn perform_synthesis(
    synthesis_magnitudes: &[f32; FFT_SIZE],
    synthesis_frequencies: &[f32; FFT_SIZE],
    last_output_phases: &mut [f32; FFT_SIZE],
    full_spectrum_buffer: &mut [microfft::Complex32; FFT_SIZE],
    analysis_window: &[f32; FFT_SIZE],
    output_samples: &mut [f32; FFT_SIZE],
) {
    // Phase vocoder synthesis - reconstruct phases for each bin
    for i in 0..(FFT_SIZE / 2) {
        let magnitude = synthesis_magnitudes[i];

        // Calculate frequency deviation from bin center
        let bin_deviation = synthesis_frequencies[i] - i as f32;

        // Convert frequency deviation to phase increment
        let mut phase_increment = bin_deviation * 2.0 * PI * HOP_SIZE as f32 / FFT_SIZE as f32;

        // Add expected phase increment for this bin's center frequency
        let bin_center_frequency = 2.0 * PI * i as f32 / FFT_SIZE as f32;
        phase_increment += bin_center_frequency * HOP_SIZE as f32;

        // Update accumulated phase for this bin
        let output_phase = wrap_phase(last_output_phases[i] + phase_increment);

        // Convert polar form (magnitude, phase) back to complex form (real, imaginary)
        let real_part = magnitude * cosf(output_phase);
        let imaginary_part = magnitude * sinf(output_phase);

        // Set positive frequency component
        full_spectrum_buffer[i] = microfft::Complex32 { re: real_part, im: imaginary_part };

        // Set negative frequency component (Hermitian symmetry for real IFFT)
        // This ensures the IFFT output will be real-valued
        if i > 0 && i < FFT_SIZE / 2 {
            full_spectrum_buffer[FFT_SIZE - i] = microfft::Complex32 {
                re: real_part,
                im: -imaginary_part, // Complex conjugate
            };
        }

        // Store phase for next synthesis frame
        last_output_phases[i] = output_phase;
    }

    // Perform inverse FFT to get time domain signal
    let time_domain_result = microfft::inverse::ifft_1024(full_spectrum_buffer);

    // Apply windowing, gain compensation, and soft limiting
    const GAIN_COMPENSATION: f32 = 2.0 / 3.0; // Compensate for 50% window overlap losses

    for i in 0..FFT_SIZE {
        // Extract real part from complex IFFT result
        let mut sample = time_domain_result[i].re;

        // Apply synthesis window (same as analysis window for perfect reconstruction)
        sample *= analysis_window[i];

        // Apply gain compensation to account for window overlap losses
        sample *= GAIN_COMPENSATION;

        // Soft limiting to prevent harsh clipping artifacts
        if sample.abs() > 0.95 {
            // Gentle compression using exponential curve
            let sign = sample.signum();
            let compressed = 0.95 - 0.05 * expf(-sample.abs());
            sample = sign * compressed;
        }

        // Store processed sample for output
        output_samples[i] = sample;
    }
}

/// Writes synthesized audio samples to the output ring buffer using overlap-add.
///
/// This function is separated from the main synthesis to minimize time spent
/// holding the output buffer lock. It performs the final overlap-add step.
///
/// # Parameters
///
/// * `output_samples` - Processed audio samples from synthesis
/// * `output_ring` - Ring buffer to receive the samples
pub fn write_synthesis_output(
    output_samples: &[f32; FFT_SIZE],
    output_ring: &RingBuffer<BUFFER_SIZE>,
) {
    // Add samples to output buffer using overlap-add (accumulation)
    for i in 0..FFT_SIZE {
        output_ring.add_at_offset(i as u32, output_samples[i]);
    }
}

pub fn normalize_sample(sample: f32, target_peak: f32) -> f32 {
    let abs_sample = fabsf(sample);
    if abs_sample > target_peak {
        // Soft limiting to prevent harsh clipping
        let ratio = target_peak / abs_sample;
        let soft_ratio = 1.0 - expf(-3.0 * ratio);
        sample * soft_ratio
    } else {
        sample
    }
}
