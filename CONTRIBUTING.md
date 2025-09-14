# Contributing to Synthphone Vocals

Thank you for your interest in contributing to Synthphone Vocals! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ (we use edition 2024)
- For embedded testing: ARM cross-compilation toolchain
- Audio hardware or virtual audio devices for testing

### Development Setup

1. **Clone the repository:**
```bash
git clone https://github.com/yourusername/synthphone_vocals
cd synthphone_vocals
```

2. **Install Rust targets (for embedded development):**
```bash
rustup target add thumbv7em-none-eabihf
rustup target add thumbv6m-none-eabi
```

3. **Run tests to verify setup:**
```bash
cargo test --all-features
cargo test --no-default-features --features embedded
```

## 🎯 Areas for Contribution

### High Priority
- **Performance optimization** - SIMD, algorithm improvements
- **Documentation** - Examples, tutorials, API docs
- **Testing** - Edge cases, embedded platforms, audio quality
- **Platform support** - New embedded targets, RTOS integration

### Medium Priority
- **New features** - Additional effects, analysis tools
- **Developer experience** - Better error messages, debugging tools
- **Examples** - Real-world applications, tutorials

### Future/Research
- **Machine learning** - Neural pitch correction
- **Advanced DSP** - New algorithms, quality improvements

## 📝 Development Guidelines

### Code Style

We use standard Rust formatting with some project-specific conventions:

```bash
# Format code
cargo fmt

# Check linting
cargo clippy --all-features -- -D warnings
```

### Key Principles

1. **Real-time First**: All code must be real-time safe
   - No heap allocations in audio processing paths
   - Bounded execution time
   - Lock-free data structures preferred

2. **Embedded Friendly**: 
   - `no_std` compatible core functionality
   - Minimal RAM usage
   - Efficient algorithms

3. **Performance Critical**:
   - Profile before optimizing
   - Benchmark significant changes
   - Consider cache efficiency

4. **Safety & Correctness**:
   - Prefer safe Rust where possible
   - Document any unsafe code thoroughly
   - Comprehensive testing

### Code Organization

```
src/
├── config.rs              # Runtime configuration
├── fft_config.rs          # FFT setup and validation
├── vocal_effects_config.rs # Processing function generation
├── process_vocal_effects.rs # Main pitch correction engine
├── process_frequencies.rs  # Frequency analysis
├── frequencies.rs         # Musical calculations  
├── keys.rs               # Musical theory
├── ring_buffer.rs        # Lock-free buffers
├── hann_window.rs        # Windowing functions
├── embedded.rs           # Embedded-specific code
└── utils.rs              # Utilities
```

### Testing Strategy

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test complete audio processing chains
3. **Property Tests**: Verify mathematical properties hold
4. **Embedded Tests**: Validate on actual hardware when possible

## 🧪 Testing Guidelines

### Running Tests

```bash
# All tests with all features
cargo test --all-features

# Embedded-only tests  
cargo test --no-default-features --features embedded

# Documentation tests
cargo test --doc

# Benchmarks
cargo bench
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_specific_behavior() {
        // Arrange
        let input = setup_test_data();
        
        // Act  
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
        assert!(result.is_valid());
    }
    
    #[test] 
    fn test_real_time_constraints() {
        use std::time::Instant;
        
        let start = Instant::now();
        let result = process_audio_block(&audio_data);
        let duration = start.elapsed();
        
        // Must complete within buffer time
        assert!(duration.as_micros() < 1000); // 1ms for 48 samples @ 48kHz
    }
}
```

### Audio Testing

For audio quality testing:

```rust
#[test]
fn test_audio_quality() {
    let input = load_test_audio("test_vocals.wav");
    let processed = process_with_vocal_effects(input);
    
    // Check for artifacts
    assert!(measure_thd(&processed) < 0.01); // < 1% THD
    assert!(measure_latency(&processed) < max_latency);
}
```

## 📚 Documentation

### Code Documentation

- **Public APIs**: Must have comprehensive doc comments
- **Complex algorithms**: Include mathematical background  
- **Examples**: Show real usage patterns
- **Safety**: Document any unsafe code or assumptions

