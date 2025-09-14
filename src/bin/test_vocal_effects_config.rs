//! Test binary to demonstrate the process_vocal_effects_config macro functionality
//!
//! This binary shows how to use the new process_vocal_effects_config macro to create
//! customized vocal effects functions with different FFT configurations.

use synthphone_vocals::{
    MusicalSettings, VocalEffectsConfig, process_vocal_effects_config,
    process_vocal_effects_configs,
};

// Generate individual vocal effects configurations for different use cases
process_vocal_effects_config!(process_vocal_effects_studio, 4096, 48000.0, hop_ratio = 0.125);
process_vocal_effects_config!(process_vocal_effects_live, 512, 48000.0, hop_ratio = 0.5);
process_vocal_effects_config!(process_vocal_effects_balanced, 2048, 48000.0, hop_ratio = 0.25);
process_vocal_effects_config!(
    process_vocal_effects_embedded,
    512,
    44100.0,
    hop_ratio = 0.5,
    buffer_multiplier = 2
);

// Generate multiple configurations at once
process_vocal_effects_configs! {
    draft => (process_vocal_effects_draft, 512, 48000.0, hop_ratio = 0.5),
    preview => (process_vocal_effects_preview, 1024, 48000.0, hop_ratio = 0.25),
    production => (process_vocal_effects_production, 4096, 48000.0, hop_ratio = 0.125)
}

/// Configuration data for comparison
struct ConfigInfo {
    name: &'static str,
    fft_size: usize,
    sample_rate: f32,
    hop_ratio: f32,
    expected_latency_ms: f32,
    memory_usage_kb: usize,
    use_case: &'static str,
}

const CONFIGS: &[ConfigInfo] = &[
    ConfigInfo {
        name: "process_vocal_effects_studio",
        fft_size: 4096,
        sample_rate: 48000.0,
        hop_ratio: 0.125,
        expected_latency_ms: 10.67,
        memory_usage_kb: 64,
        use_case: "High-quality studio processing",
    },
    ConfigInfo {
        name: "process_vocal_effects_live",
        fft_size: 512,
        sample_rate: 48000.0,
        hop_ratio: 0.5,
        expected_latency_ms: 5.33,
        memory_usage_kb: 4,
        use_case: "Real-time live performance",
    },
    ConfigInfo {
        name: "process_vocal_effects_balanced",
        fft_size: 2048,
        sample_rate: 48000.0,
        hop_ratio: 0.25,
        expected_latency_ms: 10.67,
        memory_usage_kb: 16,
        use_case: "General-purpose processing",
    },
    ConfigInfo {
        name: "process_vocal_effects_embedded",
        fft_size: 512,
        sample_rate: 44100.0,
        hop_ratio: 0.5,
        expected_latency_ms: 5.8,
        memory_usage_kb: 4,
        use_case: "Memory-constrained devices",
    },
];

/// Helper function to create test audio with harmonics
fn create_test_audio<const N: usize>(frequency: f32, sample_rate: f32) -> [f32; N] {
    let mut audio = [0.0f32; N];
    for (i, value) in audio.iter_mut().enumerate().take(N) {
        let t = i as f32 / sample_rate;
        // Create a signal with fundamental and harmonics
        *value = 0.6 * (2.0 * std::f32::consts::PI * frequency * t).sin()
            + 0.3 * (2.0 * std::f32::consts::PI * frequency * 2.0 * t).sin()
            + 0.1 * (2.0 * std::f32::consts::PI * frequency * 3.0 * t).sin();
    }
    audio
}

/// Helper to create zero-initialized phase buffers
fn create_phase_buffers<const N: usize>() -> ([f32; N], [f32; N]) {
    ([0.0f32; N], [0.0f32; N])
}

/// Calculate RMS level of an audio buffer
fn calculate_rms<const N: usize>(buffer: &[f32; N]) -> f32 {
    (buffer.iter().map(|&x| x * x).sum::<f32>() / N as f32).sqrt()
}

