//! Basic Autotune Example
//!
//! This example demonstrates the basic usage of the synthphone_vocals library
//! for real-time autotune processing. It shows how to configure and use the
//! vocal effects processing functions.

use synthphone_vocals::{AutotuneConfig, MusicalSettings, process_vocal_effects_config};

// Generate an optimized autotune function for real-time processing
process_vocal_effects_config!(
    process_vocals_realtime, // Function name
    1024,                    // FFT size (latency vs quality trade-off)
    48000.0,                 // Sample rate
    hop_ratio = 0.25         // Overlap ratio (quality vs CPU)
);

fn main() {
    println!("🎵 Basic Autotune Example");
    println!("========================");

    // Initialize configuration
    let config = AutotuneConfig::default();
    let mut settings = MusicalSettings::default();

    // Configure for C major scale
    settings.key = 0; // C major
    settings.note = 0; // Auto-detect mode (corrects to nearest note in key)
    settings.formant = 1; // Preserve formants for natural sound

    println!("Configuration:");
    println!("  Key: C Major");
    println!("  Mode: Auto-detect");
    println!("  Formant preservation: Enabled");
    println!("  FFT Size: 1024 samples");
    println!("  Sample Rate: 48000 Hz");
    println!("  Latency: ~21.3ms");
    println!();

    // Simulate processing audio buffers
    println!("Processing audio buffers...");

    // Initialize processing state
    let mut audio_buffer = [0.0f32; 1024];
    let mut input_phases = [0.0f32; 1024];
    let mut output_phases = [0.0f32; 1024];
    let mut pitch_shift_ratio = 1.0f32;

    // Simulate some audio processing cycles
    for frame in 0..10 {
        // Fill audio_buffer with simulated input samples
        // In a real application, this would come from your audio input
        fill_with_test_signal(&mut audio_buffer, frame);

        // Process the audio buffer with autotune
        let processed = process_vocals_realtime(
            &mut audio_buffer,
            &mut input_phases,
            &mut output_phases,
            pitch_shift_ratio,
            &config,
            &settings,
        );

        // In a real application, you would output these samples to your audio device
        let rms = calculate_rms(&processed);
        println!("Frame {}: Processed {} samples, RMS: {:.6}", frame + 1, processed.len(), rms);

        // Update pitch shift ratio based on processing (this would be calculated internally)
        pitch_shift_ratio = 1.0 + (frame as f32 * 0.001); // Simulate slight pitch variations
    }

    println!();
    println!("✅ Processing complete!");

    // Demonstrate different musical keys
    demonstrate_different_keys();

    // Show performance characteristics
    show_performance_info();
}

/// Fill the audio buffer with a test signal (simulated vocal input)
fn fill_with_test_signal(buffer: &mut [f32; 1024], frame: usize) {
    let base_freq = 220.0; // A3
    let sample_rate = 48000.0;

    for (i, sample) in buffer.iter_mut().enumerate() {
        let t = (frame * 1024 + i) as f32 / sample_rate;

        // Simulate a slightly off-pitch vocal with harmonics
        let fundamental = (2.0 * std::f32::consts::PI * base_freq * 1.03 * t).sin(); // 3% sharp
        let harmonic2 = 0.3 * (2.0 * std::f32::consts::PI * base_freq * 2.0 * 1.02 * t).sin();
        let harmonic3 = 0.1 * (2.0 * std::f32::consts::PI * base_freq * 3.0 * 1.01 * t).sin();

        *sample = 0.5 * (fundamental + harmonic2 + harmonic3);
    }
}

/// Calculate RMS (Root Mean Square) of the audio buffer
fn calculate_rms(buffer: &[f32; 1024]) -> f32 {
    let sum_squares: f32 = buffer.iter().map(|x| x * x).sum();
    (sum_squares / buffer.len() as f32).sqrt()
}

/// Demonstrate processing with different musical keys
fn demonstrate_different_keys() {
    println!("🎹 Musical Key Demonstrations");
    println!("============================");

    let config = AutotuneConfig::default();
    let mut settings = MusicalSettings::default();

    let keys = [(0, "C Major"), (7, "G Major"), (12, "A Minor"), (19, "E Minor")];

    for (key_index, key_name) in keys {
        settings.key = key_index;
        println!("Key: {} (index: {})", key_name, key_index);

        // You could process different audio samples here to show
        // how the autotune behavior changes with different keys
    }
    println!();
}

/// Show performance and configuration information
fn show_performance_info() {
    println!("⚡ Performance Information");
    println!("=========================");
    println!("FFT Size: 1024 samples");
    println!("Processing Latency: ~21.3ms @ 48kHz");
    println!("Memory Usage: ~8KB for buffers");
    println!("CPU Usage: Low (suitable for real-time)");
    println!();

    println!("🔧 Alternative Configurations");
    println!("=============================");
    println!("Ultra-low latency: 512 samples (~10.7ms)");
    println!("High quality: 2048 samples (~42.7ms)");
    println!("Maximum quality: 4096 samples (~85.3ms)");
    println!();

    println!("💡 Usage Tips");
    println!("=============");
    println!("- Use FFT size 512-1024 for live performance");
    println!("- Use FFT size 2048-4096 for studio recording");
    println!("- Enable formant preservation for natural sound");
    println!("- Adjust hop_ratio to balance quality vs CPU usage");
    println!("- Test on your target hardware for optimal settings");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_processing() {
        let config = AutotuneConfig::default();
        let settings = MusicalSettings::default();

        let mut audio_buffer = [0.0f32; 1024];
        let mut input_phases = [0.0f32; 1024];
        let mut output_phases = [0.0f32; 1024];

        // Fill with test signal
        fill_with_test_signal(&mut audio_buffer, 0);

        // Process should not panic
        let processed = process_vocals_realtime(
            &mut audio_buffer,
            &mut input_phases,
            &mut output_phases,
            1.0,
            &config,
            &settings,
        );

        // Verify we got valid output
        assert_eq!(processed.len(), 1024);

        // Check that processing produced some output
        let rms = calculate_rms(&processed);
        assert!(rms > 0.0);
    }

    #[test]
    fn test_rms_calculation() {
        let mut buffer = [0.0f32; 1024];

        // Test with silence
        assert_eq!(calculate_rms(&buffer), 0.0);

        // Test with constant value
        buffer.fill(0.5);
        assert!((calculate_rms(&buffer) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_different_keys() {
        let config = AutotuneConfig::default();
        let mut settings = MusicalSettings::default();

        // Test that different keys don't cause panics
        for key in 0..24 {
            settings.key = key;

            let mut audio_buffer = [0.1f32; 1024]; // Simple constant signal
            let mut input_phases = [0.0f32; 1024];
            let mut output_phases = [0.0f32; 1024];

            let _processed = process_vocals_realtime(
                &mut audio_buffer,
                &mut input_phases,
                &mut output_phases,
                1.0,
                &config,
                &settings,
            );

            // Should not panic for any valid key
        }
    }
}
