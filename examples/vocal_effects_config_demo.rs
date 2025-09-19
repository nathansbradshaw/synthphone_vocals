//! Example: Using the process_vocal_effects_config macro
//!
//! This example demonstrates how to use the `process_vocal_effects_config!` macro to create
//! custom vocal effects functions with different FFT configurations for various use cases.

use synthphone_vocals::{
    MusicalSettings, ProcessingMode, VocalEffectsConfig, process_vocal_effects_config,
    process_vocal_effects_configs,
};

// Generate individual vocal effects configurations for different use cases

// Real-time vocal effects with minimal latency (10.7ms @ 48kHz)
// Good for live performance where latency is critical
process_vocal_effects_config!(process_vocal_effects_realtime, 512, 48000.0, hop_ratio = 0.5);

// Balanced vocal effects for general use (21.3ms @ 48kHz)
// Good compromise between quality and performance
process_vocal_effects_config!(process_vocal_effects_balanced, 1024, 48000.0, hop_ratio = 0.25);

// High-quality vocal effects for studio work (42.7ms @ 48kHz)
// Better frequency resolution and quality
process_vocal_effects_config!(process_vocal_effects_studio, 2048, 48000.0, hop_ratio = 0.125);

// Ultra high-quality for post-processing (85.3ms @ 48kHz)
// Maximum quality for offline processing
process_vocal_effects_config!(process_vocal_effects_hifi, 4096, 48000.0, hop_ratio = 0.0625);

// Generate multiple configurations at once using the convenience macro
process_vocal_effects_configs! {
    draft => (process_vocal_effects_draft, 512, 44100.0, hop_ratio = 0.5),
    preview => (process_vocal_effects_preview, 1024, 44100.0, hop_ratio = 0.25),
    production => (process_vocal_effects_production, 2048, 44100.0, hop_ratio = 0.125),
}

fn main() {
    println!("vocal effects Configuration Examples");
    println!("===============================");

    // Example 1: Real-time processing
    println!("\n1. Real-time vocal effects (512 FFT, 50% hop):");
    demo_process_vocal_effects_function("Real-time", process_vocal_effects_realtime);

    // Example 2: Balanced processing
    println!("\n2. Balanced vocal effects (1024 FFT, 25% hop):");
    demo_process_vocal_effects_function("Balanced", process_vocal_effects_balanced);

    // Example 3: Studio quality
    println!("\n3. Studio vocal effects (2048 FFT, 12.5% hop):");
    demo_process_vocal_effects_function("Studio", process_vocal_effects_studio);

    // Example 4: High-fidelity processing
    println!("\n4. Hi-fi vocal effects (4096 FFT, 6.25% hop):");
    demo_process_vocal_effects_function("Hi-fi", process_vocal_effects_hifi);

    // Example 5: Using generated batch configurations
    println!("\n5. Batch-generated configurations:");
    demo_process_vocal_effects_function("Draft", process_vocal_effects_draft);
    demo_process_vocal_effects_function("Preview", process_vocal_effects_preview);
    demo_process_vocal_effects_function("Production", process_vocal_effects_production);

    // Example 6: Different musical settings
    println!("\n6. Different musical settings:");
    demo_different_settings();

    println!("\nConfiguration Guidelines:");
    print_configuration_guide();
}

/// Demonstrates using a specific vocal effects function
#[allow(clippy::type_complexity)]
fn demo_process_vocal_effects_function<const N: usize>(
    name: &str,
    process_vocal_effects_func: fn(
        &mut [f32; N],
        Option<&mut [f32; N]>,
        &mut [f32; N],
        &mut [f32; N],
        f32,
        &VocalEffectsConfig,
        &MusicalSettings,
    ) -> [f32; N],
) {
    // Create test buffers
    let mut audio_buffer = [0.0f32; N];
    let mut input_phases = [0.0f32; N];
    let mut output_phases = [0.0f32; N];

    // Fill buffer with a simple sine wave pattern for demo
    for (i, sample) in audio_buffer.iter_mut().enumerate() {
        let t = i as f32 / N as f32;
        *sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.1;
    }

    // Create configuration
    let config = VocalEffectsConfig::default();
    let settings =
        synthphone_vocals::MusicalSettings { key: 0, note: 0, formant: 0, ..Default::default() };

    // Process audio
    let result = process_vocal_effects_func(
        &mut audio_buffer,
        None,
        &mut input_phases,
        &mut output_phases,
        1.0, // Previous pitch shift ratio
        &config,
        &settings,
    );

    // Analyze results
    let input_rms = calculate_rms(&audio_buffer);
    let output_rms = calculate_rms(&result);

    println!(
        "  {} - FFT Size: {}, Input RMS: {:.4}, Output RMS: {:.4}",
        name, N, input_rms, output_rms
    );

    // Verify output is valid
    if result.iter().all(|x| x.is_finite()) {
        println!("  ✓ Output is valid (all finite values)");
    } else {
        println!("  ✗ Output contains invalid values");
    }
}

