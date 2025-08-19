# Embedded Autotune Troubleshooting Guide

This guide helps diagnose and fix common issues when using the synthphone_vocals autotune library on embedded systems, specifically the Daisy Seed.

## Quick Fix for Common Issues

If you're experiencing **quiet, choppy audio with poor autotune performance**, the most likely cause is improper buffer management. Use the new `embedded_realtime` module instead of the basic `embedded` module:

```rust
// OLD - causes choppy audio
use synthphone_vocals::embedded::{EmbeddedAutotuneState1024, process_autotune_embedded};

// NEW - smooth real-time processing
use synthphone_vocals::embedded_realtime::{RealtimeAutotuneState, process_block_realtime};
```

## Common Problems and Solutions

### 1. Audio is Quiet and Distorted

**Symptoms:**
- Very low output volume
- Crackling or popping sounds
- No audible autotune effect

**Causes & Solutions:**

#### A. Incorrect FFT Scaling
```rust
// WRONG - Missing proper normalization
let output = ifft_result[i].re * window[i];

// CORRECT - Proper scaling for microfft
let normalization = 0.5 / (FFT_SIZE as f32);
let output = ifft_result[i].re * window[i] * normalization;
```

#### B. Poor Overlap-Add Implementation
Use the `RealtimeAutotuneState` which handles overlap-add correctly:

```rust
// Initialize real-time state
let mut state = RealtimeAutotuneState::new(config);

// Process in your audio callback
audio.for_each(|input, _| {
    let output = process_sample_realtime(input, &mut state, &settings)?;
    (output, output)
});
```

### 2. Choppy Audio with Dropouts

**Symptoms:**
- Audio cuts in and out
- Rhythmic clicking or stuttering
- Irregular volume changes

**Root Cause:** The basic embedded implementation waits for full 1024-sample buffers, causing large gaps in audio output.

**Solution:** Use block-based processing with smaller buffers:

```rust
const BLOCK_SIZE: usize = 128; // Much smaller than FFT size

#[task(binds = DMA1_STR1, priority = 8)]
fn audio_handler(ctx: audio_handler::Context) {
    let mut input_block = [0.0f32; BLOCK_SIZE];
    let mut output_block = [0.0f32; BLOCK_SIZE];
    
    // Collect samples into blocks
    audio.for_each(|left_in, _right_in| {
        input_block[sample_idx] = left_in;
        sample_idx += 1;
        
        if sample_idx >= BLOCK_SIZE {
            // Process entire block at once
            process_block_realtime(&input_block, &mut output_block, &mut state, &settings)?;
            sample_idx = 0;
        }
        
        // Output from processed block
        (output_block[sample_idx], output_block[sample_idx])
    });
}
```

### 3. No Autotune Effect

**Symptoms:**
- Audio passes through unchanged
- No pitch correction occurs
- Fundamental frequency detection fails

**Debugging Steps:**

#### A. Check Pitch Correction Strength
```rust
// Too weak - barely audible
config.pitch_correction_strength = 0.1;

// Good for natural correction
config.pitch_correction_strength = 0.8;

// Very strong correction
config.pitch_correction_strength = 0.95;
```

#### B. Verify Frequency Range
```rust
// Check if input frequency is in processing range
if fundamental_freq < 80.0 || fundamental_freq > 2000.0 {
    // Frequency outside processing range
    return original_frequency;
}
```

#### C. Musical Settings Configuration
```rust
let settings = MusicalSettings {
    key: 0,     // 0 = C Major, 1 = C# Major, etc.
    note: 0,    // 0 = auto mode (finds nearest note)
    octave: 2,  // Reasonable octave for vocals
    formant: 0, // No formant shifting
};
```

### 4. High CPU Usage / Real-Time Issues

**Symptoms:**
- Audio glitches under load
- System becomes unresponsive
- Buffer overruns or underruns

**Optimizations:**

#### A. Reduce Processing Load
```rust
// Use larger hop size for less frequent processing
let config = AutotuneConfig {
    fft_size: 1024,
    hop_size: 256,  // Process every 256 samples instead of 128
    // ... other settings
};
```

