mod fft;
mod filter;
mod shift;

use rustfft::num_complex::Complex;
use std::path::Path;

/// A grayscale image represented as a complex buffer, suitable for FFT operations.
///
/// Carries `width`, `height`, and `data` together so dimensions never get out of sync.
///
/// # Example
///
/// ```
/// use freqshow::{FreqImage, Complex};
///
/// let data = vec![Complex::new(0.5, 0.0); 8 * 8];
/// let mut fi = FreqImage { width: 8, height: 8, data };
/// fi.fft_forward();
/// fi.fft_inverse();
/// assert!((fi.data[0].re - 0.5).abs() < 1e-10);
/// ```
#[derive(Clone, Debug)]
pub struct FreqImage {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Complex buffer of length `width * height`.
    pub data: Vec<Complex<f64>>,
}

impl FreqImage {
    /// Load an image from disk and convert it to a complex buffer.
    ///
    /// Color images are automatically converted to grayscale (luma8).
    pub fn open(path: impl AsRef<Path>) -> Result<Self, image::ImageError> {
        let img = image::open(path)?;
        Ok(Self::from_image(img))
    }

    /// Convert a `DynamicImage` to a complex buffer.
    ///
    /// Color images are automatically converted to grayscale (luma8).
    /// Each pixel `[0, 255]` maps to a real component in `[0.0, 1.0]`.
    #[must_use]
    pub fn from_image(img: image::DynamicImage) -> Self {
        let gray = img.into_luma8();
        let (width, height) = gray.dimensions();
        let data = gray
            .as_raw()
            .iter()
            .map(|&pix| Complex::new(pix as f64 / 255.0, 0.0))
            .collect();
        FreqImage {
            width,
            height,
            data,
        }
    }

    /// Convert the complex buffer back to a grayscale image.
    ///
    /// Takes the real component of each value, clamps to `[0.0, 1.0]`,
    /// and scales to `[0, 255]`.
    ///
    /// ```
    /// use freqshow::{FreqImage, Complex};
    ///
    /// let data = vec![Complex::new(0.5, 0.0); 4];
    /// let fi = FreqImage { width: 2, height: 2, data };
    /// let img = fi.to_image();
    /// assert_eq!(img.dimensions(), (2, 2));
    /// assert_eq!(img.as_raw()[0], 127); // 0.5 * 255 ≈ 127
    /// ```
    #[must_use]
    pub fn to_image(&self) -> image::GrayImage {
        let pixels: Vec<u8> = self
            .data
            .iter()
            .map(|c| (c.re.clamp(0.0, 1.0) * 255.0) as u8)
            .collect();
        image::GrayImage::from_raw(self.width, self.height, pixels).unwrap()
    }

    /// Render the magnitude of the buffer as a grayscale image using a log scale.
    ///
    /// Applies `ln(1 + |c|)` per pixel and normalizes to `[0, 255]`.
    ///
    /// ```
    /// use freqshow::{FreqImage, Complex};
    ///
    /// let data = vec![Complex::new(1.0, 0.0); 4];
    /// let fi = FreqImage { width: 2, height: 2, data };
    /// let vis = fi.view_fft_norm();
    /// assert_eq!(vis.dimensions(), (2, 2));
    /// ```
    #[must_use]
    pub fn view_fft_norm(&self) -> image::GrayImage {
        let log_norms: Vec<f64> = self.data.iter().map(|c| (1.0 + c.norm()).ln()).collect();
        let max = log_norms.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let pixels: Vec<u8> = log_norms
            .into_iter()
            .map(|x| {
                if max > 0.0 {
                    (x / max * 255.0) as u8
                } else {
                    0
                }
            })
            .collect();
        image::GrayImage::from_raw(self.width, self.height, pixels).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        for file in &["data/sjb-aerial.png", "data/mandrill.jpg"] {
            let original = image::open(file).unwrap().into_luma8();
            let original_pixels = original.as_raw().clone();

            let mut fi = FreqImage::open(file).unwrap();
            fi.fft_forward();
            fi.fft_inverse();
            let recovered = fi.to_image();

            for (&orig, &rec) in original_pixels.iter().zip(recovered.as_raw().iter()) {
                assert!(orig.abs_diff(rec) <= 1, "pixel mismatch: {orig} vs {rec}");
            }
        }
    }

    #[test]
    fn test_fftshift_double_shift_is_identity() {
        let fi = FreqImage::open("data/mandrill.jpg").unwrap();
        let shifted = fi.fftshift();
        let restored = shifted.fftshift();
        for (a, b) in fi.data.iter().zip(restored.data.iter()) {
            assert!((a.re - b.re).abs() < 1e-10);
        }
    }

    #[test]
    fn test_ifftshift_inverts_fftshift() {
        let fi = FreqImage::open("data/mandrill.jpg").unwrap();
        let shifted = fi.fftshift();
        let restored = shifted.ifftshift();
        for (a, b) in fi.data.iter().zip(restored.data.iter()) {
            assert!((a.re - b.re).abs() < 1e-10);
        }
    }

    #[test]
    fn test_low_high_pass_masks_sum_to_one() {
        let fi = FreqImage {
            width: 64,
            height: 64,
            data: vec![Complex::default(); 64 * 64],
        };
        let lp = fi.low_pass_mask(0.10, 0.02);
        let hp = fi.high_pass_mask(0.10, 0.02);
        for (l, h) in lp.iter().zip(hp.iter()) {
            assert!(
                (l + h - 1.0).abs() < 1e-10,
                "masks don't sum to 1: {l} + {h}"
            );
        }
    }

    #[test]
    fn test_band_pass_mask_bounded_by_low_and_high() {
        let fi = FreqImage {
            width: 64,
            height: 64,
            data: vec![Complex::default(); 64 * 64],
        };
        let bp = fi.band_pass_mask(0.05, 0.15, 0.0);
        let lp = fi.low_pass_mask(0.15, 0.0);
        let hp = fi.high_pass_mask(0.05, 0.0);
        for ((&b, &l), &h) in bp.iter().zip(lp.iter()).zip(hp.iter()) {
            assert!(
                (b - l * h).abs() < 1e-10,
                "band-pass != low*high: {b} vs {}",
                l * h
            );
        }
    }

    #[test]
    fn test_view_fft_norm_produces_correct_size() {
        let mut fi = FreqImage::open("data/mandrill.jpg").unwrap();
        fi.fft_forward();
        let vis = fi.view_fft_norm();
        assert_eq!(vis.dimensions(), (fi.width, fi.height));
    }

    #[test]
    fn test_open_nonexistent_returns_error() {
        assert!(FreqImage::open("nonexistent.png").is_err());
    }
}
