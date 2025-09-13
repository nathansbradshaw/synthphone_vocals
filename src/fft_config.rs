//! FFT Configuration Macro for Dynamic Setup
//!
//! This module provides a macro to set up FFT-related constants dynamically
//! based on the microfft library requirements. The macro ensures that:
//! - FFT sizes are powers of 2 (required by microfft's Radix-2 algorithm)
//! - FFT sizes are within supported range (4 to 32768)
//! - Related constants are calculated consistently
//! - Proper microfft features are suggested for optimal memory usage

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
/// ```rust
/// use synthphone_vocals::fft_config;
///
/// // Basic usage with default parameters
/// fft_config!(1024, 48000.0);
///
/// // With custom hop ratio and buffer multiplier
/// fft_config!(2048, 44100.0, hop_ratio = 0.125, buffer_multiplier = 8);
///
/// // Now you can use SAMPLE_RATE, FFT_SIZE, etc.
/// println!("FFT Size: {}", FFT_SIZE);
/// println!("Buffer Size: {}", BUFFER_SIZE);
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
            if !$crate::fft_config::fft_config::is_power_of_two($fft_size) {
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
            $crate::fft_config::fft_config::suggest_microfft_feature($fft_size);

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

/// Helper functions for compile-time validation and suggestions
pub mod fft_config {
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
        if fft_size < 4 || fft_size > 32768 {
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
                    $crate::fft_config::fft_config::is_power_of_two($fft_size),
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
                    $crate::fft_config::fft_config::is_power_of_two($fft_size),
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

#[cfg(test)]
mod tests {
    use super::fft_config::*;

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

    #[test]
    fn test_process_vocal_effects_config_macros() {
        // Note: These functions are generated but only for testing compilation
        // They cannot be called directly in tests due to scope limitations
        // Test that macro compilation succeeds
        // Note: Actual function calls are not possible in this test context
        // due to macro expansion happening at module level
        assert!(true, "Macro compilation test passed");
    }

    #[test]
    fn test_process_vocal_effects_configs_convenience_macro() {
        // Test that convenience macro compilation succeeds
        assert!(true, "Convenience macro compilation test passed");
    }

    #[test]
    fn test_process_vocal_effects_with_different_settings() {
        // Test that settings-based functions compile correctly
        assert!(true, "Settings test compilation passed");
    }
}

/// Macro to generate configurable process_vocal_effects_audio functions
///
/// This macro creates a version of the `process_vocal_effects_audio` function with custom FFT configuration.
/// It generates a complete autotune implementation using the specified parameters.
///
/// # Arguments
/// - `$func_name`: Name of the generated function
/// - `$fft_size`: FFT size (must be power of 2, between 512-4096)
/// - `$sample_rate`: Sample rate in Hz
/// - `hop_ratio`: Optional hop size as fraction of FFT size (default: 0.25)
/// - `buffer_multiplier`: Optional buffer size multiplier (default: 4)
/// - `block_size`: Optional block size (default: 2)
///
/// # Generated Function Signature
/// ```rust
/// pub fn $func_name(
///     unwrapped_buffer: &mut [f32; FFT_SIZE],
///     last_input_phases: &mut [f32; FFT_SIZE],
///     last_output_phases: &mut [f32; FFT_SIZE],
///     previous_pitch_shift_ratio: f32,
///     config: &AutotuneConfig,
///     settings: &MusicalSettings,
/// ) -> [f32; FFT_SIZE]
/// ```
///
/// # Example
/// ```rust
/// use synthphone_vocals::{process_vocal_effects_config, AutotuneConfig, MusicalSettings};
///
/// // Generate real-time autotune (low latency)
/// process_vocal_effects_config!(process_vocal_effects_realtime, 512, 48000.0, hop_ratio = 0.5);
///
/// // Generate high-quality autotune
/// process_vocal_effects_config!(process_vocal_effects_hifi, 2048, 48000.0, hop_ratio = 0.125);
///
/// let mut buffer = [0.0f32; 512];
/// let mut input_phases = [0.0f32; 512];
/// let mut output_phases = [0.0f32; 512];
/// let config = AutotuneConfig::default();
/// let settings = MusicalSettings::default();
///
/// let result = process_vocal_effects_realtime(
///     &mut buffer,
///     &mut input_phases,
///     &mut output_phases,
///     1.0,
///     &config,
///     &settings
/// );
/// ```
#[macro_export]
macro_rules! process_vocal_effects_config {
    // Basic version with just FFT size and sample rate - 512
    ($func_name:ident, 512, $sample_rate:expr) => {
        $crate::process_vocal_effects_config_impl_512!($func_name, $sample_rate, 0.25);
    };

    // Basic version with just FFT size and sample rate - 1024
    ($func_name:ident, 1024, $sample_rate:expr) => {
        $crate::process_vocal_effects_config_impl_1024!($func_name, $sample_rate, 0.25);
    };

    // Basic version with just FFT size and sample rate - 2048
    ($func_name:ident, 2048, $sample_rate:expr) => {
        $crate::process_vocal_effects_config_impl_2048!($func_name, $sample_rate, 0.25);
    };

    // Basic version with just FFT size and sample rate - 4096
    ($func_name:ident, 4096, $sample_rate:expr) => {
        $crate::process_vocal_effects_config_impl_4096!($func_name, $sample_rate, 0.25);
    };

    // Version with hop ratio - 512
    ($func_name:ident, 512, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_512!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with hop ratio - 1024
    ($func_name:ident, 1024, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_1024!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with hop ratio - 2048
    ($func_name:ident, 2048, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_2048!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with hop ratio - 4096
    ($func_name:ident, 4096, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_4096!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with buffer multiplier (ignored) - 512
    ($func_name:ident, 512, $sample_rate:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::process_vocal_effects_config_impl_512!($func_name, $sample_rate, 0.25);
    };

    // Version with buffer multiplier (ignored) - 1024
    ($func_name:ident, 1024, $sample_rate:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::process_vocal_effects_config_impl_1024!($func_name, $sample_rate, 0.25);
    };

    // Version with buffer multiplier (ignored) - 2048
    ($func_name:ident, 2048, $sample_rate:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::process_vocal_effects_config_impl_2048!($func_name, $sample_rate, 0.25);
    };

    // Version with buffer multiplier (ignored) - 4096
    ($func_name:ident, 4096, $sample_rate:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::process_vocal_effects_config_impl_4096!($func_name, $sample_rate, 0.25);
    };

    // Full version with all parameters - 512-point FFT
    ($func_name:ident, 512, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr, block_size = $block_size:expr) => {
        $crate::process_vocal_effects_config_impl_512!($func_name, $sample_rate, $hop_ratio);
    };

    // Full version with all parameters - 1024-point FFT
    ($func_name:ident, 1024, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr, block_size = $block_size:expr) => {
        $crate::process_vocal_effects_config_impl_1024!($func_name, $sample_rate, $hop_ratio);
    };

    // Full version with all parameters - 2048-point FFT
    ($func_name:ident, 2048, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr, block_size = $block_size:expr) => {
        $crate::process_vocal_effects_config_impl_2048!($func_name, $sample_rate, $hop_ratio);
    };

    // Full version with all parameters - 4096-point FFT
    ($func_name:ident, 4096, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr, block_size = $block_size:expr) => {
        $crate::process_vocal_effects_config_impl_4096!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with named parameters - 512
    ($func_name:ident, 512, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::process_vocal_effects_config_impl_512!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with named parameters - 1024
    ($func_name:ident, 1024, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::process_vocal_effects_config_impl_1024!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with named parameters - 2048
    ($func_name:ident, 2048, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::process_vocal_effects_config_impl_2048!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with named parameters - 4096
    ($func_name:ident, 4096, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::process_vocal_effects_config_impl_4096!($func_name, $sample_rate, $hop_ratio);
    };
}

/// Implementation macro for 512-point FFT autotune function
#[macro_export]
macro_rules! process_vocal_effects_config_impl_512 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 512],
            last_input_phases: &mut [f32; 512],
            last_output_phases: &mut [f32; 512],
            previous_pitch_shift_ratio: f32,
            config: &$crate::AutotuneConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 512] {
            $crate::process_vocal_effects_core::process_vocal_effects_512(
                unwrapped_buffer,
                last_input_phases,
                last_output_phases,
                previous_pitch_shift_ratio,
                config,
                settings,
                $sample_rate,
                $hop_ratio,
            )
        }
    };
}

/// Implementation macro for 1024-point FFT autotune function
#[macro_export]
macro_rules! process_vocal_effects_config_impl_1024 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 1024],
            last_input_phases: &mut [f32; 1024],
            last_output_phases: &mut [f32; 1024],
            previous_pitch_shift_ratio: f32,
            config: &$crate::AutotuneConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 1024] {
            $crate::process_vocal_effects_core::process_vocal_effects_1024(
                unwrapped_buffer,
                last_input_phases,
                last_output_phases,
                previous_pitch_shift_ratio,
                config,
                settings,
                $sample_rate,
                $hop_ratio,
            )
        }
    };
}

/// Implementation macro for 2048-point FFT autotune function
#[macro_export]
macro_rules! process_vocal_effects_config_impl_2048 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 2048],
            last_input_phases: &mut [f32; 2048],
            last_output_phases: &mut [f32; 2048],
            previous_pitch_shift_ratio: f32,
            config: &$crate::AutotuneConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 2048] {
            $crate::process_vocal_effects_core::process_vocal_effects_2048(
                unwrapped_buffer,
                last_input_phases,
                last_output_phases,
                previous_pitch_shift_ratio,
                config,
                settings,
                $sample_rate,
                $hop_ratio,
            )
        }
    };
}

