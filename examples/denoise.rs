//! Image denoising via low-pass filtering in the frequency domain.
//!
//! High-frequency noise is removed by keeping only the lower portion
//! of the spectrum. A smooth transition avoids ringing artifacts.
//!
//! Usage: cargo run --example denoise -- data/mandrill.jpg

use freqshow::FreqImage;
use std::fs;

const OUTPUT_DIR: &str = "output";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: denoise <image_file>");
        std::process::exit(1);
    }

    fs::create_dir_all(OUTPUT_DIR)?;

    let mut fi = FreqImage::open(&args[1])?;
    fi.fft_forward();
    let mut centered = fi.fftshift();

    // Keep lower frequencies, remove high-frequency noise.
    // Cutoff 0.15 keeps ~15% of the diagonal; smoothing 0.05 avoids hard edges.
    // Lower cutoff = more blur/denoising; higher = preserves more detail.
    let mask = centered.low_pass_mask(0.15, 0.05);
    centered.apply_filter(&mask);

    // Save the filtered spectrum for comparison.
    centered
        .view_fft_norm()
        .save(format!("{OUTPUT_DIR}/denoised_spectrum.png"))?;
    println!("Wrote {OUTPUT_DIR}/denoised_spectrum.png");

    let mut result = centered.ifftshift();
    result.fft_inverse();
    result
        .to_image()
        .save(format!("{OUTPUT_DIR}/denoised.png"))?;
    println!("Wrote {OUTPUT_DIR}/denoised.png");

    Ok(())
}
