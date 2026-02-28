#![warn(missing_docs)]

//! A library for converting images to and from the 2D frequency domain via FFT.
//!
//! # Example
//!
//! ```
//! use freqshow::{FreqImage, Complex};
//!
//! // Create a small 4x4 image from raw complex data.
//! let data: Vec<Complex<f64>> = (0..16)
//!     .map(|i| Complex::new(i as f64 / 16.0, 0.0))
//!     .collect();
//! let mut fi = FreqImage { width: 4, height: 4, data };
//!
//! // Forward FFT, inspect spectrum, then inverse FFT.
//! fi.fft_forward();
//! let spectrum = fi.fftshift();
//! let _vis = spectrum.view_fft_norm();
//! fi.fft_inverse();
//!
//! // Pixel values are recovered (within floating-point tolerance).
//! assert!((fi.data[0].re - 0.0).abs() < 1e-10);
//! ```

/// Core FFT image processing types and operations.
pub mod freq;

pub use freq::FreqImage;
pub use rustfft::num_complex::Complex;
