use core::f32::consts::PI;

use libm::{fabsf, floorf, fmodf, roundf};

use crate::audio::find_nearest_note_frequency;

#[inline(always)]
pub fn calculate_updates<const N: usize>(
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

    if new_bin < N / 2 {
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
pub fn wrap_phase(phase_in: f32) -> f32 {
    if phase_in >= 0.0 {
        return fmodf(phase_in + PI, 2.0 * PI) - PI;
    }
    fmodf(phase_in - PI, -2.0 * PI) + PI
}

#[cfg(test)]
mod tests {
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
