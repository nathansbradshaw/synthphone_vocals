//! Vocal Effects Configuration Module
//!
//! This module provides macros for generating specialized vocal effects processing functions
//! with different FFT configurations and processing modes. It handles the generation of
//! autotune, vocoder, and dry processing functions optimized for different FFT sizes and parameters.

/// Macro to generate configurable process_vocal_effects_audio functions
///
/// This macro creates a version of the vocal effects processing function with custom FFT
/// configuration and processing mode. It generates a complete implementation using the
/// specified parameters.
///
/// # Arguments
/// - `$func_name`: Name of the generated function
/// - `$fft_size`: FFT size (must be power of 2, between 512-4096)
/// - `$sample_rate`: Sample rate in Hz
/// - `mode`: Processing mode (autotune, vocode, or dry)
/// - `hop_ratio`: Optional hop size as fraction of FFT size (default: 0.25)
///
/// # Generated Function Signature
/// The function signature varies based on the processing mode:
///
/// ## Autotune Mode
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
/// ## Vocode Mode
/// ```rust,ignore
/// pub fn $func_name(
///     input_buffer: &mut [f32; FFT_SIZE],
///     carrier_buffer: &mut [f32; FFT_SIZE],
///     last_input_phases: &mut [f32; FFT_SIZE],
///     last_output_phases: &mut [f32; FFT_SIZE],
///     config: &VocalEffectsConfig,
///     settings: &MusicalSettings,
/// ) -> [f32; FFT_SIZE]
/// ```
///
/// ## Dry Mode
/// ```rust,ignore
/// pub fn $func_name(
///     unwrapped_buffer: &mut [f32; FFT_SIZE],
///     synth_buffer: Option<&mut [f32; FFT_SIZE]>,
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
/// // Generate different processing functions
/// synthphone_vocals::process_vocal_effects_config!(my_autotune, 1024, 48000.0, mode = autotune);
/// synthphone_vocals::process_vocal_effects_config!(my_vocoder, 1024, 48000.0, mode = vocode, hop_ratio = 0.125);
/// synthphone_vocals::process_vocal_effects_config!(my_dry_processor, 2048, 48000.0, mode = dry, hop_ratio = 0.25);
///
/// // Usage example for autotune
/// fn use_autotune() {
///     let mut buffer = [0.0f32; 1024];
///     let mut input_phases = [0.0f32; 1024];
///     let mut output_phases = [0.0f32; 1024];
///     let config = synthphone_vocals::VocalEffectsConfig::default();
///     let settings = synthphone_vocals::MusicalSettings::default();
///
///     let result = my_autotune(
///         &mut buffer,
///         &mut input_phases,
///         &mut output_phases,
///         1.0,
///         &config,
///         &settings
///     );
/// }
///
/// // Usage example for vocoder
/// fn use_vocoder() {
///     let mut vocal_buffer = [0.0f32; 1024];
///     let mut carrier_buffer = [0.0f32; 1024];
///     let mut input_phases = [0.0f32; 1024];
///     let mut output_phases = [0.0f32; 1024];
///     let config = synthphone_vocals::VocalEffectsConfig::default();
///     let settings = synthphone_vocals::MusicalSettings::default();
///
///     let result = my_vocoder(
///         &mut vocal_buffer,
///         &mut carrier_buffer,
///         &mut input_phases,
///         &mut output_phases,
///         &config,
///         &settings
///     );
/// }
/// ```
#[macro_export]
macro_rules! process_vocal_effects_config {
    // Basic autotune mode - 512
    ($func_name:ident, 512, $sample_rate:expr, mode = autotune) => {
        $crate::process_vocal_effects_config_impl_autotune_512!($func_name, $sample_rate, 0.25);
    };
    // Basic autotune mode - 1024
    ($func_name:ident, 1024, $sample_rate:expr, mode = autotune) => {
        $crate::process_vocal_effects_config_impl_autotune_1024!($func_name, $sample_rate, 0.25);
    };
    // Basic autotune mode - 2048
    ($func_name:ident, 2048, $sample_rate:expr, mode = autotune) => {
        $crate::process_vocal_effects_config_impl_autotune_2048!($func_name, $sample_rate, 0.25);
    };
    // Basic autotune mode - 4096
    ($func_name:ident, 4096, $sample_rate:expr, mode = autotune) => {
        $crate::process_vocal_effects_config_impl_autotune_4096!($func_name, $sample_rate, 0.25);
    };

    // Autotune mode with hop ratio - 512
    ($func_name:ident, 512, $sample_rate:expr, mode = autotune, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_autotune_512!(
            $func_name,
            $sample_rate,
            $hop_ratio
        );
    };
    // Autotune mode with hop ratio - 1024
    ($func_name:ident, 1024, $sample_rate:expr, mode = autotune, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_autotune_1024!(
            $func_name,
            $sample_rate,
            $hop_ratio
        );
    };
    // Autotune mode with hop ratio - 2048
    ($func_name:ident, 2048, $sample_rate:expr, mode = autotune, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_autotune_2048!(
            $func_name,
            $sample_rate,
            $hop_ratio
        );
    };
    // Autotune mode with hop ratio - 4096
    ($func_name:ident, 4096, $sample_rate:expr, mode = autotune, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_autotune_4096!(
            $func_name,
            $sample_rate,
            $hop_ratio
        );
    };

    // Basic vocode mode - 512
    ($func_name:ident, 512, $sample_rate:expr, mode = vocode) => {
        $crate::process_vocal_effects_config_impl_vocode_512!($func_name, $sample_rate, 0.25);
    };
    // Basic vocode mode - 1024
    ($func_name:ident, 1024, $sample_rate:expr, mode = vocode) => {
        $crate::process_vocal_effects_config_impl_vocode_1024!($func_name, $sample_rate, 0.25);
    };
    // Basic vocode mode - 2048
    ($func_name:ident, 2048, $sample_rate:expr, mode = vocode) => {
        $crate::process_vocal_effects_config_impl_vocode_2048!($func_name, $sample_rate, 0.25);
    };
    // Basic vocode mode - 4096
    ($func_name:ident, 4096, $sample_rate:expr, mode = vocode) => {
        $crate::process_vocal_effects_config_impl_vocode_4096!($func_name, $sample_rate, 0.25);
    };

    // Vocode mode with hop ratio - 512
    ($func_name:ident, 512, $sample_rate:expr, mode = vocode, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_vocode_512!($func_name, $sample_rate, $hop_ratio);
    };
    // Vocode mode with hop ratio - 1024
    ($func_name:ident, 1024, $sample_rate:expr, mode = vocode, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_vocode_1024!(
            $func_name,
            $sample_rate,
            $hop_ratio
        );
    };
    // Vocode mode with hop ratio - 2048
    ($func_name:ident, 2048, $sample_rate:expr, mode = vocode, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_vocode_2048!(
            $func_name,
            $sample_rate,
            $hop_ratio
        );
    };
    // Vocode mode with hop ratio - 4096
    ($func_name:ident, 4096, $sample_rate:expr, mode = vocode, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_vocode_4096!(
            $func_name,
            $sample_rate,
            $hop_ratio
        );
    };

    // Basic dry mode - 512
    ($func_name:ident, 512, $sample_rate:expr, mode = dry) => {
        $crate::process_vocal_effects_config_impl_dry_512!($func_name, $sample_rate, 0.25);
    };
    // Basic dry mode - 1024
    ($func_name:ident, 1024, $sample_rate:expr, mode = dry) => {
        $crate::process_vocal_effects_config_impl_dry_1024!($func_name, $sample_rate, 0.25);
    };
    // Basic dry mode - 2048
    ($func_name:ident, 2048, $sample_rate:expr, mode = dry) => {
        $crate::process_vocal_effects_config_impl_dry_2048!($func_name, $sample_rate, 0.25);
    };
    // Basic dry mode - 4096
    ($func_name:ident, 4096, $sample_rate:expr, mode = dry) => {
        $crate::process_vocal_effects_config_impl_dry_4096!($func_name, $sample_rate, 0.25);
    };

    // Dry mode with hop ratio - 512
    ($func_name:ident, 512, $sample_rate:expr, mode = dry, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_dry_512!($func_name, $sample_rate, $hop_ratio);
    };
    // Dry mode with hop ratio - 1024
    ($func_name:ident, 1024, $sample_rate:expr, mode = dry, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_dry_1024!($func_name, $sample_rate, $hop_ratio);
    };
    // Dry mode with hop ratio - 2048
    ($func_name:ident, 2048, $sample_rate:expr, mode = dry, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_dry_2048!($func_name, $sample_rate, $hop_ratio);
    };
    // Dry mode with hop ratio - 4096
    ($func_name:ident, 4096, $sample_rate:expr, mode = dry, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_dry_4096!($func_name, $sample_rate, $hop_ratio);
    };

    // Legacy compatibility - treat no mode as autotune mode
    // Basic version with just FFT size and sample rate - 512
    ($func_name:ident, 512, $sample_rate:expr) => {
        $crate::process_vocal_effects_config_impl_autotune_512!($func_name, $sample_rate, 0.25);
    };
    // Basic version with just FFT size and sample rate - 1024
    ($func_name:ident, 1024, $sample_rate:expr) => {
        $crate::process_vocal_effects_config_impl_autotune_1024!($func_name, $sample_rate, 0.25);
    };
    // Basic version with just FFT size and sample rate - 2048
    ($func_name:ident, 2048, $sample_rate:expr) => {
        $crate::process_vocal_effects_config_impl_autotune_2048!($func_name, $sample_rate, 0.25);
    };
    // Basic version with just FFT size and sample rate - 4096
    ($func_name:ident, 4096, $sample_rate:expr) => {
        $crate::process_vocal_effects_config_impl_autotune_4096!($func_name, $sample_rate, 0.25);
    };

    // Legacy compatibility - treat hop_ratio without mode as autotune
    // Version with hop ratio - 512
    ($func_name:ident, 512, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_autotune_512!(
            $func_name,
            $sample_rate,
            $hop_ratio
        );
    };
    // Version with hop ratio - 1024
    ($func_name:ident, 1024, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_autotune_1024!(
            $func_name,
            $sample_rate,
            $hop_ratio
        );
    };
    // Version with hop ratio - 2048
    ($func_name:ident, 2048, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_autotune_2048!(
            $func_name,
            $sample_rate,
            $hop_ratio
        );
    };
    // Version with hop ratio - 4096
    ($func_name:ident, 4096, $sample_rate:expr, hop_ratio = $hop_ratio:expr) => {
        $crate::process_vocal_effects_config_impl_autotune_4096!(
            $func_name,
            $sample_rate,
            $hop_ratio
        );
    };
}

