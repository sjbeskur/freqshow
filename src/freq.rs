use std::path::Path;
use rustfft::{FftPlanner, num_complex::Complex, FftDirection};

/// A grayscale image represented as a complex buffer, suitable for FFT operations.
///
/// Carries `width`, `height`, and `data` together so dimensions never get out of sync.
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
    pub fn from_image(img: image::DynamicImage) -> Self {
        let gray = img.into_luma8();
        let (width, height) = gray.dimensions();
        let data = gray
            .as_raw()
            .iter()
            .map(|&pix| Complex::new(pix as f64 / 255.0, 0.0))
            .collect();
        FreqImage { width, height, data }
    }

    /// Convert the complex buffer back to a grayscale image.
    ///
    /// Takes the real component of each value, clamps to `[0.0, 1.0]`,
    /// and scales to `[0, 255]`.
    pub fn to_image(&self) -> image::GrayImage {
        let pixels: Vec<u8> = self
            .data
            .iter()
            .map(|c| (c.re.clamp(0.0, 1.0) * 255.0) as u8)
            .collect();
        image::GrayImage::from_raw(self.width, self.height, pixels).unwrap()
    }

    /// Perform a 2D forward FFT in-place.
    ///
    /// The buffer remains in row-major layout after this call.
    pub fn fft_forward(&mut self) {
        let (w, h) = (self.width as usize, self.height as usize);
        let mut planner = FftPlanner::new();

        // Row-wise FFT.
        let fft_width = planner.plan_fft(w, FftDirection::Forward);
        let mut scratch = vec![Complex::default(); fft_width.get_inplace_scratch_len()];
        for row in self.data.chunks_exact_mut(w) {
            fft_width.process_with_scratch(row, &mut scratch);
        }

        // Transpose so columns become accessible as contiguous rows.
        let mut transposed = transpose(w, h, &self.data);

        // Column-wise FFT (operating on transposed rows).
        let fft_height = planner.plan_fft(h, FftDirection::Forward);
        scratch.resize(fft_height.get_inplace_scratch_len(), Complex::default());
        for col in transposed.chunks_exact_mut(h) {
            fft_height.process_with_scratch(col, &mut scratch);
        }

        // Transpose back to row-major so the buffer layout is always consistent.
        self.data = transpose(h, w, &transposed);
    }

    /// Perform a 2D inverse FFT in-place, including normalization.
    pub fn fft_inverse(&mut self) {
        let (w, h) = (self.width as usize, self.height as usize);
        let mut planner = FftPlanner::new();

        // Transpose so columns are contiguous.
        let mut transposed = transpose(w, h, &self.data);

        // Column-wise IFFT.
        let fft_height = planner.plan_fft(h, FftDirection::Inverse);
        let mut scratch = vec![Complex::default(); fft_height.get_inplace_scratch_len()];
        for col in transposed.chunks_exact_mut(h) {
            fft_height.process_with_scratch(col, &mut scratch);
        }

        // Transpose back to row-major.
        self.data = transpose(h, w, &transposed);

        // Row-wise IFFT.
        let fft_width = planner.plan_fft(w, FftDirection::Inverse);
        scratch.resize(fft_width.get_inplace_scratch_len(), Complex::default());
        for row in self.data.chunks_exact_mut(w) {
            fft_width.process_with_scratch(row, &mut scratch);
        }

        let norm = (w * h) as f64;
        for c in self.data.iter_mut() {
            *c /= norm;
        }
    }

    /// Shift the DC component to the center of the spectrum (like MATLAB's `fftshift`).
    pub fn fftshift(&self) -> Self {
        let (w, h) = (self.width as usize, self.height as usize);
        let data = quadrant_shift(w, h, &self.data, w / 2, h / 2);
        FreqImage { width: self.width, height: self.height, data }
    }

    /// Shift the DC component back to the corners (inverse of `fftshift`).
    ///
    /// For even dimensions this is identical to `fftshift`. For odd dimensions
    /// this correctly shifts by `ceil(N/2)`.
    pub fn ifftshift(&self) -> Self {
        let (w, h) = (self.width as usize, self.height as usize);
        let data = quadrant_shift(w, h, &self.data, w.div_ceil(2), h.div_ceil(2));
        FreqImage { width: self.width, height: self.height, data }
    }

    /// Render the magnitude of the buffer as a grayscale image using a log scale.
    ///
    /// Applies `ln(1 + |c|)` per pixel and normalizes to `[0, 255]`.
    pub fn view_fft_norm(&self) -> image::GrayImage {
        let log_norms: Vec<f64> = self.data.iter().map(|c| (1.0 + c.norm()).ln()).collect();
        let max = log_norms.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let pixels: Vec<u8> = log_norms
            .into_iter()
            .map(|x| if max > 0.0 { (x / max * 255.0) as u8 } else { 0 })
            .collect();
        image::GrayImage::from_raw(self.width, self.height, pixels).unwrap()
    }

    /// Generate a low-pass filter mask, for use on `fftshift`'d data.
    ///
    /// `cutoff` and `smoothing` are fractions of `sqrt(width² + height²)`.
    /// Frequencies within `cutoff - smoothing/2` pass; beyond `cutoff + smoothing/2`
    /// are blocked. Use `smoothing = 0.0` for a hard cutoff.
    pub fn low_pass_mask(&self, cutoff: f64, smoothing: f64) -> Vec<f64> {
        make_radial_mask(self.width as usize, self.height as usize, cutoff, smoothing)
    }

    /// Generate a high-pass filter mask, for use on `fftshift`'d data.
    ///
    /// `cutoff` and `smoothing` are fractions of `sqrt(width² + height²)`.
    /// Frequencies beyond `cutoff + smoothing/2` pass; within `cutoff - smoothing/2`
    /// are blocked. Use `smoothing = 0.0` for a hard cutoff.
    pub fn high_pass_mask(&self, cutoff: f64, smoothing: f64) -> Vec<f64> {
        make_radial_mask(self.width as usize, self.height as usize, cutoff, smoothing)
            .into_iter()
            .map(|v| 1.0 - v)
            .collect()
    }

    /// Apply a filter mask in-place (element-wise multiplication).
    pub fn apply_filter(&mut self, mask: &[f64]) {
        for (c, &m) in self.data.iter_mut().zip(mask.iter()) {
            *c *= m;
        }
    }
}

