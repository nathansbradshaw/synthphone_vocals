//! Lock-free ring buffer implementation for single-producer, single-consumer scenarios.
//!
//! This module provides a high-performance, lock-free ring buffer optimized for audio
//! processing applications where one thread produces data and another consumes it.

use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicU32, Ordering},
};

/// A lock-free ring buffer for single-producer, single-consumer (SPSC) scenarios.
///
/// This ring buffer is optimized for audio processing where samples need to be written
/// by one thread (typically an interrupt handler) and read by another (typically the
/// main processing loop).
///
/// # Safety
///
/// This implementation is **only safe** when used with exactly one producer thread
/// and one consumer thread. Using multiple producers or consumers will result in
/// undefined behavior.
///
/// # Generic Parameters
///
/// * `N` - The buffer capacity. **Must be a power of two** for efficient modulo operations
///   using bit masking (e.g., 1024, 2048, 4096).
///
/// # Examples
///
/// ```rust
/// use synthphone_e_vocal_dsp::ring_buffer::RingBuffer;
/// let mut buffer: RingBuffer<1024> = RingBuffer::new();
///
/// // Producer thread
/// buffer.push(0.5);
/// buffer.push(-0.3);
///
/// // Consumer thread
/// let sample1 = buffer.pop(); // 0.5
/// let sample2 = buffer.pop(); // -0.3
/// ```
pub struct RingBuffer<const N: usize> {
    /// The actual buffer storage. UnsafeCell allows interior mutability.
    buf: UnsafeCell<[f32; N]>,
    /// Atomic write index (producer position)
    write: AtomicU32,
    /// Atomic read index (consumer position)
    read: AtomicU32,
}

// Safety – single producer / single consumer.
unsafe impl<const N: usize> Sync for RingBuffer<N> {}

impl<const N: usize> Default for RingBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> RingBuffer<N> {
    /// Creates a new ring buffer with the write pointer offset by the specified amount.
    ///
    /// This is useful when you want to pre-allocate space in the buffer or when
    /// synchronizing with external systems that expect a specific initial offset.
    ///
    /// # Parameters
    ///
    /// * `offset` - Initial write index offset
    ///
    /// # Example
    ///
    /// ```rust
    /// use synthphone_e_vocal_dsp::ring_buffer::RingBuffer;
    /// let buffer: RingBuffer<1024> = RingBuffer::with_offset(512);
    /// ```
    pub fn with_offset(offset: u32) -> Self {
        Self {
            buf: UnsafeCell::new([0.0; N]),
            write: AtomicU32::new(offset),
            read: AtomicU32::new(0),
        }
    }

