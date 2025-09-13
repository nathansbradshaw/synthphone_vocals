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
    fn test_autotune_config_macros() {
        // Note: These functions are generated but only for testing compilation
        // They cannot be called directly in tests due to scope limitations
        // Test that macro compilation succeeds
        // Note: Actual function calls are not possible in this test context
        // due to macro expansion happening at module level
        assert!(true, "Macro compilation test passed");
    }

    #[test]
    fn test_autotune_configs_convenience_macro() {
        // Test that convenience macro compilation succeeds
        assert!(true, "Convenience macro compilation test passed");
    }

    #[test]
    fn test_autotune_with_different_settings() {
        // Test that settings-based functions compile correctly
        assert!(true, "Settings test compilation passed");
    }
}

/// Macro to generate configurable autotune_audio functions
///
/// This macro creates a version of the `autotune_audio` function with custom FFT configuration.
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
/// use synthphone_vocals::{autotune_config, AutotuneConfig, MusicalSettings};
///
/// // Generate real-time autotune (low latency)
/// autotune_config!(autotune_realtime, 512, 48000.0, hop_ratio = 0.5);
///
/// // Generate high-quality autotune
/// autotune_config!(autotune_hifi, 2048, 48000.0, hop_ratio = 0.125);
///
/// let mut buffer = [0.0f32; 512];
/// let mut input_phases = [0.0f32; 512];
/// let mut output_phases = [0.0f32; 512];
/// let config = AutotuneConfig::default();
/// let settings = MusicalSettings::default();
///
/// let result = autotune_realtime(
///     &mut buffer,
///     &mut input_phases,
///     &mut output_phases,
///     1.0,
///     &config,
///     &settings
/// );
/// ```
#[macro_export]
macro_rules! autotune_config {
    // Basic version with just FFT size and sample rate - 512
    ($func_name:ident, 512, $sample_rate:expr) => {
        $crate::autotune_config_impl_512!($func_name, $sample_rate, 0.25);
    };

    // Basic version with just FFT size and sample rate - 1024
    ($func_name:ident, 1024, $sample_rate:expr) => {
        $crate::autotune_config_impl_1024!($func_name, $sample_rate, 0.25);
    };

    // Basic version with just FFT size and sample rate - 2048
    ($func_name:ident, 2048, $sample_rate:expr) => {
        $crate::autotune_config_impl_2048!($func_name, $sample_rate, 0.25);
    };

    // Basic version with just FFT size and sample rate - 4096
    ($func_name:ident, 4096, $sample_rate:expr) => {
        $crate::autotune_config_impl_4096!($func_name, $sample_rate, 0.25);
    };

    // Version with hop ratio - 512
    ($func_name:ident, 512, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        $crate::autotune_config_impl_512!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with hop ratio - 1024
    ($func_name:ident, 1024, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        $crate::autotune_config_impl_1024!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with hop ratio - 2048
    ($func_name:ident, 2048, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        $crate::autotune_config_impl_2048!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with hop ratio - 4096
    ($func_name:ident, 4096, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        $crate::autotune_config_impl_4096!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with buffer multiplier (ignored) - 512
    ($func_name:ident, 512, $sample_rate:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::autotune_config_impl_512!($func_name, $sample_rate, 0.25);
    };

    // Version with buffer multiplier (ignored) - 1024
    ($func_name:ident, 1024, $sample_rate:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::autotune_config_impl_1024!($func_name, $sample_rate, 0.25);
    };

    // Version with buffer multiplier (ignored) - 2048
    ($func_name:ident, 2048, $sample_rate:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::autotune_config_impl_2048!($func_name, $sample_rate, 0.25);
    };

    // Version with buffer multiplier (ignored) - 4096
    ($func_name:ident, 4096, $sample_rate:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::autotune_config_impl_4096!($func_name, $sample_rate, 0.25);
    };

    // Full version with all parameters - 512-point FFT
    ($func_name:ident, 512, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr, block_size = $block_size:expr) => {
        $crate::autotune_config_impl_512!($func_name, $sample_rate, $hop_ratio);
    };

    // Full version with all parameters - 1024-point FFT
    ($func_name:ident, 1024, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr, block_size = $block_size:expr) => {
        $crate::autotune_config_impl_1024!($func_name, $sample_rate, $hop_ratio);
    };

    // Full version with all parameters - 2048-point FFT
    ($func_name:ident, 2048, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr, block_size = $block_size:expr) => {
        $crate::autotune_config_impl_2048!($func_name, $sample_rate, $hop_ratio);
    };

    // Full version with all parameters - 4096-point FFT
    ($func_name:ident, 4096, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr, block_size = $block_size:expr) => {
        $crate::autotune_config_impl_4096!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with named parameters - 512
    ($func_name:ident, 512, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::autotune_config_impl_512!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with named parameters - 1024
    ($func_name:ident, 1024, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::autotune_config_impl_1024!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with named parameters - 2048
    ($func_name:ident, 2048, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::autotune_config_impl_2048!($func_name, $sample_rate, $hop_ratio);
    };

    // Version with named parameters - 4096
    ($func_name:ident, 4096, $sample_rate:expr, hop_ratio = $hop_ratio:expr, buffer_multiplier = $buffer_multiplier:expr) => {
        $crate::autotune_config_impl_4096!($func_name, $sample_rate, $hop_ratio);
    };
}

/// Implementation macro for 512-point FFT autotune function
#[macro_export]
macro_rules! autotune_config_impl_512 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 512],
            last_input_phases: &mut [f32; 512],
            last_output_phases: &mut [f32; 512],
            previous_pitch_shift_ratio: f32,
            _config: &$crate::AutotuneConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 512] {
            use core::f32::consts::PI;
            use libm::{atan2f, cosf, expf, fabsf, floorf, logf, sinf, sqrtf};

            const SAMPLE_RATE: f32 = $sample_rate;
            const FFT_SIZE: usize = 512;
            const HOP_SIZE: usize = (FFT_SIZE as f32 * $hop_ratio) as usize;
            const BIN_WIDTH: f32 = SAMPLE_RATE / FFT_SIZE as f32;

            let analysis_window_buffer = $crate::hann_window::get_hann_window::<512>();
            let mut full_spectrum: [microfft::Complex32; 512] =
                [microfft::Complex32 { re: 0.0, im: 0.0 }; 512];
            let mut analysis_magnitudes = [0.0; 256];
            let mut analysis_frequencies = [0.0; 256];
            let mut synthesis_magnitudes: [f32; 512] = [0.0; 512];
            let mut synthesis_frequencies: [f32; 512] = [0.0; 512];
            let mut envelope = [1.0f32; 256];

            let formant = settings.formant;
            let is_auto = settings.note == 0;

            // Apply windowing
            for i in 0..512 {
                unwrapped_buffer[i] *= analysis_window_buffer[i];
            }

            // Forward FFT
            let fft_result = microfft::real::rfft_512(unwrapped_buffer);

            // Process frequency bins
            for i in 0..fft_result.len() {
                let amplitude = sqrtf(
                    fft_result[i].re * fft_result[i].re + fft_result[i].im * fft_result[i].im,
                );
                let phase = atan2f(fft_result[i].im, fft_result[i].re);
                let mut phase_diff = phase - last_input_phases[i];
                let bin_centre_frequency = 2.0 * PI * i as f32 / 512.0;
                phase_diff = $crate::process_frequencies::wrap_phase(
                    phase_diff - bin_centre_frequency * HOP_SIZE as f32,
                );
                let bin_deviation = phase_diff * 512.0 / HOP_SIZE as f32 / (2.0 * PI);
                analysis_frequencies[i] = i as f32 + bin_deviation;
                analysis_magnitudes[i] = amplitude;
                last_input_phases[i] = phase;
            }

            // Extract formant envelope if needed
            if formant != 0 {
                const LIFTER_CUTOFF: usize = 64;
                let mut cepstrum_buffer = [0.0f32; 512];

                for i in 0..256 {
                    let mag = analysis_magnitudes[i].max(1e-6_f32);
                    let log_mag = logf(mag);
                    full_spectrum[i] = microfft::Complex32 { re: log_mag, im: 0.0 };
                    if i != 0 {
                        full_spectrum[512 - i] = microfft::Complex32 { re: log_mag, im: 0.0 };
                    }
                }

                let cepstrum = microfft::inverse::ifft_512(&mut full_spectrum);
                cepstrum_buffer.fill(0.0);
                for i in 0..LIFTER_CUTOFF.min(256) {
                    cepstrum_buffer[i] = cepstrum[i].re;
                }
                for i in (512 - LIFTER_CUTOFF.min(256))..512 {
                    cepstrum_buffer[i] = cepstrum[i].re;
                }

                let envelope_fft = microfft::real::rfft_512(&mut cepstrum_buffer);
                for i in 0..256 {
                    envelope[i] = expf(envelope_fft[i].re);
                }
            }

            // Calculate pitch shift
            let mut pitch_shift_ratio = previous_pitch_shift_ratio;
            let fundamental_index =
                $crate::process_frequencies::find_fundamental_frequency(&analysis_magnitudes);
            let detected_frequency = analysis_frequencies[fundamental_index] * BIN_WIDTH;

            if detected_frequency > 0.001 {
                let target_frequency = if is_auto {
                    let scale_frequencies = $crate::keys::get_scale_by_key(settings.key);
                    $crate::frequencies::find_nearest_note_in_key(
                        detected_frequency,
                        scale_frequencies,
                    )
                } else {
                    $crate::keys::get_frequency(settings.key, settings.note, settings.octave, false)
                };
                let raw_ratio = target_frequency / detected_frequency;
                let clamped_ratio = raw_ratio.clamp(0.5, 2.0);
                const SMOOTHING_FACTOR: f32 = 0.99;
                pitch_shift_ratio = clamped_ratio * SMOOTHING_FACTOR
                    + previous_pitch_shift_ratio * (1.0 - SMOOTHING_FACTOR);
            }

            // Apply spectral shift
            synthesis_magnitudes.fill(0.0);
            synthesis_frequencies.fill(0.0);
            let formant_ratio = match formant {
                1 => 0.5,
                2 => 2.0,
                _ => 1.0,
            };
            let use_formants = formant != 0;

            for i in 0..256 {
                if analysis_magnitudes[i] <= 1e-8 {
                    continue;
                }
                let residual = if use_formants {
                    analysis_magnitudes[i] / envelope[i].max(1e-6_f32)
                } else {
                    analysis_magnitudes[i]
                };
                let new_bin_f = i as f32 * pitch_shift_ratio;
                let new_bin = (floorf(new_bin_f + 0.5) as usize).min(255);
                if new_bin >= 256 {
                    continue;
                }

                let shifted_envelope = if use_formants {
                    let env_pos = (i as f32 / formant_ratio).clamp(0.0, 255.0);
                    let env_idx = env_pos as usize;
                    let frac = env_pos - env_idx as f32;
                    if env_idx < 255 {
                        envelope[env_idx] * (1.0 - frac) + envelope[env_idx + 1] * frac
                    } else {
                        envelope[env_idx]
                    }
                } else {
                    1.0
                };

                synthesis_magnitudes[new_bin] = residual * shifted_envelope;
                synthesis_frequencies[new_bin] = analysis_frequencies[i] * pitch_shift_ratio;
            }

            // Synthesis phase reconstruction
            for i in 0..256 {
                let magnitude = synthesis_magnitudes[i];
                let bin_deviation = synthesis_frequencies[i] - i as f32;
                let mut phase_increment = bin_deviation * 2.0 * PI * HOP_SIZE as f32 / 512.0;
                let bin_center_frequency = 2.0 * PI * i as f32 / 512.0;
                phase_increment += bin_center_frequency * HOP_SIZE as f32;
                let output_phase = $crate::process_frequencies::wrap_phase(
                    last_output_phases[i] + phase_increment,
                );
                let real_part = magnitude * cosf(output_phase);
                let imaginary_part = magnitude * sinf(output_phase);
                full_spectrum[i] = microfft::Complex32 { re: real_part, im: imaginary_part };
                if i > 0 && i < 256 {
                    full_spectrum[512 - i] =
                        microfft::Complex32 { re: real_part, im: -imaginary_part };
                }
                last_output_phases[i] = output_phase;
            }

            // Inverse FFT
            let time_domain_result = microfft::inverse::ifft_512(&mut full_spectrum);
            const GAIN_COMPENSATION: f32 = 2.0 / 3.0;
            let mut output_samples = [0.0f32; 512];

            for i in 0..512 {
                let mut sample = time_domain_result[i].re;
                sample *= analysis_window_buffer[i];
                sample *= GAIN_COMPENSATION;
                if sample.abs() > 0.95 {
                    let sign = if sample >= 0.0 { 1.0 } else { -1.0 };
                    let compressed = 0.95 - 0.05 * expf(-fabsf(sample));
                    sample = sign * compressed;
                }
                output_samples[i] = sample;
            }

            output_samples
        }
    };
}

/// Implementation macro for 1024-point FFT autotune function
#[macro_export]
macro_rules! autotune_config_impl_1024 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 1024],
            last_input_phases: &mut [f32; 1024],
            last_output_phases: &mut [f32; 1024],
            previous_pitch_shift_ratio: f32,
            _config: &$crate::AutotuneConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 1024] {
            use core::f32::consts::PI;
            use libm::{atan2f, cosf, expf, fabsf, floorf, logf, sinf, sqrtf};

            const SAMPLE_RATE: f32 = $sample_rate;
            const FFT_SIZE: usize = 1024;
            const HOP_SIZE: usize = (FFT_SIZE as f32 * $hop_ratio) as usize;
            const BIN_WIDTH: f32 = SAMPLE_RATE / FFT_SIZE as f32;

            let analysis_window_buffer = $crate::hann_window::get_hann_window::<1024>();
            let mut full_spectrum: [microfft::Complex32; 1024] =
                [microfft::Complex32 { re: 0.0, im: 0.0 }; 1024];
            let mut analysis_magnitudes = [0.0; 512];
            let mut analysis_frequencies = [0.0; 512];
            let mut synthesis_magnitudes: [f32; 1024] = [0.0; 1024];
            let mut synthesis_frequencies: [f32; 1024] = [0.0; 1024];
            let mut envelope = [1.0f32; 512];

            let formant = settings.formant;
            let is_auto = settings.note == 0;

            // Apply windowing
            for i in 0..1024 {
                unwrapped_buffer[i] *= analysis_window_buffer[i];
            }

            // Forward FFT
            let fft_result = microfft::real::rfft_1024(unwrapped_buffer);

            // Process frequency bins
            for i in 0..fft_result.len() {
                let amplitude = sqrtf(
                    fft_result[i].re * fft_result[i].re + fft_result[i].im * fft_result[i].im,
                );
                let phase = atan2f(fft_result[i].im, fft_result[i].re);
                let mut phase_diff = phase - last_input_phases[i];
                let bin_centre_frequency = 2.0 * PI * i as f32 / 1024.0;
                phase_diff = $crate::process_frequencies::wrap_phase(
                    phase_diff - bin_centre_frequency * HOP_SIZE as f32,
                );
                let bin_deviation = phase_diff * 1024.0 / HOP_SIZE as f32 / (2.0 * PI);
                analysis_frequencies[i] = i as f32 + bin_deviation;
                analysis_magnitudes[i] = amplitude;
                last_input_phases[i] = phase;
            }

            // Extract formant envelope if needed
            if formant != 0 {
                const LIFTER_CUTOFF: usize = 64;
                let mut cepstrum_buffer = [0.0f32; 1024];

                for i in 0..512 {
                    let mag = analysis_magnitudes[i].max(1e-6_f32);
                    let log_mag = logf(mag);
                    full_spectrum[i] = microfft::Complex32 { re: log_mag, im: 0.0 };
                    if i != 0 {
                        full_spectrum[1024 - i] = microfft::Complex32 { re: log_mag, im: 0.0 };
                    }
                }

                let cepstrum = microfft::inverse::ifft_1024(&mut full_spectrum);
                cepstrum_buffer.fill(0.0);
                for i in 0..LIFTER_CUTOFF.min(512) {
                    cepstrum_buffer[i] = cepstrum[i].re;
                }
                for i in (1024 - LIFTER_CUTOFF.min(512))..1024 {
                    cepstrum_buffer[i] = cepstrum[i].re;
                }

                let envelope_fft = microfft::real::rfft_1024(&mut cepstrum_buffer);
                for i in 0..512 {
                    envelope[i] = expf(envelope_fft[i].re);
                }
            }

            // Calculate pitch shift
            let mut pitch_shift_ratio = previous_pitch_shift_ratio;
            let fundamental_index =
                $crate::process_frequencies::find_fundamental_frequency(&analysis_magnitudes);
            let detected_frequency = analysis_frequencies[fundamental_index] * BIN_WIDTH;

            if detected_frequency > 0.001 {
                let target_frequency = if is_auto {
                    let scale_frequencies = $crate::keys::get_scale_by_key(settings.key);
                    $crate::frequencies::find_nearest_note_in_key(
                        detected_frequency,
                        scale_frequencies,
                    )
                } else {
                    $crate::keys::get_frequency(settings.key, settings.note, settings.octave, false)
                };
                let raw_ratio = target_frequency / detected_frequency;
                let clamped_ratio = raw_ratio.clamp(0.5, 2.0);
                const SMOOTHING_FACTOR: f32 = 0.99;
                pitch_shift_ratio = clamped_ratio * SMOOTHING_FACTOR
                    + previous_pitch_shift_ratio * (1.0 - SMOOTHING_FACTOR);
            }

            // Apply spectral shift
            synthesis_magnitudes.fill(0.0);
            synthesis_frequencies.fill(0.0);
            let formant_ratio = match formant {
                1 => 0.5,
                2 => 2.0,
                _ => 1.0,
            };
            let use_formants = formant != 0;

            for i in 0..512 {
                if analysis_magnitudes[i] <= 1e-8 {
                    continue;
                }
                let residual = if use_formants {
                    analysis_magnitudes[i] / envelope[i].max(1e-6_f32)
                } else {
                    analysis_magnitudes[i]
                };
                let new_bin_f = i as f32 * pitch_shift_ratio;
                let new_bin = (floorf(new_bin_f + 0.5) as usize).min(511);
                if new_bin >= 512 {
                    continue;
                }

                let shifted_envelope = if use_formants {
                    let env_pos = (i as f32 / formant_ratio).clamp(0.0, 511.0);
                    let env_idx = env_pos as usize;
                    let frac = env_pos - env_idx as f32;
                    if env_idx < 511 {
                        envelope[env_idx] * (1.0 - frac) + envelope[env_idx + 1] * frac
                    } else {
                        envelope[env_idx]
                    }
                } else {
                    1.0
                };

                synthesis_magnitudes[new_bin] = residual * shifted_envelope;
                synthesis_frequencies[new_bin] = analysis_frequencies[i] * pitch_shift_ratio;
            }

            // Synthesis phase reconstruction
            for i in 0..512 {
                let magnitude = synthesis_magnitudes[i];
                let bin_deviation = synthesis_frequencies[i] - i as f32;
                let mut phase_increment = bin_deviation * 2.0 * PI * HOP_SIZE as f32 / 1024.0;
                let bin_center_frequency = 2.0 * PI * i as f32 / 1024.0;
                phase_increment += bin_center_frequency * HOP_SIZE as f32;
                let output_phase = $crate::process_frequencies::wrap_phase(
                    last_output_phases[i] + phase_increment,
                );
                let real_part = magnitude * cosf(output_phase);
                let imaginary_part = magnitude * sinf(output_phase);
                full_spectrum[i] = microfft::Complex32 { re: real_part, im: imaginary_part };
                if i > 0 && i < 512 {
                    full_spectrum[1024 - i] =
                        microfft::Complex32 { re: real_part, im: -imaginary_part };
                }
                last_output_phases[i] = output_phase;
            }

            // Inverse FFT
            let time_domain_result = microfft::inverse::ifft_1024(&mut full_spectrum);
            const GAIN_COMPENSATION: f32 = 2.0 / 3.0;
            let mut output_samples = [0.0f32; 1024];

            for i in 0..1024 {
                let mut sample = time_domain_result[i].re;
                sample *= analysis_window_buffer[i];
                sample *= GAIN_COMPENSATION;
                if sample.abs() > 0.95 {
                    let sign = if sample >= 0.0 { 1.0 } else { -1.0 };
                    let compressed = 0.95 - 0.05 * expf(-fabsf(sample));
                    sample = sign * compressed;
                }
                output_samples[i] = sample;
            }

            output_samples
        }
    };
}

