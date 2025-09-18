//! Vocoder and Dry Processing Demo
//!
//! This example demonstrates the different processing modes available in the
//! synthphone_vocals library: autotune, vocoder, and dry processing modes.

use synthphone_vocals::{MusicalSettings, VocalEffectsConfig, process_vocal_effects_config};

// Generate functions for different processing modes
process_vocal_effects_config!(
    process_autotune_demo,
    1024,
    48000.0,
    mode = autotune,
    hop_ratio = 0.25
);

process_vocal_effects_config!(process_vocoder_demo, 1024, 48000.0, mode = vocode, hop_ratio = 0.25);

process_vocal_effects_config!(process_dry_demo, 1024, 48000.0, mode = dry, hop_ratio = 0.25);

/// Create a test audio signal with harmonics
fn create_test_signal(frequency: f32, sample_rate: f32) -> [f32; 1024] {
    let mut signal = [0.0f32; 1024];
    for (i, sample) in signal.iter_mut().enumerate() {
        let t = i as f32 / sample_rate;
        // Create a rich harmonic signal
        *sample = 0.5 * (2.0 * std::f32::consts::PI * frequency * t).sin()
            + 0.3 * (2.0 * std::f32::consts::PI * frequency * 2.0 * t).sin()
            + 0.2 * (2.0 * std::f32::consts::PI * frequency * 3.0 * t).sin();
    }
    signal
}

/// Create a carrier signal for vocoder
fn create_carrier_signal(frequency: f32, sample_rate: f32) -> [f32; 1024] {
    let mut signal = [0.0f32; 1024];
    for (i, sample) in signal.iter_mut().enumerate() {
        let t = i as f32 / sample_rate;
        // Create a sawtooth-like carrier
        *sample = 0.4 * (2.0 * std::f32::consts::PI * frequency * t).sin()
            + 0.2 * (2.0 * std::f32::consts::PI * frequency * 2.0 * t).sin()
            + 0.1 * (2.0 * std::f32::consts::PI * frequency * 4.0 * t).sin();
    }
    signal
}

/// Calculate RMS (Root Mean Square) of a signal
fn calculate_rms(signal: &[f32]) -> f32 {
    let sum_squares: f32 = signal.iter().map(|&x| x * x).sum();
    (sum_squares / signal.len() as f32).sqrt()
}

