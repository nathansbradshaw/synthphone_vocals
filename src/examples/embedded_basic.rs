//! Basic embedded usage example

#![no_std]
#![no_main]

use autotune::{
    AutotuneConfig, MusicalSettings,
    embedded::{EmbeddedAutotuneState1024, process_autotune_embedded},
};

fn autotune_example() {
    // Create configuration for embedded system
    let config = AutotuneConfig {
        fft_size: 1024,
        hop_size: 128,
        sample_rate: 48000.0,
        ..Default::default()
    };

    // Create embedded state (uses fixed arrays, no heap allocation)
    let mut state = EmbeddedAutotuneState1024::new(config);

    // Set musical parameters
    let settings = MusicalSettings {
        key: 0,  // C Major
        note: 0, // Auto mode
        octave: 2,
        formant: 0,
    };

    // Process audio buffer
    let input_buffer = [0.0f32; 1024];
    let mut output_buffer = [0.0f32; 1024];

    match process_autotune_embedded(&input_buffer, &mut output_buffer, &mut state, &settings) {
        Ok(()) => {
            // Process output_buffer...
        }
        Err(e) => {
            // Handle error...
        }
    }
}