    /// Creates a new ring buffer with both read and write pointers at zero.
    ///
    /// The buffer is initialized with all zeros and ready for immediate use.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use synthphone_e_vocal_dsp::ring_buffer::RingBuffer;
    /// let buffer: RingBuffer<1024> = RingBuffer::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            buf: UnsafeCell::new([0.0; N]),
            write: AtomicU32::new(0),
            read: AtomicU32::new(0),
        }
    }

    /// Pushes a single sample into the ring buffer.
    ///
    /// This method should only be called from the producer thread. It writes
    /// the value at the current write position and advances the write pointer.
    ///
    /// If the buffer is full, this will overwrite the oldest unread data.
    ///
    /// # Parameters
    ///
    /// * `v` - The sample value to write
    ///
    /// # Example
    ///
    /// ```rust
    /// use synthphone_e_vocal_dsp::ring_buffer::RingBuffer;
    /// let mut buffer: RingBuffer<1024> = RingBuffer::new();
    /// buffer.push(0.5);
    /// buffer.push(0.25);
    /// assert_eq!(buffer.available_samples(), 2);
    /// ```
    pub fn push(&self, v: f32) {
        let w = self.write.load(Ordering::Relaxed);
        unsafe { (*self.buf.get())[w as usize & (N - 1)] = v };
        self.write.store(w.wrapping_add(1), Ordering::Release);
    }

    /// Pops a single sample from the ring buffer.
    ///
    /// This method should only be called from the consumer thread. It reads
    /// the value at the current read position, clears that position to zero,
    /// and advances the read pointer.
    ///
    /// If no data is available, this returns 0.0 (from previously cleared slots).
    ///
    /// # Returns
    ///
    /// The sample value that was read from the buffer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use synthphone_e_vocal_dsp::ring_buffer::RingBuffer;
    /// let mut buffer: RingBuffer<1024> = RingBuffer::new();
    /// buffer.push(0.75);
    /// let sample = buffer.pop(); // Returns 0.75
    /// ```
    pub fn pop(&self) -> f32 {
        let r = self.read.load(Ordering::Relaxed);
        let v = unsafe {
            let cell = &mut (*self.buf.get())[r as usize & (N - 1)];
            let old_val = *cell;
            *cell = 0.0; // Clear after reading
            old_val
        };
        self.read.store(r.wrapping_add(1), Ordering::Release);
        v
    }

    /// Returns the current write index.
    ///
    /// This can be used for synchronization or to determine how much data
    /// has been written to the buffer since initialization.
    ///
    /// # Returns
    ///
    /// The current write position as a monotonically increasing counter.
    ///
    /// # Example
    ///
    /// ```rust
    /// use synthphone_e_vocal_dsp::ring_buffer::RingBuffer;
    /// let mut buffer: RingBuffer<1024> = RingBuffer::new();
    /// buffer.advance_write(10);
    /// assert_eq!(buffer.write_index(), 10);
    /// ```
    pub fn write_index(&self) -> u32 {
        self.write.load(core::sync::atomic::Ordering::Relaxed)
    }

    /// This method performs overlap-add synthesis by accumulating the provided
    /// samples into the buffer at consecutive positions starting from the current
    /// read position. This is commonly used in audio synthesis where overlapping
    /// frames need to be summed together.
    ///
    /// # Parameters
    ///
    /// * `samples` - Array of audio samples to add to the buffer
    ///
    /// # Example
    ///
    /// ```rust
    /// use synthphone_e_vocal_dsp::ring_buffer::RingBuffer;
    /// let buffer: RingBuffer<1024> = RingBuffer::new();
    /// let synthesis_frame = [0.1, 0.2, 0.3, 0.4];
    /// buffer.write_overlapped_samples(&synthesis_frame);
    /// ```
    pub fn write_overlapped_samples<const FRAME_SIZE: usize>(&self, samples: &[f32; FRAME_SIZE]) {
        // Add samples to buffer using overlap-add (accumulation)
        for (i, &sample) in samples.iter().enumerate() {
            self.add_at_offset(i as u32, sample);
        }
    }

    /// Advances the write pointer by `n` positions without writing data.
    ///
    /// This is useful for reserving space in the buffer or when data is written
    /// directly via other means (e.g., DMA) and you need to update the write pointer.
    ///
    /// # Parameters
    ///
    /// * `n` - Number of positions to advance the write pointer
    ///
    /// # Example
    ///
    /// ```rust
    /// use synthphone_e_vocal_dsp::ring_buffer::RingBuffer;
    /// let mut buffer: RingBuffer<1024> = RingBuffer::new();
    /// ```
    pub fn advance_write(&self, n: u32) {
        use core::sync::atomic::Ordering;

        #[cfg(feature = "std")]
        {
            // Use critical section for std targets
            critical_section::with(|_| {
                let current = self.write.load(Ordering::Relaxed);
                let new = current.wrapping_add(n);
                self.write.store(new, Ordering::Relaxed);
            });
        }

        #[cfg(not(feature = "std"))]
        {
            // For embedded targets, disable interrupts if available, otherwise rely on single-producer guarantee
            #[cfg(feature = "cortex-m")]
            {
                cortex_m::interrupt::free(|_| {
                    let current = self.write.load(Ordering::Relaxed);
                    let new = current.wrapping_add(n);
                    self.write.store(new, Ordering::Relaxed);
                });
            }

            #[cfg(not(feature = "cortex-m"))]
            {
                // For other embedded targets without atomic RMW, use compiler barriers
                core::sync::atomic::compiler_fence(Ordering::Acquire);
                let current = self.write.load(Ordering::Relaxed);
                let new = current.wrapping_add(n);
                self.write.store(new, Ordering::Relaxed);
                core::sync::atomic::compiler_fence(Ordering::Release);
            }
        }
    }

    /// Adds a value to the buffer at a position relative to the current read pointer.
    ///
    /// This is useful for implementing delay lines or feedback systems where you
    /// need to modify samples at specific delays relative to the current read position.
    ///
    /// # Parameters
    ///
    /// * `offset` - Offset from the current read position (0 = current read position)
    /// * `val` - Value to add to the existing sample at that position
    ///
    /// # Example
    ///
    /// ```rust
    /// use synthphone_e_vocal_dsp::ring_buffer::RingBuffer;
    /// let mut buffer: RingBuffer<1024> = RingBuffer::new();
    /// buffer.add_at_offset(512, 1.0); // Add 1.0 at position 512 samples ago
    /// ```
    pub fn add_at_offset(&self, offset: u32, val: f32) {
        let idx = self.read.load(Ordering::Relaxed).wrapping_add(offset);
        unsafe {
            let cell = &mut (*self.buf.get())[idx as usize & (N - 1)];
            *cell += val;
        }
    }

    /// Returns the number of samples available for reading.
    ///
    /// This is the difference between the write and read pointers, indicating
    /// how many samples have been written but not yet read.
    ///
    /// # Returns
    ///
    /// The number of samples ready to be consumed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use synthphone_e_vocal_dsp::ring_buffer::RingBuffer;
    /// let mut buffer: RingBuffer<1024> = RingBuffer::new();
    /// buffer.push(0.5);
    /// buffer.push(0.25);
    /// assert_eq!(buffer.available_samples(), 2);
    ///
    /// buffer.pop();
    /// assert_eq!(buffer.available_samples(), 1);
    /// ```
    pub fn available_samples(&self) -> u32 {
        let w = self.write.load(Ordering::Acquire);
        let r = self.read.load(Ordering::Acquire);
        w.wrapping_sub(r)
    }

    /// Copies the most recently written block of samples into the destination array.
    ///
    /// This method copies the last `LEN` samples that were written to the buffer,
    /// with the oldest sample first. The operation is performed within an interrupt-free
    /// critical section to ensure consistency.
    ///
    /// # Generic Parameters
    ///
    /// * `LEN` - Number of samples to copy
    ///
    /// # Parameters
    ///
    /// * `dest` - Destination array to copy the samples into
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use synthphone_e_vocal_dsp::ring_buffer::RingBuffer;
    /// let buffer: RingBuffer<1024> = RingBuffer::new();
    /// let mut block = [0.0f32; 32];
    /// buffer.latest_block(&mut block); // Copy latest 32 samples
    /// ```
    pub fn latest_block<const LEN: usize>(&self, dest: &mut [f32; LEN]) {
        #[cfg(feature = "std")]
        {
            critical_section::with(|_| {
                let w = self.write.load(Ordering::Acquire);
                for (i, value) in dest.iter_mut().enumerate().take(LEN) {
                    let idx = w.wrapping_sub(LEN as u32).wrapping_add(i as u32);
                    *value = unsafe { (*self.buf.get())[idx as usize & (N - 1)] };
                }
            });
        }

        #[cfg(not(feature = "std"))]
        {
            // For embedded targets without std, use atomic snapshot approach
            // This is safe for single-producer/single-consumer usage
            let w = self.write.load(Ordering::Acquire);
            for (i, value) in dest.iter_mut().enumerate().take(LEN) {
                let idx = w.wrapping_sub(LEN as u32).wrapping_add(i as u32);
                *value = unsafe { (*self.buf.get())[idx as usize & (N - 1)] };
            }
        }
    }

    /// Copies a block of samples ending at the specified write index.
    ///
    /// This method copies `LEN` samples from the buffer, with the block ending
    /// at the given write index position. This is useful for processing historical
    /// data or implementing lookahead algorithms.
    ///
    /// # Generic Parameters
    ///
    /// * `LEN` - Number of samples to copy
    ///
    /// # Parameters
    ///
    /// * `write_idx` - The write index position where the block should end
    /// * `dst` - Destination array to copy the samples into
    ///
    /// # Example
    ///
    /// ```rust
    /// use synthphone_e_vocal_dsp::ring_buffer::RingBuffer;
    /// let mut buffer: RingBuffer<1024> = RingBuffer::new();
    /// // ... write some samples ...
    /// let write_pos = buffer.write_index();
    /// let mut block = [0.0f32; 32];
    /// buffer.block_from(write_pos, &mut block); // Copy 32 samples ending at write_pos
    /// ```
    pub fn block_from<const LEN: usize>(&self, write_idx: u32, dst: &mut [f32; LEN]) {
        for (i, item) in dst.iter_mut().enumerate().take(LEN) {
            let idx = write_idx.wrapping_sub(LEN as u32).wrapping_add(i as u32);
            *item = unsafe { (*self.buf.get())[idx as usize & (N - 1)] };
        }
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic_operations() {
        let buffer: RingBuffer<1024> = RingBuffer::new();

        // Test initial state
        assert_eq!(buffer.available_samples(), 0);
        assert_eq!(buffer.write_index(), 0);

        // Test push and pop
        buffer.push(0.5);
        buffer.push(0.25);
        assert_eq!(buffer.available_samples(), 2);

        let sample1 = buffer.pop();
        assert!((sample1 - 0.5).abs() < f32::EPSILON);
        assert_eq!(buffer.available_samples(), 1);

        let sample2 = buffer.pop();
        assert!((sample2 - 0.25).abs() < f32::EPSILON);
        assert_eq!(buffer.available_samples(), 0);
    }

    #[test]
    fn test_ring_buffer_with_offset() {
        let buffer: RingBuffer<1024> = RingBuffer::with_offset(512);
        assert_eq!(buffer.write_index(), 512);
        assert_eq!(buffer.available_samples(), 512);
    }

    #[test]
    fn test_advance_write() {
        let buffer: RingBuffer<1024> = RingBuffer::new();
        buffer.advance_write(10);
        assert_eq!(buffer.write_index(), 10);
        assert_eq!(buffer.available_samples(), 10);
    }

    #[test]
    fn test_add_at_offset() {
        let buffer: RingBuffer<1024> = RingBuffer::new();

        // Push some initial data
        buffer.push(1.0);
        buffer.push(2.0);

        // Add value at offset 0 (current read position)
        buffer.add_at_offset(0, 0.5);

        // Pop and check the modified value
        let sample = buffer.pop();
        assert!((sample - 1.5).abs() < f32::EPSILON); // 1.0 + 0.5
    }

    #[test]
    fn test_latest_block() {
        let buffer: RingBuffer<1024> = RingBuffer::new();

        // Push some test data
        for i in 0..8 {
            buffer.push(i as f32);
        }

        // Get latest 4 samples
        let mut block = [0.0f32; 4];
        buffer.latest_block(&mut block);

        // Should contain samples [4.0, 5.0, 6.0, 7.0]
        assert!((block[0] - 4.0).abs() < f32::EPSILON);
        assert!((block[1] - 5.0).abs() < f32::EPSILON);
        assert!((block[2] - 6.0).abs() < f32::EPSILON);
        assert!((block[3] - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_block_from() {
        let buffer: RingBuffer<1024> = RingBuffer::new();

        // Push some test data
        for i in 0..8 {
            buffer.push(i as f32);
        }

        let write_idx = buffer.write_index();
        let mut block = [0.0f32; 4];
        buffer.block_from(write_idx, &mut block);

        // Should contain the last 4 samples written
        assert!((block[0] - 4.0).abs() < f32::EPSILON);
        assert!((block[1] - 5.0).abs() < f32::EPSILON);
        assert!((block[2] - 6.0).abs() < f32::EPSILON);
        assert!((block[3] - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ring_buffer_wrap_around() {
        let buffer: RingBuffer<4> = RingBuffer::new(); // Small buffer for testing wrap

        // Fill buffer beyond capacity - this will overwrite old data
        for i in 0..8 {
            buffer.push(i as f32);
        }

        // Buffer tracks 8 samples written, but only has space for 4
        assert_eq!(buffer.available_samples(), 8);

        // Due to wrapping, the buffer storage contains the last 4 values: [4, 5, 6, 7]
        // But we can pop 8 times due to the write/read pointer difference
        // First 4 pops get the stored values: 4, 5, 6, 7
        for i in 0..4 {
            let sample = buffer.pop();
            assert!(
                (sample - (i + 4) as f32).abs() < f32::EPSILON,
                "Expected {}, got {}",
                (i + 4) as f32,
                sample
            );
        }

        // Next 4 pops get zeros (from previously cleared slots)
        for _ in 0..4 {
            let sample = buffer.pop();
            assert!((sample - 0.0).abs() < f32::EPSILON);
        }

        // After popping all, buffer should be empty
        assert_eq!(buffer.available_samples(), 0);
    }

    #[test]
    fn test_ring_buffer_overwrite_behavior() {
        let buffer: RingBuffer<4> = RingBuffer::new(); // Small buffer

        // Fill buffer to capacity
        for i in 0..4 {
            buffer.push(i as f32);
        }
        assert_eq!(buffer.available_samples(), 4);

        // Pop all values - should get 0, 1, 2, 3
        for i in 0..4 {
            let sample = buffer.pop();
            assert!((sample - i as f32).abs() < f32::EPSILON);
        }

        // Buffer is now empty
        assert_eq!(buffer.available_samples(), 0);

        // Fill again with new values
        for i in 10..14 {
            buffer.push(i as f32);
        }

        // Should get the new values: 10, 11, 12, 13
        for i in 10..14 {
            let sample = buffer.pop();
            assert!((sample - i as f32).abs() < f32::EPSILON);
        }
    }
}