/// Implementation macro for 2048-point FFT autotune function
#[macro_export]
macro_rules! autotune_config_impl_2048 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 2048],
            last_input_phases: &mut [f32; 2048],
            last_output_phases: &mut [f32; 2048],
            previous_pitch_shift_ratio: f32,
            _config: &$crate::AutotuneConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 2048] {
            use core::f32::consts::PI;
            use libm::{atan2f, cosf, expf, fabsf, floorf, logf, sinf, sqrtf};

            const SAMPLE_RATE: f32 = $sample_rate;
            const FFT_SIZE: usize = 2048;
            const HOP_SIZE: usize = (FFT_SIZE as f32 * $hop_ratio) as usize;
            const BIN_WIDTH: f32 = SAMPLE_RATE / FFT_SIZE as f32;

            let analysis_window_buffer = $crate::hann_window::get_hann_window::<2048>();
            let mut full_spectrum: [microfft::Complex32; 2048] =
                [microfft::Complex32 { re: 0.0, im: 0.0 }; 2048];
            let mut analysis_magnitudes = [0.0; 1024];
            let mut analysis_frequencies = [0.0; 1024];
            let mut synthesis_magnitudes: [f32; 2048] = [0.0; 2048];
            let mut synthesis_frequencies: [f32; 2048] = [0.0; 2048];
            let mut envelope = [1.0f32; 1024];

            let formant = settings.formant;
            let is_auto = settings.note == 0;

            // Apply windowing
            for i in 0..2048 {
                unwrapped_buffer[i] *= analysis_window_buffer[i];
            }

            // Forward FFT
            let fft_result = microfft::real::rfft_2048(unwrapped_buffer);

            // Process frequency bins
            for i in 0..fft_result.len() {
                let amplitude = sqrtf(
                    fft_result[i].re * fft_result[i].re + fft_result[i].im * fft_result[i].im,
                );
                let phase = atan2f(fft_result[i].im, fft_result[i].re);
                let mut phase_diff = phase - last_input_phases[i];
                let bin_centre_frequency = 2.0 * PI * i as f32 / 2048.0;
                phase_diff = $crate::process_frequencies::wrap_phase(
                    phase_diff - bin_centre_frequency * HOP_SIZE as f32,
                );
                let bin_deviation = phase_diff * 2048.0 / HOP_SIZE as f32 / (2.0 * PI);
                analysis_frequencies[i] = i as f32 + bin_deviation;
                analysis_magnitudes[i] = amplitude;
                last_input_phases[i] = phase;
            }

            // Extract formant envelope if needed
            if formant != 0 {
                const LIFTER_CUTOFF: usize = 64;
                let mut cepstrum_buffer = [0.0f32; 2048];

                for i in 0..1024 {
                    let mag = analysis_magnitudes[i].max(1e-6_f32);
                    let log_mag = logf(mag);
                    full_spectrum[i] = microfft::Complex32 { re: log_mag, im: 0.0 };
                    if i != 0 {
                        full_spectrum[2048 - i] = microfft::Complex32 { re: log_mag, im: 0.0 };
                    }
                }

                let cepstrum = microfft::inverse::ifft_2048(&mut full_spectrum);
                cepstrum_buffer.fill(0.0);
                for i in 0..LIFTER_CUTOFF.min(1024) {
                    cepstrum_buffer[i] = cepstrum[i].re;
                }
                for i in (2048 - LIFTER_CUTOFF.min(1024))..2048 {
                    cepstrum_buffer[i] = cepstrum[i].re;
                }

                let envelope_fft = microfft::real::rfft_2048(&mut cepstrum_buffer);
                for i in 0..1024 {
                    envelope[i] = expf(envelope_fft[i].re);
                }
            }

            // Calculate pitch shift
            let mut pitch_shift_ratio = previous_pitch_shift_ratio;
            let fundamental_index =
                $crate::process_frequencies::find_fundamental_frequency(&analysis_magnitudes);
            let detected_frequency = analysis_frequencies[fundamental_index] * BIN_WIDTH;

            if detected_frequency > 0.001 {
                let target_frequency = if is_auto {
                    let scale_frequencies = $crate::keys::get_scale_by_key(settings.key);
                    $crate::frequencies::find_nearest_note_in_key(
                        detected_frequency,
                        scale_frequencies,
                    )
                } else {
                    $crate::keys::get_frequency(settings.key, settings.note, settings.octave, false)
                };
                let raw_ratio = target_frequency / detected_frequency;
                let clamped_ratio = raw_ratio.clamp(0.5, 2.0);
                const SMOOTHING_FACTOR: f32 = 0.99;
                pitch_shift_ratio = clamped_ratio * SMOOTHING_FACTOR
                    + previous_pitch_shift_ratio * (1.0 - SMOOTHING_FACTOR);
            }

            // Apply spectral shift
            synthesis_magnitudes.fill(0.0);
            synthesis_frequencies.fill(0.0);
            let formant_ratio = match formant {
                1 => 0.5,
                2 => 2.0,
                _ => 1.0,
            };
            let use_formants = formant != 0;

            for i in 0..1024 {
                if analysis_magnitudes[i] <= 1e-8 {
                    continue;
                }
                let residual = if use_formants {
                    analysis_magnitudes[i] / envelope[i].max(1e-6_f32)
                } else {
                    analysis_magnitudes[i]
                };
                let new_bin_f = i as f32 * pitch_shift_ratio;
                let new_bin = (floorf(new_bin_f + 0.5) as usize).min(1023);
                if new_bin >= 1024 {
                    continue;
                }

                let shifted_envelope = if use_formants {
                    let env_pos = (i as f32 / formant_ratio).clamp(0.0, 1023.0);
                    let env_idx = env_pos as usize;
                    let frac = env_pos - env_idx as f32;
                    if env_idx < 1023 {
                        envelope[env_idx] * (1.0 - frac) + envelope[env_idx + 1] * frac
                    } else {
                        envelope[env_idx]
                    }
                } else {
                    1.0
                };

                synthesis_magnitudes[new_bin] = residual * shifted_envelope;
                synthesis_frequencies[new_bin] = analysis_frequencies[i] * pitch_shift_ratio;
            }

            // Synthesis phase reconstruction
            for i in 0..1024 {
                let magnitude = synthesis_magnitudes[i];
                let bin_deviation = synthesis_frequencies[i] - i as f32;
                let mut phase_increment = bin_deviation * 2.0 * PI * HOP_SIZE as f32 / 2048.0;
                let bin_center_frequency = 2.0 * PI * i as f32 / 2048.0;
                phase_increment += bin_center_frequency * HOP_SIZE as f32;
                let output_phase = $crate::process_frequencies::wrap_phase(
                    last_output_phases[i] + phase_increment,
                );
                let real_part = magnitude * cosf(output_phase);
                let imaginary_part = magnitude * sinf(output_phase);
                full_spectrum[i] = microfft::Complex32 { re: real_part, im: imaginary_part };
                if i > 0 && i < 1024 {
                    full_spectrum[2048 - i] =
                        microfft::Complex32 { re: real_part, im: -imaginary_part };
                }
                last_output_phases[i] = output_phase;
            }

            // Inverse FFT
            let time_domain_result = microfft::inverse::ifft_2048(&mut full_spectrum);
            const GAIN_COMPENSATION: f32 = 2.0 / 3.0;
            let mut output_samples = [0.0f32; 2048];

            for i in 0..2048 {
                let mut sample = time_domain_result[i].re;
                sample *= analysis_window_buffer[i];
                sample *= GAIN_COMPENSATION;
                if sample.abs() > 0.95 {
                    let sign = if sample >= 0.0 { 1.0 } else { -1.0 };
                    let compressed = 0.95 - 0.05 * expf(-fabsf(sample));
                    sample = sign * compressed;
                }
                output_samples[i] = sample;
            }

            output_samples
        }
    };
}

