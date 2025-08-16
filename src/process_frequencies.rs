use core::f32::consts::PI;

use libm::{expf, fabsf, floorf, fmodf, logf, roundf};
use microfft;

use crate::frequencies::find_nearest_note_frequency;

//TODO this should be passed in
const FFT_SIZE: usize = 1024;

#[inline(always)]
pub fn calculate_updates(
    index: usize,
    analysis_frequencies: &[f32],
    analysis_magnitudes: &[f32],
    transition_speed: f32,
) -> Option<(usize, f32, f32)> {
    if index >= analysis_frequencies.len() || index >= analysis_magnitudes.len() {
        return None;
    }
    let exact_frequency = analysis_frequencies[index];
    let target_frequency = find_nearest_note_frequency(exact_frequency);
    let pitch_shift = target_frequency / exact_frequency;

    let new_bin = floorf(index as f32 * pitch_shift + 0.5) as usize;

    if new_bin < FFT_SIZE / 2 {
        let updated_magnitude = transition_speed * analysis_magnitudes[new_bin]
            + (1.0 - transition_speed) * analysis_magnitudes[index];
        let updated_frequency = exact_frequency * pitch_shift;
        Some((new_bin, updated_magnitude, updated_frequency))
    } else {
        None
    }
}

#[inline(always)]
pub fn find_fundamental_frequency(analysis_magnitudes: &[f32]) -> usize {
    let mut max_magnitude = 0.0;
    let mut fundamental_bin = 0;
    for (i, &magnitude) in analysis_magnitudes.iter().enumerate() {
        if magnitude > max_magnitude {
            max_magnitude = magnitude;
            fundamental_bin = i;
        }
    }
    fundamental_bin
}

#[inline(always)]
pub fn collect_harmonics(fundamental_index: usize) -> [usize; 8] {
    let mut harmonics = [0; 8];
    for n in 1..=8 {
        let harmonic_index = fundamental_index * n;
        harmonics[n - 1] = harmonic_index;
    }
    harmonics
}

#[inline(always)]
pub fn sample_rate_reduce(
    sample: f32,
    factor: i32,
    hold_counter: &mut i32,
    held_value: &mut f32,
) -> f32 {
    // If we're at the start of the "hold" cycle, update the held sample
    if *hold_counter == 0 {
        *held_value = sample;
    }
    // Increment the hold_counter (wrapping around "factor")
    //TODO: this can cause a panic if devide by 0

    if factor != 0 {
        *hold_counter = (*hold_counter + 1) % factor;
    }

    // Always return the held_value (which may have just been updated)
    *held_value
}

#[inline(always)]
pub fn bitcrush(sample: f32, bit_depth: u8) -> f32 {
    let levels = (1u64 << bit_depth) as f32;
    // Normalize sample from [-1,1] to [0,1]
    let normalized = (sample + 1.0) / 2.0;
    // Quantize the sample using libm's roundf
    let quantized = roundf(normalized * levels) / levels;
    // Map back to [-1,1]
    quantized * 2.0 - 1.0
}

#[inline(always)]
pub fn normalize_sample(sample: f32, target_peak: f32) -> f32 {
    let abs_sample = fabsf(sample);
    if abs_sample > target_peak {
        // Scale the sample down to target_peak while preserving its sign.
        sample * (target_peak / abs_sample)
    } else {
        sample
    }
}

