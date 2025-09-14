//! FFT Configuration Module for Dynamic Setup
//!
//! This module provides macros and utilities for setting up FFT-related constants
//! and configurations dynamically based on the microfft library requirements.
//! The module ensures that:
//! - FFT sizes are powers of 2 (required by microfft's Radix-2 algorithm)
//! - FFT sizes are within supported range (4 to 32768)
//! - Related constants are calculated consistently
//! - Proper microfft features are suggested for optimal memory usage
#![allow(dead_code)]
/// Macro to set up FFT configuration with dynamic sizing
///
/// This macro generates all the necessary constants for FFT processing
/// based on a given FFT size and sample rate.
///
/// # Arguments
/// - `fft_size`: Must be a power of 2 between 4 and 32768
/// - `sample_rate`: Sample rate in Hz (f32)
/// - `hop_ratio`: Optional hop size as fraction of FFT size (default: 0.25)
/// - `buffer_multiplier`: Optional buffer size multiplier (default: 4)
/// - `block_size`: Optional block size (default: 2)
///
/// # Generated Constants
/// - `SAMPLE_RATE`: The provided sample rate
/// - `FFT_SIZE`: The FFT size (must be power of 2)
/// - `BUFFER_SIZE`: FFT_SIZE * buffer_multiplier
/// - `HOP_SIZE`: FFT_SIZE * hop_ratio (rounded to usize)
/// - `BLOCK_SIZE`: Block size for processing
/// - `BIN_WIDTH`: Frequency resolution per bin
///
/// # Example
/// ```rust,no_run
/// use synthphone_vocals::fft_config;
///
/// // Basic usage with default parameters (in one scope)
/// mod basic_config {
///     use synthphone_vocals::fft_config;
///     fft_config!(1024, 48000.0);
///
///     pub fn print_config() {
///         println!("FFT Size: {}", FFT_SIZE);
///         println!("Buffer Size: {}", BUFFER_SIZE);
///     }
/// }
///
/// // With custom parameters (in another scope)
/// mod custom_config {
///     use synthphone_vocals::fft_config;
///     fft_config!(2048, 44100.0, hop_ratio = 0.125, buffer_multiplier = 8);
///
///     pub fn print_config() {
///         println!("FFT Size: {}", FFT_SIZE);
///         println!("Buffer Size: {}", BUFFER_SIZE);
///     }
/// }
/// ```
///
/// # Microfft Features
/// To optimize memory usage, add the appropriate feature to your Cargo.toml:
/// - For FFT sizes up to 1024: `features = ["size-1024"]`
/// - For FFT sizes up to 4096: `features = ["size-4096"]` (default)
/// - For larger sizes: `features = ["size-8192"]`, `features = ["size-16384"]`, etc.
#[macro_export]
macro_rules! fft_config {
    // Basic version with just FFT size and sample rate
    ($fft_size:expr, $sample_rate:expr) => {
        fft_config!(
            $fft_size,
            $sample_rate,
            hop_ratio = 0.25,
            buffer_multiplier = 4,
            block_size = 2
        );
    };

    // Version with hop ratio specified
    ($fft_size:expr, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        fft_config!(
            $fft_size,
            $sample_rate,
            hop_ratio = $hop_ratio,
            buffer_multiplier = 4,
            block_size = 2
        );
    };

    // Version with buffer multiplier specified
    ($fft_size:expr, $sample_rate:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        fft_config!(
            $fft_size,
            $sample_rate,
            hop_ratio = 0.25,
            buffer_multiplier = $buffer_multiplier,
            block_size = 2
        );
    };

    // Full version with all parameters
    ($fft_size:expr, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr, block_size = $block_size:expr) => {
        // Compile-time validation that FFT_SIZE is a power of 2
        const _: () = {
            if !$crate::fft_config::is_power_of_two($fft_size) {
                panic!("FFT_SIZE must be a power of 2");
            }
            if $fft_size < 4 || $fft_size > 32768 {
                panic!("FFT_SIZE must be between 4 and 32768");
            }
            if $hop_ratio <= 0.0 || $hop_ratio > 1.0 {
                panic!("hop_ratio must be between 0.0 and 1.0");
            }
            if $buffer_multiplier < 1 {
                panic!("buffer_multiplier must be at least 1");
            }
        };

        pub const SAMPLE_RATE: f32 = $sample_rate;
        pub const FFT_SIZE: usize = $fft_size;
        pub const BUFFER_SIZE: usize = FFT_SIZE * $buffer_multiplier;
        pub const HOP_SIZE: usize = (FFT_SIZE as f32 * $hop_ratio) as usize;
        pub const BLOCK_SIZE: usize = $block_size;
        pub const BIN_WIDTH: f32 = SAMPLE_RATE / FFT_SIZE as f32;

        // Compile-time suggestion for optimal microfft features
        const _FFT_FEATURE_SUGGESTION: &str =
            $crate::fft_config::suggest_microfft_feature($fft_size);

        // Optional: Print feature suggestion at compile time (commented out to avoid spam)
        // const _: () = {
        //     let _ = _FFT_FEATURE_SUGGESTION;
        //     // This would ideally print the suggestion, but const eval printing is limited
        // };
    };

    // Version with named parameters - simplified to avoid macro complexity
    ($fft_size:expr, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        fft_config!(
            $fft_size,
            $sample_rate,
            hop_ratio = $hop_ratio,
            buffer_multiplier = $buffer_multiplier,
            block_size = 2
        );
    };
}