```rust
/// Processes audio with real-time vocal pitch correction.
///
/// This function performs pitch correction using a phase vocoder approach,
/// with automatic pitch detection and correction to the nearest note in 
/// the specified musical key.
///
/// # Parameters
/// - `buffer`: Audio samples to process (modified in-place)
/// - `config`: Vocal effects configuration parameters
/// - `settings`: Musical key and correction settings
///
/// # Returns
/// Processed audio buffer with pitch correction applied
///
/// # Performance
/// This function is designed for real-time use and completes processing
/// within the audio buffer timeframe on typical embedded hardware.
///
/// # Example
/// ```rust
/// let mut audio = [0.0f32; 1024];
/// let config = VocalEffectsConfig::default();  
/// let settings = MusicalSettings { key: 0, ..Default::default() };
///
/// let result = process_vocal_effects(&mut audio, &config, &settings);
/// ```
pub fn process_vocal_effects(
    buffer: &mut [f32],
    config: &VocalEffectsConfig, 
    settings: &MusicalSettings
) -> ProcessResult {
    // Implementation...
}
```

### Examples and Tutorials

When adding examples:
- Include complete, runnable code
- Explain the trade-offs and configuration choices
- Show both basic and advanced usage
- Test examples as part of CI

## 🔧 Pull Request Process

### Before Submitting

1. **Test thoroughly:**
```bash
cargo test --all-features
cargo clippy --all-features -- -D warnings
cargo fmt --check
```

2. **Update documentation:**
   - Add/update doc comments for public APIs
   - Update README if adding major features
   - Add examples for new functionality

3. **Consider performance:**
   - Run benchmarks if changing performance-critical code
   - Profile on embedded hardware if available

### PR Description Template

```markdown
## Summary
Brief description of changes and motivation.

## Type of Change
- [ ] Bug fix
- [ ] New feature  
- [ ] Performance improvement
- [ ] Documentation update
- [ ] Refactoring

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass  
- [ ] Tested on embedded hardware (if applicable)
- [ ] Audio quality verified (if applicable)

## Performance Impact
Describe any performance implications, with benchmarks if relevant.

## Breaking Changes
List any breaking API changes and migration path.
```

### Review Process

1. **Automated checks** must pass (CI/CD)
2. **Code review** by maintainers
3. **Testing** on multiple platforms if applicable
4. **Documentation review** for public APIs

## 🐛 Bug Reports

### Before Reporting

1. **Search existing issues** for duplicates
2. **Test with latest version**
3. **Minimize reproduction case**

### Bug Report Template

```markdown
## Bug Description
Clear description of the bug and expected behavior.

## Reproduction Steps
1. Step 1
2. Step 2  
3. Step 3

## Environment
- OS: [e.g. Linux, Windows, embedded]
- Rust version: [e.g. 1.70.0]
- Target: [e.g. x86_64, thumbv7em-none-eabihf]
- Features: [e.g. default, embedded, formant-shifting]

## Audio Details (if applicable)
- Sample rate: [e.g. 48kHz]
- Buffer size: [e.g. 1024 samples]
- Input characteristics: [e.g. vocal, instrumental]

## Additional Context
Any other relevant information, logs, or screenshots.
```

## 🎵 Audio Quality Guidelines

When contributing audio processing code:

### Quality Metrics
- **THD+N**: < 0.1% for clean signals
- **Latency**: Minimize while maintaining quality
- **Artifacts**: Avoid audible glitches or discontinuities  
- **CPU usage**: Profile and optimize hot paths

### Testing Audio Changes
1. Test with various input types (vocal, instrumental, noise)
2. Check for artifacts at buffer boundaries
3. Verify real-time performance constraints
4. Test edge cases (silence, clipping, extreme pitch)

## 🏗️ Architecture Decisions

### When Adding New Features

Consider:
1. **Real-time constraints**: Will this work in embedded contexts?
2. **Memory usage**: Static allocation preferred
3. **API design**: Consistent with existing patterns
4. **Performance**: Profile impact on critical paths
5. **Testing**: How will this be validated?

### Major Changes

For significant architectural changes:
1. **Open an issue** to discuss the approach first
2. **Create an RFC** for complex features
3. **Prototype** performance-critical changes
4. **Plan migration** for breaking changes

## 📞 Communication

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: General questions, ideas
- **Pull Requests**: Code contributions, documentation

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Synthphone Vocals! 🎵