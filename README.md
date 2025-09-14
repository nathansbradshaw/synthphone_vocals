# Synthphone Vocals

[![Crates.io](https://img.shields.io/crates/v/synthphone_vocals.svg)](https://crates.io/crates/synthphone_vocals)
[![Documentation](https://docs.rs/synthphone_vocals/badge.svg)](https://docs.rs/synthphone_vocals)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, real-time vocal effects library for embedded and desktop applications. Built with `no_std` support and optimized for minimal latency audio processing.

## ✨ Features

- **🎵 Real-time Autotune**: Phase vocoder-based pitch correction with musical key awareness
- **⚡ Ultra-low Latency**: Configurable FFT sizes from 512 to 4096 samples
- **🎛️ Formant Processing**: Optional formant preservation and shifting
- **🎹 Musical Intelligence**: Support for all 12 musical keys and scales
- **🔧 Embedded Ready**: `no_std` compatible with ARM Cortex-M support
- **📊 Flexible Configuration**: Dynamic FFT setup with compile-time validation
- **🎯 Zero-allocation**: Lock-free ring buffers and static memory usage
- **📈 Performance Oriented**: SIMD-friendly algorithms and efficient memory layouts

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
synthphone_vocals = "0.1.1"

# For embedded applications
synthphone_vocals = { version = "0.1.1", default-features = false, features = ["embedded"] }

# For desktop with all features
synthphone_vocals = { version = "0.1.1", features = ["std", "formant-shifting"] }
```

### Basic Usage

```rust
use synthphone_vocals::{
    AutotuneConfig, MusicalSettings, 
    process_vocal_effects_config
};

// Generate an optimized autotune function for your use case
process_vocal_effects_config!(
    process_vocals_realtime,  // Function name
    1024,                     // FFT size (latency vs quality trade-off)
    48000.0,                  // Sample rate
    hop_ratio = 0.25         // Overlap ratio (quality vs CPU)
);

fn main() {
    // Initialize configuration
    let config = AutotuneConfig::default();
    let mut settings = MusicalSettings::default();
    settings.key = 0;    // C major
    settings.note = 0;   // Auto-detect mode
    
    // Process audio buffers
    let mut audio_buffer = [0.0f32; 1024];
    let mut input_phases = [0.0f32; 1024];
    let mut output_phases = [0.0f32; 1024];
    
    // Fill audio_buffer with input samples...
    
    let processed = process_vocals_realtime(
        &mut audio_buffer,
        &mut input_phases,
        &mut output_phases,
        1.0, // Previous pitch shift ratio
        &config,
        &settings
    );
    
    // Use processed audio...
}
```

## 🏗️ Architecture

The library is organized into focused modules for maintainability and performance:

### Core Processing Modules

- **`process_vocal_effects`** - Main autotune processing engine
- **`process_frequencies`** - Fundamental frequency detection and analysis  
- **`frequencies`** - Musical note frequency calculations and mappings
- **`keys`** - Musical key and scale definitions

### Configuration & Setup

- **`fft_config`** - FFT parameter configuration and validation
- **`vocal_effects_config`** - Specialized vocal processing function generation
- **`config`** - Runtime configuration structures
- **`state`** - Musical settings and state management

### Audio Infrastructure

- **`ring_buffer`** - Lock-free circular buffers for real-time audio
- **`hann_window`** - Windowing functions for spectral analysis
- **`oscillator`** - Signal generation utilities

### Platform Support

- **`embedded`** - ARM Cortex-M specific optimizations
- **`utils`** - Cross-platform utilities and helpers

## ⚙️ Configuration Options

### FFT Size vs Performance Trade-offs

| FFT Size | Latency @ 48kHz | CPU Usage | Quality | Best For |
|----------|----------------|-----------|---------|----------|
| 512      | 10.7ms         | Very Low  | Basic   | Live performance |
| 1024     | 21.3ms         | Low       | Good    | Real-time apps |
| 2048     | 42.7ms         | Medium    | High    | Studio recording |
| 4096     | 85.3ms         | High      | Excellent| Post-processing |

### Hop Ratio Settings

- **0.0625** (1/16): Highest quality, 93.75% overlap, most CPU intensive
- **0.125** (1/8): Very high quality, 87.5% overlap, high CPU usage  
- **0.25** (1/4): Good quality, 75% overlap, moderate CPU usage (default)
- **0.5** (1/2): Lower quality, 50% overlap, lowest CPU usage

### Example Configurations

```rust
// Ultra-low latency for live performance
process_vocal_effects_config!(live_autotune, 512, 48000.0, hop_ratio = 0.5);

// Balanced for real-time applications  
process_vocal_effects_config!(realtime_autotune, 1024, 48000.0, hop_ratio = 0.25);

// High quality for recording
process_vocal_effects_config!(studio_autotune, 2048, 48000.0, hop_ratio = 0.125);

// Maximum quality for post-processing
process_vocal_effects_config!(offline_autotune, 4096, 48000.0, hop_ratio = 0.0625);
```

## 🎹 Musical Features

### Supported Keys and Modes

```rust
use synthphone_vocals::{MusicalSettings, get_key_name};

let mut settings = MusicalSettings::default();

// Major keys (0-11)
settings.key = 0;  // C major
settings.key = 7;  // G major

// Minor keys (12-23) 
settings.key = 12; // A minor
settings.key = 19; // E minor

println!("Current key: {}", get_key_name(settings.key));
```

### Note Targeting

```rust
// Automatic pitch detection and correction
settings.note = 0;  // Auto mode (default)

// Lock to specific notes (1-12)
settings.note = 1;  // Lock to C
settings.note = 8;  // Lock to G
```

### Formant Processing

```rust
// Preserve natural voice character
settings.formant = 0;  // No formant processing (fastest)
settings.formant = 1;  // Preserve formants (natural sound)
settings.formant = 2;  // Shift formants (voice character effects)
```

## 🔧 Embedded Usage

For ARM Cortex-M and other embedded platforms:

```toml
[dependencies]
synthphone_vocals = { 
    version = "0.1.1", 
    default-features = false, 
    features = ["embedded"] 
}
```

```rust
#![no_std]
#![no_main]

use synthphone_vocals::{fft_config, embedded::process_audio_block_embedded};
use cortex_m_rt::entry;

// Configure for embedded constraints
fft_config!(512, 48000.0, hop_ratio = 0.5, buffer_multiplier = 2);

#[entry]
fn main() -> ! {
    // Initialize your audio hardware...
    
    let config = AutotuneConfig::default();
    let settings = MusicalSettings::default();
    
    loop {
        // Get audio from ADC/I2S...
        let mut audio_block = [0.0f32; BLOCK_SIZE];
        
        let processed = process_audio_block_embedded(
            &audio_block,
            &config,
            &settings
        );
        
        // Send to DAC/I2S...
    }
}
```

## 📊 Performance Optimization

### Memory Usage Optimization

Configure `microfft` features for optimal memory usage:

```toml
[dependencies.microfft]
default-features = false
features = ["size-1024"]  # Match your maximum FFT size
```

Available features: `size-512`, `size-1024`, `size-2048`, `size-4096`

### Real-time Guidelines

1. **Choose appropriate FFT size** based on latency requirements
2. **Use power-of-2 buffer sizes** for optimal performance
3. **Enable formant processing only when needed** (adds ~20% CPU overhead)
4. **Consider hop ratio trade-offs** between quality and CPU usage
5. **Profile on target hardware** to validate real-time performance

## 🛠️ Advanced Configuration

### Multiple Quality Levels

```rust
use synthphone_vocals::process_vocal_effects_configs;

// Generate multiple configurations at once
process_vocal_effects_configs! {
    fast => (process_fast, 512, 48000.0, hop_ratio = 0.5),
    balanced => (process_balanced, 1024, 48000.0, hop_ratio = 0.25),
    quality => (process_quality, 2048, 48000.0, hop_ratio = 0.125),
    premium => (process_premium, 4096, 48000.0, hop_ratio = 0.0625)
}

// Use based on performance requirements
let result = match quality_level {
    QualityLevel::Fast => process_fast(&mut buffer, ...),
    QualityLevel::Balanced => process_balanced(&mut buffer, ...),
    QualityLevel::Quality => process_quality(&mut buffer, ...),
    QualityLevel::Premium => process_premium(&mut buffer, ...),
};
```

### Runtime Configuration

```rust
use synthphone_vocals::fft_config::{FFTConfig, validate_config};

// Runtime validation and configuration
let config = FFTConfig::new(1024, 48000.0, 0.25, 4, 2)?;

println!("Buffer size: {}", config.buffer_size());
println!("Hop size: {}", config.hop_size());
println!("Frequency resolution: {:.2} Hz", config.bin_width());
```

## 📚 Examples

Check the `examples/` directory for complete applications:

- **`basic_autotune.rs`** - Simple desktop autotune application
- **`embedded_demo.rs`** - ARM Cortex-M4 real-time processing
- **`multi_quality.rs`** - Dynamic quality switching
- **`musical_modes.rs`** - Key and scale demonstration

## 🔬 Testing

Run the test suite:

```bash
# All tests
cargo test

# Documentation tests
cargo test --doc  

# Embedded target tests (requires target)
cargo test --target thumbv7em-none-eabihf --features embedded
```

## 🎯 Roadmap

- [ ] **SIMD Acceleration**: AVX2/NEON optimizations for desktop/mobile
- [ ] **Additional Effects**: Reverb, chorus, and delay modules  
- [ ] **Real-time Analysis**: Pitch tracking and formant visualization
- [ ] **Machine Learning**: Neural network-based pitch correction
- [ ] **Multi-channel**: Stereo and surround sound processing
- [ ] **Plugin Formats**: VST3/AU/LV2 plugin wrappers

## 🤝 Contributing

Contributions are welcome! Please see our [contributing guidelines](CONTRIBUTING.md).

### Development Setup

```bash
git clone https://github.com/yourusername/synthphone_vocals
cd synthphone_vocals
cargo test --all-features
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **microfft** - Fast FFT implementation for embedded systems
- **cortex-m** - ARM Cortex-M runtime and peripherals
- The Rust audio community for inspiration and feedback

---

**Built with ❤️ in Rust for the audio community**