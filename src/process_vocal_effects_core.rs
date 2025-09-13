//! Core Autotune Implementation
//!
//! This module contains shared autotune processing functions that use generics
//! to eliminate code duplication across different FFT size configurations.

use crate::{AutotuneConfig, MusicalSettings};
use core::f32::consts::PI;
use libm::{atan2f, cosf, expf, fabsf, floorf, logf, sinf, sqrtf};

/// Trait for FFT operations to abstract over different sizes
pub trait FftOps<const N: usize, const HALF_N: usize> {
    /// Perform forward real FFT
    fn forward_fft(input: &mut [f32; N]) -> &mut [microfft::Complex32];

    /// Perform inverse complex FFT
    fn inverse_fft(spectrum: &mut [microfft::Complex32; N]) -> &mut [microfft::Complex32; N];

    /// Get the Hann window for this FFT size
    fn get_hann_window() -> &'static [f32; N];
}

/// FFT operations for 512-point FFT
pub struct Fft512;
impl FftOps<512, 256> for Fft512 {
    fn forward_fft(input: &mut [f32; 512]) -> &mut [microfft::Complex32] {
        microfft::real::rfft_512(input)
    }

    fn inverse_fft(spectrum: &mut [microfft::Complex32; 512]) -> &mut [microfft::Complex32; 512] {
        microfft::inverse::ifft_512(spectrum)
    }

    fn get_hann_window() -> &'static [f32; 512] {
        &crate::hann_window::HANN_WINDOW_512
    }
}

/// FFT operations for 1024-point FFT
pub struct Fft1024;
impl FftOps<1024, 512> for Fft1024 {
    fn forward_fft(input: &mut [f32; 1024]) -> &mut [microfft::Complex32] {
        microfft::real::rfft_1024(input)
    }

    fn inverse_fft(spectrum: &mut [microfft::Complex32; 1024]) -> &mut [microfft::Complex32; 1024] {
        microfft::inverse::ifft_1024(spectrum)
    }

    fn get_hann_window() -> &'static [f32; 1024] {
        &crate::hann_window::HANN_WINDOW_1024
    }
}

/// FFT operations for 2048-point FFT
pub struct Fft2048;
impl FftOps<2048, 1024> for Fft2048 {
    fn forward_fft(input: &mut [f32; 2048]) -> &mut [microfft::Complex32] {
        microfft::real::rfft_2048(input)
    }

    fn inverse_fft(spectrum: &mut [microfft::Complex32; 2048]) -> &mut [microfft::Complex32; 2048] {
        microfft::inverse::ifft_2048(spectrum)
    }

    fn get_hann_window() -> &'static [f32; 2048] {
        &crate::hann_window::HANN_WINDOW_2048
    }
}

/// FFT operations for 4096-point FFT
pub struct Fft4096;
impl FftOps<4096, 2048> for Fft4096 {
    fn forward_fft(input: &mut [f32; 4096]) -> &mut [microfft::Complex32] {
        microfft::real::rfft_4096(input)
    }

    fn inverse_fft(spectrum: &mut [microfft::Complex32; 4096]) -> &mut [microfft::Complex32; 4096] {
        microfft::inverse::ifft_4096(spectrum)
    }

    fn get_hann_window() -> &'static [f32; 4096] {
        &crate::hann_window::HANN_WINDOW_4096
    }
}

/// Extract cepstral envelope for formant preservation using generic FFT operations
fn extract_cepstral_envelope<const N: usize, const HALF_N: usize, F>(
    analysis_magnitudes: &[f32; HALF_N],
    envelope: &mut [f32; HALF_N],
) where
    F: FftOps<N, HALF_N>,
{
    const LIFTER_CUTOFF: usize = 64;
    let mut full_spectrum = [microfft::Complex32 { re: 0.0, im: 0.0 }; N];
    let mut cepstrum_buffer = [0.0f32; N];

    // Compute log spectrum
    for i in 0..HALF_N {
        let mag = analysis_magnitudes[i].max(1e-6_f32);
        let log_mag = logf(mag);
        full_spectrum[i] = microfft::Complex32 { re: log_mag, im: 0.0 };
        if i != 0 {
            full_spectrum[N - i] = microfft::Complex32 { re: log_mag, im: 0.0 };
        }
    }

    // Inverse FFT to get cepstrum
    let cepstrum = F::inverse_fft(&mut full_spectrum);

    // Apply liftering (low-pass in cepstral domain)
    cepstrum_buffer.fill(0.0);
    for i in 0..LIFTER_CUTOFF.min(HALF_N) {
        cepstrum_buffer[i] = cepstrum[i].re;
    }
    for i in (N - LIFTER_CUTOFF.min(HALF_N))..N {
        cepstrum_buffer[i] = cepstrum[i].re;
    }

    // Forward FFT to get smoothed envelope
    let envelope_fft = F::forward_fft(&mut cepstrum_buffer);
    for i in 0..HALF_N {
        envelope[i] = expf(envelope_fft[i].re);
    }
}

