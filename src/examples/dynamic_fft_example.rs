//! Example demonstrating the use of the FFT configuration macro
//!
//! This example shows how to replace hardcoded FFT constants with the
//! dynamic fft_config macro, providing flexibility while maintaining
//! compile-time safety and optimization.

use synthphone_vocals::fft_config;

/// Example 1: Basic usage matching the original embedded.rs configuration
mod original_config {
    use synthphone_vocals::fft_config;

    // This replaces the original hardcoded constants:
    // pub const SAMPLE_RATE: f32 = 48_014.312;
    // pub const FFT_SIZE: usize = 1024;
    // pub const BUFFER_SIZE: usize = FFT_SIZE * 4;
    // pub const HOP_SIZE: usize = 256;
    // pub const BLOCK_SIZE: usize = 2;
    // pub const BIN_WIDTH: f32 = SAMPLE_RATE as f32 / FFT_SIZE as f32 * 2.0;

    fft_config!(1024, 48_014.312);

    pub fn demonstrate_original_config() {
        println!("=== Original Configuration ===");
        println!("Sample Rate: {} Hz", SAMPLE_RATE);
        println!("FFT Size: {}", FFT_SIZE);
        println!("Buffer Size: {}", BUFFER_SIZE);
        println!("Hop Size: {}", HOP_SIZE);
        println!("Block Size: {}", BLOCK_SIZE);
        println!("Bin Width: {} Hz", BIN_WIDTH);
        println!();
    }
}

/// Example 2: High-quality configuration for better frequency resolution
mod high_quality_config {
    use synthphone_vocals::fft_config;

    // Higher FFT size for better frequency resolution
    // Smaller hop size for better time resolution
    fft_config!(4096, 48000.0, hop_ratio = 0.125, buffer_multiplier = 8);

    pub fn demonstrate_high_quality_config() {
        println!("=== High Quality Configuration ===");
        println!("Sample Rate: {} Hz", SAMPLE_RATE);
        println!("FFT Size: {}", FFT_SIZE);
        println!("Buffer Size: {}", BUFFER_SIZE);
        println!("Hop Size: {} ({}% overlap)", HOP_SIZE, (1.0 - 0.125) * 100.0);
        println!("Block Size: {}", BLOCK_SIZE);
        println!("Bin Width: {:.2} Hz", BIN_WIDTH);
        println!(
            "Memory usage: ~{} KB (without bitrev tables)",
            (FFT_SIZE * 4 + BUFFER_SIZE * 4) / 1024
        );
        println!();
    }
}

/// Example 3: Fast processing configuration for real-time applications
mod fast_config {
    use synthphone_vocals::fft_config;

    // Smaller FFT size for faster processing
    // Larger hop size for less CPU usage
    fft_config!(512, 44100.0, hop_ratio = 0.5, buffer_multiplier = 2);

    pub fn demonstrate_fast_config() {
        println!("=== Fast Processing Configuration ===");
        println!("Sample Rate: {} Hz", SAMPLE_RATE);
        println!("FFT Size: {}", FFT_SIZE);
        println!("Buffer Size: {}", BUFFER_SIZE);
        println!("Hop Size: {} ({}% overlap)", HOP_SIZE, (1.0 - 0.5) * 100.0);
        println!("Block Size: {}", BLOCK_SIZE);
        println!("Bin Width: {:.2} Hz", BIN_WIDTH);
        println!("Processing latency: ~{:.2} ms", (HOP_SIZE as f32 / SAMPLE_RATE) * 1000.0);
        println!();
    }
}

/// Example 4: Using struct-based configuration for multiple setups
mod multi_config {
    use synthphone_vocals::{fft_config, fft_config_struct};

    // Define multiple configurations as structs
    fft_config_struct!(VoiceConfig, 2048, 48000.0, hop_ratio = 0.25);
    fft_config_struct!(MusicConfig, 4096, 48000.0, hop_ratio = 0.125);
    fft_config_struct!(RealtimeConfig, 1024, 48000.0, hop_ratio = 0.5);

    pub fn demonstrate_multi_config() {
        println!("=== Multiple Configuration Example ===");

        println!("Voice Processing Config:");
        println!("  FFT Size: {}", VoiceConfig::FFT_SIZE);
        println!("  Bin Width: {:.2} Hz", VoiceConfig::BIN_WIDTH);
        println!("  Hop Size: {}", VoiceConfig::HOP_SIZE);

        println!("Music Processing Config:");
        println!("  FFT Size: {}", MusicConfig::FFT_SIZE);
        println!("  Bin Width: {:.2} Hz", MusicConfig::BIN_WIDTH);
        println!("  Hop Size: {}", MusicConfig::HOP_SIZE);

        println!("Realtime Config:");
        println!("  FFT Size: {}", RealtimeConfig::FFT_SIZE);
        println!("  Bin Width: {:.2} Hz", RealtimeConfig::BIN_WIDTH);
        println!("  Hop Size: {}", RealtimeConfig::HOP_SIZE);
        println!();
    }
}

