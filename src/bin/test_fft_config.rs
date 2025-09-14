//! Test binary to demonstrate the FFT configuration macro usage
//!
//! This binary shows how to use the new dynamic FFT configuration system
//! to replace hardcoded constants with flexible, validated configurations.

use synthphone_vocals::{fft_config, fft_config_struct};

// Example 1: Basic configuration matching original embedded.rs
fn print_original_config() {
    use synthphone_vocals::fft_config;

    // This replaces the hardcoded constants from embedded.rs
    fft_config!(1024, 48_014.312);

    println!("=== Original Configuration ===");
    println!("Sample Rate: {} Hz", SAMPLE_RATE);
    println!("FFT Size: {}", FFT_SIZE);
    println!("Buffer Size: {}", BUFFER_SIZE);
    println!("Hop Size: {}", HOP_SIZE);
    println!("Block Size: {}", BLOCK_SIZE);
    println!("Bin Width: {:.2} Hz", BIN_WIDTH);
    println!();
}

// Example 2: High-quality configuration
fn print_high_quality_config() {
    use synthphone_vocals::fft_config;

    fft_config!(4096, 48000.0, hop_ratio = 0.125, buffer_multiplier = 8);

    println!("=== High Quality Configuration ===");
    println!("Sample Rate: {} Hz", SAMPLE_RATE);
    println!("FFT Size: {}", FFT_SIZE);
    println!("Buffer Size: {}", BUFFER_SIZE);
    println!("Hop Size: {} (87.5% overlap)", HOP_SIZE);
    println!("Block Size: {}", BLOCK_SIZE);
    println!("Bin Width: {:.2} Hz", BIN_WIDTH);
    println!("Memory usage: ~{} KB", (FFT_SIZE * 4 + BUFFER_SIZE * 4) / 1024);
    println!();
}

// Example 3: Fast/real-time configuration
fn print_realtime_config() {
    use synthphone_vocals::fft_config;

    fft_config!(512, 48000.0, hop_ratio = 0.5, buffer_multiplier = 2);

    println!("=== Real-time Configuration ===");
    println!("Sample Rate: {} Hz", SAMPLE_RATE);
    println!("FFT Size: {}", FFT_SIZE);
    println!("Buffer Size: {}", BUFFER_SIZE);
    println!("Hop Size: {} (50% overlap)", HOP_SIZE);
    println!("Block Size: {}", BLOCK_SIZE);
    println!("Bin Width: {:.2} Hz", BIN_WIDTH);
    println!("Processing latency: ~{:.2} ms", (HOP_SIZE as f32 / SAMPLE_RATE) * 1000.0);
    println!();
}

// Example 4: Multiple configurations using struct approach
fft_config_struct!(VoiceProcessing, 2048, 48000.0, hop_ratio = 0.25);
fft_config_struct!(MusicAnalysis, 4096, 48000.0, hop_ratio = 0.125);
fft_config_struct!(EmbeddedDevice, 512, 44100.0, hop_ratio = 0.5);

fn print_struct_configs() {
    println!("=== Struct-based Configurations ===");

    println!("Voice Processing Config:");
    println!("  FFT Size: {}", VoiceProcessing::FFT_SIZE);
    println!("  Bin Width: {:.2} Hz", VoiceProcessing::BIN_WIDTH);
    println!("  Hop Size: {}", VoiceProcessing::HOP_SIZE);

    println!("Music Analysis Config:");
    println!("  FFT Size: {}", MusicAnalysis::FFT_SIZE);
    println!("  Bin Width: {:.2} Hz", MusicAnalysis::BIN_WIDTH);
    println!("  Hop Size: {}", MusicAnalysis::HOP_SIZE);

    println!("Embedded Device Config:");
    println!("  FFT Size: {}", EmbeddedDevice::FFT_SIZE);
    println!("  Bin Width: {:.2} Hz", EmbeddedDevice::BIN_WIDTH);
    println!("  Hop Size: {}", EmbeddedDevice::HOP_SIZE);
    println!();
}