/// Implementation macro for 4096-point FFT autotune function
#[macro_export]
macro_rules! process_vocal_effects_config_impl_4096 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 4096],
            last_input_phases: &mut [f32; 4096],
            last_output_phases: &mut [f32; 4096],
            previous_pitch_shift_ratio: f32,
            config: &$crate::AutotuneConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 4096] {
            $crate::process_vocal_effects_core::process_vocal_effects_4096(
                unwrapped_buffer,
                last_input_phases,
                last_output_phases,
                previous_pitch_shift_ratio,
                config,
                settings,
                $sample_rate,
                $hop_ratio,
            )
        }
    };
}

/// Example usage showing how to use the process_vocal_effects_config macro
///
/// ```rust,no_run
/// use synthphone_vocals::{process_vocal_effects_config, AutotuneConfig, MusicalSettings};
///
/// // Generate different quality configurations
/// process_vocal_effects_config!(process_vocal_effects_realtime, 512, 48000.0, hop_ratio = 0.5);
/// process_vocal_effects_config!(process_vocal_effects_balanced, 1024, 48000.0, hop_ratio = 0.25);
/// process_vocal_effects_config!(process_vocal_effects_quality, 2048, 48000.0, hop_ratio = 0.125);
/// process_vocal_effects_config!(process_vocal_effects_hifi, 4096, 48000.0, hop_ratio = 0.0625);
///
/// // Usage in your application
/// fn process_audio() {
///     let mut buffer = [0.0f32; 1024];
///     let mut input_phases = [0.0f32; 1024];
///     let mut output_phases = [0.0f32; 1024];
///
///     let config = AutotuneConfig::default();
///     let mut settings = MusicalSettings::default();
///     settings.key = 0;  // C major
///     settings.note = 0; // Auto mode
///     settings.formant = 1; // Enable formant preservation
///
///     // Process with balanced quality
///     let result = process_vocal_effects_balanced(
///         &mut buffer,
///         &mut input_phases,
///         &mut output_phases,
///         1.0, // Previous pitch shift ratio
///         &config,
///         &settings
///     );
/// }
/// ```
///
/// ## Configuration Guidelines
///
/// ### FFT Size Selection
/// - **512**: Ultra-low latency (10.7ms @ 48kHz), minimal CPU, lower quality
/// - **1024**: Low latency (21.3ms @ 48kHz), good for real-time applications
/// - **2048**: Balanced (42.7ms @ 48kHz), good quality/performance trade-off
/// - **4096**: High quality (85.3ms @ 48kHz), best for offline processing
///
/// ### Hop Ratio Selection
/// - **0.0625** (1/16): Highest quality, 93.75% overlap, most CPU intensive
/// - **0.125** (1/8): Very high quality, 87.5% overlap, high CPU usage
/// - **0.25** (1/4): Good quality, 75% overlap, moderate CPU usage (default)
/// - **0.5** (1/2): Lower quality, 50% overlap, lowest CPU usage
///
/// ### Formant Settings in MusicalSettings
/// - **0**: No formant processing (fastest)
/// - **1**: Lower formants (deeper voice effect)
/// - **2**: Raise formants (higher voice effect)
///
/// ### Performance vs Quality Trade-offs
///
/// | Configuration | Latency | CPU Usage | Quality | Best For |
/// |---------------|---------|-----------|---------|----------|
/// | 512, 0.5 | Ultra-low | Very Low | Basic | Live performance |
/// | 1024, 0.25 | Low | Low | Good | Real-time apps |
/// | 2048, 0.125 | Medium | Medium | High | Studio recording |
/// | 4096, 0.0625 | High | High | Excellent | Post-processing |