#[inline(always)]
pub fn cepstral_smoothing(input_magnitude: &[f32; FFT_SIZE]) -> [f32; FFT_SIZE / 2] {
    // Step 1: Compute log magnitude for each FFT bin.
    let mut log_spec: [microfft::Complex32; FFT_SIZE] =
        [microfft::Complex32 { re: 0.0, im: 0.0 }; FFT_SIZE];
    for i in 0..FFT_SIZE {
        // Use libm::logf instead of .ln(), adding a small constant to avoid log(0)
        log_spec[i].re = logf(input_magnitude[i].max(1e-12));
    }

    // Step 2: Compute the cepstrum by doing an inverse FFT on the log magnitude spectrum.
    let mut cepstrum = log_spec; // Copy to a mutable array.
    let _ = microfft::inverse::ifft_1024(&mut cepstrum);

    // Step 3: Low-pass filter the cepstrum.
    let cutoff = 20;
    for i in cutoff..(FFT_SIZE - cutoff) {
        cepstrum[i].re = 0.0;
        cepstrum[i].im = 0.0;
    }

    // Step 4: Transform back to the frequency domain: FFT of the filtered cepstrum.
    let mut filtered_input: [f32; FFT_SIZE] = [0.0; FFT_SIZE];
    for i in 0..FFT_SIZE {
        filtered_input[i] = cepstrum[i].re;
    }
    let smoothed_log_spec = microfft::real::rfft_1024(&mut filtered_input);

    // Step 5: Exponentiate to recover the smoothed magnitude envelope.
    let mut envelope = [0.0f32; FFT_SIZE / 2];
    for i in 0..(FFT_SIZE / 2) {
        envelope[i] = expf(smoothed_log_spec[i].re);
    }

    envelope
}

#[inline(always)]
pub fn wrap_phase(phase_in: f32) -> f32 {
    if phase_in >= 0.0 {
        return fmodf(phase_in + PI, 2.0 * PI) - PI;
    }
    fmodf(phase_in - PI, -2.0 * PI) + PI
}

#[cfg(test)]
mod tests {
    #[cfg(any(feature = "std", not(feature = "embedded")))]
    use alloc::vec;

    use super::*;

