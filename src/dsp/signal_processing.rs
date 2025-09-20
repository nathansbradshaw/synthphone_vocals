use libm::{expf, logf};

use crate::{MusicalSettings, dsp::FftOps};

/// Extract cepstral envelope for formant preservation using generic FFT operations
pub fn extract_cepstral_envelope<const N: usize, const HALF_N: usize, F>(
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

pub fn calculate_pitch_shift(
    analysis_magnitudes: &[f32],
    analysis_frequencies: &[f32],
    previous_pitch_shift_ratio: f32,
    settings: &MusicalSettings,
    bin_width: f32,
) -> f32 {
    let mut pitch_shift_ratio = previous_pitch_shift_ratio;
    let fundamental_index =
        crate::dsp::frequency_analysis::find_fundamental_frequency(analysis_magnitudes);
    let detected_frequency = analysis_frequencies[fundamental_index] * bin_width;

    if detected_frequency > 0.001 {
        let target_frequency = if settings.note == 0 {
            let scale_frequencies = crate::audio::keys::get_scale_by_key(settings.key);
            crate::audio::frequencies::find_nearest_note_in_key(
                detected_frequency,
                scale_frequencies,
            )
        } else {
            crate::audio::keys::get_frequency(settings.key, settings.note, settings.octave, false)
        };
        let raw_ratio = target_frequency / detected_frequency;
        let clamped_ratio = raw_ratio.clamp(0.5, 2.0);
        const SMOOTHING_FACTOR: f32 = 0.99;
        pitch_shift_ratio = clamped_ratio * SMOOTHING_FACTOR
            + previous_pitch_shift_ratio * (1.0 - SMOOTHING_FACTOR);
    }

    pitch_shift_ratio
}
