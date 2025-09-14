#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Core modules
pub mod config;
pub mod error;
pub mod fft_config;
pub mod state;
pub mod vocal_effects_config;

// Audio processing modules
pub mod frequencies;
pub mod hann_window;
pub mod keys;
pub mod process_frequencies;
pub mod process_vocal_effects;

// Buffer management
pub mod ring_buffer;

// Utility modules
pub mod utils;

// Platform-specific modules
#[cfg(feature = "embedded")]
#[cfg_attr(docsrs, doc(cfg(feature = "embedded")))]
pub mod embedded;

// Existing modules (kept for compatibility)
pub mod fade;
pub mod normal_phase_advance;
pub mod oscillator;

// Re-export main API
pub use config::VocalEffectsConfig;
pub use error::VocalEffectsError;
pub use state::MusicalSettings;

// Re-export commonly used functions
pub use frequencies::{find_nearest_note_frequency, find_nearest_note_in_key};
pub use keys::{get_frequency, get_key, get_key_name, get_scale_by_key};
pub use process_frequencies::{find_fundamental_frequency, wrap_phase};
