use freqshow::FreqImage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: freq_out <image_file>");
        std::process::exit(1);
    }

    let mut fi = FreqImage::open(&args[1])?;
    fi.fft_forward();

    // Visualize the centered spectrum.
    let centered = fi.fftshift();
    centered.view_fft_norm().save("spectrum.png")?;
    println!("Wrote spectrum.png");

    // Low-pass filter: keep the central 10% of the diagonal, 2% smooth transition.
    let mut lp = centered.clone();
    let lp_mask = lp.low_pass_mask(0.10, 0.02);
    lp.apply_filter(&lp_mask);
    let mut lp = lp.ifftshift();
    lp.fft_inverse();
    lp.to_image().save("low_pass.png")?;
    println!("Wrote low_pass.png");

    // High-pass filter: block the central 10%, 2% smooth transition.
    let mut hp = centered.clone();
    let hp_mask = hp.high_pass_mask(0.10, 0.02);
    hp.apply_filter(&hp_mask);
    let mut hp = hp.ifftshift();
    hp.fft_inverse();
    hp.to_image().save("high_pass.png")?;
    println!("Wrote high_pass.png");

    Ok(())
}