/// Convenience macro to create a configuration struct instead of constants
/// Useful when you need multiple configurations in the same scope
#[macro_export]
macro_rules! fft_config_struct {
    // Version with hop_ratio parameter (must come first for proper matching)
    ($name:ident, $fft_size:expr, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl $name {
            pub const SAMPLE_RATE: f32 = $sample_rate;
            pub const FFT_SIZE: usize = $fft_size;
            pub const BUFFER_SIZE: usize = Self::FFT_SIZE * 4;
            pub const HOP_SIZE: usize = (Self::FFT_SIZE as f32 * $hop_ratio) as usize;
            pub const BLOCK_SIZE: usize = 2;
            pub const BIN_WIDTH: f32 = Self::SAMPLE_RATE / Self::FFT_SIZE as f32;

            // Compile-time validation
            const _VALIDATE: () = {
                assert!(
                    $crate::fft_config::is_power_of_two($fft_size),
                    "FFT_SIZE must be a power of 2"
                );
                assert!(
                    $fft_size >= 4 && $fft_size <= 32768,
                    "FFT_SIZE must be between 4 and 32768"
                );
            };
        }
    };

    // Basic version with default parameters
    ($name:ident, $fft_size:expr, $sample_rate:expr) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl $name {
            pub const SAMPLE_RATE: f32 = $sample_rate;
            pub const FFT_SIZE: usize = $fft_size;
            pub const BUFFER_SIZE: usize = Self::FFT_SIZE * 4;
            pub const HOP_SIZE: usize = (Self::FFT_SIZE as f32 * 0.25) as usize;
            pub const BLOCK_SIZE: usize = 2;
            pub const BIN_WIDTH: f32 = Self::SAMPLE_RATE / Self::FFT_SIZE as f32;

            // Compile-time validation
            const _VALIDATE: () = {
                assert!(
                    $crate::fft_config::is_power_of_two($fft_size),
                    "FFT_SIZE must be a power of 2"
                );
                assert!(
                    $fft_size >= 4 && $fft_size <= 32768,
                    "FFT_SIZE must be between 4 and 32768"
                );
            };
        }
    };
}

/// Check if a number is a power of 2 at compile time
pub const fn is_power_of_two(n: usize) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

/// Suggest the optimal microfft feature for a given FFT size
pub const fn suggest_microfft_feature(fft_size: usize) -> &'static str {
    match fft_size {
        1..=4 => "size-4",
        5..=8 => "size-8",
        9..=16 => "size-16",
        17..=32 => "size-32",
        33..=64 => "size-64",
        65..=128 => "size-128",
        129..=256 => "size-256",
        257..=512 => "size-512",
        513..=1024 => "size-1024",
        1025..=2048 => "size-2048",
        2049..=4096 => "size-4096", // This is the default
        4097..=8192 => "size-8192",
        8193..=16384 => "size-16384",
        16385..=32768 => "size-32768",
        _ => "size-32768", // Maximum supported
    }
}

/// Get the next power of 2 greater than or equal to n
pub const fn next_power_of_two(mut n: usize) -> usize {
    if n <= 1 {
        return 1;
    }
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    #[cfg(target_pointer_width = "64")]
    {
        n |= n >> 32;
    }
    n + 1
}

/// Validate FFT configuration at runtime
pub fn validate_config(
    fft_size: usize,
    sample_rate: f32,
    hop_ratio: f32,
) -> Result<(), &'static str> {
    if !is_power_of_two(fft_size) {
        return Err("FFT size must be a power of 2");
    }
    if !(4..=32768).contains(&fft_size) {
        return Err("FFT size must be between 4 and 32768");
    }
    if sample_rate <= 0.0 {
        return Err("Sample rate must be positive");
    }
    if hop_ratio <= 0.0 || hop_ratio > 1.0 {
        return Err("Hop ratio must be between 0.0 and 1.0");
    }
    Ok(())
}

/// Configuration struct for FFT parameters
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FFTConfig {
    pub fft_size: usize,
    pub sample_rate: f32,
    pub hop_ratio: f32,
    pub buffer_multiplier: usize,
    pub block_size: usize,
}

impl FFTConfig {
    /// Create a new FFT configuration with validation
    pub fn new(
        fft_size: usize,
        sample_rate: f32,
        hop_ratio: f32,
        buffer_multiplier: usize,
        block_size: usize,
    ) -> Result<Self, &'static str> {
        validate_config(fft_size, sample_rate, hop_ratio)?;

        if buffer_multiplier < 1 {
            return Err("buffer_multiplier must be at least 1");
        }

        if block_size < 1 {
            return Err("block_size must be at least 1");
        }

