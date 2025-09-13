//! Core Autotune Implementation
//!
//! This module contains shared autotune processing functions that eliminate
//! code duplication across different FFT size configurations.

use crate::{AutotuneConfig, MusicalSettings};
use core::f32::consts::PI;
use libm::{atan2f, cosf, expf, fabsf, floorf, logf, sinf, sqrtf};

/// Extract cepstral envelope for formant preservation (512-point)
fn extract_cepstral_envelope_512(analysis_magnitudes: &[f32; 256], envelope: &mut [f32; 256]) {
    const LIFTER_CUTOFF: usize = 64;
    let mut full_spectrum = [microfft::Complex32 { re: 0.0, im: 0.0 }; 512];
    let mut cepstrum_buffer = [0.0f32; 512];

    for i in 0..256 {
        let mag = analysis_magnitudes[i].max(1e-6_f32);
        let log_mag = logf(mag);
        full_spectrum[i] = microfft::Complex32 { re: log_mag, im: 0.0 };
        if i != 0 {
            full_spectrum[512 - i] = microfft::Complex32 { re: log_mag, im: 0.0 };
        }
    }

    let cepstrum = microfft::inverse::ifft_512(&mut full_spectrum);
    cepstrum_buffer.fill(0.0);
    for i in 0..LIFTER_CUTOFF.min(256) {
        cepstrum_buffer[i] = cepstrum[i].re;
    }
    for i in (512 - LIFTER_CUTOFF.min(256))..512 {
        cepstrum_buffer[i] = cepstrum[i].re;
    }

    let envelope_fft = microfft::real::rfft_512(&mut cepstrum_buffer);
    for i in 0..256 {
        envelope[i] = expf(envelope_fft[i].re);
    }
}

/// Extract cepstral envelope for formant preservation (1024-point)
fn extract_cepstral_envelope_1024(analysis_magnitudes: &[f32; 512], envelope: &mut [f32; 512]) {
    const LIFTER_CUTOFF: usize = 64;
    let mut full_spectrum = [microfft::Complex32 { re: 0.0, im: 0.0 }; 1024];
    let mut cepstrum_buffer = [0.0f32; 1024];

    for i in 0..512 {
        let mag = analysis_magnitudes[i].max(1e-6_f32);
        let log_mag = logf(mag);
        full_spectrum[i] = microfft::Complex32 { re: log_mag, im: 0.0 };
        if i != 0 {
            full_spectrum[1024 - i] = microfft::Complex32 { re: log_mag, im: 0.0 };
        }
    }

    let cepstrum = microfft::inverse::ifft_1024(&mut full_spectrum);
    cepstrum_buffer.fill(0.0);
    for i in 0..LIFTER_CUTOFF.min(512) {
        cepstrum_buffer[i] = cepstrum[i].re;
    }
    for i in (1024 - LIFTER_CUTOFF.min(512))..1024 {
        cepstrum_buffer[i] = cepstrum[i].re;
    }

    let envelope_fft = microfft::real::rfft_1024(&mut cepstrum_buffer);
    for i in 0..512 {
        envelope[i] = expf(envelope_fft[i].re);
    }
}

/// Extract cepstral envelope for formant preservation (2048-point)
fn extract_cepstral_envelope_2048(analysis_magnitudes: &[f32; 1024], envelope: &mut [f32; 1024]) {
    const LIFTER_CUTOFF: usize = 64;
    let mut full_spectrum = [microfft::Complex32 { re: 0.0, im: 0.0 }; 2048];
    let mut cepstrum_buffer = [0.0f32; 2048];

    for i in 0..1024 {
        let mag = analysis_magnitudes[i].max(1e-6_f32);
        let log_mag = logf(mag);
        full_spectrum[i] = microfft::Complex32 { re: log_mag, im: 0.0 };
        if i != 0 {
            full_spectrum[2048 - i] = microfft::Complex32 { re: log_mag, im: 0.0 };
        }
    }

    let cepstrum = microfft::inverse::ifft_2048(&mut full_spectrum);
    cepstrum_buffer.fill(0.0);
    for i in 0..LIFTER_CUTOFF.min(1024) {
        cepstrum_buffer[i] = cepstrum[i].re;
    }
    for i in (2048 - LIFTER_CUTOFF.min(1024))..2048 {
        cepstrum_buffer[i] = cepstrum[i].re;
    }

    let envelope_fft = microfft::real::rfft_2048(&mut cepstrum_buffer);
    for i in 0..1024 {
        envelope[i] = expf(envelope_fft[i].re);
    }
}