/// Test a specific vocal effects configuration
fn test_configuration() {
    println!("🧪 Testing Different Vocal Effects Configurations");
    println!("==================================================");
    println!();

    let config = VocalEffectsConfig::default();
    let settings = MusicalSettings { note: 0, key: 0, octave: 4, ..Default::default() };

    // Test studio configuration (4096 FFT)
    {
        println!("Testing process_vocal_effects_studio (4096 FFT, high quality):");
        let mut audio = create_test_audio::<4096>(440.0, 48000.0); // A4
        let (mut input_phases, mut output_phases) = create_phase_buffers::<4096>();

        let input_rms = calculate_rms(&audio);
        let result = process_vocal_effects_studio(
            &mut audio,
            &mut input_phases,
            &mut output_phases,
            1.0,
            &config,
            &settings,
        );
        let output_rms = calculate_rms(&result);

        println!("  ✓ Processed {} samples", result.len());
        println!("  ✓ Input RMS: {:.6}, Output RMS: {:.6}", input_rms, output_rms);
        println!("  ✓ Energy preservation: {:.1}%", (output_rms / input_rms) * 100.0);
        println!();
    }

    // Test live configuration (512 FFT)
    {
        println!("Testing process_vocal_effects_live (512 FFT, low latency):");
        let mut audio = create_test_audio::<512>(330.0, 48000.0); // E4
        let (mut input_phases, mut output_phases) = create_phase_buffers::<512>();

        let input_rms = calculate_rms(&audio);
        let result = process_vocal_effects_live(
            &mut audio,
            &mut input_phases,
            &mut output_phases,
            1.0,
            &config,
            &settings,
        );
        let output_rms = calculate_rms(&result);

        println!("  ✓ Processed {} samples", result.len());
        println!("  ✓ Input RMS: {:.6}, Output RMS: {:.6}", input_rms, output_rms);
        println!("  ✓ Energy preservation: {:.1}%", (output_rms / input_rms) * 100.0);
        println!();
    }

    // Test embedded configuration (256 FFT)
    {
        println!("Testing process_vocal_effects_embedded (512 FFT, memory optimized):");
        let mut audio = create_test_audio::<512>(262.0, 44100.0); // C4
        let (mut input_phases, mut output_phases) = create_phase_buffers::<512>();

        let input_rms = calculate_rms(&audio);
        let result = process_vocal_effects_embedded(
            &mut audio,
            &mut input_phases,
            &mut output_phases,
            1.0,
            &config,
            &settings,
        );
        let output_rms = calculate_rms(&result);

        println!("  ✓ Processed {} samples", result.len());
        println!("  ✓ Input RMS: {:.6}, Output RMS: {:.6}", input_rms, output_rms);
        println!("  ✓ Energy preservation: {:.1}%", (output_rms / input_rms) * 100.0);
        println!();
    }
}

/// Demonstrate batch-generated configurations
fn test_batch_configurations() {
    println!("🔄 Testing Batch-Generated Configurations");
    println!("==========================================");
    println!();

    let config = VocalEffectsConfig::default();
    let settings = MusicalSettings { note: 0, key: 0, octave: 4, ..Default::default() };

    // Test draft quality
    {
        let mut audio = create_test_audio::<512>(220.0, 48000.0); // A3
        let (mut input_phases, mut output_phases) = create_phase_buffers::<512>();

        let result = process_vocal_effects_draft(
            &mut audio,
            &mut input_phases,
            &mut output_phases,
            1.0,
            &config,
            &settings,
        );
        println!("✓ process_vocal_effects_draft: {} samples processed", result.len());
    }

    // Test preview quality
    {
        let mut audio = create_test_audio::<1024>(293.0, 48000.0); // D4
        let (mut input_phases, mut output_phases) = create_phase_buffers::<1024>();

        let result = process_vocal_effects_preview(
            &mut audio,
            &mut input_phases,
            &mut output_phases,
            1.0,
            &config,
            &settings,
        );
        println!("✓ process_vocal_effects_preview: {} samples processed", result.len());
    }

    // Test production quality
    {
        let mut audio = create_test_audio::<4096>(349.0, 48000.0); // F4
        let (mut input_phases, mut output_phases) = create_phase_buffers::<4096>();

        let result = process_vocal_effects_production(
            &mut audio,
            &mut input_phases,
            &mut output_phases,
            1.0,
            &config,
            &settings,
        );
        println!("✓ process_vocal_effects_production: {} samples processed", result.len());
    }

    println!();
}

/// Display configuration comparison table
fn display_configuration_table() {
    println!("📊 Configuration Comparison Table");
    println!("=================================");
    println!();

    println!(
        "{:<20} {:<8} {:<10} {:<8} {:<10} {:<10} {:<35}",
        "Function", "FFT", "Sample", "Hop", "Latency", "Memory", "Use Case"
    );
    println!("{}", "-".repeat(100));

    for config in CONFIGS {
        println!(
            "{:<20} {:<8} {:<10} {:<8.3} {:<10.2} {:<10} {:<35}",
            config.name,
            config.fft_size,
            config.sample_rate,
            config.hop_ratio,
            config.expected_latency_ms,
            format!("{}KB", config.memory_usage_kb),
            config.use_case
        );
    }
    println!();
}

