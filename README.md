# freqshow

[![Crates.io](https://img.shields.io/crates/v/freqshow)](https://crates.io/crates/freqshow)
[![docs.rs](https://img.shields.io/docsrs/freqshow)](https://docs.rs/freqshow)
[![CI](https://github.com/sjbeskur/freqshow/actions/workflows/ci.yml/badge.svg)](https://github.com/sjbeskur/freqshow/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/freqshow)](https://github.com/sjbeskur/freqshow#license)

A Rust library for converting images to and from the 2D frequency domain via FFT.

| Original | Spectrum | Edge Detection |
|----------|----------|----------------|
| ![Lena](docs/images/lena.jpg) | ![Spectrum](docs/images/lena_spectrum.png) | ![Edges](docs/images/lena_edges.png) |

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
freqshow = "0.3"
```

### Example

```rust
use freqshow::FreqImage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load any image (color images are converted to grayscale automatically).
    let mut fi = FreqImage::open("photo.jpg")?;

    // Forward FFT.
    fi.fft_forward();

    // Center DC and apply a low-pass filter.
    let mut centered = fi.fftshift();
    let mask = centered.low_pass_mask(0.10, 0.02);
    centered.apply_filter(&mask);

    // Inverse FFT and save the result.
    let mut result = centered.ifftshift();
    result.fft_inverse();
    result.to_image().save("filtered.png")?;

    Ok(())
}
```

### API overview

| Method | Description |
|--------|-------------|
| `FreqImage::open(path)` | Load an image from disk (returns `Result`) |
| `FreqImage::from_image(img)` | Convert a `DynamicImage` to a complex buffer |
| `fi.fft_forward()` | 2D forward FFT in-place |
| `fi.fft_inverse()` | 2D inverse FFT in-place (normalized) |
| `fi.fftshift()` / `fi.ifftshift()` | Center / uncenter the DC component |
| `fi.to_image()` | Convert back to a `GrayImage` |
| `fi.view_fft_norm()` | Log-scale magnitude visualization |
| `fi.low_pass_mask(cutoff, smoothing)` | Generate a low-pass filter mask |
| `fi.high_pass_mask(cutoff, smoothing)` | Generate a high-pass filter mask |
| `fi.band_pass_mask(low, high, smoothing)` | Generate a band-pass filter mask |
| `fi.apply_filter(mask)` | Apply a filter mask in-place |

## Examples

```bash
cargo run --example freq_out -- data/mandrill.jpg      # spectrum, low/high-pass
cargo run --example edge_detect -- data/mandrill.jpg   # edge detection
cargo run --example sharpen -- data/mandrill.jpg       # sharpening
cargo run --example denoise -- data/mandrill.jpg       # denoising
```

All output is written to the `output/` directory.

## License

Licensed under either of [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0) or [MIT license](http://opensource.org/licenses/MIT) at your option.