/// Implementation macro for 512-point FFT autotune function
#[macro_export]
macro_rules! process_vocal_effects_config_impl_autotune_512 {
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

/// Implementation macro for 1024-point FFT autotune function
#[macro_export]
macro_rules! process_vocal_effects_config_impl_autotune_1024 {
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

/// Implementation macro for 2048-point FFT autotune function
#[macro_export]
macro_rules! process_vocal_effects_config_impl_autotune_2048 {
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

/// Implementation macro for 4096-point FFT autotune function
#[macro_export]
macro_rules! process_vocal_effects_config_impl_autotune_4096 {
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

/// Implementation macros for vocode functions
#[macro_export]
macro_rules! process_vocal_effects_config_impl_vocode_512 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            input_buffer: &mut [f32; 512],
            carrier_buffer: &mut [f32; 512],
            last_input_phases: &mut [f32; 512],
            last_output_phases: &mut [f32; 512],
            config: &$crate::VocalEffectsConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 512] {
            $crate::process_vocal_effects::process_vocode_512(
                input_buffer,
                carrier_buffer,
                last_input_phases,
                last_output_phases,
                config,
                settings,
                $sample_rate,
                $hop_ratio,
            )
        }
    };
}

#[macro_export]
macro_rules! process_vocal_effects_config_impl_vocode_1024 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            input_buffer: &mut [f32; 1024],
            carrier_buffer: &mut [f32; 1024],
            last_input_phases: &mut [f32; 1024],
            last_output_phases: &mut [f32; 1024],
            config: &$crate::VocalEffectsConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 1024] {
            $crate::process_vocal_effects::process_vocode_1024(
                input_buffer,
                carrier_buffer,
                last_input_phases,
                last_output_phases,
                config,
                settings,
                $sample_rate,
                $hop_ratio,
            )
        }
    };
}