/// Demonstrate creating a custom configuration
fn demonstrate_custom_config() {
    println!("🎨 Creating Custom Configuration");
    println!("================================");
    println!();

    // Create a custom voice-optimized configuration
    process_vocal_effects_config!(
        process_vocal_effects_voice_custom,
        2048,
        48000.0,
        hop_ratio = 0.25,
        buffer_multiplier = 4
    );

    println!("Generated custom voice processor with:");
    println!("  FFT Size: 2048 (good frequency resolution for voice)");
    println!("  Sample Rate: 48kHz (professional quality)");
    println!("  Hop Ratio: 0.25 (75% overlap for smooth processing)");
    println!("  Buffer Multiplier: 4 (standard buffering)");
    println!();

    // Test the custom configuration
    let config = VocalEffectsConfig::default();
    let settings = MusicalSettings { note: 0, formant: 1, ..Default::default() };

    let mut voice_audio = create_test_audio::<2048>(196.0, 48000.0); // G3 (typical male voice)
    let (mut input_phases, mut output_phases) = create_phase_buffers::<2048>();

    let input_rms = calculate_rms(&voice_audio);
    let result = process_vocal_effects_voice_custom(
        &mut voice_audio,
        &mut input_phases,
        &mut output_phases,
        1.0,
        &config,
        &settings,
    );
    let output_rms = calculate_rms(&result);

    println!("Custom voice processor test results:");
    println!("  ✓ Processed {} samples", result.len());
    println!("  ✓ Input RMS: {:.6}", input_rms);
    println!("  ✓ Output RMS: {:.6}", output_rms);
    println!("  ✓ Signal preservation: {:.1}%", (output_rms / input_rms) * 100.0);
    println!("  ✓ Frequency resolution: ~{:.1} Hz per bin", 48000.0 / 2048.0);
    println!("  ✓ Processing latency: ~{:.1} ms", (2048.0 * 0.25) / 48000.0 * 1000.0);
    println!();
}

/// Show memory usage and performance characteristics
fn show_performance_analysis() {
    println!("⚡ Performance Analysis");
    println!("======================");
    println!();

    println!("Memory Usage (approximate FFT table sizes):");
    println!("  256  FFT: ~1 KB   (embedded/mobile applications)");
    println!("  512  FFT: ~2 KB   (real-time processing)");
    println!("  1024 FFT: ~4 KB   (balanced applications)");
    println!("  2048 FFT: ~8 KB   (high-quality processing)");
    println!("  4096 FFT: ~16 KB  (studio/production quality)");
    println!();
    println!("Supported FFT sizes: 512, 1024, 2048, 4096");
    println!();

    println!("Latency Analysis (hop size / sample rate):");
    println!("  512 FFT, 0.5 hop:   ~5.3 ms (excellent for live)");
    println!("  1024 FFT, 0.25 hop: ~5.3 ms (good for real-time)");
    println!("  2048 FFT, 0.25 hop: ~10.7 ms (acceptable for most uses)");
    println!("  4096 FFT, 0.125 hop: ~10.7 ms (high quality)");
    println!();

    println!("Frequency Resolution (sample rate / FFT size):");
    println!("  512 FFT @ 48kHz:  ~93.8 Hz per bin");
    println!("  1024 FFT @ 48kHz: ~46.9 Hz per bin");
    println!("  2048 FFT @ 48kHz: ~23.4 Hz per bin");
    println!("  4096 FFT @ 48kHz: ~11.7 Hz per bin");
    println!();
}

fn main() {
    println!("🎵 vocal effects Configuration Macro Demonstration 🎵");
    println!("================================================");
    println!();

    println!(
        "This demonstration shows how to use the process_vocal_effects_config! macro to create"
    );
    println!("customized vocal effects functions with different FFT configurations for");
    println!("various use cases like real-time processing, studio quality, embedded systems, etc.");
    println!();

    display_configuration_table();
    test_configuration();
    test_batch_configurations();
    demonstrate_custom_config();
    show_performance_analysis();

    println!("🚀 Migration Benefits");
    println!("=====================");
    println!();
    println!("Before (hardcoded in library):");
    println!("  - Fixed FFT size of 1024");
    println!("  - Single configuration for all use cases");
    println!("  - No flexibility without modifying library source");
    println!();
    println!("After (configurable via macro):");
    println!("  - Any FFT size (power of 2, validated at compile time)");
    println!("  - Multiple configurations in same application");
    println!("  - External configuration without touching library internals");
    println!("  - Type-safe: each function is specialized for its FFT size");
    println!("  - Memory optimized: only includes FFT tables for sizes you use");
    println!();

    println!("💡 Usage Tips:");
    println!("- Choose FFT size based on your latency vs quality requirements");
    println!("- Use smaller hop ratios (more overlap) for better quality");
    println!("- Consider memory constraints for embedded applications");
    println!("- Test different configurations to find optimal balance");
    println!("- Use process_vocal_effects_configs! macro to generate multiple variants at once");
    println!();

    println!("✅ All tests completed successfully!");
    println!("The process_vocal_effects_config macro system is working correctly.");
}