/// Calculate pitch shift ratio
fn calculate_pitch_shift(
    analysis_magnitudes: &[f32],
    analysis_frequencies: &[f32],
    previous_pitch_shift_ratio: f32,
    settings: &MusicalSettings,
    bin_width: f32,
) -> f32 {
    let mut pitch_shift_ratio = previous_pitch_shift_ratio;
    let fundamental_index =
        crate::process_frequencies::find_fundamental_frequency(analysis_magnitudes);
    let detected_frequency = analysis_frequencies[fundamental_index] * bin_width;

    if detected_frequency > 0.001 {
        let target_frequency = if settings.note == 0 {
            let scale_frequencies = crate::keys::get_scale_by_key(settings.key);
            crate::frequencies::find_nearest_note_in_key(detected_frequency, scale_frequencies)
        } else {
            crate::keys::get_frequency(settings.key, settings.note, settings.octave, false)
        };
        let raw_ratio = target_frequency / detected_frequency;
        let clamped_ratio = raw_ratio.clamp(0.5, 2.0);
        const SMOOTHING_FACTOR: f32 = 0.99;
        pitch_shift_ratio = clamped_ratio * SMOOTHING_FACTOR
            + previous_pitch_shift_ratio * (1.0 - SMOOTHING_FACTOR);
    }

    pitch_shift_ratio
}

/// Generic autotune processing function that works with different FFT sizes
fn process_vocal_effects_generic<const N: usize, const HALF_N: usize, F>(
    unwrapped_buffer: &mut [f32; N],
    last_input_phases: &mut [f32; N],
    last_output_phases: &mut [f32; N],
    previous_pitch_shift_ratio: f32,
    _config: &AutotuneConfig,
    settings: &MusicalSettings,
    sample_rate: f32,
    hop_ratio: f32,
) -> [f32; N]
where
    F: FftOps<N, HALF_N>,
{
    const GAIN_COMPENSATION: f32 = 2.0 / 3.0;

    let hop_size = (N as f32 * hop_ratio) as usize;
    let bin_width = sample_rate / N as f32;

    let analysis_window_buffer = F::get_hann_window();
    let mut full_spectrum = [microfft::Complex32 { re: 0.0, im: 0.0 }; N];
    let mut analysis_magnitudes = [0.0; HALF_N];
    let mut analysis_frequencies = [0.0; HALF_N];
    let mut synthesis_magnitudes = [0.0; N];
    let mut synthesis_frequencies = [0.0; N];
    let mut envelope = [1.0f32; HALF_N];

    let formant = settings.formant;

    // Apply windowing
    for i in 0..N {
        unwrapped_buffer[i] *= analysis_window_buffer[i];
    }

    // Forward FFT
    let fft_result = F::forward_fft(unwrapped_buffer);

    // Process frequency bins - limit to the actual number of bins we have arrays for
    let num_bins = HALF_N.min(fft_result.len());
    for i in 0..num_bins {
        let amplitude =
            sqrtf(fft_result[i].re * fft_result[i].re + fft_result[i].im * fft_result[i].im);
        let phase = atan2f(fft_result[i].im, fft_result[i].re);
        let mut phase_diff = phase - last_input_phases[i];
        let bin_centre_frequency = 2.0 * PI * i as f32 / N as f32;
        phase_diff = crate::process_frequencies::wrap_phase(
            phase_diff - bin_centre_frequency * hop_size as f32,
        );
        let bin_deviation = phase_diff * N as f32 / hop_size as f32 / (2.0 * PI);
        analysis_frequencies[i] = i as f32 + bin_deviation;
        analysis_magnitudes[i] = amplitude;
        last_input_phases[i] = phase;
    }

    // Extract formant envelope if needed
    if formant != 0 {
        extract_cepstral_envelope::<N, HALF_N, F>(&analysis_magnitudes, &mut envelope);
    }

    // Calculate pitch shift
    let pitch_shift_ratio = calculate_pitch_shift(
        &analysis_magnitudes,
        &analysis_frequencies,
        previous_pitch_shift_ratio,
        settings,
        bin_width,
    );

    // Apply spectral shift
    synthesis_magnitudes.fill(0.0);
    synthesis_frequencies.fill(0.0);
    let formant_ratio = match formant {
        1 => 0.5,
        2 => 2.0,
        _ => 1.0,
    };
    let use_formants = formant != 0;

    for i in 0..num_bins {
        if analysis_magnitudes[i] <= 1e-8 {
            continue;
        }
        let residual = if use_formants {
            analysis_magnitudes[i] / envelope[i].max(1e-6_f32)
        } else {
            analysis_magnitudes[i]
        };
        let new_bin_f = i as f32 * pitch_shift_ratio;
        let new_bin = (floorf(new_bin_f + 0.5) as usize).min(num_bins - 1);
        if new_bin >= num_bins {
            continue;
        }

        let shifted_envelope = if use_formants {
            let env_pos = (i as f32 / formant_ratio).clamp(0.0, (num_bins - 1) as f32);
            let env_idx = env_pos as usize;
            let frac = env_pos - env_idx as f32;
            if env_idx < num_bins - 1 {
                envelope[env_idx] * (1.0 - frac) + envelope[env_idx + 1] * frac
            } else {
                envelope[env_idx]
            }
        } else {
            1.0
        };

        synthesis_magnitudes[new_bin] = residual * shifted_envelope;
        synthesis_frequencies[new_bin] = analysis_frequencies[i] * pitch_shift_ratio;
    }

    // Synthesis phase reconstruction
    for i in 0..num_bins {
        let magnitude = synthesis_magnitudes[i];
        let bin_deviation = synthesis_frequencies[i] - i as f32;
        let mut phase_increment = bin_deviation * 2.0 * PI * hop_size as f32 / N as f32;
        let bin_center_frequency = 2.0 * PI * i as f32 / N as f32;
        phase_increment += bin_center_frequency * hop_size as f32;
        let output_phase =
            crate::process_frequencies::wrap_phase(last_output_phases[i] + phase_increment);
        let real_part = magnitude * cosf(output_phase);
        let imaginary_part = magnitude * sinf(output_phase);
        full_spectrum[i] = microfft::Complex32 { re: real_part, im: imaginary_part };
        if i > 0 && i < num_bins {
            full_spectrum[N - i] = microfft::Complex32 { re: real_part, im: -imaginary_part };
        }
        last_output_phases[i] = output_phase;
    }

    // Inverse FFT
    let time_domain_result = F::inverse_fft(&mut full_spectrum);
    let mut output_samples = [0.0f32; N];

    for i in 0..N {
        let mut sample = time_domain_result[i].re;
        sample *= analysis_window_buffer[i];
        sample *= GAIN_COMPENSATION;
        if sample.abs() > 0.95 {
            let sign = if sample >= 0.0 { 1.0 } else { -1.0 };
            let compressed = 0.95 - 0.05 * expf(-fabsf(sample));
            sample = sign * compressed;
        }
        output_samples[i] = sample;
    }

    output_samples
}