#### B. Use Appropriate Configuration Preset
```rust
// Use configuration optimized for your use case
let config = AutotuneConfig::realtime();      // Balanced latency/CPU
// let config = AutotuneConfig::low_latency(); // Interactive applications
// let config = AutotuneConfig::high_quality(); // Non-interactive/battery
```

#### C. Compiler Optimizations
Add to your `Cargo.toml`:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### 5. Memory Issues

**Symptoms:**
- Stack overflow errors
- System crashes
- Insufficient RAM errors

**Solutions:**

#### A. Increase Stack Size
In your linker script:
```
MEMORY
{
  /* Increase stack size */
  RAM : ORIGIN = 0x20000000, LENGTH = 512K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
_stack_size = 32K;  /* Increase from default */
```

#### B. Monitor Memory Usage
```rust
#[init]
fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
    // Check available stack space
    let stack_used = estimate_stack_usage();
    info!("Estimated stack usage: {} bytes", stack_used);
    
    // Ensure we have enough margin
    if stack_used > 24000 {  // 24KB threshold
        info!("Warning: High stack usage detected");
    }
    
    // ... rest of init
}
```

## Performance Tuning

### Latency vs Quality Trade-offs

| Hop Size | Latency @ 48kHz | CPU Usage | Quality |
|----------|----------------|-----------|---------|
| 128      | ~2.7ms         | High      | Best    |
| 256      | ~5.3ms         | Medium    | Good    |
| 512      | ~10.7ms        | Low       | Fair    |

### Recommended Configurations

#### Interactive Applications (Low Latency)
```rust
let config = AutotuneConfig::low_latency();
// Or customize:
// AutotuneConfig {
//     fft_size: 1024,
//     hop_size: 128,
//     pitch_correction_strength: 0.7,
//     transition_speed: 0.3,
//     min_frequency: 80.0,
//     max_frequency: 2000.0,
// }
```

#### Real-Time Processing (Balanced)
```rust
let config = AutotuneConfig::realtime();
// Or customize:
// AutotuneConfig {
//     fft_size: 1024,
//     hop_size: 256,
//     pitch_correction_strength: 0.8,
//     transition_speed: 0.2,
//     min_frequency: 80.0,
//     max_frequency: 2000.0,
// }
```

#### High-Quality Processing (Lower CPU)
```rust
let config = AutotuneConfig::high_quality();
// Or customize:
// AutotuneConfig {
//     fft_size: 1024,
//     hop_size: 512,
//     pitch_correction_strength: 0.85,
//     transition_speed: 0.15,
//     min_frequency: 60.0,
//     max_frequency: 4000.0,
// }
```

## Hardware Configuration Guidelines

### STM32H7 Series (e.g., STM32H750)

#### Clock Configuration
Ensure proper clock setup for audio processing:
```rust
// In your system initialization
let ccdr = device
    .PWR
    .constrain()
    .and_freeze(
        device.RCC.constrain()
            .use_hse(16.mhz())      // External crystal
            .sys_ck(400.mhz())      // Maximum system clock
            .hclk(200.mhz())        // AHB clock
            .pclk1(100.mhz())       // APB1 clock
            .pclk2(100.mhz())       // APB2 clock
            .freeze()
    );
```

#### Audio Interface Setup
```rust
// Configure SAI for optimal performance
let sai_config = SaiConfig::new()
    .frame_sync_active_high(false)
    .clock_strobe(ClockStrobe::Rising)
    .bit_order(BitOrder::MsbFirst)
    .frame_sync_offset(FrameSyncOffset::BeforeFirstBit);
```

#### FPU Configuration
Ensure FPU is enabled for floating-point operations:
```rust
// In your boot sequence
#[pre_init]
unsafe fn pre_init() {
    // Enable FPU
    let cpacr = core::ptr::read_volatile(0xE000_ED88 as *const u32);
    core::ptr::write_volatile(0xE000_ED88 as *mut u32, cpacr | (0xF << 20));
}
```

### General STM32H7 Optimization

#### Cache Configuration
Enable caches for better performance:
```rust
// Enable instruction and data caches
cortex_m::interrupt::free(|_| {
    let mut cp = CorePeripherals::take().unwrap();
    cp.SCB.enable_icache();
    cp.SCB.enable_dcache(&mut cp.CPUID);
});
```