        Ok(Self { fft_size, sample_rate, hop_ratio, buffer_multiplier, block_size })
    }

    /// Create a default configuration for the given FFT size and sample rate
    pub fn default_for(fft_size: usize, sample_rate: f32) -> Result<Self, &'static str> {
        Self::new(fft_size, sample_rate, 0.25, 4, 2)
    }

    /// Get the buffer size for this configuration
    pub const fn buffer_size(&self) -> usize {
        self.fft_size * self.buffer_multiplier
    }

    /// Get the hop size for this configuration
    pub fn hop_size(&self) -> usize {
        (self.fft_size as f32 * self.hop_ratio) as usize
    }

    /// Get the frequency bin width for this configuration
    pub fn bin_width(&self) -> f32 {
        self.sample_rate / self.fft_size as f32
    }

    /// Get the suggested microfft feature for this configuration
    pub const fn suggested_microfft_feature(&self) -> &'static str {
        suggest_microfft_feature(self.fft_size)
    }
}

impl Default for FFTConfig {
    fn default() -> Self {
        Self {
            fft_size: 1024,
            sample_rate: 48000.0,
            hop_ratio: 0.25,
            buffer_multiplier: 4,
            block_size: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_power_of_two() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(4));
        assert!(is_power_of_two(8));
        assert!(is_power_of_two(1024));
        assert!(is_power_of_two(2048));

        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(5));
        assert!(!is_power_of_two(1023));
    }

    #[test]
    fn test_next_power_of_two() {
        assert_eq!(next_power_of_two(1), 1);
        assert_eq!(next_power_of_two(2), 2);
        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(5), 8);
        assert_eq!(next_power_of_two(1000), 1024);
        assert_eq!(next_power_of_two(1024), 1024);
        assert_eq!(next_power_of_two(1025), 2048);
    }

    #[test]
    fn test_suggest_microfft_feature() {
        assert_eq!(suggest_microfft_feature(4), "size-4");
        assert_eq!(suggest_microfft_feature(16), "size-16");
        assert_eq!(suggest_microfft_feature(1024), "size-1024");
        assert_eq!(suggest_microfft_feature(4096), "size-4096");
        assert_eq!(suggest_microfft_feature(8192), "size-8192");
    }

    #[test]
    fn test_validate_config() {
        assert!(validate_config(1024, 48000.0, 0.25).is_ok());
        assert!(validate_config(2048, 44100.0, 0.125).is_ok());

        assert!(validate_config(1023, 48000.0, 0.25).is_err()); // Not power of 2
        assert!(validate_config(1024, -48000.0, 0.25).is_err()); // Negative sample rate
        assert!(validate_config(1024, 48000.0, 0.0).is_err()); // Zero hop ratio
        assert!(validate_config(1024, 48000.0, 1.5).is_err()); // Hop ratio > 1.0
    }

    #[test]
    fn test_fft_config_struct() {
        let config = FFTConfig::new(1024, 48000.0, 0.25, 4, 2).unwrap();

        assert_eq!(config.fft_size, 1024);
        assert_eq!(config.sample_rate, 48000.0);
        assert_eq!(config.hop_ratio, 0.25);
        assert_eq!(config.buffer_size(), 4096);
        assert_eq!(config.hop_size(), 256);
        assert_eq!(config.bin_width(), 46.875);
        assert_eq!(config.suggested_microfft_feature(), "size-1024");
    }

    #[test]
    fn test_fft_config_default_for() {
        let config = FFTConfig::default_for(2048, 44100.0).unwrap();

        assert_eq!(config.fft_size, 2048);
        assert_eq!(config.sample_rate, 44100.0);
        assert_eq!(config.hop_ratio, 0.25);
        assert_eq!(config.buffer_multiplier, 4);
        assert_eq!(config.block_size, 2);
    }

    #[test]
    fn test_basic_fft_config_macro() {
        // Test basic macro usage
        mod test_basic {
            use crate::fft_config;
            fft_config!(1024, 48000.0);

            pub fn verify() {
                assert_eq!(FFT_SIZE, 1024);
                assert_eq!(SAMPLE_RATE, 48000.0);
                assert_eq!(BUFFER_SIZE, 4096); // 1024 * 4
                assert_eq!(HOP_SIZE, 256); // 1024 * 0.25
                assert_eq!(BLOCK_SIZE, 2);
            }
        }

        test_basic::verify();
    }

    #[test]
    fn test_fft_config_struct_macro() {
        use crate::fft_config_struct;

        // Test struct-based configuration
        fft_config_struct!(TestConfig, 2048, 44100.0);
        fft_config_struct!(TestConfigWithHop, 1024, 48000.0, hop_ratio = 0.125);

        assert_eq!(TestConfig::FFT_SIZE, 2048);
        assert_eq!(TestConfig::SAMPLE_RATE, 44100.0);
        assert_eq!(TestConfig::HOP_SIZE, 512); // 2048 * 0.25

        assert_eq!(TestConfigWithHop::FFT_SIZE, 1024);
        assert_eq!(TestConfigWithHop::HOP_SIZE, 128); // 1024 * 0.125
    }
}
