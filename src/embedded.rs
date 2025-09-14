use libm::{expf, fabsf};

use crate::ring_buffer::RingBuffer;

/// Writes synthesized audio samples to the output ring buffer using overlap-add.
///
/// This function is separated from the main synthesis to minimize time spent
/// holding the output buffer lock. It performs the final overlap-add step.
///
/// # Parameters
///
/// * `output_samples` - Processed audio samples from synthesis
/// * `output_ring` - Ring buffer to receive the samples
pub fn write_synthesis_output<const N: usize, const BUFFER_SIZE: usize>(
    output_samples: &[f32; N],
    output_ring: &RingBuffer<BUFFER_SIZE>,
) {
    // Add samples to output buffer using overlap-add (accumulation)
    for (i, sample) in output_samples.iter().enumerate().take(N) {
        output_ring.add_at_offset(i as u32, *sample);
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