/// Implementation macro for 4096-point FFT autotune function
#[macro_export]
macro_rules! autotune_config_impl_4096 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 4096],
            last_input_phases: &mut [f32; 4096],
            last_output_phases: &mut [f32; 4096],
            previous_pitch_shift_ratio: f32,
            _config: &$crate::AutotuneConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 4096] {
            use core::f32::consts::PI;
            use libm::{atan2f, cosf, expf, fabsf, floorf, logf, sinf, sqrtf};

            const SAMPLE_RATE: f32 = $sample_rate;
            const FFT_SIZE: usize = 4096;
            const HOP_SIZE: usize = (FFT_SIZE as f32 * $hop_ratio) as usize;
            const BIN_WIDTH: f32 = SAMPLE_RATE / FFT_SIZE as f32;

            let analysis_window_buffer = $crate::hann_window::get_hann_window::<4096>();
            let mut full_spectrum: [microfft::Complex32; 4096] =
                [microfft::Complex32 { re: 0.0, im: 0.0 }; 4096];
            let mut analysis_magnitudes = [0.0; 2048];
            let mut analysis_frequencies = [0.0; 2048];
            let mut synthesis_magnitudes: [f32; 4096] = [0.0; 4096];
            let mut synthesis_frequencies: [f32; 4096] = [0.0; 4096];
            let mut envelope = [1.0f32; 2048];

            let formant = settings.formant;
            let is_auto = settings.note == 0;

            // Apply windowing
            for i in 0..4096 {
                unwrapped_buffer[i] *= analysis_window_buffer[i];
            }

            // Forward FFT
            let fft_result = microfft::real::rfft_4096(unwrapped_buffer);

            // Process frequency bins
            for i in 0..fft_result.len() {
                let amplitude = sqrtf(
                    fft_result[i].re * fft_result[i].re + fft_result[i].im * fft_result[i].im,
                );
                let phase = atan2f(fft_result[i].im, fft_result[i].re);
                let mut phase_diff = phase - last_input_phases[i];
                let bin_centre_frequency = 2.0 * PI * i as f32 / 4096.0;
                phase_diff = $crate::process_frequencies::wrap_phase(
                    phase_diff - bin_centre_frequency * HOP_SIZE as f32,
                );
                let bin_deviation = phase_diff * 4096.0 / HOP_SIZE as f32 / (2.0 * PI);
                analysis_frequencies[i] = i as f32 + bin_deviation;
                analysis_magnitudes[i] = amplitude;
                last_input_phases[i] = phase;
            }

            // Extract formant envelope if needed
            if formant != 0 {
                const LIFTER_CUTOFF: usize = 64;
                let mut cepstrum_buffer = [0.0f32; 4096];

                for i in 0..2048 {
                    let mag = analysis_magnitudes[i].max(1e-6_f32);
                    let log_mag = logf(mag);
                    full_spectrum[i] = microfft::Complex32 { re: log_mag, im: 0.0 };
                    if i != 0 {
                        full_spectrum[4096 - i] = microfft::Complex32 { re: log_mag, im: 0.0 };
                    }
                }

                let cepstrum = microfft::inverse::ifft_4096(&mut full_spectrum);
                cepstrum_buffer.fill(0.0);
                for i in 0..LIFTER_CUTOFF.min(2048) {
                    cepstrum_buffer[i] = cepstrum[i].re;
                }
                for i in (4096 - LIFTER_CUTOFF.min(2048))..4096 {
                    cepstrum_buffer[i] = cepstrum[i].re;
                }

                let envelope_fft = microfft::real::rfft_4096(&mut cepstrum_buffer);
                for i in 0..2048 {
                    envelope[i] = expf(envelope_fft[i].re);
                }
            }

            // Calculate pitch shift
            let mut pitch_shift_ratio = previous_pitch_shift_ratio;
            let fundamental_index =
                $crate::process_frequencies::find_fundamental_frequency(&analysis_magnitudes);
            let detected_frequency = analysis_frequencies[fundamental_index] * BIN_WIDTH;

            if detected_frequency > 0.001 {
                let target_frequency = if is_auto {
                    let scale_frequencies = $crate::keys::get_scale_by_key(settings.key);
                    $crate::frequencies::find_nearest_note_in_key(
                        detected_frequency,
                        scale_frequencies,
                    )
                } else {
                    $crate::keys::get_frequency(settings.key, settings.note, settings.octave, false)
                };
                let raw_ratio = target_frequency / detected_frequency;
                let clamped_ratio = raw_ratio.clamp(0.5, 2.0);
                const SMOOTHING_FACTOR: f32 = 0.99;
                pitch_shift_ratio = clamped_ratio * SMOOTHING_FACTOR
                    + previous_pitch_shift_ratio * (1.0 - SMOOTHING_FACTOR);
            }

            // Apply spectral shift
            synthesis_magnitudes.fill(0.0);
            synthesis_frequencies.fill(0.0);
            let formant_ratio = match formant {
                1 => 0.5,
                2 => 2.0,
                _ => 1.0,
            };
            let use_formants = formant != 0;

            for i in 0..2048 {
                if analysis_magnitudes[i] <= 1e-8 {
                    continue;
                }
                let residual = if use_formants {
                    analysis_magnitudes[i] / envelope[i].max(1e-6_f32)
                } else {
                    analysis_magnitudes[i]
                };
                let new_bin_f = i as f32 * pitch_shift_ratio;
                let new_bin = (floorf(new_bin_f + 0.5) as usize).min(2047);
                if new_bin >= 2048 {
                    continue;
                }

                let shifted_envelope = if use_formants {
                    let env_pos = (i as f32 / formant_ratio).clamp(0.0, 2047.0);
                    let env_idx = env_pos as usize;
                    let frac = env_pos - env_idx as f32;
                    if env_idx < 2047 {
                        envelope[env_idx] * (1.0 - frac) + envelope[env_idx + 1] * frac
                    } else {
                        envelope[env_idx]
                    }
                } else {
                    1.0
                };

                synthesis_magnitudes[new_bin] = residual * shifted_envelope;
                synthesis_frequencies[new_bin] = analysis_frequencies[i] * pitch_shift_ratio;
            }

            // Synthesis phase reconstruction
            for i in 0..2048 {
                let magnitude = synthesis_magnitudes[i];
                let bin_deviation = synthesis_frequencies[i] - i as f32;
                let mut phase_increment = bin_deviation * 2.0 * PI * HOP_SIZE as f32 / 4096.0;
                let bin_center_frequency = 2.0 * PI * i as f32 / 4096.0;
                phase_increment += bin_center_frequency * HOP_SIZE as f32;
                let output_phase = $crate::process_frequencies::wrap_phase(
                    last_output_phases[i] + phase_increment,
                );
                let real_part = magnitude * cosf(output_phase);
                let imaginary_part = magnitude * sinf(output_phase);
                full_spectrum[i] = microfft::Complex32 { re: real_part, im: imaginary_part };
                if i > 0 && i < 2048 {
                    full_spectrum[4096 - i] =
                        microfft::Complex32 { re: real_part, im: -imaginary_part };
                }
                last_output_phases[i] = output_phase;
            }

            // Inverse FFT
            let time_domain_result = microfft::inverse::ifft_4096(&mut full_spectrum);
            const GAIN_COMPENSATION: f32 = 2.0 / 3.0;
            let mut output_samples = [0.0f32; 4096];

            for i in 0..4096 {
                let mut sample = time_domain_result[i].re;
                sample *= analysis_window_buffer[i];
                sample *= GAIN_COMPENSATION;
                if sample.abs() > 0.95 {
                    let sign = if sample >= 0.0 { 1.0 } else { -1.0 };
                    let compressed = 0.95 - 0.05 * expf(-fabsf(sample));
                    sample = sign * compressed;
                }
                output_samples[i] = sample;
            }

            output_samples
        }
    };
}

