/// Musical settings for autotune processing
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MusicalSettings {
    /// Musical key (0-23, see keys module for mapping)
    pub key: i32,
    /// Specific note (0 = auto mode, 1-9 = specific note in scale)
    pub note: i32,
    /// Octave setting
    pub octave: i32,
    /// Formant shift mode (0 = none, 1 = lower, 2 = higher)
    pub formant: i32,
}

impl Default for MusicalSettings {
    fn default() -> Self {
        Self {
            key: 0,  // C Major
            note: 0, // Auto mode
            octave: 2,
            formant: 0, // No formant shift
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_musical_settings_default() {
        let settings = MusicalSettings::default();
        assert_eq!(settings.key, 0);
        assert_eq!(settings.note, 0);
        assert_eq!(settings.octave, 2);
        assert_eq!(settings.formant, 0);
    }
}