/// Extract cepstral envelope for formant preservation (4096-point)
fn extract_cepstral_envelope_4096(analysis_magnitudes: &[f32; 2048], envelope: &mut [f32; 2048]) {
    const LIFTER_CUTOFF: usize = 64;
    let mut full_spectrum = [microfft::Complex32 { re: 0.0, im: 0.0 }; 4096];
    let mut cepstrum_buffer = [0.0f32; 4096];

    for i in 0..2048 {
        let mag = analysis_magnitudes[i].max(1e-6_f32);
        let log_mag = logf(mag);
        full_spectrum[i] = microfft::Complex32 { re: log_mag, im: 0.0 };
        if i != 0 {
            full_spectrum[4096 - i] = microfft::Complex32 { re: log_mag, im: 0.0 };
        }
    }

    let cepstrum = microfft::inverse::ifft_4096(&mut full_spectrum);
    cepstrum_buffer.fill(0.0);
    for i in 0..LIFTER_CUTOFF.min(2048) {
        cepstrum_buffer[i] = cepstrum[i].re;
    }
    for i in (4096 - LIFTER_CUTOFF.min(2048))..4096 {
        cepstrum_buffer[i] = cepstrum[i].re;
    }

    let envelope_fft = microfft::real::rfft_4096(&mut cepstrum_buffer);
    for i in 0..2048 {
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

/// Specialized autotune function for 512-point FFT
pub fn process_vocal_effects_512(
    unwrapped_buffer: &mut [f32; 512],
    last_input_phases: &mut [f32; 512],
    last_output_phases: &mut [f32; 512],
    previous_pitch_shift_ratio: f32,
    _config: &AutotuneConfig,
    settings: &MusicalSettings,
    sample_rate: f32,
    hop_ratio: f32,
) -> [f32; 512] {
    const FFT_SIZE: usize = 512;
    const HALF_SIZE: usize = 256;
    const GAIN_COMPENSATION: f32 = 2.0 / 3.0;

    let hop_size = (FFT_SIZE as f32 * hop_ratio) as usize;
    let bin_width = sample_rate / FFT_SIZE as f32;

    let analysis_window_buffer = crate::hann_window::get_hann_window::<512>();
    let mut full_spectrum = [microfft::Complex32 { re: 0.0, im: 0.0 }; 512];
    let mut analysis_magnitudes = [0.0; 256];
    let mut analysis_frequencies = [0.0; 256];
    let mut synthesis_magnitudes = [0.0; 512];
    let mut synthesis_frequencies = [0.0; 512];
    let mut envelope = [1.0f32; 256];

    let formant = settings.formant;

    // Apply windowing
    for i in 0..512 {
        unwrapped_buffer[i] *= analysis_window_buffer[i];
    }

    // Forward FFT
    let fft_result = microfft::real::rfft_512(unwrapped_buffer);

    // Process frequency bins
    for i in 0..fft_result.len() {
        let amplitude =
            sqrtf(fft_result[i].re * fft_result[i].re + fft_result[i].im * fft_result[i].im);
        let phase = atan2f(fft_result[i].im, fft_result[i].re);
        let mut phase_diff = phase - last_input_phases[i];
        let bin_centre_frequency = 2.0 * PI * i as f32 / 512.0;
        phase_diff = crate::process_frequencies::wrap_phase(
            phase_diff - bin_centre_frequency * hop_size as f32,
        );
        let bin_deviation = phase_diff * 512.0 / hop_size as f32 / (2.0 * PI);
        analysis_frequencies[i] = i as f32 + bin_deviation;
        analysis_magnitudes[i] = amplitude;
        last_input_phases[i] = phase;
    }

    // Extract formant envelope if needed
    if formant != 0 {
        extract_cepstral_envelope_512(&analysis_magnitudes, &mut envelope);
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

    for i in 0..HALF_SIZE {
        if analysis_magnitudes[i] <= 1e-8 {
            continue;
        }
        let residual = if use_formants {
            analysis_magnitudes[i] / envelope[i].max(1e-6_f32)
        } else {
            analysis_magnitudes[i]
        };
        let new_bin_f = i as f32 * pitch_shift_ratio;
        let new_bin = (floorf(new_bin_f + 0.5) as usize).min(HALF_SIZE - 1);
        if new_bin >= HALF_SIZE {
            continue;
        }

        let shifted_envelope = if use_formants {
            let env_pos = (i as f32 / formant_ratio).clamp(0.0, (HALF_SIZE - 1) as f32);
            let env_idx = env_pos as usize;
            let frac = env_pos - env_idx as f32;
            if env_idx < HALF_SIZE - 1 {
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
    for i in 0..HALF_SIZE {
        let magnitude = synthesis_magnitudes[i];
        let bin_deviation = synthesis_frequencies[i] - i as f32;
        let mut phase_increment = bin_deviation * 2.0 * PI * hop_size as f32 / 512.0;
        let bin_center_frequency = 2.0 * PI * i as f32 / 512.0;
        phase_increment += bin_center_frequency * hop_size as f32;
        let output_phase =
            crate::process_frequencies::wrap_phase(last_output_phases[i] + phase_increment);
        let real_part = magnitude * cosf(output_phase);
        let imaginary_part = magnitude * sinf(output_phase);
        full_spectrum[i] = microfft::Complex32 { re: real_part, im: imaginary_part };
        if i > 0 && i < HALF_SIZE {
            full_spectrum[512 - i] = microfft::Complex32 { re: real_part, im: -imaginary_part };
        }
        last_output_phases[i] = output_phase;
    }

    // Inverse FFT
    let time_domain_result = microfft::inverse::ifft_512(&mut full_spectrum);
    let mut output_samples = [0.0f32; 512];

    for i in 0..512 {
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

/// Specialized autotune function for 1024-point FFT
pub fn process_vocal_effects_1024(
    unwrapped_buffer: &mut [f32; 1024],
    last_input_phases: &mut [f32; 1024],
    last_output_phases: &mut [f32; 1024],
    previous_pitch_shift_ratio: f32,
    _config: &AutotuneConfig,
    settings: &MusicalSettings,
    sample_rate: f32,
    hop_ratio: f32,
) -> [f32; 1024] {
    const FFT_SIZE: usize = 1024;
    const HALF_SIZE: usize = 512;
    const GAIN_COMPENSATION: f32 = 2.0 / 3.0;

    let hop_size = (FFT_SIZE as f32 * hop_ratio) as usize;
    let bin_width = sample_rate / FFT_SIZE as f32;

    let analysis_window_buffer = crate::hann_window::get_hann_window::<1024>();
    let mut full_spectrum = [microfft::Complex32 { re: 0.0, im: 0.0 }; 1024];
    let mut analysis_magnitudes = [0.0; 512];
    let mut analysis_frequencies = [0.0; 512];
    let mut synthesis_magnitudes = [0.0; 1024];
    let mut synthesis_frequencies = [0.0; 1024];
    let mut envelope = [1.0f32; 512];

    let formant = settings.formant;

    // Apply windowing
    for i in 0..1024 {
        unwrapped_buffer[i] *= analysis_window_buffer[i];
    }

    // Forward FFT
    let fft_result = microfft::real::rfft_1024(unwrapped_buffer);

    // Process frequency bins
    for i in 0..fft_result.len() {
        let amplitude =
            sqrtf(fft_result[i].re * fft_result[i].re + fft_result[i].im * fft_result[i].im);
        let phase = atan2f(fft_result[i].im, fft_result[i].re);
        let mut phase_diff = phase - last_input_phases[i];
        let bin_centre_frequency = 2.0 * PI * i as f32 / 1024.0;
        phase_diff = crate::process_frequencies::wrap_phase(
            phase_diff - bin_centre_frequency * hop_size as f32,
        );
        let bin_deviation = phase_diff * 1024.0 / hop_size as f32 / (2.0 * PI);
        analysis_frequencies[i] = i as f32 + bin_deviation;
        analysis_magnitudes[i] = amplitude;
        last_input_phases[i] = phase;
    }

    // Extract formant envelope if needed
    if formant != 0 {
        extract_cepstral_envelope_1024(&analysis_magnitudes, &mut envelope);
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

    for i in 0..HALF_SIZE {
        if analysis_magnitudes[i] <= 1e-8 {
            continue;
        }
        let residual = if use_formants {
            analysis_magnitudes[i] / envelope[i].max(1e-6_f32)
        } else {
            analysis_magnitudes[i]
        };
        let new_bin_f = i as f32 * pitch_shift_ratio;
        let new_bin = (floorf(new_bin_f + 0.5) as usize).min(HALF_SIZE - 1);
        if new_bin >= HALF_SIZE {
            continue;
        }

        let shifted_envelope = if use_formants {
            let env_pos = (i as f32 / formant_ratio).clamp(0.0, (HALF_SIZE - 1) as f32);
            let env_idx = env_pos as usize;
            let frac = env_pos - env_idx as f32;
            if env_idx < HALF_SIZE - 1 {
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
    for i in 0..HALF_SIZE {
        let magnitude = synthesis_magnitudes[i];
        let bin_deviation = synthesis_frequencies[i] - i as f32;
        let mut phase_increment = bin_deviation * 2.0 * PI * hop_size as f32 / 1024.0;
        let bin_center_frequency = 2.0 * PI * i as f32 / 1024.0;
        phase_increment += bin_center_frequency * hop_size as f32;
        let output_phase =
            crate::process_frequencies::wrap_phase(last_output_phases[i] + phase_increment);
        let real_part = magnitude * cosf(output_phase);
        let imaginary_part = magnitude * sinf(output_phase);
        full_spectrum[i] = microfft::Complex32 { re: real_part, im: imaginary_part };
        if i > 0 && i < HALF_SIZE {
            full_spectrum[1024 - i] = microfft::Complex32 { re: real_part, im: -imaginary_part };
        }
        last_output_phases[i] = output_phase;
    }

    // Inverse FFT
    let time_domain_result = microfft::inverse::ifft_1024(&mut full_spectrum);
    let mut output_samples = [0.0f32; 1024];

    for i in 0..1024 {
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

/// Specialized autotune function for 2048-point FFT
pub fn process_vocal_effects_2048(
    unwrapped_buffer: &mut [f32; 2048],
    last_input_phases: &mut [f32; 2048],
    last_output_phases: &mut [f32; 2048],
    previous_pitch_shift_ratio: f32,
    _config: &AutotuneConfig,
    settings: &MusicalSettings,
    sample_rate: f32,
    hop_ratio: f32,
) -> [f32; 2048] {
    const FFT_SIZE: usize = 2048;
    const HALF_SIZE: usize = 1024;
    const GAIN_COMPENSATION: f32 = 2.0 / 3.0;

    let hop_size = (FFT_SIZE as f32 * hop_ratio) as usize;
    let bin_width = sample_rate / FFT_SIZE as f32;

    let analysis_window_buffer = crate::hann_window::get_hann_window::<2048>();
    let mut full_spectrum = [microfft::Complex32 { re: 0.0, im: 0.0 }; 2048];
    let mut analysis_magnitudes = [0.0; 1024];
    let mut analysis_frequencies = [0.0; 1024];
    let mut synthesis_magnitudes = [0.0; 2048];
    let mut synthesis_frequencies = [0.0; 2048];
    let mut envelope = [1.0f32; 1024];

    let formant = settings.formant;

    // Apply windowing
    for i in 0..2048 {
        unwrapped_buffer[i] *= analysis_window_buffer[i];
    }

    // Forward FFT
    let fft_result = microfft::real::rfft_2048(unwrapped_buffer);

    // Process frequency bins
    for i in 0..fft_result.len() {
        let amplitude =
            sqrtf(fft_result[i].re * fft_result[i].re + fft_result[i].im * fft_result[i].im);
        let phase = atan2f(fft_result[i].im, fft_result[i].re);
        let mut phase_diff = phase - last_input_phases[i];
        let bin_centre_frequency = 2.0 * PI * i as f32 / 2048.0;
        phase_diff = crate::process_frequencies::wrap_phase(
            phase_diff - bin_centre_frequency * hop_size as f32,
        );
        let bin_deviation = phase_diff * 2048.0 / hop_size as f32 / (2.0 * PI);
        analysis_frequencies[i] = i as f32 + bin_deviation;
        analysis_magnitudes[i] = amplitude;
        last_input_phases[i] = phase;
    }

    // Extract formant envelope if needed
    if formant != 0 {
        extract_cepstral_envelope_2048(&analysis_magnitudes, &mut envelope);
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

    for i in 0..HALF_SIZE {
        if analysis_magnitudes[i] <= 1e-8 {
            continue;
        }
        let residual = if use_formants {
            analysis_magnitudes[i] / envelope[i].max(1e-6_f32)
        } else {
            analysis_magnitudes[i]
        };
        let new_bin_f = i as f32 * pitch_shift_ratio;
        let new_bin = (floorf(new_bin_f + 0.5) as usize).min(HALF_SIZE - 1);
        if new_bin >= HALF_SIZE {
            continue;
        }

        let shifted_envelope = if use_formants {
            let env_pos = (i as f32 / formant_ratio).clamp(0.0, (HALF_SIZE - 1) as f32);
            let env_idx = env_pos as usize;
            let frac = env_pos - env_idx as f32;
            if env_idx < HALF_SIZE - 1 {
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
    for i in 0..HALF_SIZE {
        let magnitude = synthesis_magnitudes[i];
        let bin_deviation = synthesis_frequencies[i] - i as f32;
        let mut phase_increment = bin_deviation * 2.0 * PI * hop_size as f32 / 2048.0;
        let bin_center_frequency = 2.0 * PI * i as f32 / 2048.0;
        phase_increment += bin_center_frequency * hop_size as f32;
        let output_phase =
            crate::process_frequencies::wrap_phase(last_output_phases[i] + phase_increment);
        let real_part = magnitude * cosf(output_phase);
        let imaginary_part = magnitude * sinf(output_phase);
        full_spectrum[i] = microfft::Complex32 { re: real_part, im: imaginary_part };
        if i > 0 && i < HALF_SIZE {
            full_spectrum[2048 - i] = microfft::Complex32 { re: real_part, im: -imaginary_part };
        }
        last_output_phases[i] = output_phase;
    }

    // Inverse FFT
    let time_domain_result = microfft::inverse::ifft_2048(&mut full_spectrum);
    let mut output_samples = [0.0f32; 2048];

    for i in 0..2048 {
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

/// Specialized autotune function for 4096-point FFT
pub fn process_vocal_effects_4096(
    unwrapped_buffer: &mut [f32; 4096],
    last_input_phases: &mut [f32; 4096],
    last_output_phases: &mut [f32; 4096],
    previous_pitch_shift_ratio: f32,
    _config: &AutotuneConfig,
    settings: &MusicalSettings,
    sample_rate: f32,
    hop_ratio: f32,
) -> [f32; 4096] {
    const FFT_SIZE: usize = 4096;
    const HALF_SIZE: usize = 2048;
    const GAIN_COMPENSATION: f32 = 2.0 / 3.0;

    let hop_size = (FFT_SIZE as f32 * hop_ratio) as usize;
    let bin_width = sample_rate / FFT_SIZE as f32;

    let analysis_window_buffer = crate::hann_window::get_hann_window::<4096>();
    let mut full_spectrum = [microfft::Complex32 { re: 0.0, im: 0.0 }; 4096];
    let mut analysis_magnitudes = [0.0; 2048];
    let mut analysis_frequencies = [0.0; 2048];
    let mut synthesis_magnitudes = [0.0; 4096];
    let mut synthesis_frequencies = [0.0; 4096];
    let mut envelope = [1.0f32; 2048];

    let formant = settings.formant;

    // Apply windowing
    for i in 0..4096 {
        unwrapped_buffer[i] *= analysis_window_buffer[i];
    }

    // Forward FFT
    let fft_result = microfft::real::rfft_4096(unwrapped_buffer);

    // Process frequency bins
    for i in 0..fft_result.len() {
        let amplitude =
            sqrtf(fft_result[i].re * fft_result[i].re + fft_result[i].im * fft_result[i].im);
        let phase = atan2f(fft_result[i].im, fft_result[i].re);
        let mut phase_diff = phase - last_input_phases[i];
        let bin_centre_frequency = 2.0 * PI * i as f32 / 4096.0;
        phase_diff = crate::process_frequencies::wrap_phase(
            phase_diff - bin_centre_frequency * hop_size as f32,
        );
        let bin_deviation = phase_diff * 4096.0 / hop_size as f32 / (2.0 * PI);
        analysis_frequencies[i] = i as f32 + bin_deviation;
        analysis_magnitudes[i] = amplitude;
        last_input_phases[i] = phase;
    }

    // Extract formant envelope if needed
    if formant != 0 {
        extract_cepstral_envelope_4096(&analysis_magnitudes, &mut envelope);
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

    for i in 0..HALF_SIZE {
        if analysis_magnitudes[i] <= 1e-8 {
            continue;
        }
        let residual = if use_formants {
            analysis_magnitudes[i] / envelope[i].max(1e-6_f32)
        } else {
            analysis_magnitudes[i]
        };
        let new_bin_f = i as f32 * pitch_shift_ratio;
        let new_bin = (floorf(new_bin_f + 0.5) as usize).min(HALF_SIZE - 1);
        if new_bin >= HALF_SIZE {
            continue;
        }

        let shifted_envelope = if use_formants {
            let env_pos = (i as f32 / formant_ratio).clamp(0.0, (HALF_SIZE - 1) as f32);
            let env_idx = env_pos as usize;
            let frac = env_pos - env_idx as f32;
            if env_idx < HALF_SIZE - 1 {
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
    for i in 0..HALF_SIZE {
        let magnitude = synthesis_magnitudes[i];
        let bin_deviation = synthesis_frequencies[i] - i as f32;
        let mut phase_increment = bin_deviation * 2.0 * PI * hop_size as f32 / 4096.0;
        let bin_center_frequency = 2.0 * PI * i as f32 / 4096.0;
        phase_increment += bin_center_frequency * hop_size as f32;
        let output_phase =
            crate::process_frequencies::wrap_phase(last_output_phases[i] + phase_increment);
        let real_part = magnitude * cosf(output_phase);
        let imaginary_part = magnitude * sinf(output_phase);
        full_spectrum[i] = microfft::Complex32 { re: real_part, im: imaginary_part };
        if i > 0 && i < HALF_SIZE {
            full_spectrum[4096 - i] = microfft::Complex32 { re: real_part, im: -imaginary_part };
        }
        last_output_phases[i] = output_phase;
    }

    // Inverse FFT
    let time_domain_result = microfft::inverse::ifft_4096(&mut full_spectrum);
    let mut output_samples = [0.0f32; 4096];

    for i in 0..4096 {
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