fn demonstrate_validation() {
    

    println!("=== Configuration Validation ===");

    let test_cases = [
        (1024, 48000.0, 0.25, "Valid standard config"),
        (2048, 44100.0, 0.125, "Valid high-res config"),
        (1023, 48000.0, 0.25, "Invalid: not power of 2"),
        (1024, -48000.0, 0.25, "Invalid: negative sample rate"),
        (1024, 48000.0, 0.0, "Invalid: zero hop ratio"),
        (1024, 48000.0, 1.5, "Invalid: hop ratio > 1.0"),
        (2, 48000.0, 0.25, "Invalid: FFT size too small"),
        (65536, 48000.0, 0.25, "Invalid: FFT size too large"),
    ];

    for (fft_size, sample_rate, hop_ratio, description) in test_cases {
        match synthphone_vocals::fft_config::validate_config(fft_size, sample_rate, hop_ratio) {
            Ok(()) => println!("✓ {}", description),
            Err(e) => println!("✗ {}: {}", description, e),
        }
    }
    println!();
}

fn suggest_microfft_features() {
    

    println!("=== Microfft Feature Suggestions ===");
    println!("Add these features to your Cargo.toml for optimal memory usage:");
    println!();

    let fft_sizes = [64, 256, 512, 1024, 2048, 4096, 8192];

    for &size in &fft_sizes {
        let feature = synthphone_vocals::fft_config::suggest_microfft_feature(size);
        println!("FFT Size {:>4}: features = [\"{}\"]", size, feature);
    }

    println!();
    println!("Example Cargo.toml configuration:");
    println!("[dependencies.microfft]");
    println!("default-features = false");
    println!("features = [\"size-4096\"]  # Adjust based on your maximum FFT size");
    println!();
}

fn performance_comparison() {
    println!("=== Performance Comparison ===");

    let configs = [
        ("Fast", 512, 0.5),
        ("Balanced", 1024, 0.25),
        ("Quality", 2048, 0.125),
        ("High-Res", 4096, 0.0625),
    ];

    println!(
        "{:<12} {:<8} {:<8} {:<10} {:<12} {:<10}",
        "Config", "FFT", "Hop", "Bin Width", "Latency", "Memory"
    );
    println!("{}", "-".repeat(66));

    for (name, fft_size, hop_ratio) in configs {
        let sample_rate = 48000.0;
        let hop_size = (fft_size as f32 * hop_ratio) as usize;
        let bin_width = sample_rate / fft_size as f32;
        let latency_ms = (hop_size as f32 / sample_rate) * 1000.0;
        let memory_kb = (fft_size * 8) / 1024; // Rough estimate

        println!(
            "{:<12} {:<8} {:<8} {:<10.2} {:<12.2} {:<10}KB",
            name, fft_size, hop_size, bin_width, latency_ms, memory_kb
        );
    }
    println!();
}

fn main() {
    println!("🎵 FFT Configuration Macro Demonstration 🎵");
    println!("===========================================");
    println!();

    // Show different configuration examples
    print_original_config();
    print_high_quality_config();
    print_realtime_config();
    print_struct_configs();

    // Demonstrate validation
    demonstrate_validation();

    // Show microfft feature suggestions
    suggest_microfft_features();

    // Performance comparison
    performance_comparison();

    println!("💡 Key Benefits of the FFT Config Macro:");
    println!("- Compile-time validation ensures FFT sizes are powers of 2");
    println!("- Automatic calculation of related constants (buffer size, hop size, etc.)");
    println!("- Memory optimization suggestions for microfft features");
    println!("- Multiple configurations can coexist in the same project");
    println!("- Runtime validation available for dynamic configurations");
    println!();

    println!("🚀 Migration from hardcoded constants:");
    println!("Before: pub const FFT_SIZE: usize = 1024;");
    println!("        pub const BUFFER_SIZE: usize = FFT_SIZE * 4;");
    println!("        // ... more constants");
    println!();
    println!("After:  fft_config!(1024, 48000.0);");
    println!("        // All constants generated automatically with validation!");
}
