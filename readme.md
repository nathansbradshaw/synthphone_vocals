# Synthphone Vocals Library

A real-time pitch correction library designed for both embedded and desktop applications.

## Features

- ✅ Real-time pitch correction using phase vocoder
- ✅ Musical key and scale support (24 major/minor keys)
- ✅ **Truly `no_std`** - Zero heap allocation in embedded mode
- ✅ Both embedded and desktop environments
- ✅ Configurable FFT sizes and processing parameters
- ✅ Optional formant shifting
- ✅ Low-latency processing suitable for real-time audio
- ✅ RTIC integration for ARM Cortex-M microcontrollers
- ✅ Stack-only memory allocation (~16KB total)

## Quick Start

### Embedded Usage (Truly no_std)

```rust
#![no_std]
use synthphone_vocals::{
    AutotuneConfig, MusicalSettings,
    embedded::{EmbeddedAutotuneState1024, process_autotune_embedded}
};

// Create configuration - no heap allocation!
let config = AutotuneConfig {
    fft_size: 1024,
    hop_size: 256,
    sample_rate: 48000.0,
    pitch_correction_strength: 0.8,
    ..Default::default()
};

// Create embedded state - stack allocated only
let mut state = EmbeddedAutotuneState1024::new(config);

// Set musical parameters
let settings = MusicalSettings {
    key: 0, // C Major
    note: 0, // Auto mode (snap to nearest note)
    octave: 2,
    formant: 0, // No formant shifting
};

// Process audio with fixed-size arrays
let input = [0.0f32; 1024];
let mut output = [0.0f32; 1024];

process_autotune_embedded(&input, &mut output, &mut state, &settings)?;
```

### Desktop Usage

```rust
use synthphone_vocals::{AutotuneConfig, AutotuneState, MusicalSettings, process_autotune};

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
```

## Embedded Targets

### Cargo.toml
```toml
[dependencies]
# For embedded/no_std - zero heap allocation
synthphone_vocals = { version = "0.1.0", default-features = false, features = ["embedded"] }

# For desktop/std - with dynamic allocation  
synthphone_vocals = { version = "0.1.0", features = ["std"] }
```

### Supported Platforms
- **Daisy Seed (STM32H750)** - Primary target
- **ARM Cortex-M4F/M7** with FPU support
- **Minimum 64KB RAM** (128KB+ recommended)
- **RTIC 2.0** integration ready

See `EMBEDDED_README.md` for detailed embedded usage guide.
