#![warn(missing_docs)]

//! A library for converting images to and from the 2D frequency domain via FFT.

/// Core FFT image processing types and operations.
pub mod freq;

pub use freq::FreqImage;
pub use rustfft::num_complex::Complex;
