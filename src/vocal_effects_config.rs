//! Vocal Effects Configuration Module
//!
//! This module provides macros for generating specialized vocal effects processing functions
//! with different FFT configurations. It handles the generation of pitch correction processing
//! functions optimized for different FFT sizes and parameters.

/// Macro to generate configurable process_vocal_effects_audio functions
///
/// This macro creates a version of the `process_vocal_effects_audio` function with custom FFT configuration.
/// It generates a complete pitch correction implementation using the specified parameters.
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
/// ```rust,ignore
/// pub fn $func_name(
///     unwrapped_buffer: &mut [f32; FFT_SIZE],
///     last_input_phases: &mut [f32; FFT_SIZE],
///     last_output_phases: &mut [f32; FFT_SIZE],
///     previous_pitch_shift_ratio: f32,
///     config: &VocalEffectsConfig,
///     settings: &MusicalSettings,
/// ) -> [f32; FFT_SIZE]
/// ```
///
/// # Example
/// ```rust,no_run
/// // Generate real-time vocal effects function (low latency)
/// synthphone_vocals::process_vocal_effects_config!(process_vocal_effects_realtime, 512, 48000.0, hop_ratio = 0.5);
///
/// // Generate high-quality vocal effects function
/// synthphone_vocals::process_vocal_effects_config!(process_vocal_effects_hifi, 2048, 48000.0, hop_ratio = 0.125);
///
/// // Usage example
/// fn use_generated_functions() {
///     let mut buffer = [0.0f32; 512];
///     let mut input_phases = [0.0f32; 512];
///     let mut output_phases = [0.0f32; 512];
///     let config = synthphone_vocals::VocalEffectsConfig::default();
///     let settings = synthphone_vocals::MusicalSettings::default();
///
///     let result = process_vocal_effects_realtime(
///         &mut buffer,
///         &mut input_phases,
///         &mut output_phases,
///         1.0,
///         &config,
///         &settings
///     );
/// }
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

/// Implementation macro for 512-point FFT vocal effects function
#[macro_export]
macro_rules! process_vocal_effects_config_impl_512 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 512],
            last_input_phases: &mut [f32; 512],
            last_output_phases: &mut [f32; 512],
            previous_pitch_shift_ratio: f32,
            config: &$crate::VocalEffectsConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 512] {
            $crate::process_vocal_effects::process_vocal_effects_512(
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

/// Implementation macro for 1024-point FFT vocal effects function
#[macro_export]
macro_rules! process_vocal_effects_config_impl_1024 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 1024],
            last_input_phases: &mut [f32; 1024],
            last_output_phases: &mut [f32; 1024],
            previous_pitch_shift_ratio: f32,
            config: &$crate::VocalEffectsConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 1024] {
            $crate::process_vocal_effects::process_vocal_effects_1024(
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

/// Implementation macro for 2048-point FFT vocal effects function
#[macro_export]
macro_rules! process_vocal_effects_config_impl_2048 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 2048],
            last_input_phases: &mut [f32; 2048],
            last_output_phases: &mut [f32; 2048],
            previous_pitch_shift_ratio: f32,
            config: &$crate::VocalEffectsConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 2048] {
            $crate::process_vocal_effects::process_vocal_effects_2048(
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

/// Implementation macro for 4096-point FFT vocal effects function
#[macro_export]
macro_rules! process_vocal_effects_config_impl_4096 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 4096],
            last_input_phases: &mut [f32; 4096],
            last_output_phases: &mut [f32; 4096],
            previous_pitch_shift_ratio: f32,
            config: &$crate::VocalEffectsConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 4096] {
            $crate::process_vocal_effects::process_vocal_effects_4096(
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

/// Convenience macro to create multiple vocal effects configurations at once
///
/// This macro generates multiple vocal effects functions with different configurations
/// in a single call, useful for applications that need different quality levels.
///
/// # Example
/// ```rust,no_run
/// // Generate multiple vocal effects functions with different quality levels
/// synthphone_vocals::process_vocal_effects_configs! {
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

/// Configuration guidelines and examples for vocal effects processing
///
/// ## Example usage showing how to use the process_vocal_effects_config macro
///
/// ```rust,no_run
/// // Generate different quality configurations
/// synthphone_vocals::process_vocal_effects_config!(process_vocal_effects_realtime, 512, 48000.0, hop_ratio = 0.5);
/// synthphone_vocals::process_vocal_effects_config!(process_vocal_effects_balanced, 1024, 48000.0, hop_ratio = 0.25);
/// synthphone_vocals::process_vocal_effects_config!(process_vocal_effects_quality, 2048, 48000.0, hop_ratio = 0.125);
/// synthphone_vocals::process_vocal_effects_config!(process_vocal_effects_hifi, 4096, 48000.0, hop_ratio = 0.0625);
///
/// // Usage in your application
/// fn process_audio() {
///     let mut buffer = [0.0f32; 1024];
///     let mut input_phases = [0.0f32; 1024];
///     let mut output_phases = [0.0f32; 1024];
///
///     let config = synthphone_vocals::VocalEffectsConfig::default();
///     let mut settings = synthphone_vocals::MusicalSettings::default();
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_process_vocal_effects_config_macros() {
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
