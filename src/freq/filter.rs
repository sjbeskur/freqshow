use super::FreqImage;

impl FreqImage {
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