    #[test]
    fn test_find_nearest_note_frequency_exact_match() {
        let frequency = 440.0;
        let expected = 440.0;
        let result = find_nearest_note_frequency(frequency);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_find_nearest_note_frequency_in_between() {
        let frequency = 445.0;
        let expected = 440.0;
        let result = find_nearest_note_frequency(frequency);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_find_nearest_note_frequency_below_range() {
        let frequency = 10.0;
        let expected = 16.35;
        let result = find_nearest_note_frequency(frequency);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_find_nearest_note_frequency_above_range() {
        let frequency = 5000.0;
        let result = find_nearest_note_frequency(frequency);
        // Should be close to the highest frequency in our generated scales
        assert!(result > 4900.0 && result < 5100.0);
    }

    #[test]
    fn test_find_nearest_note_frequency_mid_point() {
        let frequency = 55.0;
        let result = find_nearest_note_frequency(frequency);
        // Should be close to 55.0 Hz (A1)
        assert!((result - 55.0).abs() < 0.1);
    }

    #[test]
    fn test_find_nearest_note_frequency_edge_case_low() {
        let frequency = 16.0;
        let result = find_nearest_note_frequency(frequency);
        // Should be close to C0 (16.35 Hz)
        assert!((result - 16.35).abs() < 0.1);
    }

    #[test]
    fn test_find_nearest_note_frequency_edge_case_high() {
        let frequency = 4999.0;
        let result = find_nearest_note_frequency(frequency);
        // Should be close to the highest frequency in our generated scales
        assert!(result > 4900.0 && result < 5100.0);
    }

    #[test]
    fn test_find_nearest_note_frequency_very_close_lower() {
        let frequency = 110.1;
        let expected = 110.0;
        let result = find_nearest_note_frequency(frequency);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_find_nearest_note_frequency_very_close_upper() {
        let frequency = 109.9;
        let expected = 110.0;
        let result = find_nearest_note_frequency(frequency);
        assert_eq!(result, expected);
    }

    #[cfg(any(feature = "std", not(feature = "embedded")))]
    #[test]
    fn test_calculate_updates_within_bounds() {
        let analysis_frequencies = vec![440.0, 880.0, 1760.0];
        let analysis_magnitudes = vec![1.0, 0.5, 0.25];
        let transition_speed = 0.1;

        let result =
            calculate_updates(0, &analysis_frequencies, &analysis_magnitudes, transition_speed);
        assert!(result.is_some());
        let (new_bin, updated_magnitude, updated_frequency) = result.unwrap();
        assert_eq!(new_bin, 0);
        assert!((updated_magnitude - 1.0).abs() < 1e-6);
        assert!((updated_frequency - 440.0).abs() < 1e-6);

        let result =
            calculate_updates(1, &analysis_frequencies, &analysis_magnitudes, transition_speed);
        assert!(result.is_some());
        let (new_bin, updated_magnitude, updated_frequency) = result.unwrap();
        assert_eq!(new_bin, 1);
        assert!((updated_magnitude - 0.5).abs() < 1e-6);
        assert!((updated_frequency - 880.0).abs() < 1e-6);
    }

    #[cfg(any(feature = "std", not(feature = "embedded")))]
    #[test]
    fn test_calculate_updates_out_of_bounds() {
        let analysis_frequencies = vec![440.0, 880.0, 1760.0];
        let analysis_magnitudes = vec![1.0, 0.5, 0.25];
        let transition_speed = 0.1;

        // This index should be out of bounds
        let result = calculate_updates(
            FFT_SIZE / 2,
            &analysis_frequencies,
            &analysis_magnitudes,
            transition_speed,
        );
        assert!(result.is_none());
    }

    #[cfg(any(feature = "std", not(feature = "embedded")))]
    #[test]
    fn test_calculate_updates_with_transition() {
        let analysis_frequencies = vec![440.0, 880.0, 1760.0];
        let analysis_magnitudes = vec![1.0, 0.5, 0.25];
        let transition_speed = 0.5;

        let result =
            calculate_updates(0, &analysis_frequencies, &analysis_magnitudes, transition_speed);
        assert!(result.is_some());
        let (new_bin, updated_magnitude, updated_frequency) = result.unwrap();
        assert_eq!(new_bin, 0);
        assert!((updated_magnitude - 1.0).abs() < 1e-6);
        assert!((updated_frequency - 440.0).abs() < 1e-6);

        let result =
            calculate_updates(1, &analysis_frequencies, &analysis_magnitudes, transition_speed);
        assert!(result.is_some());
        let (new_bin, updated_magnitude, updated_frequency) = result.unwrap();
        assert_eq!(new_bin, 1);
        assert!((updated_magnitude - 0.5).abs() < 1e-6);
        assert!((updated_frequency - 880.0).abs() < 1e-6);
    }
}

#[cfg(test)]
mod detect_fun_freq_tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let analysis_magnitudes: [f32; 0] = [];
        let result = find_fundamental_frequency(&analysis_magnitudes);
        assert_eq!(result, 0, "Empty input should return index 0");
    }

    #[test]
    fn test_single_element() {
        let analysis_magnitudes = [1.0];
        let result = find_fundamental_frequency(&analysis_magnitudes);
        assert_eq!(result, 0, "Single element should return index 0");
    }

    #[test]
    fn test_all_zeros() {
        let analysis_magnitudes = [0.0, 0.0, 0.0];
        let result = find_fundamental_frequency(&analysis_magnitudes);
        assert_eq!(result, 0, "All zeros should return index 0");
    }

    #[test]
    fn test_positive_magnitudes() {
        let analysis_magnitudes = [0.1, 0.5, 0.3, 0.8, 0.2];
        let result = find_fundamental_frequency(&analysis_magnitudes);
        assert_eq!(result, 3, "Maximum magnitude at index 3");
    }

    #[test]
    fn test_mixed_sign_magnitudes() {
        let analysis_magnitudes = [-0.5, 0.2, 0.3, -0.1, 0.4];
        let result = find_fundamental_frequency(&analysis_magnitudes);
        assert_eq!(result, 4, "Maximum magnitude at index 4");
    }

    #[test]
    fn test_multiple_maximums() {
        let analysis_magnitudes = [0.5, 0.5, 0.5];
        let result = find_fundamental_frequency(&analysis_magnitudes);
        assert_eq!(result, 0, "First occurrence of maximum magnitude at index 0");
    }

    #[test]
    fn test_max_at_start() {
        let analysis_magnitudes = [0.9, 0.5, 0.3, 0.8, 0.2];
        let result = find_fundamental_frequency(&analysis_magnitudes);
        assert_eq!(result, 0, "Maximum magnitude at the start index 0");
    }

    #[test]
    fn test_max_at_end() {
        let analysis_magnitudes = [0.1, 0.5, 0.3, 0.8, 1.0];
        let result = find_fundamental_frequency(&analysis_magnitudes);
        assert_eq!(result, 4, "Maximum magnitude at the end index 4");
    }
}
