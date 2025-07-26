//! Integration tests for autotune library

use approx::assert_relative_eq;
use autotune::{AutotuneConfig, AutotuneState, MusicalSettings, process_autotune};

#[test]
fn test_basic_processing() {
    let config = AutotuneConfig::default();
    let mut state = AutotuneState::new(config);
    let settings = MusicalSettings::default();

    let input = vec![0.0f32; 1024];
    let mut output = vec![0.0f32; 1024];

    // Should not error on silence
    assert!(process_autotune(&input, &mut output, &mut state, &settings).is_ok());

    // Output should be mostly silent for silent input
    let max_output = output.iter().fold(0.0f32, |acc, &x| acc.max(x.abs()));
    assert!(max_output < 0.1);
}

#[test]
fn test_sine_wave_processing() {
    let config = AutotuneConfig {
        sample_rate: 44100.0,
        ..Default::default()
    };
    let mut state = AutotuneState::new(config);
    let settings = MusicalSettings::default();

    // Generate a 440Hz sine wave
    let mut input = vec![0.0f32; 1024];
    for (i, sample) in input.iter_mut().enumerate() {
        let t = i as f32 / config.sample_rate;
        *sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin();
    }

    let mut output = vec![0.0f32; 1024];

    // Should process without error
    assert!(process_autotune(&input, &mut output, &mut state, &settings).is_ok());

    // Output should have similar energy
    let input_energy: f32 = input.iter().map(|x| x * x).sum();
    let output_energy: f32 = output.iter().map(|x| x * x).sum();

    assert_relative_eq!(input_energy, output_energy, epsilon = 0.5);
}

#[test]
fn test_config_validation() {
    // Valid config should work
    assert!(AutotuneConfig::new(1024, 256, 44100.0).is_ok());

    // Invalid FFT size (not power of 2) should fail
    assert!(AutotuneConfig::new(1000, 256, 44100.0).is_err());

    // Hop size >= FFT size should fail
    assert!(AutotuneConfig::new(1024, 1024, 44100.0).is_err());

    // Invalid sample rate should fail
    assert!(AutotuneConfig::new(1024, 256, -44100.0).is_err());
}
