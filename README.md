# Synthphone Vocals

<!--[![Crates.io](https://img.shields.io/crates/v/synthphone_e_vocal_dsp.svg)]()-->
<!--[![Documentation](https://docs.rs/synthphone_e_vocal_dsp/badge.svg)]()-->
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, real-time vocal effects library for embedded and desktop applications. Features pitch correction, vocoding, formant shifting, and musical key awareness with `no_std` support and optimized for minimal latency audio processing.

## ✨ Features

- **🎵 Real-time Pitch Correction**: Phase vocoder-based vocal processing with musical key awareness
- **🎤 Vocoder Effects**: Apply vocal formants to carrier signals for classic vocoder sounds
- **⚡ Ultra-low Latency**: Configurable FFT sizes from 512 to 4096 samples
- **🎛️ Formant Processing**: Cepstral-based formant preservation and shifting
- **🎹 Musical Intelligence**: Support for all 12 major and minor keys with automatic scale detection
- **🔧 Embedded Ready**: `no_std` compatible with ARM Cortex-M support
- **📊 Flexible Configuration**: Dynamic FFT setup with compile-time validation
- **🎯 Zero-allocation**: Lock-free ring buffers and static memory usage
- **📈 Performance Oriented**: Generic algorithms optimized for different FFT sizes

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
synthphone_e_vocal_dsp = "0.0.1"

# For embedded applications
synthphone_e_vocal_dsp = { version = "0.1.1", default-features = false, features = ["embedded"] }
```

### Basic Usage

```rust
use synthphone-e-vocal_dsp::{
    VocalEffectsConfig, MusicalSettings, ProcessingMode,
    process_vocal_effects_1024
};

fn main() {
    // Initialize configuration
    let config = VocalEffectsConfig::default();
    let mut settings = MusicalSettings::default();
    settings.key = 0;    // C major
    settings.note = 0;   // Auto-detect mode
    settings.mode = ProcessingMode::Autotune;

    // Process audio buffers
    let mut audio_buffer = [0.0f32; 1024];
    let mut input_phases = [0.0f32; 1024];
    let mut output_phases = [0.0f32; 1024];

    // Fill audio_buffer with input samples...

    let processed = process_vocal_effects_1024(
        &mut audio_buffer,
        None, // No carrier buffer for autotune mode
        &mut input_phases,
        &mut output_phases,
        1.0, // Previous pitch shift ratio
        &config,
        &settings
    );

    // Use processed audio...
}
```
