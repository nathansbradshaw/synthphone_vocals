# Configuration Guide

This guide helps you choose the right configuration for your autotune application based on your requirements, not your hardware. The library provides flexible configuration options that can be adapted to any embedded system.

## Quick Start

Choose a preset based on your primary requirement:

```rust
// For interactive applications where latency matters most
let config = AutotuneConfig::low_latency();

// For balanced real-time processing (recommended starting point)
let config = AutotuneConfig::realtime();

// For best quality when latency is less critical
let config = AutotuneConfig::high_quality();
```

## Configuration Parameters

### Core Parameters

#### `fft_size: usize`
- **What it affects**: Frequency resolution and processing complexity
- **Current limitation**: Must be 1024 (may be expanded in future versions)
- **Trade-offs**: Larger = better frequency resolution but more CPU usage

#### `hop_size: usize`
- **What it affects**: Latency and overlap amount
- **Range**: 64 - 512 (must be less than fft_size)
- **Trade-offs**: 
  - Smaller = lower latency but higher CPU usage
  - Larger = higher latency but lower CPU usage

#### `sample_rate: f32`
- **What it affects**: Time resolution and frequency mapping
- **Common values**: 44100.0, 48000.0
- **Notes**: Must match your audio hardware sample rate

#### `pitch_correction_strength: f32`
- **Range**: 0.0 - 1.0
- **What it affects**: How strongly notes are corrected
- **Values**:
  - 0.0 = No correction (passthrough)
  - 0.5 = Subtle correction
  - 0.8 = Strong correction (typical)
  - 1.0 = Maximum correction (can sound robotic)

#### `transition_speed: f32`
- **Range**: 0.01 - 1.0
- **What it affects**: How quickly pitch changes occur
- **Values**:
  - 0.1 = Slow, smooth transitions
  - 0.2 = Natural transitions (typical)
  - 0.5 = Fast transitions
  - 1.0 = Instantaneous (can cause artifacts)

#### `min_frequency: f32` / `max_frequency: f32`
- **What it affects**: Frequency range for pitch detection and correction
- **Purpose**: Improves accuracy by filtering out noise and irrelevant frequencies
- **Common ranges**:
  - Vocals: 80.0 - 1200.0 Hz
  - Full range: 60.0 - 4000.0 Hz
  - Instruments: Varies by instrument

## Choosing Configuration Based on Requirements

### Priority: Low Latency
**Use Case**: Live performance, interactive applications, real-time effects

```rust
let config = AutotuneConfig {
    fft_size: 1024,
    hop_size: 128,        // Small hop = low latency
    sample_rate: 48000.0,
    transition_speed: 0.3, // Faster transitions
    pitch_correction_strength: 0.7,
    min_frequency: 80.0,
    max_frequency: 2000.0,
};
```

**Characteristics**:
- Latency: ~2.7ms at 48kHz
- CPU Usage: High
- Quality: Good

### Priority: Balanced Performance
**Use Case**: General embedded audio processing, most applications

```rust
let config = AutotuneConfig {
    fft_size: 1024,
    hop_size: 256,        // Balanced hop size
    sample_rate: 48000.0,
    transition_speed: 0.2,
    pitch_correction_strength: 0.8,
    min_frequency: 80.0,
    max_frequency: 2000.0,
};
```

**Characteristics**:
- Latency: ~5.3ms at 48kHz
- CPU Usage: Medium
- Quality: Good to Excellent

### Priority: High Quality
**Use Case**: Studio processing, non-interactive applications, recording

```rust
let config = AutotuneConfig {
    fft_size: 1024,
    hop_size: 512,        // Large hop = high quality
    sample_rate: 48000.0,
    transition_speed: 0.15, // Smooth transitions
    pitch_correction_strength: 0.85,
    min_frequency: 60.0,
    max_frequency: 4000.0, // Wider frequency range
};
```

**Characteristics**:
- Latency: ~10.7ms at 48kHz
- CPU Usage: Low
- Quality: Excellent

### Priority: Low CPU Usage
**Use Case**: Battery-powered devices, systems with limited processing power

```rust
let config = AutotuneConfig {
    fft_size: 1024,
    hop_size: 512,        // Large hop reduces CPU load
    sample_rate: 44100.0, // Slightly lower sample rate
    transition_speed: 0.2,
    pitch_correction_strength: 0.75,
    min_frequency: 100.0, // Narrower range reduces processing
    max_frequency: 1000.0,
};
```

**Characteristics**:
- CPU Usage: Minimal
- Battery Life: Extended
- Quality: Good (within frequency range)

## Application-Specific Configurations

### Vocal Processing
```rust
let config = AutotuneConfig {
    fft_size: 1024,
    hop_size: 256,
    sample_rate: 48000.0,
    transition_speed: 0.15,        // Smooth for natural vocals
    pitch_correction_strength: 0.8,
    min_frequency: 80.0,          // Human vocal range
    max_frequency: 1200.0,        // Focus on fundamental frequencies
};
```

### Instrument Processing
```rust
let config = AutotuneConfig {
    fft_size: 1024,
    hop_size: 200,
    sample_rate: 48000.0,
    transition_speed: 0.25,        // Slightly faster for instruments
    pitch_correction_strength: 0.7, // Less aggressive
    min_frequency: 60.0,          // Lower for bass instruments
    max_frequency: 2000.0,        // Higher for treble instruments
};
```

