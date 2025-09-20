/// Trait for FFT operations to abstract over different sizes
pub trait FftOps<const N: usize, const HALF_N: usize> {
    /// Perform forward real FFT
    fn forward_fft(input: &mut [f32; N]) -> &mut [microfft::Complex32];

    /// Perform inverse complex FFT
    fn inverse_fft(spectrum: &mut [microfft::Complex32; N]) -> &mut [microfft::Complex32; N];

    /// Get the Hann window for this FFT size
    fn get_hann_window() -> &'static [f32; N];
}

/// FFT operations for 512-point FFT
pub struct Fft512;
impl FftOps<512, 256> for Fft512 {
    fn forward_fft(input: &mut [f32; 512]) -> &mut [microfft::Complex32] {
        microfft::real::rfft_512(input)
    }

    fn inverse_fft(spectrum: &mut [microfft::Complex32; 512]) -> &mut [microfft::Complex32; 512] {
        microfft::inverse::ifft_512(spectrum)
    }

    fn get_hann_window() -> &'static [f32; 512] {
        &crate::hann_window::HANN_WINDOW_512
    }
}

/// FFT operations for 1024-point FFT
pub struct Fft1024;
impl FftOps<1024, 512> for Fft1024 {
    fn forward_fft(input: &mut [f32; 1024]) -> &mut [microfft::Complex32] {
        microfft::real::rfft_1024(input)
    }

    fn inverse_fft(spectrum: &mut [microfft::Complex32; 1024]) -> &mut [microfft::Complex32; 1024] {
        microfft::inverse::ifft_1024(spectrum)
    }

    fn get_hann_window() -> &'static [f32; 1024] {
        &crate::hann_window::HANN_WINDOW_1024
    }
}

/// FFT operations for 2048-point FFT
pub struct Fft2048;
impl FftOps<2048, 1024> for Fft2048 {
    fn forward_fft(input: &mut [f32; 2048]) -> &mut [microfft::Complex32] {
        microfft::real::rfft_2048(input)
    }

    fn inverse_fft(spectrum: &mut [microfft::Complex32; 2048]) -> &mut [microfft::Complex32; 2048] {
        microfft::inverse::ifft_2048(spectrum)
    }

    fn get_hann_window() -> &'static [f32; 2048] {
        &crate::hann_window::HANN_WINDOW_2048
    }
}

/// FFT operations for 4096-point FFT
pub struct Fft4096;
impl FftOps<4096, 2048> for Fft4096 {
    fn forward_fft(input: &mut [f32; 4096]) -> &mut [microfft::Complex32] {
        microfft::real::rfft_4096(input)
    }

    fn inverse_fft(spectrum: &mut [microfft::Complex32; 4096]) -> &mut [microfft::Complex32; 4096] {
        microfft::inverse::ifft_4096(spectrum)
    }

    fn get_hann_window() -> &'static [f32; 4096] {
        &crate::hann_window::HANN_WINDOW_4096
    }
}
