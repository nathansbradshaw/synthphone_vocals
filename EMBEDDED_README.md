# Embedded Usage Guide

This guide explains how to use the `synthphone_vocals` library in embedded (no_std) environments, specifically with RTIC on ARM Cortex-M microcontrollers like the Daisy Seed.

## Features

- ✅ **Truly no_std** - No heap allocation or global allocator required
- ✅ **Real-time processing** - Designed for audio interrupt contexts
- ✅ **Fixed memory usage** - All buffers are stack-allocated at compile time
- ✅ **Low latency** - ~21ms latency at 48kHz (1024 samples)
- ✅ **RTIC integration** - Resource sharing with proper locking

## Memory Requirements

The embedded implementation uses only stack-allocated memory:

- `EmbeddedAutotuneState1024`: ~8KB
- Processing buffers: ~8KB additional during processing
- **Total**: ~16KB stack memory

Ensure your microcontroller has sufficient RAM and stack space.

## Quick Start

### 1. Add to Cargo.toml

```toml
[dependencies]
synthphone_vocals = { version = "0.1.0", default-features = false, features = ["embedded"] }
```

**Important**: Use `default-features = false` to disable std library support.

### 2. Basic RTIC Integration

```rust
#![no_main]
#![no_std]

use rtic::app;
use synthphone_vocals::{
    AutotuneConfig, MusicalSettings,
    embedded::{EmbeddedAutotuneState1024, process_autotune_embedded},
};

#[app(device = stm32h7xx_hal::stm32, peripherals = true)]
mod app {
    use super::*;
    
    #[shared]
    struct Shared {
        autotune_state: EmbeddedAutotuneState1024,
        musical_settings: MusicalSettings,
    }

    #[local]
    struct Local {
        input_buffer: [f32; 1024],
        output_buffer: [f32; 1024],
        buffer_index: usize,
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        let config = AutotuneConfig {
            fft_size: 1024,
            hop_size: 256,
            sample_rate: 48000.0,
            pitch_correction_strength: 0.8,
            transition_speed: 0.1,
            ..Default::default()
        };

        let settings = MusicalSettings {
            key: 0,    // C Major
            note: 0,   // Auto mode
            octave: 2,
            formant: 0,
        };

        (
            Shared {
                autotune_state: EmbeddedAutotuneState1024::new(config),
                musical_settings: settings,
            },
            Local {
                input_buffer: [0.0; 1024],
                output_buffer: [0.0; 1024],
                buffer_index: 0,
            },
            init::Monotonics(),
        )
    }

    #[task(
        binds = DMA1_STR1,  // Your audio interrupt
        local = [input_buffer, output_buffer, buffer_index],
        shared = [autotune_state, musical_settings],
        priority = 8
    )]
    fn audio_handler(mut ctx: audio_handler::Context) {
        // Your audio processing code here
        // See examples/daisy_seed_example.rs for complete implementation
    }
}
```

## Supported Platforms

### Tested Platforms
- **Daisy Seed (STM32H750)** - Primary development platform
- **STM32H7 series** - Should work on all variants with sufficient RAM

### Requirements
- **ARM Cortex-M4F/M7** with FPU (floating-point operations)
- **Minimum 64KB RAM** (128KB+ recommended)
- **Audio peripheral** (I2S, SAI, etc.)
- **RTIC 2.0** for resource management

### Potential Platforms
- STM32F4 series (with sufficient RAM)
- STM32F7 series
- ESP32 (with appropriate feature flags)
- Nordic nRF52 series (may need reduced buffer sizes)

## Configuration

### Musical Settings

```rust
let settings = MusicalSettings {
    key: 0,     // Musical key (0-23, see keys module)
    note: 0,    // 0 = auto mode, 1-9 = specific note in scale
    octave: 2,  // Octave setting
    formant: 0, // Formant shift (0 = none, 1 = lower, 2 = higher)
};
```

### Performance Settings

```rust
let config = AutotuneConfig {
    fft_size: 1024,                    // Must be 1024 for embedded
    hop_size: 256,                     // Overlap amount
    sample_rate: 48000.0,              // Your audio sample rate
    pitch_correction_strength: 0.8,    // 0.0 = no correction, 1.0 = full correction
    transition_speed: 0.1,             // Speed of pitch changes (0.0-1.0)
    ..Default::default()
};
```

## Real-time Considerations

### Latency
- **Buffer size**: 1024 samples
- **Latency at 48kHz**: ~21ms
- **Latency at 44.1kHz**: ~23ms

### CPU Usage
- Approximately **15-25%** on STM32H750 at 400MHz
- Leaves plenty of headroom for other processing

### Memory Layout
- All processing uses **stack allocation only**
- No dynamic memory allocation
- Predictable memory usage pattern

## Advanced Usage

### Dynamic Settings Changes

```rust
// Task to change musical key
#[task(shared = [musical_settings], priority = 2)]
fn change_key(mut ctx: change_key::Context, new_key: i32) {
    ctx.shared.musical_settings.lock(|settings| {
        settings.key = new_key;
    });
}

// Usage: change_key::spawn(5).ok(); // Change to F Major
```

### Error Handling

```rust
let result = process_autotune_embedded(
    &input_buffer,
    &mut output_buffer,
    &mut autotune_state,
    &settings,
);

match result {
    Ok(()) => {
        // Process successful, use output_buffer
    },
    Err(_) => {
        // Error occurred, fall back to passthrough
        output_buffer.copy_from_slice(&input_buffer);
    }
}
```

### Multi-channel Processing

For stereo processing, create separate state instances:

```rust
#[shared]
struct Shared {
    left_autotune: EmbeddedAutotuneState1024,
    right_autotune: EmbeddedAutotuneState1024,
    settings: MusicalSettings,
}
```

## Troubleshooting

### Compilation Issues

**Error**: "no global memory allocator found"
- **Solution**: Ensure `default-features = false` in Cargo.toml

**Error**: "cannot find function `process_autotune`"
- **Solution**: Use `process_autotune_embedded` instead of `process_autotune`

### Runtime Issues

**Audio dropouts or glitches**:
- Check stack size (increase if needed)
- Verify interrupt priorities
- Monitor CPU usage

**No audio output**:
- Verify buffer management in audio callback
- Check that `frames_ready` flag is handled correctly
- Ensure proper overlap-add implementation

### Memory Issues

**Stack overflow**:
- Increase stack size in linker script
- Consider reducing buffer sizes for lower-end MCUs

**Insufficient RAM**:
- Use smaller FFT sizes (requires code modifications)
- Reduce number of simultaneous processing instances

## Performance Tips

1. **Compiler Optimization**: Use `opt-level = "s"` or `opt-level = 3` for release builds
2. **FPU**: Ensure FPU is enabled for floating-point operations
3. **Interrupt Priority**: Use high priority (7-8) for audio interrupts
4. **Cache**: Enable instruction and data cache on supported MCUs

## Example Projects

See `examples/daisy_seed_example.rs` for a complete working implementation with:
- RTIC integration
- Audio buffer management
- Real-time parameter control
- Error handling

## Limitations

- **FFT Size**: Currently fixed at 1024 points
- **Sample Rates**: Tested at 44.1kHz and 48kHz
- **Mono Processing**: Stereo requires separate instances
- **Key Changes**: Require reprocessing (small latency)

## Support

For embedded-specific issues:
1. Check memory usage with your linker map
2. Verify FPU configuration
3. Test with a simple passthrough first
4. Monitor real-time performance with oscilloscope/logic analyzer

The embedded implementation prioritizes deterministic behavior and real-time performance over flexibility.