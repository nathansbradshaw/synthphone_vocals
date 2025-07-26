//! Desktop file processing example

use autotune::{AutotuneConfig, AutotuneState, MusicalSettings, process_autotune};
use hound::{WavReader, WavSpec, WavWriter};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Open input file
    let mut reader = WavReader::open("input.wav")?;
    let spec = reader.spec();

    // Create autotune configuration
    let config = AutotuneConfig {
        fft_size: 1024,
        hop_size: 256,
        sample_rate: spec.sample_rate as f32,
        transition_speed: 0.1,
        pitch_correction_strength: 0.8,
        ..Default::default()
    };

    // Create state
    let mut state = AutotuneState::new(config);

    // Set musical parameters
    let settings = MusicalSettings {
        key: 0,  // C Major
        note: 0, // Auto mode
        octave: 2,
        formant: 0,
    };

    // Create output file
    let mut writer = WavWriter::create("output.wav", spec)?;

    // Process audio in chunks
    let mut input_buffer = vec![0.0f32; config.fft_size];
    let mut output_buffer = vec![0.0f32; config.fft_size];

    for chunk in reader
        .samples::<f32>()
        .collect::<Result<Vec<_>, _>>()?
        .chunks(config.hop_size)
    {
        // Fill input buffer (with overlap)
        input_buffer.rotate_left(config.hop_size);
        for (i, &sample) in chunk.iter().enumerate() {
            if i < config.hop_size {
                input_buffer[config.fft_size - config.hop_size + i] = sample;
            }
        }

        // Process
        process_autotune(&input_buffer, &mut output_buffer, &mut state, &settings)?;

        // Write output (just the new samples)
        for &sample in &output_buffer[..config.hop_size] {
            writer.write_sample(sample)?;
        }
    }

    writer.finalize()?;
    println!("Processing complete!");
    Ok(())
}
