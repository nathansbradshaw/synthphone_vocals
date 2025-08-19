#![no_main]
#![no_std]

use rtic::app;

#[app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
)]
mod app {
    const BLOCK_SIZE: usize = 128; // Smaller block size for better real-time performance

    use libdaisy::logger;
    use libdaisy::{audio, system};
    use log::info;
    use synthphone_vocals::{
        AutotuneConfig, MusicalSettings,
        embedded_realtime::{RealtimeAutotuneState, process_block_realtime},
    };

    #[shared]
    struct Shared {
        autotune_state: RealtimeAutotuneState,
        musical_settings: MusicalSettings,
        bypass_mode: bool,
    }

    #[local]
    struct Local {
        audio: audio::Audio,
        led_counter: u32,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        logger::init();

        let mut core = ctx.core;
        let device = ctx.device;
        let ccdr = system::System::init_clocks(device.PWR, device.RCC, &device.SYSCFG);
        let system = libdaisy::system_init!(core, device, ccdr, BLOCK_SIZE);

        info!("Initializing Real-time Daisy Seed Autotune v0.2.0");

        // Use real-time configuration optimized for embedded systems
        let config = AutotuneConfig::realtime();

        let musical_settings = MusicalSettings {
            key: 0,  // C Major
            note: 0, // Auto mode (snap to nearest note in key)
            octave: 2,
            formant: 0, // No formant shifting
        };

        info!("Configuration:");
        info!("  FFT Size: {}", config.fft_size);
        info!("  Hop Size: {}", config.hop_size);
        info!("  Sample Rate: {} Hz", config.sample_rate);
        info!("  Correction Strength: {:.2}", config.pitch_correction_strength);
        info!("  Transition Speed: {:.2}", config.transition_speed);
        info!("  Block Size: {} samples", BLOCK_SIZE);

        let autotune_state = RealtimeAutotuneState::new(config);

        info!("Real-time autotune initialized successfully!");
        info!("Memory usage: Stack-allocated circular buffers");
        info!("Latency: ~{}ms", (config.hop_size as f32 / config.sample_rate * 1000.0) as u32);

        (
            Shared { autotune_state, musical_settings, bypass_mode: false },
            Local { audio: system.audio, led_counter: 0 },
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi(); // Wait for interrupt - saves power
        }
    }