/// Demonstrates different musical settings
fn demo_different_settings() {
    let mut buffer = [0.0f32; 1024];
    let mut input_phases = [0.0f32; 1024];
    let mut output_phases = [0.0f32; 1024];

    // Fill with test signal
    for (i, sample) in buffer.iter_mut().enumerate() {
        let t = i as f32 / 1024.0;
        *sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.1;
    }

    let config = VocalEffectsConfig::default();

    // Auto mode (snap to nearest note in key)
    let settings_auto =
        synthphone_vocals::MusicalSettings { key: 0, note: 0, formant: 0, ..Default::default() };

    let result_auto = process_vocal_effects_balanced(
        &mut buffer.clone(),
        None,
        &mut input_phases.clone(),
        &mut output_phases.clone(),
        1.0,
        &config,
        &settings_auto,
    );

    // Manual mode (specific note)
    let settings_manual = synthphone_vocals::MusicalSettings {
        key: 0,
        note: 1,
        octave: 4,
        formant: 1,
        mode: ProcessingMode::Autotune,
    };

    let result_manual = process_vocal_effects_balanced(
        &mut buffer,
        None,
        &mut input_phases,
        &mut output_phases,
        1.0,
        &config,
        &settings_manual,
    );

    println!("  Auto mode RMS: {:.4}", calculate_rms(&result_auto));
    println!("  Manual mode RMS: {:.4}", calculate_rms(&result_manual));
}

/// Calculate RMS (Root Mean Square) of a signal
fn calculate_rms(signal: &[f32]) -> f32 {
    let sum_squares: f32 = signal.iter().map(|&x| x * x).sum();
    (sum_squares / signal.len() as f32).sqrt()
}

/// Print configuration guidelines
fn print_configuration_guide() {
    println!();
    println!("FFT Size Guidelines:");
    println!("  512   - Ultra-low latency (10.7ms @ 48kHz), minimal CPU, basic quality");
    println!("  1024  - Low latency (21.3ms @ 48kHz), good for real-time applications");
    println!("  2048  - Balanced (42.7ms @ 48kHz), good quality/performance trade-off");
    println!("  4096  - High quality (85.3ms @ 48kHz), best for offline processing");
    println!();
    println!("Hop Ratio Guidelines:");
    println!("  0.0625 (1/16) - Highest quality, 93.75% overlap, most CPU intensive");
    println!("  0.125  (1/8)  - Very high quality, 87.5% overlap, high CPU usage");
    println!("  0.25   (1/4)  - Good quality, 75% overlap, moderate CPU usage (default)");
    println!("  0.5    (1/2)  - Lower quality, 50% overlap, lowest CPU usage");
    println!();
    println!("Formant Settings:");
    println!("  0 - No formant processing (fastest)");
    println!("  1 - Lower formants (deeper voice effect)");
    println!("  2 - Raise formants (higher voice effect)");
    println!();
    println!("Use Cases:");
    println!("  Live Performance: 512 FFT, 0.5 hop ratio");
    println!("  Real-time Apps:   1024 FFT, 0.25 hop ratio");
    println!("  Studio Recording: 2048 FFT, 0.125 hop ratio");
    println!("  Post-processing:  4096 FFT, 0.0625 hop ratio");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_functions_work() {
        let mut buffer_512 = [0.1f32; 512];
        let mut phases_512 = [0.0f32; 512];
        let config = VocalEffectsConfig::default();
        let settings = MusicalSettings::default();

        let result = process_vocal_effects_realtime(
            &mut buffer_512,
            &mut phases_512.clone(),
            &mut phases_512,
            1.0,
            &config,
            &settings,
        );

        assert!(result.iter().all(|x| x.is_finite()));
    }

    #[test]
    fn test_rms_calculation() {
        let signal = [1.0, 0.0, -1.0, 0.0];
        let rms = calculate_rms(&signal);
        assert!((rms - 0.7071).abs() < 0.001); // sqrt(2)/2 ≈ 0.7071
    }
}
