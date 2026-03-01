//! Edge detection via high-pass filtering in the frequency domain.
//!
//! Usage: cargo run --example edge_detect -- data/mandrill.jpg

use freqshow::FreqImage;
use std::fs;

const OUTPUT_DIR: &str = "output";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: edge_detect <image_file>");
        std::process::exit(1);
    }

    fs::create_dir_all(OUTPUT_DIR)?;

    let mut fi = FreqImage::open(&args[1])?;
    fi.fft_forward();
    let mut centered = fi.fftshift();

    // Block low frequencies (smooth regions), keep high frequencies (edges).
    // A small cutoff catches fine edges; increase for coarser edges.
    // Cutoff	Effect
    // 0.02	    Fine edges, lots of detail
    // 0.05	    Current default
    // 0.10	    Thicker, more prominent edges
    // 0.15	    Very thick edges, coarse features only

    let mask = centered.high_pass_mask(0.02, 0.02);
    centered.apply_filter(&mask);

    let mut result = centered.ifftshift();
    result.fft_inverse();
    result.to_image().save(format!("{OUTPUT_DIR}/edges.png"))?;
    println!("Wrote {OUTPUT_DIR}/edges.png");

    Ok(())
}
