#![no_main]
#![no_std]

use rtic::app;

mod autotune;
mod ring_buffer;

#[app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    dispatchers = [DMA1_STR0, DMA1_STR2]
)]
mod app {

    use libdaisy::logger;
    use libdaisy::{audio, system};
    use libm::{expf, fabsf};
    use log::warn;
    use synthphone_vocals::{AutotuneConfig, MusicalSettings};

    use crate::autotune::{autotune_audio, write_synthesis_output};
    use crate::ring_buffer::RingBuffer;
    pub const SAMPLE_RATE: f32 = 48_014.312;
    pub const FFT_SIZE: usize = 1024;
    pub const BUFFER_SIZE: usize = FFT_SIZE * 4;
    pub const HOP_SIZE: usize = 256;
    pub const BLOCK_SIZE: usize = 2;
    pub const BIN_WIDTH: f32 = SAMPLE_RATE as f32 / FFT_SIZE as f32 * 2.0;

    #[shared]
    struct Shared {
        in_ring: RingBuffer<BUFFER_SIZE>,
        out_ring: RingBuffer<BUFFER_SIZE>,
        in_pointer_cached: u32,
    }

    #[local]
    struct Local {
        audio: audio::Audio,
        buffer: audio::AudioBuffer,
        hop_counter: u32,
        last_input_phases: [f32; FFT_SIZE],
        last_output_phases: [f32; FFT_SIZE],
        previous_pitch_shift_ratio: f32,
        fft_overflow_count: u32,
        audio_underrun_count: u32,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        logger::init();

        let mut core = ctx.core;
        let device = ctx.device;
        let ccdr = system::System::init_clocks(device.PWR, device.RCC, &device.SYSCFG);
        let system = libdaisy::system_init!(core, device, ccdr, BLOCK_SIZE);
        let buffer = [(0.0, 0.0); audio::BLOCK_SIZE_MAX];

        (
            Shared {
                in_ring: RingBuffer::new(),
                out_ring: RingBuffer::with_offset((FFT_SIZE + (2 * HOP_SIZE)) as u32),
                in_pointer_cached: 0,
            },
            Local {
                buffer,
                audio: system.audio,
                hop_counter: 0,
                previous_pitch_shift_ratio: 1.0,
                last_input_phases: [0.0; FFT_SIZE],
                last_output_phases: [0.0; FFT_SIZE],
                fft_overflow_count: 0,
                audio_underrun_count: 0,
            },
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi(); // Wait for interrupt
        }
    }

    #[task(
        binds = DMA1_STR1,
        local = [audio, buffer, hop_counter, audio_underrun_count],
        shared = [in_ring, out_ring, in_pointer_cached],
        priority = 8
    )]
    fn audio_handler(mut ctx: audio_handler::Context) {
        let audio = ctx.local.audio;
        let buffer = ctx.local.buffer;

        if audio.get_stereo(buffer) {
            for (_left, right) in &buffer.as_slice()[..BLOCK_SIZE] {
                let sample = *right;

                // Lock to write to in_buffer
                ctx.shared.in_ring.lock(|in_ring| in_ring.push(sample));

                // Get output sample with fallback to input
                let out_sample = ctx.shared.out_ring.lock(|out_ring| out_ring.pop());

                // Gentle normalization to prevent clipping
                let normalized_sample = normalize_sample(out_sample, 0.6);

                // Check and handle hop counter
                if *ctx.local.hop_counter >= HOP_SIZE as u32 {
                    *ctx.local.hop_counter = 0;

                    let pointer = ctx.shared.in_ring.lock(|in_ring| in_ring.write_index());

                    ctx.shared.in_pointer_cached.lock(|cache| {
                        *cache = pointer;
                    });

                    // Run FFT Process in new software task
                    if process_autotune::spawn().is_err() {
                        warn!("Could not spawn FFT task - processing overload");
                        // Continue with unprocessed audio to prevent complete silence
                    }
                }

                *ctx.local.hop_counter += 1;

                // Output the processed audio
                if audio.push_stereo((normalized_sample, normalized_sample)).is_err() {
                    warn!("Failed to write audio data");
                }
            }
        } else {
            warn!("Error reading audio data!");
        }
    }

    #[task(
        shared = [in_ring, out_ring, in_pointer_cached],
        local = [last_input_phases,
        last_output_phases,
        previous_pitch_shift_ratio,
        fft_overflow_count],
        priority = 6,
    )]
    fn process_autotune(mut ctx: process_autotune::Context) {
        let musical_settings = MusicalSettings::default();
        let config = AutotuneConfig::default();
        let mut unwrapped_buffer = [0.0; FFT_SIZE];

        let write_idx = ctx.shared.in_pointer_cached.lock(|in_pointer| *in_pointer);

        ctx.shared
            .in_ring
            .lock(|rb| rb.block_from::<FFT_SIZE>(write_idx, &mut unwrapped_buffer));

        let synthesis_output = autotune_audio(
            &mut unwrapped_buffer,
            ctx.local.last_input_phases,
            ctx.local.last_output_phases,
            *ctx.local.previous_pitch_shift_ratio,
            &config,
            &musical_settings,
        );

        ctx.shared.out_ring.lock(|output_ring| {
            write_synthesis_output(&synthesis_output, output_ring);
        });
    }

    #[inline(always)]
    pub fn normalize_sample(sample: f32, target_peak: f32) -> f32 {
        let abs_sample = fabsf(sample);
        if abs_sample > target_peak {
            // Soft limiting to prevent harsh clipping
            let ratio = target_peak / abs_sample;
            let soft_ratio = 1.0 - expf(-3.0 * ratio);
            sample * soft_ratio
        } else {
            sample
        }
    }
}
