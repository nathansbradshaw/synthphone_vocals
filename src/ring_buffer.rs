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
/// let buffer: RingBuffer<1024> = RingBuffer::new();
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
    /// # Examples
    ///
    /// ```rust
    /// // Create buffer with 128 samples of "headroom"
    /// let buffer: RingBuffer<1024> = RingBuffer::with_offset(128);
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
    /// # Examples
    ///
    /// ```rust
    /// const BUFFER: RingBuffer<1024> = RingBuffer::new();
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
    /// # Examples
    ///
    /// ```rust
    /// let buffer: RingBuffer<1024> = RingBuffer::new();
    /// buffer.push(0.75);
    /// buffer.push(-0.25);
    /// ```
    #[inline(always)]
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
    /// # Examples
    ///
    /// ```rust
    /// let buffer: RingBuffer<1024> = RingBuffer::new();
    /// buffer.push(0.5);
    /// let sample = buffer.pop(); // Returns 0.5
    /// ```
    #[inline(always)]
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
    /// # Examples
    ///
    /// ```rust
    /// let buffer: RingBuffer<1024> = RingBuffer::new();
    /// buffer.push(0.5);
    /// assert_eq!(buffer.write_index(), 1);
    /// ```
    #[inline(always)]
    pub fn write_index(&self) -> u32 {
        self.write.load(core::sync::atomic::Ordering::Relaxed)
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
    /// # Examples
    ///
    /// ```rust
    /// let buffer: RingBuffer<1024> = RingBuffer::new();
    /// buffer.advance_write(64); // Reserve 64 samples
    /// ```
    #[inline(always)]
    pub fn advance_write(&self, n: u32) {
        use core::sync::atomic::Ordering;
        self.write.fetch_add(n, Ordering::Relaxed);
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
    /// # Examples
    ///
    /// ```rust
    /// let buffer: RingBuffer<1024> = RingBuffer::new();
    /// buffer.add_at_offset(10, 0.5); // Add 0.5 to sample 10 positions ahead
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
    /// # Examples
    ///
    /// ```rust
    /// let buffer: RingBuffer<1024> = RingBuffer::new();
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
    /// # Examples
    ///
    /// ```rust
    /// let buffer: RingBuffer<1024> = RingBuffer::new();
    /// // ... write some samples ...
    /// let mut block = [0.0f32; 64];
    /// buffer.latest_block(&mut block); // Copy last 64 samples
    /// ```
    pub fn latest_block<const LEN: usize>(&self, dest: &mut [f32; LEN]) {
        cortex_m::interrupt::free(|_| {
            let w = self.write.load(Ordering::Acquire);
            for i in 0..LEN {
                let idx = w.wrapping_sub(LEN as u32).wrapping_add(i as u32);
                dest[i] = unsafe { (*self.buf.get())[idx as usize & (N - 1)] };
            }
        });
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
    /// # Examples
    ///
    /// ```rust
    /// let buffer: RingBuffer<1024> = RingBuffer::new();
    /// // ... write some samples ...
    /// let write_pos = buffer.write_index();
    /// let mut block = [0.0f32; 32];
    /// buffer.block_from(write_pos, &mut block); // Copy 32 samples ending at write_pos
    /// ```
    pub fn block_from<const LEN: usize>(&self, write_idx: u32, dst: &mut [f32; LEN]) {
        for i in 0..LEN {
            let idx = write_idx.wrapping_sub(LEN as u32).wrapping_add(i as u32);
            dst[i] = unsafe { (*self.buf.get())[idx as usize & (N - 1)] };
        }
    }
}