/// Example 5: Runtime configuration validation
mod runtime_validation {
    use synthphone_vocals::fft_config;

    pub fn demonstrate_runtime_validation() {
        println!("=== Runtime Validation Example ===");

        let test_cases = [
            (1024, 48000.0, 0.25),  // Valid
            (1023, 48000.0, 0.25),  // Invalid: not power of 2
            (1024, -48000.0, 0.25), // Invalid: negative sample rate
            (1024, 48000.0, 0.0),   // Invalid: zero hop ratio
            (1024, 48000.0, 1.5),   // Invalid: hop ratio > 1.0
            (2, 48000.0, 0.25),     // Invalid: FFT size too small
            (65536, 48000.0, 0.25), // Invalid: FFT size too large
        ];

        for (fft_size, sample_rate, hop_ratio) in test_cases {
            match fft_config::validate_config(fft_size, sample_rate, hop_ratio) {
                Ok(()) => {
                    println!("✓ Config ({}, {}, {}) is valid", fft_size, sample_rate, hop_ratio)
                }
                Err(e) => println!(
                    "✗ Config ({}, {}, {}) is invalid: {}",
                    fft_size, sample_rate, hop_ratio, e
                ),
            }
        }
        println!();
    }
}

/// Example 6: Microfft feature suggestions
mod feature_suggestions {
    use synthphone_vocals::fft_config;

    pub fn demonstrate_feature_suggestions() {
        println!("=== Microfft Feature Suggestions ===");
        println!("Add these features to your Cargo.toml for optimal memory usage:");
        println!();

        let fft_sizes = [64, 256, 1024, 2048, 4096, 8192];

        for &size in &fft_sizes {
            let feature = fft_config::suggest_microfft_feature(size);
            println!("FFT Size {}: features = [\"{}\"]", size, feature);
        }

        println!();
        println!("Example Cargo.toml configuration:");
        println!("[dependencies.microfft]");
        println!("default-features = false");
        println!("features = [\"size-4096\"]  # Adjust based on your maximum FFT size");
        println!();
    }
}

/// Example 7: Performance comparison utility
mod performance_comparison {
    use synthphone_vocals::fft_config;

    pub fn demonstrate_performance_comparison() {
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
}

/// Main demonstration function
pub fn run_examples() {
    println!("🎵 FFT Configuration Macro Examples 🎵");
    println!("=====================================");
    println!();

    original_config::demonstrate_original_config();
    high_quality_config::demonstrate_high_quality_config();
    fast_config::demonstrate_fast_config();
    multi_config::demonstrate_multi_config();
    runtime_validation::demonstrate_runtime_validation();
    feature_suggestions::demonstrate_feature_suggestions();
    performance_comparison::demonstrate_performance_comparison();

    println!("💡 Tips for choosing FFT configuration:");
    println!("- Larger FFT sizes = better frequency resolution, more CPU/memory");
    println!("- Smaller hop ratios = better time resolution, more CPU usage");
    println!("- Power of 2 sizes are required for microfft compatibility");
    println!("- Consider your target platform's memory and CPU constraints");
    println!("- Use runtime validation during development to catch errors early");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_original_config_constants() {
        use original_config::*;

        assert_eq!(FFT_SIZE, 1024);
        assert_eq!(BUFFER_SIZE, FFT_SIZE * 4);
        assert_eq!(HOP_SIZE, 256); // 1024 * 0.25
        assert_eq!(BLOCK_SIZE, 2);
        assert!((SAMPLE_RATE - 48_014.312).abs() < 0.001);
    }

    #[test]
    fn test_multi_config_structs() {
        use multi_config::*;

        assert_eq!(VoiceConfig::FFT_SIZE, 2048);
        assert_eq!(MusicConfig::FFT_SIZE, 4096);
        assert_eq!(RealtimeConfig::FFT_SIZE, 1024);

        // Test that hop sizes are calculated correctly
        assert_eq!(VoiceConfig::HOP_SIZE, 512); // 2048 * 0.25
        assert_eq!(MusicConfig::HOP_SIZE, 512); // 4096 * 0.125
        assert_eq!(RealtimeConfig::HOP_SIZE, 512); // 1024 * 0.5
    }
}