### Creative/Effect Processing
```rust
let config = AutotuneConfig {
    fft_size: 1024,
    hop_size: 128,
    sample_rate: 48000.0,
    transition_speed: 0.5,         // Fast for robotic effect
    pitch_correction_strength: 0.95, // Very strong correction
    min_frequency: 80.0,
    max_frequency: 2000.0,
};
```

## Performance vs Quality Trade-offs

| Parameter | Lower Value | Higher Value |
|-----------|-------------|--------------|
| `hop_size` | Lower latency, Higher CPU | Higher latency, Lower CPU |
| `pitch_correction_strength` | More natural, Subtle effect | More robotic, Strong effect |
| `transition_speed` | Smoother, Slower response | Faster response, Possible artifacts |
| `min/max_frequency` range | Lower CPU, Less accuracy | Higher CPU, Better accuracy |

## Latency Calculation

```
Latency (seconds) = hop_size / sample_rate
Latency (ms) = (hop_size / sample_rate) * 1000
```

Examples at 48kHz:
- hop_size 128: ~2.7ms
- hop_size 256: ~5.3ms  
- hop_size 512: ~10.7ms

## CPU Usage Estimation

Relative CPU usage (higher = more processing):
```
CPU_factor = (sample_rate / hop_size) * frequency_range_factor

Where frequency_range_factor = (max_freq - min_freq) / 2000
```

## Memory Usage

All configurations use the same memory (stack-allocated):
- Input buffer: 4KB (1024 × 4 bytes)
- Output buffer: 4KB
- FFT working space: ~8KB during processing
- State arrays: ~8KB
- **Total**: ~16KB stack memory

## Validation and Testing

### Validate Your Configuration
```rust
impl AutotuneConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.hop_size >= self.fft_size {
            return Err("hop_size must be less than fft_size");
        }
        if self.sample_rate <= 0.0 {
            return Err("sample_rate must be positive");
        }
        if !(0.0..=1.0).contains(&self.pitch_correction_strength) {
            return Err("pitch_correction_strength must be between 0.0 and 1.0");
        }
        if !(0.01..=1.0).contains(&self.transition_speed) {
            return Err("transition_speed must be between 0.01 and 1.0");
        }
        if self.min_frequency >= self.max_frequency {
            return Err("min_frequency must be less than max_frequency");
        }
        Ok(())
    }
}
```

### Test Your Configuration
```rust
// Test with known frequency
fn test_configuration(config: &AutotuneConfig) {
    let mut state = RealtimeAutotuneState::new(*config);
    let settings = MusicalSettings::default();
    
    // Generate test tone at A4 (440 Hz)
    let samples_per_second = config.sample_rate as usize;
    for i in 0..samples_per_second {
        let t = i as f32 / config.sample_rate;
        let input = (2.0 * PI * 440.0 * t).sin() * 0.5;
        
        let result = process_sample_realtime(input, &mut state, &settings);
        assert!(result.is_ok());
    }
}
```

## Common Pitfalls

### 1. **hop_size Too Small**
- **Problem**: Excessive CPU usage, potential real-time violations
- **Solution**: Increase hop_size, monitor CPU usage

### 2. **pitch_correction_strength Too High**
- **Problem**: Robotic, unnatural sound
- **Solution**: Reduce to 0.7-0.8 for natural correction

### 3. **Frequency Range Too Wide**
- **Problem**: Processing noise and irrelevant frequencies
- **Solution**: Narrow the range to your specific use case

### 4. **transition_speed Too High**
- **Problem**: Audible artifacts, clicking sounds
- **Solution**: Reduce to 0.1-0.3 for smooth transitions

### 5. **Sample Rate Mismatch**
- **Problem**: Incorrect pitch detection and correction
- **Solution**: Ensure config sample_rate matches hardware

## Advanced Optimization

### Dynamic Configuration
```rust
// Adjust parameters based on system load
fn adaptive_config(base_config: AutotuneConfig, cpu_usage: f32) -> AutotuneConfig {
    let mut config = base_config;
    
    if cpu_usage > 0.8 {
        // Reduce processing load
        config.hop_size = (config.hop_size * 2).min(512);
        config.max_frequency = config.max_frequency.min(1500.0);
    } else if cpu_usage < 0.3 {
        // Increase quality
        config.hop_size = (config.hop_size / 2).max(64);
        config.max_frequency = config.max_frequency.max(2000.0);
    }
    
    config
}
```

### Environment-Specific Tuning
```rust
// Adjust for different acoustic environments
fn environment_config(base_config: AutotuneConfig, environment: Environment) -> AutotuneConfig {
    let mut config = base_config;
    
    match environment {
        Environment::Studio => {
            // Clean environment - can use full range
            config.min_frequency = 60.0;
            config.max_frequency = 4000.0;
        },
        Environment::Live => {
            // Noisy environment - focus on vocal range
            config.min_frequency = 100.0;
            config.max_frequency = 1200.0;
            config.transition_speed = 0.3; // Faster response
        },
        Environment::Practice => {
            // Balanced settings
            config.min_frequency = 80.0;
            config.max_frequency = 2000.0;
        }
    }
    
    config
}
```

## Summary

Choose your configuration based on your primary requirement:

1. **Low Latency**: Small hop_size (128), fast transitions
2. **High Quality**: Large hop_size (512), smooth transitions  
3. **Low CPU**: Large hop_size, narrow frequency range
4. **Natural Sound**: Medium correction strength (0.7-0.8), slow transitions
5. **Creative Effect**: High correction strength (0.9+), fast transitions

Remember: you can always start with a preset and adjust individual parameters based on your specific needs and system capabilities.