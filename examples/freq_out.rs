use freqshow::freq::{read_image};
use rustfft::num_complex::Complex;
use image::{GrayImage};


fn main() {
    println!("Don't freq out: {}","enter a filename");
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 1 {
        panic!("Usage: freqshow <image_file_name>");
    }
    let image = read_image( args[1].clone()); //mandrill.jpg


    let img = image::open("data/mandrill.jpg")?.into_luma8();
    let (width, height) = img.dimensions();

    // Convert the image buffer to complex numbers to be able to compute the FFT.
    let mut img_buffer: Vec<Complex<f64>> = img
        .as_raw()
        .iter()
        .map(|&pix| Complex::new(pix as f64 / 255.0, 0.0))
        .collect();
    fft_2d(width as usize, height as usize, &mut img_buffer);

    // Shift opposite quadrants of the fft (like matlab fftshift).
    img_buffer = fftshift(height as usize, width as usize, &img_buffer);

    // Apply a low-pass filter (10% radius, smoothed on 2%).
    let low_pass = low_pass_filter(height as usize, width as usize);
    let fft_low_pass: Vec<Complex<f64>> = low_pass
        .iter()
        .zip(&img_buffer)
        .map(|(l, b)| l * b)
        .collect();
    let fft_norm_img_low = view_fft_norm(height, width, &fft_low_pass);

    fft_norm_img_low.save("data/fft_low_pass.png");

}

/// Convert the norm of the (transposed) FFT 2d transform into an image for visualization.
/// Use a logarithm scale.
fn view_fft_norm(width: u32, height: u32, img_buffer: &[Complex<f64>]) -> GrayImage {
    let fft_log_norm: Vec<f64> = img_buffer.iter().map(|c| c.norm().ln()).collect();
    let max_norm = fft_log_norm.iter().cloned().fold(0.0 / 0.0, f64::max);
    let fft_norm_u8: Vec<u8> = fft_log_norm
        .into_iter()
        .map(|x| ((x / max_norm) * 255.0) as u8)
        .collect();
    GrayImage::from_raw(width, height, fft_norm_u8).unwrap()
}

/// Apply a low-pass filter (6% radius, smoothed on 2%).
fn low_pass_filter(width: usize, height: usize) -> Vec<f64> {
    let diagonal = ((width * width + height * height) as f64).sqrt();
    let radius_in_sqr = (0.06 * diagonal).powi(2);
    let radius_out_sqr = (0.08 * diagonal).powi(2);
    let center_x = (width - 1) as f64 / 2.0;
    let center_y = (height - 1) as f64 / 2.0;
    let mut buffer = vec![0.0; width * height];
    for (i, row) in buffer.chunks_exact_mut(width).enumerate() {
        for (j, pix) in row.iter_mut().enumerate() {
            let dist_sqr = (center_x - j as f64).powi(2) + (center_y - i as f64).powi(2);
            *pix = if dist_sqr < radius_in_sqr {
                1.0
            } else if dist_sqr > radius_out_sqr {
                0.0
            } else {
                ((radius_out_sqr - dist_sqr) / (radius_out_sqr - radius_in_sqr)).powi(2)
            }
        }
    }
    buffer
}
