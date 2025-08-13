# Synthphone Vocals

[![CI](https://github.com/YOUR_USERNAME/synthphone_vocals/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/synthphone_vocals/actions)
[![Security Audit](https://github.com/YOUR_USERNAME/synthphone_vocals/workflows/Security%20Audit/badge.svg)](https://github.com/YOUR_USERNAME/synthphone_vocals/actions)
[![Benchmark](https://github.com/YOUR_USERNAME/synthphone_vocals/workflows/Benchmark/badge.svg)](https://github.com/YOUR_USERNAME/synthphone_vocals/actions)
[![Crates.io](https://img.shields.io/crates/v/synthphone_vocals.svg)](https://crates.io/crates/synthphone_vocals)
[![Documentation](https://docs.rs/synthphone_vocals/badge.svg)](https://docs.rs/synthphone_vocals)
[![License](https://img.shields.io/crates/l/synthphone_vocals.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.70+-ab6000.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

A real-time autotune and vocal processing library designed for both embedded and desktop applications, with compile-time generated frequency tables and Hann windows for optimal performance and memory efficiency.

## 🎵 Features

- ✅ **Real-time pitch correction** using phase vocoder techniques
- ✅ **24 musical keys** (major/minor scales) with compile-time generated frequency tables
- ✅ **No-std compatible** for embedded systems (ARM Cortex-M, etc.)
- ✅ **Desktop support** with full standard library features  
- ✅ **Compile-time optimization** - all frequency tables and windows generated at compile time
- ✅ **Configurable FFT sizes** (64, 128, 256, 512, 1024, 2048, 4096)
- ✅ **Low-latency processing** suitable for real-time audio applications
- ✅ **Memory efficient** with pre-computed lookup tables
- ✅ **Mathematical precision** using high-accuracy polynomial approximations

## 🚀 Quick Start

### Desktop Usage

```rust
use synthphone_vocals::{
    core::process_autotune,
    state::{AutotuneState, MusicalSettings},
    frequencies::*
};

// Create state with default configuration
let mut state = AutotuneState::default();

// Configure musical settings
let settings = MusicalSettings {
    key: 0,          // C Major
    note: 0,         // Auto mode (snap to nearest note)
    octave: 2,       // Octave offset
    formant: 0,      // No formant shifting
};

// Process audio buffer
let input = vec![0.0f32; 1024];
let mut output = vec![0.0f32; 1024];

process_autotune(&input, &mut output, &mut state, &settings);
```

### Embedded/No-Std Usage

```rust
#![no_std]

use synthphone_vocals::{
    core::process_autotune,
    state::{AutotuneState, MusicalSettings},
    frequencies::C_MAJOR_SCALE_FREQUENCIES,
    hann_window::HANN_1024,
};

// Pre-allocated buffers for embedded systems
static mut INPUT_BUFFER: [f32; 1024] = [0.0; 1024];
static mut OUTPUT_BUFFER: [f32; 1024] = [0.0; 1024];

// Initialize state (can be done at compile time)
static mut STATE: AutotuneState = AutotuneState::new_default();

fn process_audio_frame() {
    let settings = MusicalSettings {
        key: 0, note: 0, octave: 2, formant: 0
    };
    
    unsafe {
        process_autotune(
            &INPUT_BUFFER,
            &mut OUTPUT_BUFFER,
            &mut STATE,
            &settings
        );
    }
}
```

## 🏗️ Architecture

The library is built with a modular architecture optimized for both performance and memory efficiency:

### Core Modules

- **`frequencies`** - Compile-time generated frequency tables for all 24 musical keys
- **`hann_window`** - Compile-time generated Hann windows for various FFT sizes
- **`core`** - Main processing engine with FFT-based pitch correction
- **`state`** - State management and configuration structures
- **`process_frequencies`** - Frequency analysis and pitch detection algorithms

### Compile-Time Optimization

All frequency tables and windowing functions are generated at compile time using `const` functions:

```rust
// Generated at compile time - zero runtime cost
pub const C_MAJOR_SCALE_FREQUENCIES: [f32; 70] = generate_major_scale_frequencies(0);
pub const HANN_1024: HannWindow<1024> = HannWindow::new();
```

## 📊 Performance

### Benchmarks

- **Real-time processing**: <1ms latency for 1024-sample buffers at 44.1kHz
- **Memory usage**: <64KB static allocation for embedded targets
- **CPU usage**: <5% on ARM Cortex-M4 at 168MHz

### Supported Targets

- ✅ `x86_64-unknown-linux-gnu` (Desktop Linux)
- ✅ `x86_64-pc-windows-msvc` (Desktop Windows)
- ✅ `x86_64-apple-darwin` (Desktop macOS)
- ✅ `thumbv7em-none-eabihf` (ARM Cortex-M4F/M7F)
- ✅ `thumbv6m-none-eabi` (ARM Cortex-M0/M0+)

## 🧪 Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test

# Run tests with no_std
cargo test --no-default-features

# Run benchmarks
cargo bench

# Run with different targets
cargo test --target thumbv7em-none-eabihf --no-default-features
```

### Test Coverage

- ✅ Unit tests for all core algorithms
- ✅ Integration tests for audio processing pipeline
- ✅ Compile-time validation of generated tables
- ✅ Cross-platform compatibility tests
- ✅ Performance regression tests

## 🔧 Development

### Prerequisites

- Rust 1.70+ (MSRV)
- For embedded targets: `rustup target add thumbv7em-none-eabihf`

### Building

```bash
# Standard build
cargo build

# Release build with optimizations
cargo build --release

# Embedded build (no_std)
cargo build --target thumbv7em-none-eabihf --no-default-features

# Run linting
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Run tests and linting (`cargo test && cargo clippy`)
4. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
5. Push to the branch (`git push origin feature/AmazingFeature`)
6. Open a Pull Request

All PRs are automatically tested with our comprehensive CI pipeline including:
- Multi-platform builds (Linux, Windows, macOS)
- Cross-compilation to embedded targets
- Security audits and dependency checks
- Performance benchmarks
- Code formatting and linting

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔄 CI/CD Pipeline

This project uses GitHub Actions for comprehensive continuous integration:

### Workflows

- **CI Pipeline** (`ci.yml`):
  - ✅ Multi-version testing (stable, beta, nightly)
  - ✅ Cross-platform builds (Linux, Windows, macOS)
  - ✅ Embedded targets (ARM Cortex-M4F, M0+)
  - ✅ Code formatting (`cargo fmt`)
  - ✅ Linting (`cargo clippy`)
  - ✅ Documentation building
  - ✅ MSRV compatibility (Rust 1.70+)

- **Security Audit** (`security.yml`):
  - 🔒 Daily dependency vulnerability scans
  - 📦 Outdated dependency checks
  - ⚖️ License compatibility verification
  - 🧹 Unused dependency detection

- **Benchmarking** (`benchmark.yml`):
  - 📊 Performance regression detection
  - 🎯 Memory usage validation
  - 📈 Binary size tracking
  - 🚀 Benchmark result storage

All PRs must pass the complete CI pipeline before merging, ensuring high code quality and cross-platform compatibility.

## 🙏 Acknowledgments

- Built with [microfft](https://crates.io/crates/microfft) for embedded FFT processing
- Inspired by classic autotune and pitch correction algorithms
- Mathematical foundations based on phase vocoder techniques
