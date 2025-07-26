#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # Autotune Library
//!
//! A real-time autotune library designed to work in both embedded and desktop environments.
//!
//! ## Features
//!
//! - Real-time pitch correction
//! - Musical key and scale support
//! - Formant shifting (optional)
//! - Both embedded (no_std) and desktop (std) support
//! - Configurable FFT sizes and processing parameters
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use autotune::{AutotuneConfig, AutotuneState, MusicalSettings, process_autotune};
//!
//! let config = AutotuneConfig::default();
//! let mut state = AutotuneState::new(config);
//! let settings = MusicalSettings::default();
//!
//! let input = vec![0.0f32; 1024];
//! let mut output = vec![0.0f32; 1024];
//!
//! process_autotune(&input, &mut output, &mut state, &settings)?;
//! ```

extern crate alloc;

// Core modules
pub mod config;
pub mod core;
pub mod error;
pub mod state;

// Audio processing modules
pub mod frequencies;
pub mod hann_window;
pub mod keys;
pub mod process_frequencies;

// Buffer management
pub mod circular_buffer;
pub mod ring_buffer;

// Utility modules
pub mod utils;

// Platform-specific modules
#[cfg(feature = "embedded")]
#[cfg_attr(docsrs, doc(cfg(feature = "embedded")))]
pub mod embedded;

#[cfg(feature = "desktop")]
#[cfg_attr(docsrs, doc(cfg(feature = "desktop")))]
pub mod desktop;

// Existing modules (kept for compatibility)
pub mod fade;
pub mod normal_phase_advance;
pub mod oscillator;

// Re-export main API
pub use config::AutotuneConfig;
pub use core::process_autotune;
pub use error::AutotuneError;
pub use state::{AutotuneState, MusicalSettings};

// Re-export commonly used functions
pub use frequencies::{find_nearest_note_frequency, find_nearest_note_in_key};
pub use keys::{get_frequency, get_key, get_key_name, get_scale_by_key};
pub use process_frequencies::{find_fundamental_frequency, wrap_phase};
