#![no_main]
#![no_std]

use rtic::app;

#[app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
)]
mod app {
    const BLOCK_SIZE: usize = 128;

    use libdaisy::logger;
    use libdaisy::{audio, system};
    use log::info;
    use synthphone_vocals::{
        AutotuneConfig, MusicalSettings,
        embedded::{EmbeddedAutotuneState1024, process_autotune_embedded},
    };

    #[shared]
    struct Shared {
        autotune_state: EmbeddedAutotuneState1024,
        musical_settings: MusicalSettings,
    }

    #[local]
    struct Local {
        audio: audio::Audio,
        input_buffer: [f32; 1024],
        output_buffer: [f32; 1024],
        buffer_index: usize,
        output_index: usize,
        frames_ready: bool,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        logger::init();

        let mut core = ctx.core;
        let device = ctx.device;
        let ccdr = system::System::init_clocks(device.PWR, device.RCC, &device.SYSCFG);
        let system = libdaisy::system_init!(core, device, ccdr, BLOCK_SIZE);

        info!("Initializing truly no_std Daisy Seed autotune...");

        let config = AutotuneConfig {
            fft_size: 1024,
            hop_size: 256,
            sample_rate: 48000.0,
            pitch_correction_strength: 0.8,
            transition_speed: 0.1,
            ..Default::default()
        };

        let musical_settings = MusicalSettings {
            key: 0,  // C Major
            note: 0, // Auto mode (snap to nearest note in key)
            octave: 2,
            formant: 0, // No formant shifting
        };

        info!("Ready for truly no_std audio processing!");
        info!("Memory usage: ~16KB stack allocation only");

        (
            Shared { autotune_state: EmbeddedAutotuneState1024::new(config), musical_settings },
            Local {
                audio: system.audio,
                input_buffer: [0.0; 1024],
                output_buffer: [0.0; 1024],
                buffer_index: 0,
                output_index: 0,
                frames_ready: false,
            },
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(
        binds = DMA1_STR1,
        local = [audio, input_buffer, output_buffer, buffer_index, output_index, frames_ready],
        shared = [autotune_state, musical_settings],
        priority = 8
    )]
    fn audio_handler(mut ctx: audio_handler::Context) {
        let audio = ctx.local.audio;

        audio.for_each(|left_in, _right_in| {
            // Fill input buffer
            ctx.local.input_buffer[*ctx.local.buffer_index] = left_in;
            *ctx.local.buffer_index += 1;

            // Process when buffer is full (1024 samples collected)
            if *ctx.local.buffer_index >= 1024 {
                // Get musical settings (copy to avoid holding lock)
                let local_musical_settings = ctx.shared.musical_settings.lock(|settings| *settings);

                // Process autotune with truly no_std implementation
                let result = ctx.shared.autotune_state.lock(|autotune_state| {
                    process_autotune_embedded(
                        ctx.local.input_buffer,
                        ctx.local.output_buffer,
                        autotune_state,
                        &local_musical_settings,
                    )
                });

                // Handle any processing errors by falling back to passthrough
                if result.is_err() {
                    ctx.local.output_buffer.copy_from_slice(ctx.local.input_buffer);
                }

                // Reset for next frame
                *ctx.local.buffer_index = 0;
                *ctx.local.output_index = 0;
                *ctx.local.frames_ready = true;
            }

            // Output processed audio or passthrough during initial buffering
            let output = if *ctx.local.frames_ready && *ctx.local.output_index < 1024 {
                let sample = ctx.local.output_buffer[*ctx.local.output_index];
                *ctx.local.output_index += 1;

                // Mark as not ready when we've output all samples
                if *ctx.local.output_index >= 1024 {
                    *ctx.local.frames_ready = false;
                }

                sample
            } else {
                // Passthrough during buffering phase
                left_in
            };

            // Output mono to both channels
            (output, output)
        });
    }

    // Optional: Task to change musical key (can be triggered by button, MIDI, etc.)
    #[task(shared = [musical_settings], priority = 2)]
    fn change_key(mut ctx: change_key::Context, new_key: i32) {
        ctx.shared.musical_settings.lock(|settings| {
            settings.key = new_key;
        });
    }

    // Optional: Task to adjust pitch correction strength
    #[task(shared = [autotune_state], priority = 2)]
    fn adjust_correction_strength(mut ctx: adjust_correction_strength::Context, strength: f32) {
        ctx.shared.autotune_state.lock(|state| {
            state.config.pitch_correction_strength = strength.clamp(0.0, 1.0);
        });
    }

    // Example: Button handler to cycle through keys
    /*
    #[task(binds = EXTI15_10, priority = 1)]
    fn button_handler(_: button_handler::Context) {
        static mut CURRENT_KEY: i32 = 0;
        unsafe {
            *CURRENT_KEY = (*CURRENT_KEY + 1) % 12; // Cycle through 12 keys
            change_key::spawn(*CURRENT_KEY).ok();
        }
    }
    */
}