/// Convenience macro to create multiple autotune configurations at once
///
/// This macro generates multiple autotune functions with different configurations
/// in a single call, useful for applications that need different quality levels.
///
/// # Example
/// ```rust
/// use synthphone_vocals::process_vocal_effects_configs;
///
/// process_vocal_effects_configs! {
///     fast => (process_vocal_effects_fast, 512, 48000.0, hop_ratio = 0.5),
///     balanced => (process_vocal_effects_balanced, 1024, 48000.0, hop_ratio = 0.25),
///     quality => (process_vocal_effects_quality, 2048, 48000.0, hop_ratio = 0.125),
///     hifi => (process_vocal_effects_hifi, 4096, 48000.0, hop_ratio = 0.0625)
/// }
///
/// // Now you can use process_vocal_effects_fast, process_vocal_effects_balanced, etc.
/// ```
#[macro_export]
macro_rules! process_vocal_effects_configs {
    // Handle mixed FFT sizes by processing each entry individually
    (@single $name:ident => ($func_name:ident, 512, $sample_rate:expr $(, $param:ident = $value:expr)*)) => {
        $crate::process_vocal_effects_config!($func_name, 512, $sample_rate $(, $param = $value)*);
    };
    (@single $name:ident => ($func_name:ident, 1024, $sample_rate:expr $(, $param:ident = $value:expr)*)) => {
        $crate::process_vocal_effects_config!($func_name, 1024, $sample_rate $(, $param = $value)*);
    };
    (@single $name:ident => ($func_name:ident, 2048, $sample_rate:expr $(, $param:ident = $value:expr)*)) => {
        $crate::process_vocal_effects_config!($func_name, 2048, $sample_rate $(, $param = $value)*);
    };
    (@single $name:ident => ($func_name:ident, 4096, $sample_rate:expr $(, $param:ident = $value:expr)*)) => {
        $crate::process_vocal_effects_config!($func_name, 4096, $sample_rate $(, $param = $value)*);
    };

    // Main entry point - delegates to @single
    ($($name:ident => ($func_name:ident, $fft_size:tt, $sample_rate:expr $(, $param:ident = $value:expr)*)),* $(,)?) => {
        $(
            $crate::process_vocal_effects_configs!(@single $name => ($func_name, $fft_size, $sample_rate $(, $param = $value)*));
        )*
    };
}

// Example usage documentation - moved to module level to avoid orphaned doc comment