    #[task(
        binds = DMA1_STR1,
        local = [audio, led_counter],
        shared = [autotune_state, musical_settings, bypass_mode],
        priority = 8
    )]
    fn audio_handler(mut ctx: audio_handler::Context) {
        let audio = ctx.local.audio;

        // Process audio in blocks for better efficiency
        let mut input_block = [0.0f32; BLOCK_SIZE];
        let mut output_block = [0.0f32; BLOCK_SIZE];
        let mut sample_idx = 0;

        // Get shared resources (quick copy to minimize lock time)
        let (local_settings, bypass) = ctx
            .shared
            .musical_settings
            .lock(|settings| ctx.shared.bypass_mode.lock(|bypass| (*settings, *bypass)));

        audio.for_each(|left_in, _right_in| {
            // Collect samples into block
            input_block[sample_idx] = left_in;
            sample_idx += 1;

            // Process block when full
            if sample_idx >= BLOCK_SIZE {
                if !bypass {
                    // Process with autotune
                    let result = ctx.shared.autotune_state.lock(|autotune_state| {
                        process_block_realtime(
                            &input_block,
                            &mut output_block,
                            autotune_state,
                            &local_settings,
                        )
                    });

                    // Handle processing errors gracefully
                    if result.is_err() {
                        // Fall back to passthrough with slight attenuation
                        for i in 0..BLOCK_SIZE {
                            output_block[i] = input_block[i] * 0.8;
                        }
                    }
                } else {
                    // Bypass mode - direct passthrough
                    output_block.copy_from_slice(&input_block);
                }

                sample_idx = 0;
            }

            // Output processed audio (or previous block during processing)
            let output = if sample_idx == 0 && !output_block.is_empty() {
                output_block[BLOCK_SIZE - 1] // Use last sample from processed block
            } else if sample_idx > 0 {
                output_block[sample_idx - 1] // Use corresponding sample from current block
            } else {
                left_in * 0.8 // Safe fallback
            };

            // Apply soft limiting to prevent clipping
            let limited_output = soft_limit(output);

            // LED indicator (blink every ~1000 samples)
            *ctx.local.led_counter += 1;
            if *ctx.local.led_counter % 1000 == 0 {
                // Toggle LED or other indicator here if available
            }

            // Output mono to both channels
            (limited_output, limited_output)
        });
    }

    // Task to change musical key
    #[task(shared = [musical_settings], priority = 2)]
    fn change_key(mut ctx: change_key::Context, new_key: i32) {
        ctx.shared.musical_settings.lock(|settings| {
            settings.key = new_key.clamp(0, 23); // Ensure valid key range
        });
        info!("Changed to key: {}", new_key);
    }

    // Task to adjust pitch correction strength
    #[task(shared = [autotune_state], priority = 2)]
    fn adjust_correction_strength(mut ctx: adjust_correction_strength::Context, strength: f32) {
        ctx.shared.autotune_state.lock(|state| {
            state.config.pitch_correction_strength = strength.clamp(0.0, 1.0);
        });
        info!("Correction strength: {:.2}", strength);
    }

    // Task to adjust transition speed
    #[task(shared = [autotune_state], priority = 2)]
    fn adjust_transition_speed(mut ctx: adjust_transition_speed::Context, speed: f32) {
        ctx.shared.autotune_state.lock(|state| {
            state.config.transition_speed = speed.clamp(0.01, 1.0);
        });
        info!("Transition speed: {:.2}", speed);
    }

    // Task to toggle bypass mode
    #[task(shared = [bypass_mode], priority = 2)]
    fn toggle_bypass(mut ctx: toggle_bypass::Context) {
        let new_bypass = ctx.shared.bypass_mode.lock(|bypass| {
            *bypass = !*bypass;
            *bypass
        });
        info!("Bypass mode: {}", if new_bypass { "ON" } else { "OFF" });
    }

    // Task to change octave
    #[task(shared = [musical_settings], priority = 2)]
    fn change_octave(mut ctx: change_octave::Context, octave: i32) {
        ctx.shared.musical_settings.lock(|settings| {
            settings.octave = octave.clamp(-2, 6); // Reasonable octave range
        });
        info!("Changed to octave: {}", octave);
    }

    // Task to reset autotune state (useful for clearing artifacts)
    #[task(shared = [autotune_state], priority = 2)]
    fn reset_autotune(mut ctx: reset_autotune::Context) {
        ctx.shared.autotune_state.lock(|state| {
            state.reset();
        });
        info!("Autotune state reset");
    }

    // Example button handlers (uncomment and adapt for your hardware)
    /*
    #[task(binds = EXTI15_10, priority = 3)]
    fn button1_handler(_: button1_handler::Context) {
        // Cycle through keys
        static mut CURRENT_KEY: i32 = 0;
        unsafe {
            *CURRENT_KEY = (*CURRENT_KEY + 1) % 12; // Major keys only
            change_key::spawn(*CURRENT_KEY).ok();
        }
    }

    #[task(binds = EXTI9_5, priority = 3)]
    fn button2_handler(_: button2_handler::Context) {
        // Toggle bypass
        toggle_bypass::spawn().ok();
    }

    #[task(binds = EXTI4, priority = 3)]
    fn button3_handler(_: button3_handler::Context) {
        // Reset autotune state
        reset_autotune::spawn().ok();
    }
    */

    // Example encoder handler for real-time parameter control
    /*
    #[task(binds = TIM2, priority = 4)]
    fn encoder_handler(_: encoder_handler::Context) {
        // Read encoder value and map to correction strength
        // let encoder_value = read_encoder(); // Your encoder reading function
        // let strength = map_range(encoder_value, 0, 100, 0.0, 1.0);
        // adjust_correction_strength::spawn(strength).ok();
    }
    */
}

/// Soft limiter to prevent harsh clipping
fn soft_limit(input: f32) -> f32 {
    let threshold = 0.8;
    let ratio = 4.0;

    if input.abs() > threshold {
        let excess = input.abs() - threshold;
        let compressed = excess / ratio;
        let limited = threshold + compressed;
        if input >= 0.0 { limited } else { -limited }
    } else {
        input
    }
}

/// Map a value from one range to another
#[allow(dead_code)]
fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let normalized = (value - in_min) / (in_max - in_min);
    out_min + normalized * (out_max - out_min)
}
