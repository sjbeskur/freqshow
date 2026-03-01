//! Image sharpening by boosting high-frequency components.
//!
//! The idea: `sharpened = original + strength * high_pass(original)`
//! In the frequency domain we combine the original spectrum with an
//! amplified high-pass version, then inverse-FFT the result.
//!
//! Usage: cargo run --example sharpen -- data/mandrill.jpg

use freqshow::{Complex, FreqImage};
use std::fs;

const OUTPUT_DIR: &str = "output";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: sharpen <image_file>");
        std::process::exit(1);
    }

    fs::create_dir_all(OUTPUT_DIR)?;

    let mut fi = FreqImage::open(&args[1])?;
    fi.fft_forward();
    let centered = fi.fftshift();

    // Extract high-frequency detail.
    let hp_mask = centered.high_pass_mask(0.05, 0.02);

    // Build a sharpening mask: 1.0 + strength * high_pass_mask.
    // strength > 0 boosts edges; try 1.0–3.0.
    let strength = 2.0;
    let sharpen_mask: Vec<f64> = hp_mask.iter().map(|&h| 1.0 + strength * h).collect();

    let mut sharpened = centered.clone();
    sharpened.apply_filter(&sharpen_mask);

    let mut result = sharpened.ifftshift();
    result.fft_inverse();

    // Clamp values that exceed [0,1] from the boost.
    for c in &mut result.data {
        *c = Complex::new(c.re.clamp(0.0, 1.0), 0.0);
    }

    result
        .to_image()
        .save(format!("{OUTPUT_DIR}/sharpened.png"))?;
    println!("Wrote {OUTPUT_DIR}/sharpened.png (strength={strength})");

    Ok(())
}
