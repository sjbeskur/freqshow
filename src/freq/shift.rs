use rustfft::num_complex::Complex;
use super::FreqImage;

impl FreqImage {
    /// Shift the DC component to the center of the spectrum (like MATLAB's `fftshift`).
    ///
    /// ```
    /// use freqshow::{FreqImage, Complex};
    ///
    /// let mut data = vec![Complex::default(); 4];
    /// data[0] = Complex::new(1.0, 0.0); // DC at top-left
    /// let fi = FreqImage { width: 2, height: 2, data };
    /// let shifted = fi.fftshift();
    /// // DC moves to center (bottom-right for 2x2).
    /// assert_eq!(shifted.data[3].re, 1.0);
    /// ```
    #[must_use]
    pub fn fftshift(&self) -> Self {
        let (w, h) = (self.width as usize, self.height as usize);
        let data = quadrant_shift(w, h, &self.data, w / 2, h / 2);
        FreqImage { width: self.width, height: self.height, data }
    }

    /// Shift the DC component back to the corners (inverse of `fftshift`).
    ///
    /// For even dimensions this is identical to `fftshift`. For odd dimensions
    /// this correctly shifts by `ceil(N/2)`.
    #[must_use]
    pub fn ifftshift(&self) -> Self {
        let (w, h) = (self.width as usize, self.height as usize);
        let data = quadrant_shift(w, h, &self.data, w.div_ceil(2), h.div_ceil(2));
        FreqImage { width: self.width, height: self.height, data }
    }
}

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