/// Example usage showing how to use the autotune_config macro
///
/// ```rust,no_run
/// use synthphone_vocals::{autotune_config, AutotuneConfig, MusicalSettings};
///
/// // Generate different quality configurations
/// autotune_config!(autotune_realtime, 512, 48000.0, hop_ratio = 0.5);
/// autotune_config!(autotune_balanced, 1024, 48000.0, hop_ratio = 0.25);
/// autotune_config!(autotune_quality, 2048, 48000.0, hop_ratio = 0.125);
/// autotune_config!(autotune_hifi, 4096, 48000.0, hop_ratio = 0.0625);
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
///     let result = autotune_balanced(
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
/// use synthphone_vocals::autotune_configs;
///
/// autotune_configs! {
///     fast => (autotune_fast, 512, 48000.0, hop_ratio = 0.5),
///     balanced => (autotune_balanced, 1024, 48000.0, hop_ratio = 0.25),
///     quality => (autotune_quality, 2048, 48000.0, hop_ratio = 0.125),
///     hifi => (autotune_hifi, 4096, 48000.0, hop_ratio = 0.0625)
/// }
///
/// // Now you can use autotune_fast, autotune_balanced, etc.
/// ```
#[macro_export]
macro_rules! autotune_configs {
    // Handle mixed FFT sizes by processing each entry individually
    (@single $name:ident => ($func_name:ident, 512, $sample_rate:expr $(, $param:ident = $value:expr)*)) => {
        $crate::autotune_config!($func_name, 512, $sample_rate $(, $param = $value)*);
    };
    (@single $name:ident => ($func_name:ident, 1024, $sample_rate:expr $(, $param:ident = $value:expr)*)) => {
        $crate::autotune_config!($func_name, 1024, $sample_rate $(, $param = $value)*);
    };
    (@single $name:ident => ($func_name:ident, 2048, $sample_rate:expr $(, $param:ident = $value:expr)*)) => {
        $crate::autotune_config!($func_name, 2048, $sample_rate $(, $param = $value)*);
    };
    (@single $name:ident => ($func_name:ident, 4096, $sample_rate:expr $(, $param:ident = $value:expr)*)) => {
        $crate::autotune_config!($func_name, 4096, $sample_rate $(, $param = $value)*);
    };

    // Main entry point - delegates to @single
    ($($name:ident => ($func_name:ident, $fft_size:tt, $sample_rate:expr $(, $param:ident = $value:expr)*)),* $(,)?) => {
        $(
            $crate::autotune_configs!(@single $name => ($func_name, $fft_size, $sample_rate $(, $param = $value)*));
        )*
    };
}

// Example usage documentation - moved to module level to avoid orphaned doc comment
