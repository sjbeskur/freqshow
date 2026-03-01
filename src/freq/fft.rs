use super::FreqImage;
use rustfft::{FftDirection, FftPlanner, num_complex::Complex};

impl FreqImage {
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
}

pub(crate) fn transpose<T: Copy + Default>(width: usize, height: usize, matrix: &[T]) -> Vec<T> {
    let mut transposed = vec![T::default(); matrix.len()];
    for row in 0..height {
        for col in 0..width {
            transposed[col * height + row] = matrix[row * width + col];
        }
    }
    transposed
}