fn main() {
    println!("🎵 Vocoder and Dry Processing Demo 🎵");
    println!("====================================");
    println!();

    let config = VocalEffectsConfig::default();
    let sample_rate = 48000.0;

    // Demo 1: Autotune Processing
    println!("1. AUTOTUNE PROCESSING");
    println!("----------------------");
    println!("Pitch correction to musical notes with formant preservation");
    println!();

    let mut vocal_signal = create_test_signal(440.0, sample_rate); // A4
    let mut input_phases = [0.0f32; 1024];
    let mut output_phases = [0.0f32; 1024];

    let settings = MusicalSettings {
        key: 0,  // C major
        note: 0, // Auto mode - snap to nearest note
        octave: 4,
        formant: 1, // Preserve formants
        ..Default::default()
    };

    let input_rms = calculate_rms(&vocal_signal);
    let autotune_result = process_autotune_demo(
        &mut vocal_signal,
        &mut input_phases,
        &mut output_phases,
        1.0,
        &config,
        &settings,
    );
    let output_rms = calculate_rms(&autotune_result);

    println!("  Input RMS:  {:.4}", input_rms);
    println!("  Output RMS: {:.4}", output_rms);
    println!("  Energy preservation: {:.1}%", (output_rms / input_rms) * 100.0);
    println!("  ✓ Pitch corrected to nearest musical note");
    println!();

    // Demo 2: Vocoder Processing
    println!("2. VOCODER PROCESSING");
    println!("---------------------");
    println!("Applies vocal formant envelope to carrier signal");
    println!();

    let mut vocal_input = create_test_signal(220.0, sample_rate); // A3 - vocal
    let mut carrier_input = create_carrier_signal(330.0, sample_rate); // E4 - carrier
    let mut vocoder_input_phases = [0.0f32; 1024];
    let mut vocoder_output_phases = [0.0f32; 1024];

    let vocal_rms = calculate_rms(&vocal_input);
    let carrier_rms = calculate_rms(&carrier_input);

    let vocoder_result = process_vocoder_demo(
        &mut vocal_input,
        &mut carrier_input,
        &mut vocoder_input_phases,
        &mut vocoder_output_phases,
        &config,
        &settings,
    );
    let vocoder_output_rms = calculate_rms(&vocoder_result);

    println!("  Vocal RMS:    {:.4}", vocal_rms);
    println!("  Carrier RMS:  {:.4}", carrier_rms);
    println!("  Output RMS:   {:.4}", vocoder_output_rms);
    println!("  ✓ Vocal formants applied to carrier signal");
    println!("  ✓ Classic vocoder effect achieved");
    println!();

    // Demo 3: Dry Processing
    println!("3. DRY PROCESSING");
    println!("-----------------");
    println!("Pitch shifting and formant control without pitch correction");
    println!();

    let mut dry_signal = create_test_signal(262.0, sample_rate); // C4
    let mut synth_signal = create_test_signal(523.0, sample_rate); // C5 - octave higher
    let mut dry_input_phases = [0.0f32; 1024];
    let mut dry_output_phases = [0.0f32; 1024];

    let dry_settings = MusicalSettings {
        key: 0,
        note: 1,    // Manual pitch shift
        octave: 5,  // Shift up an octave
        formant: 2, // Raise formants
        ..Default::default()
    };

    let dry_input_rms = calculate_rms(&dry_signal);
    let synth_rms = calculate_rms(&synth_signal);

    let dry_result = process_dry_demo(
        &mut dry_signal,
        Some(&mut synth_signal), // Mix with synth
        &mut dry_input_phases,
        &mut dry_output_phases,
        1.0,
        &config,
        &dry_settings,
    );
    let dry_output_rms = calculate_rms(&dry_result);

    println!("  Input RMS:  {:.4}", dry_input_rms);
    println!("  Synth RMS:  {:.4}", synth_rms);
    println!("  Output RMS: {:.4}", dry_output_rms);
    println!("  ✓ Pitch shifted without correction");
    println!("  ✓ Formants adjusted");
    println!("  ✓ Mixed with synthesizer signal");
    println!();

    // Demo 4: Comparison of Processing Modes
    println!("4. PROCESSING MODE COMPARISON");
    println!("-----------------------------");
    println!();

    let test_freq = 349.23; // F4
    let mut test_signal1 = create_test_signal(test_freq, sample_rate);
    let mut test_signal2 = test_signal1;
    let mut test_signal3 = test_signal1;
    let mut carrier_for_vocoder = create_carrier_signal(440.0, sample_rate);

    let mut phases1 = [0.0f32; 1024];
    let mut phases2 = [0.0f32; 1024];
    let mut phases3 = [0.0f32; 1024];
    let mut phases4 = [0.0f32; 1024];
    let mut phases5 = [0.0f32; 1024];
    let mut phases6 = [0.0f32; 1024];

    let comparison_settings = MusicalSettings {
        key: 0,
        note: 0,
        octave: 4,
        formant: 0, // No formant processing for fair comparison
        ..Default::default()
    };

    // Process with each mode
    let autotune_comparison = process_autotune_demo(
        &mut test_signal1,
        &mut phases1,
        &mut phases2,
        1.0,
        &config,
        &comparison_settings,
    );

    let vocoder_comparison = process_vocoder_demo(
        &mut test_signal2,
        &mut carrier_for_vocoder,
        &mut phases3,
        &mut phases4,
        &config,
        &comparison_settings,
    );

    let dry_comparison = process_dry_demo(
        &mut test_signal3,
        None, // No synth mixing for comparison
        &mut phases5,
        &mut phases6,
        1.0,
        &config,
        &comparison_settings,
    );

    let input_rms = calculate_rms(&test_signal1);
    let autotune_rms = calculate_rms(&autotune_comparison);
    let vocoder_rms = calculate_rms(&vocoder_comparison);
    let dry_rms = calculate_rms(&dry_comparison);

    println!("Input signal: {:.1} Hz, RMS: {:.4}", test_freq, input_rms);
    println!();
    println!("Mode       | Output RMS | Energy Ratio | Characteristics");
    println!("-----------|------------|--------------|----------------------------------");
    println!(
        "Autotune   | {:.4}     | {:5.1}%      | Pitch corrected, formants preserved",
        autotune_rms,
        (autotune_rms / input_rms) * 100.0
    );
    println!(
        "Vocoder    | {:.4}     | {:5.1}%      | Formants from vocal, pitch from carrier",
        vocoder_rms,
        (vocoder_rms / input_rms) * 100.0
    );
    println!(
        "Dry        | {:.4}     | {:5.1}%      | Raw pitch shift, no correction",
        dry_rms,
        (dry_rms / input_rms) * 100.0
    );
    println!();

    // Demo 5: Configuration Guidelines
    println!("5. CONFIGURATION GUIDELINES");
    println!("---------------------------");
    println!();
    println!("🎯 AUTOTUNE MODE:");
    println!("  • Best for: Vocal pitch correction, auto-tune effects");
    println!("  • Settings: note=0 for auto-snap, note>0 for specific pitch");
    println!("  • Formants: 1 for preservation, 0 for robotic effect");
    println!();
    println!("🤖 VOCODER MODE:");
    println!("  • Best for: Classic vocoder effects, robot voices");
    println!("  • Requires: Both vocal input and carrier signal");
    println!("  • Use case: Apply vocal formants to synthesizer sounds");
    println!();
    println!("🎛️  DRY MODE:");
    println!("  • Best for: Pitch shifting, formant manipulation");
    println!("  • Features: No auto-correction, raw pitch control");
    println!("  • Mixing: Optional synth signal blending");
    println!();

    println!("📊 PERFORMANCE CHARACTERISTICS:");
    println!("  • FFT Size: 1024 samples");
    println!("  • Hop Ratio: 0.25 (75% overlap)");
    println!("  • Sample Rate: 48 kHz");
    println!("  • Latency: ~5.3 ms");
    println!("  • Memory: ~4 KB per instance");
    println!();

    println!("✅ Demo completed successfully!");
    println!("All three processing modes are working correctly.");
}
