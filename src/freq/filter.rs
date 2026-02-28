use super::FreqImage;

impl FreqImage {
    /// Generate a low-pass filter mask, for use on `fftshift`'d data.
    ///
    /// `cutoff` and `smoothing` are fractions of `sqrt(width² + height²)`.
    /// Frequencies within `cutoff - smoothing/2` pass; beyond `cutoff + smoothing/2`
    /// are blocked. Use `smoothing = 0.0` for a hard cutoff.
    ///
    /// ```
    /// use freqshow::{FreqImage, Complex};
    ///
    /// let fi = FreqImage { width: 64, height: 64, data: vec![Complex::default(); 64 * 64] };
    /// let mask = fi.low_pass_mask(0.1, 0.0);
    /// assert_eq!(mask.len(), 64 * 64);
    /// // Center pixel (DC) should pass through.
    /// assert_eq!(mask[32 * 64 + 32], 1.0);
    /// ```
    #[must_use]
    pub fn low_pass_mask(&self, cutoff: f64, smoothing: f64) -> Vec<f64> {
        make_radial_mask(self.width as usize, self.height as usize, cutoff, smoothing)
    }

    /// Generate a high-pass filter mask, for use on `fftshift`'d data.
    ///
    /// `cutoff` and `smoothing` are fractions of `sqrt(width² + height²)`.
    /// Frequencies beyond `cutoff + smoothing/2` pass; within `cutoff - smoothing/2`
    /// are blocked. Use `smoothing = 0.0` for a hard cutoff.
    #[must_use]
    pub fn high_pass_mask(&self, cutoff: f64, smoothing: f64) -> Vec<f64> {
        make_radial_mask(self.width as usize, self.height as usize, cutoff, smoothing)
            .into_iter()
            .map(|v| 1.0 - v)
            .collect()
    }

    /// Generate a band-pass filter mask, for use on `fftshift`'d data.
    ///
    /// Passes frequencies between `low_cutoff` and `high_cutoff` (fractions of
    /// `sqrt(width² + height²)`). `smoothing` controls the transition width at
    /// both edges. Equivalent to element-wise multiplication of a low-pass mask
    /// at `high_cutoff` and a high-pass mask at `low_cutoff`.
    ///
    /// # Panics
    ///
    /// Panics if `low_cutoff >= high_cutoff`.
    ///
    /// ```
    /// use freqshow::{FreqImage, Complex};
    ///
    /// let fi = FreqImage { width: 64, height: 64, data: vec![Complex::default(); 64 * 64] };
    /// let bp = fi.band_pass_mask(0.05, 0.15, 0.0);
    /// // DC component (center) is blocked by the high-pass portion.
    /// assert_eq!(bp[32 * 64 + 32], 0.0);
    /// ```
    #[must_use]
    pub fn band_pass_mask(&self, low_cutoff: f64, high_cutoff: f64, smoothing: f64) -> Vec<f64> {
        assert!(
            low_cutoff < high_cutoff,
            "low_cutoff ({low_cutoff}) must be less than high_cutoff ({high_cutoff})"
        );
        let lp = self.low_pass_mask(high_cutoff, smoothing);
        let hp = self.high_pass_mask(low_cutoff, smoothing);
        lp.into_iter().zip(hp).map(|(l, h)| l * h).collect()
    }

    /// Apply a filter mask in-place (element-wise multiplication).
    ///
    /// # Panics
    ///
    /// Panics if `mask.len()` does not equal `width * height`.
    ///
    /// ```
    /// use freqshow::{FreqImage, Complex};
    ///
    /// let data = vec![Complex::new(1.0, 0.0); 4];
    /// let mut fi = FreqImage { width: 2, height: 2, data };
    /// fi.apply_filter(&[0.5, 0.5, 1.0, 1.0]);
    /// assert_eq!(fi.data[0].re, 0.5);
    /// assert_eq!(fi.data[2].re, 1.0);
    /// ```
    pub fn apply_filter(&mut self, mask: &[f64]) {
        assert_eq!(
            mask.len(),
            self.data.len(),
            "mask length ({}) must equal image size ({}x{} = {})",
            mask.len(),
            self.width,
            self.height,
            self.data.len(),
        );
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