/// Specialized autotune function for 512-point FFT
pub fn process_vocal_effects_512(
    unwrapped_buffer: &mut [f32; 512],
    last_input_phases: &mut [f32; 512],
    last_output_phases: &mut [f32; 512],
    previous_pitch_shift_ratio: f32,
    config: &AutotuneConfig,
    settings: &MusicalSettings,
    sample_rate: f32,
    hop_ratio: f32,
) -> [f32; 512] {
    process_vocal_effects_generic::<512, 256, Fft512>(
        unwrapped_buffer,
        last_input_phases,
        last_output_phases,
        previous_pitch_shift_ratio,
        config,
        settings,
        sample_rate,
        hop_ratio,
    )
}

/// Specialized autotune function for 1024-point FFT
pub fn process_vocal_effects_1024(
    unwrapped_buffer: &mut [f32; 1024],
    last_input_phases: &mut [f32; 1024],
    last_output_phases: &mut [f32; 1024],
    previous_pitch_shift_ratio: f32,
    config: &AutotuneConfig,
    settings: &MusicalSettings,
    sample_rate: f32,
    hop_ratio: f32,
) -> [f32; 1024] {
    process_vocal_effects_generic::<1024, 512, Fft1024>(
        unwrapped_buffer,
        last_input_phases,
        last_output_phases,
        previous_pitch_shift_ratio,
        config,
        settings,
        sample_rate,
        hop_ratio,
    )
}

/// Specialized autotune function for 2048-point FFT
pub fn process_vocal_effects_2048(
    unwrapped_buffer: &mut [f32; 2048],
    last_input_phases: &mut [f32; 2048],
    last_output_phases: &mut [f32; 2048],
    previous_pitch_shift_ratio: f32,
    config: &AutotuneConfig,
    settings: &MusicalSettings,
    sample_rate: f32,
    hop_ratio: f32,
) -> [f32; 2048] {
    process_vocal_effects_generic::<2048, 1024, Fft2048>(
        unwrapped_buffer,
        last_input_phases,
        last_output_phases,
        previous_pitch_shift_ratio,
        config,
        settings,
        sample_rate,
        hop_ratio,
    )
}

/// Specialized autotune function for 4096-point FFT
pub fn process_vocal_effects_4096(
    unwrapped_buffer: &mut [f32; 4096],
    last_input_phases: &mut [f32; 4096],
    last_output_phases: &mut [f32; 4096],
    previous_pitch_shift_ratio: f32,
    config: &AutotuneConfig,
    settings: &MusicalSettings,
    sample_rate: f32,
    hop_ratio: f32,
) -> [f32; 4096] {
    process_vocal_effects_generic::<4096, 2048, Fft4096>(
        unwrapped_buffer,
        last_input_phases,
        last_output_phases,
        previous_pitch_shift_ratio,
        config,
        settings,
        sample_rate,
        hop_ratio,
    )
}
