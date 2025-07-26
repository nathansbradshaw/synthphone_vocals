# Autotune Library

A real-time autotune library designed for both embedded and desktop applications.

## Features

- ✅ Real-time pitch correction using phase vocoder
- ✅ Musical key and scale support (24 major/minor keys)
- ✅ Both embedded (`no_std`) and desktop environments
- ✅ Configurable FFT sizes and processing parameters
- ✅ Optional formant shifting
- ✅ Low-latency processing suitable for real-time audio

## Quick Start

### Desktop Usage

```rust
use autotune::{AutotuneConfig, AutotuneState, MusicalSettings, process_autotune};

// Create configuration
let config = AutotuneConfig {
    fft_size: 1024,
    hop_size: 256,
    sample_rate: 44100.0,
    pitch_correction_strength: 0.8,
    ..Default::default()
};

// Create state
let mut state = AutotuneState::new(config);

// Set musical parameters
let settings = MusicalSettings {
    key: 0, // C Major
    note: 0, // Auto mode (snap to nearest note)
    octave: 2,
    formant: 0, // No formant shifting
};

// Process audio
let input = vec![0.0f32; 1024];
let mut output = vec![0.0f32; 1024];

process_autotune(&input, &mut output, &mut state, &settings)?;