#[macro_export]
macro_rules! process_vocal_effects_config_impl_vocode_2048 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            input_buffer: &mut [f32; 2048],
            carrier_buffer: &mut [f32; 2048],
            last_input_phases: &mut [f32; 2048],
            last_output_phases: &mut [f32; 2048],
            config: &$crate::VocalEffectsConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 2048] {
            $crate::process_vocal_effects::process_vocode_2048(
                input_buffer,
                carrier_buffer,
                last_input_phases,
                last_output_phases,
                config,
                settings,
                $sample_rate,
                $hop_ratio,
            )
        }
    };
}

#[macro_export]
macro_rules! process_vocal_effects_config_impl_vocode_4096 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            input_buffer: &mut [f32; 4096],
            carrier_buffer: &mut [f32; 4096],
            last_input_phases: &mut [f32; 4096],
            last_output_phases: &mut [f32; 4096],
            config: &$crate::VocalEffectsConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 4096] {
            $crate::process_vocal_effects::process_vocode_4096(
                input_buffer,
                carrier_buffer,
                last_input_phases,
                last_output_phases,
                config,
                settings,
                $sample_rate,
                $hop_ratio,
            )
        }
    };
}

/// Implementation macros for dry functions
#[macro_export]
macro_rules! process_vocal_effects_config_impl_dry_512 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 512],
            synth_buffer: Option<&mut [f32; 512]>,
            last_input_phases: &mut [f32; 512],
            last_output_phases: &mut [f32; 512],
            previous_pitch_shift_ratio: f32,
            config: &$crate::VocalEffectsConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 512] {
            $crate::process_vocal_effects::process_dry_512(
                unwrapped_buffer,
                synth_buffer,
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

#[macro_export]
macro_rules! process_vocal_effects_config_impl_dry_1024 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 1024],
            synth_buffer: Option<&mut [f32; 1024]>,
            last_input_phases: &mut [f32; 1024],
            last_output_phases: &mut [f32; 1024],
            previous_pitch_shift_ratio: f32,
            config: &$crate::VocalEffectsConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 1024] {
            $crate::process_vocal_effects::process_dry_1024(
                unwrapped_buffer,
                synth_buffer,
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

#[macro_export]
macro_rules! process_vocal_effects_config_impl_dry_2048 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 2048],
            synth_buffer: Option<&mut [f32; 2048]>,
            last_input_phases: &mut [f32; 2048],
            last_output_phases: &mut [f32; 2048],
            previous_pitch_shift_ratio: f32,
            config: &$crate::VocalEffectsConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 2048] {
            $crate::process_vocal_effects::process_dry_2048(
                unwrapped_buffer,
                synth_buffer,
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

#[macro_export]
macro_rules! process_vocal_effects_config_impl_dry_4096 {
    ($func_name:ident, $sample_rate:expr, $hop_ratio:expr) => {
        pub fn $func_name(
            unwrapped_buffer: &mut [f32; 4096],
            synth_buffer: Option<&mut [f32; 4096]>,
            last_input_phases: &mut [f32; 4096],
            last_output_phases: &mut [f32; 4096],
            previous_pitch_shift_ratio: f32,
            config: &$crate::VocalEffectsConfig,
            settings: &$crate::MusicalSettings,
        ) -> [f32; 4096] {
            $crate::process_vocal_effects::process_dry_4096(
                unwrapped_buffer,
                synth_buffer,
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
/// // Generate multiple vocal effects functions with different modes and quality levels
/// synthphone_vocals::process_vocal_effects_configs! {
///     autotune_fast => (process_autotune_fast, 512, 48000.0, mode = autotune, hop_ratio = 0.5),
///     autotune_balanced => (process_autotune_balanced, 1024, 48000.0, mode = autotune, hop_ratio = 0.25),
///     vocoder_quality => (process_vocoder_quality, 2048, 48000.0, mode = vocode, hop_ratio = 0.125),
///     dry_hifi => (process_dry_hifi, 4096, 48000.0, mode = dry, hop_ratio = 0.0625)
/// }
///
/// // Now you can use process_autotune_fast, process_vocoder_quality, etc.
/// ```
#[macro_export]
macro_rules! process_vocal_effects_configs {
    // Handle mixed FFT sizes and modes by processing each entry individually
    (@single $name:ident => ($func_name:ident, 512, $sample_rate:expr, mode = $mode:tt $(, $param:ident = $value:tt)*)) => {
        $crate::process_vocal_effects_config!($func_name, 512, $sample_rate, mode = $mode $(, $param = $value)*);
    };
    (@single $name:ident => ($func_name:ident, 1024, $sample_rate:expr, mode = $mode:tt $(, $param:ident = $value:tt)*)) => {
        $crate::process_vocal_effects_config!($func_name, 1024, $sample_rate, mode = $mode $(, $param = $value)*);
    };
    (@single $name:ident => ($func_name:ident, 2048, $sample_rate:expr, mode = $mode:tt $(, $param:ident = $value:tt)*)) => {
        $crate::process_vocal_effects_config!($func_name, 2048, $sample_rate, mode = $mode $(, $param = $value)*);
    };
    (@single $name:ident => ($func_name:ident, 4096, $sample_rate:expr, mode = $mode:tt $(, $param:ident = $value:tt)*)) => {
        $crate::process_vocal_effects_config!($func_name, 4096, $sample_rate, mode = $mode $(, $param = $value)*);
    };

    // Legacy compatibility - without mode specification (defaults to autotune)
    (@single $name:ident => ($func_name:ident, 512, $sample_rate:expr $(, $param:ident = $value:tt)*)) => {
        $crate::process_vocal_effects_config!($func_name, 512, $sample_rate $(, $param = $value)*);
    };
    (@single $name:ident => ($func_name:ident, 1024, $sample_rate:expr $(, $param:ident = $value:tt)*)) => {
        $crate::process_vocal_effects_config!($func_name, 1024, $sample_rate $(, $param = $value)*);
    };
    (@single $name:ident => ($func_name:ident, 2048, $sample_rate:expr $(, $param:ident = $value:tt)*)) => {
        $crate::process_vocal_effects_config!($func_name, 2048, $sample_rate $(, $param = $value)*);
    };
    (@single $name:ident => ($func_name:ident, 4096, $sample_rate:expr $(, $param:ident = $value:tt)*)) => {
        $crate::process_vocal_effects_config!($func_name, 4096, $sample_rate $(, $param = $value)*);
    };

    // Main entry point - delegates to @single
    ($($name:ident => ($func_name:ident, $fft_size:tt, $sample_rate:expr $(, $param:ident = $value:tt)*)),* $(,)?) => {
        $(
            $crate::process_vocal_effects_configs!(@single $name => ($func_name, $fft_size, $sample_rate $(, $param = $value)*));
        )*
    };
}

/// Configuration guidelines and examples for vocal effects processing
///
/// ## Example usage showing how to use the unified process_vocal_effects_config macro
///
/// ```rust,no_run
/// // Generate different processing mode functions
/// synthphone_vocals::process_vocal_effects_config!(my_autotune, 1024, 48000.0, mode = autotune);
/// synthphone_vocals::process_vocal_effects_config!(my_vocoder, 1024, 48000.0, mode = vocode, hop_ratio = 0.125);
/// synthphone_vocals::process_vocal_effects_config!(my_dry_processor, 2048, 48000.0, mode = dry, hop_ratio = 0.25);
///
/// // Generate multiple configurations at once
/// synthphone_vocals::process_vocal_effects_configs! {
///     autotune_fast => (process_autotune_fast, 512, 48000.0, mode = autotune, hop_ratio = 0.5),
///     autotune_balanced => (process_autotune_balanced, 1024, 48000.0, mode = autotune, hop_ratio = 0.25),
///     vocoder_quality => (process_vocoder_quality, 2048, 48000.0, mode = vocode, hop_ratio = 0.125),
///     dry_hifi => (process_dry_hifi, 4096, 48000.0, mode = dry, hop_ratio = 0.0625)
/// }
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
///     // Process with autotune
///     let result = my_autotune(
///         &mut buffer,
///         &mut input_phases,
///         &mut output_phases,
///         1.0, // Previous pitch shift ratio
///         &config,
///         &settings
///     );
///
///     // Process with vocoder
///     let mut vocal_buffer = [0.0f32; 1024];
///     let mut carrier_buffer = [0.0f32; 1024]; // Your synthesizer signal
///     let vocode_result = my_vocoder(
///         &mut vocal_buffer,
///         &mut carrier_buffer,
///         &mut input_phases,
///         &mut output_phases,
///         &config,
///         &settings
///     );
///
///     // Process with dry mode
///     let synth_buffer = Some(&mut [0.0f32; 2048]);
///     let dry_result = my_dry_processor(
///         &mut [0.0f32; 2048],
///         synth_buffer,
///         &mut [0.0f32; 2048],
///         &mut [0.0f32; 2048],
///         1.0,
///         &config,
///         &settings
///     );
/// }
/// ```
///
/// ## Configuration Guidelines
///
/// ### Processing Modes
/// - **autotune**: Pitch correction to musical notes with formant preservation
/// - **vocode**: Applies vocal formant envelope to carrier signal (classic vocoder)
/// - **dry**: Pitch shifting and formant control without pitch correction
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
mod tests {}