impl Clone for FreqImage {
    fn clone(&self) -> Self {
        FreqImage {
            width: self.width,
            height: self.height,
            data: self.data.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn quadrant_shift(
    width: usize,
    height: usize,
    buffer: &[Complex<f64>],
    shift_w: usize,
    shift_h: usize,
) -> Vec<Complex<f64>> {
    let mut shifted = vec![Complex::default(); buffer.len()];
    for row in 0..height {
        for col in 0..width {
            let new_row = (row + shift_h) % height;
            let new_col = (col + shift_w) % width;
            shifted[new_row * width + new_col] = buffer[row * width + col];
        }
    }
    shifted
}

fn make_radial_mask(width: usize, height: usize, cutoff: f64, smoothing: f64) -> Vec<f64> {
    let diagonal = ((width * width + height * height) as f64).sqrt();
    let r_in_sq = ((cutoff - smoothing / 2.0).max(0.0) * diagonal).powi(2);
    let r_out_sq = ((cutoff + smoothing / 2.0) * diagonal).powi(2);
    let cx = (width - 1) as f64 / 2.0;
    let cy = (height - 1) as f64 / 2.0;
    let mut mask = vec![0.0f64; width * height];
    for (i, row) in mask.chunks_exact_mut(width).enumerate() {
        for (j, pix) in row.iter_mut().enumerate() {
            let d2 = (cx - j as f64).powi(2) + (cy - i as f64).powi(2);
            *pix = if d2 <= r_in_sq {
                1.0
            } else if d2 >= r_out_sq {
                0.0
            } else {
                ((r_out_sq - d2) / (r_out_sq - r_in_sq)).powi(2)
            };
        }
    }
    mask
}

fn transpose<T: Copy + Default>(width: usize, height: usize, matrix: &[T]) -> Vec<T> {
    let mut transposed = vec![T::default(); matrix.len()];
    for row in 0..height {
        for col in 0..width {
            transposed[col * height + row] = matrix[row * width + col];
        }
    }
    transposed
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        for file in &["img/sjb-aerial.png", "img/mandrill.jpg"] {
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
        let fi = FreqImage::open("img/mandrill.jpg").unwrap();
        let shifted = fi.fftshift();
        let restored = shifted.fftshift();
        for (a, b) in fi.data.iter().zip(restored.data.iter()) {
            assert!((a.re - b.re).abs() < 1e-10);
        }
    }

    #[test]
    fn test_ifftshift_inverts_fftshift() {
        let fi = FreqImage::open("img/mandrill.jpg").unwrap();
        let shifted = fi.fftshift();
        let restored = shifted.ifftshift();
        for (a, b) in fi.data.iter().zip(restored.data.iter()) {
            assert!((a.re - b.re).abs() < 1e-10);
        }
    }

    #[test]
    fn test_low_high_pass_masks_sum_to_one() {
        let fi = FreqImage { width: 64, height: 64, data: vec![Complex::default(); 64 * 64] };
        let lp = fi.low_pass_mask(0.10, 0.02);
        let hp = fi.high_pass_mask(0.10, 0.02);
        for (l, h) in lp.iter().zip(hp.iter()) {
            assert!((l + h - 1.0).abs() < 1e-10, "masks don't sum to 1: {l} + {h}");
        }
    }

    #[test]
    fn test_view_fft_norm_produces_correct_size() {
        let mut fi = FreqImage::open("img/mandrill.jpg").unwrap();
        fi.fft_forward();
        let vis = fi.view_fft_norm();
        assert_eq!(vis.dimensions(), (fi.width, fi.height));
    }

    #[test]
    fn test_open_nonexistent_returns_error() {
        assert!(FreqImage::open("nonexistent.png").is_err());
    }
}
