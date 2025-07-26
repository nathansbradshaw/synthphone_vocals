pub const FFT_SIZE: usize = 1024;

/// Const function to generate Hann window values
/// This uses a polynomial approximation for cosine to work in const context
const fn hann_window_value(n: usize, total_size: usize) -> f32 {
    if total_size <= 1 {
        return 1.0;
    }

    let n_f = n as f32;
    let size_minus_1 = (total_size - 1) as f32;
    let normalized = n_f / size_minus_1;

    // Use Chebyshev polynomial approximation for cos(2πx)
    // cos(2πx) ≈ T₀ - T₂x² + T₄x⁴ - T₆x⁶ + ...
    // where Tₙ are Chebyshev coefficients
    let x = 2.0 * normalized - 1.0; // Map [0,1] to [-1,1] for Chebyshev
    let x2 = x * x;
    let x4 = x2 * x2;
    let x6 = x4 * x2;

    // Chebyshev approximation for cos(π(x+1)) which equals cos(2πnormalized)
    let cos_approx = 1.0 - 2.0 * x2 + (2.0 / 3.0) * x4 - (4.0 / 45.0) * x6;

    0.5 * (1.0 - cos_approx)
}

/// Macro to generate a Hann window array at compile time
macro_rules! hann_window_array {
    ($size:expr) => {{
        const SIZE: usize = $size;
        const fn generate() -> [f32; SIZE] {
            let mut window = [0.0; SIZE];
            let mut i = 0;
            while i < SIZE {
                window[i] = hann_window_value(i, SIZE);
                i += 1;
            }
            window
        }
        generate()
    }};
}

/// Generic const function to create Hann windows
/// This can be used in const contexts
pub const fn create_hann_window<const N: usize>() -> [f32; N] {
    let mut window = [0.0; N];
    let mut i = 0;
    while i < N {
        window[i] = hann_window_value(i, N);
        i += 1;
    }
    window
}

/// Struct to hold window data with const generic size
pub struct HannWindow<const N: usize> {
    data: [f32; N],
}

impl<const N: usize> HannWindow<N> {
    /// Create a new Hann window at compile time
    pub const fn new() -> Self {
        Self {
            data: create_hann_window::<N>(),
        }
    }

    /// Get the window data
    pub const fn data(&self) -> &[f32; N] {
        &self.data
    }

    /// Get a reference to the window as a slice
    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }
}

/// Static instances for common sizes
pub static HANN_64: HannWindow<64> = HannWindow::new();
pub static HANN_128: HannWindow<128> = HannWindow::new();
pub static HANN_256: HannWindow<256> = HannWindow::new();
pub static HANN_512: HannWindow<512> = HannWindow::new();
pub static HANN_1024: HannWindow<1024> = HannWindow::new();
pub static HANN_2048: HannWindow<2048> = HannWindow::new();
pub static HANN_4096: HannWindow<4096> = HannWindow::new();

/// Function to get a Hann window for any size (computed at compile time when possible)
pub const fn get_hann_window<const N: usize>() -> [f32; N] {
    create_hann_window::<N>()
}

// Pre-computed arrays for common sizes using the macro
pub const HANN_WINDOW_64: [f32; 64] = hann_window_array!(64);
pub const HANN_WINDOW_128: [f32; 128] = hann_window_array!(128);
pub const HANN_WINDOW_256: [f32; 256] = hann_window_array!(256);
pub const HANN_WINDOW_512: [f32; 512] = hann_window_array!(512);
pub const HANN_WINDOW_1024: [f32; 1024] = hann_window_array!(1024);
pub const HANN_WINDOW_2048: [f32; 2048] = hann_window_array!(2048);
pub const HANN_WINDOW_4096: [f32; 4096] = hann_window_array!(4096);

// Backwards compatibility
pub const HANN_WINDOW: [f32; FFT_SIZE] = HANN_WINDOW_1024;

/// Convenience function for runtime access to static windows
pub fn get_static_hann_window(size: usize) -> Option<&'static [f32]> {
    match size {
        64 => Some(HANN_64.as_slice()),
        128 => Some(HANN_128.as_slice()),
        256 => Some(HANN_256.as_slice()),
        512 => Some(HANN_512.as_slice()),
        1024 => Some(HANN_1024.as_slice()),
        2048 => Some(HANN_2048.as_slice()),
        4096 => Some(HANN_4096.as_slice()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_window_generation() {
        const WINDOW: [f32; 16] = get_hann_window::<16>();

        // Test boundary conditions
        assert!((WINDOW[0] - 0.0).abs() < 1e-5);
        assert!((WINDOW[15] - 0.0).abs() < 1e-5);

        // Test that middle values are reasonable
        assert!(WINDOW[8] > 0.5);
    }

    #[test]
    fn test_window_struct() {
        const WINDOW: HannWindow<32> = HannWindow::new();
        let data = WINDOW.data();

        assert_eq!(data.len(), 32);
        assert!((data[0] - 0.0).abs() < 1e-5);
        assert!((data[31] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_static_windows() {
        let window_512 = get_static_hann_window(512).unwrap();
        assert_eq!(window_512.len(), 512);

        let window_1024 = get_static_hann_window(1024).unwrap();
        assert_eq!(window_1024.len(), 1024);

        // Test non-existent size
        assert!(get_static_hann_window(333).is_none());
    }

    #[test]
    fn test_symmetry() {
        const WINDOW: [f32; 64] = get_hann_window::<64>();

        // Test symmetry
        for i in 0..32 {
            let left = WINDOW[i];
            let right = WINDOW[63 - i];
            assert!(
                (left - right).abs() < 1e-4,
                "Window not symmetric at {} vs {}: {} vs {}",
                i,
                63 - i,
                left,
                right
            );
        }
    }

    #[test]
    fn test_backwards_compatibility() {
        assert_eq!(HANN_WINDOW.len(), 1024);
        assert_eq!(HANN_WINDOW_512.len(), 512);
        assert!((HANN_WINDOW[0] - 0.0).abs() < 1e-5);
        assert!((HANN_WINDOW[1023] - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_macro_generated_arrays() {
        // Test that macro-generated arrays work
        assert_eq!(HANN_WINDOW_256.len(), 256);
        assert!((HANN_WINDOW_256[0] - 0.0).abs() < 1e-5);
        assert!((HANN_WINDOW_256[255] - 0.0).abs() < 1e-5);
        assert!(HANN_WINDOW_256[128] > 0.8); // Middle should be close to 1.0
    }
}