## Debugging Techniques

### 1. Audio Path Verification
```rust
// Add bypass mode for testing
if bypass_mode {
    return input_sample; // Direct passthrough
}
```

### 2. Processing State Monitoring
```rust
// Log processing statistics
static mut DEBUG_COUNTER: u32 = 0;
unsafe {
    DEBUG_COUNTER += 1;
    if DEBUG_COUNTER % 1000 == 0 {
        info!("Processed {} frames, freq: {:.1} Hz", 
              DEBUG_COUNTER, fundamental_frequency);
    }
}
```

### 3. Buffer Health Checks
```rust
// Verify buffer integrity
fn check_buffer_health(state: &RealtimeAutotuneState) -> bool {
    // Check for NaN or infinite values
    for &sample in &state.input_buffer {
        if !sample.is_finite() {
            return false;
        }
    }
    true
}
```

### 4. Real-Time Performance Monitoring
```rust
// Measure processing time
let start_time = get_cycle_count();
process_block_realtime(&input, &mut output, &mut state, &settings)?;
let end_time = get_cycle_count();
let processing_cycles = end_time - start_time;

// Convert to microseconds (assuming 400MHz clock)
let processing_us = processing_cycles / 400;
info!("Processing time: {} μs", processing_us);
```

## Error Handling

### Graceful Degradation
```rust
// Handle processing errors gracefully
let result = process_block_realtime(&input, &mut output, &mut state, &settings);
match result {
    Ok(()) => {
        // Use processed output
        output_sample = output[i];
    }
    Err(AutotuneError::BufferSizeMismatch) => {
        // Reset state and use passthrough
        state.reset();
        output_sample = input_sample * 0.8;
    }
    Err(_) => {
        // Other errors - use passthrough
        output_sample = input_sample * 0.8;
    }
}
```

### State Recovery
```rust
// Automatic state recovery
static mut ERROR_COUNT: u32 = 0;
unsafe {
    ERROR_COUNT += 1;
    if ERROR_COUNT > 100 {
        // Too many errors - reset everything
        state.reset();
        ERROR_COUNT = 0;
        info!("Autotune state reset due to excessive errors");
    }
}
```

## Testing and Validation

### Unit Test Framework
```rust
#[cfg(test)]
mod embedded_tests {
    use super::*;
    
    #[test]
    fn test_realtime_processing_stability() {
            let config = AutotuneConfig::realtime();
        let mut state = RealtimeAutotuneState::new(config);
        let settings = MusicalSettings::default();
        
        // Process 10 seconds of audio
        for _ in 0..(48000 * 10) {
            let input = generate_test_tone(440.0, 48000.0); // A4
            let result = process_sample_realtime(input, &mut state, &settings);
            assert!(result.is_ok());
            assert!(result.unwrap().is_finite());
        }
    }
}
```

### Integration Testing
Use a logic analyzer or oscilloscope to verify:
- Audio output continuity
- Processing timing consistency
- Buffer overflow/underflow detection

## Performance Benchmarks

Expected performance on ARM Cortex-M7 @ 400MHz (e.g., STM32H750):
- **Processing Time**: 15-25% CPU usage
- **Memory Usage**: ~16KB stack allocation
- **Latency**: 2.7ms - 10.7ms depending on hop size
- **Audio Quality**: 16-bit equivalent or better

If your performance doesn't match these benchmarks, check:
1. Compiler optimization settings
2. FPU configuration
3. Clock settings
4. Cache configuration
5. Interrupt priority configuration

## Getting Help

If you're still experiencing issues:

1. **Check the Examples**: Use the real-time examples as a reference
2. **Enable Debug Logging**: Add detailed logging to track processing flow
3. **Profile Memory Usage**: Use tools like `cargo-call-stack` to analyze memory usage
4. **Test with Bypass Mode**: Verify your audio pipeline works without autotune
5. **Reduce to Minimal Case**: Start with a simple sine wave input

For performance-critical applications, consider:
- Using fixed-point arithmetic instead of floating-point
- Implementing custom FFT optimizations
- Using DMA for audio buffer transfers
- Offloading some processing to dedicated hardware accelerators