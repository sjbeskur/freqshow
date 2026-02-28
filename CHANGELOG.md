# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-02-28

### Added
- Band-pass filter (`band_pass_mask`) combining low-pass and high-pass masks
- `#[must_use]` annotations on all methods returning values
- Input validation: `apply_filter` panics if mask length doesn't match image size
- `band_pass_mask` panics if `low_cutoff >= high_cutoff`
- Doc-tests on all public methods and crate-level example
- Criterion benchmarks (`cargo bench --bench fft_bench`)
- GitHub Actions CI (check, test, clippy, fmt, doc-tests, benchmarks)
- README badges (crates.io, docs.rs, CI, benchmarks, license)

### Changed
- Split `src/freq.rs` into `src/freq/` module directory:
  - `mod.rs` — struct definition, I/O, tests
  - `fft.rs` — forward/inverse FFT
  - `filter.rs` — mask generation and application
  - `shift.rs` — fftshift/ifftshift
- Updated `documentation` field in Cargo.toml to point to docs.rs

## [0.2.2] - 2026-02-28

### Fixed
- Documentation link now points to docs.rs instead of GitHub

## [0.2.1] - 2026-02-28

### Added
- `FreqImage` struct wrapping width, height, and complex buffer
- All operations as methods on `FreqImage`
- `FreqImage::open()` returns `Result` for proper error handling
- `FreqImage::from_image()` accepts `DynamicImage` (auto-converts color to grayscale)
- `Derive(Clone, Debug)` on `FreqImage`

### Changed
- Refactored from free functions to struct-based API

## [0.1.2] - 2026-02-28

### Added
- `fft_inverse` — 2D inverse FFT with normalization
- `fftshift` / `ifftshift` — DC component centering (odd-dimension aware)
- `view_fft_norm` — log-scale magnitude visualization
- `low_pass_filter` and `high_pass_filter` with smooth radial masks
- `apply_filter` for element-wise mask application

### Fixed
- Buffer layout: `fft_forward` now transposes back to row-major so layout is consistent

## [0.1.0] - Initial release

### Added
- `read_image` — load image from disk
- `dynimg2complex` — convert grayscale pixels to complex buffer
- `fft_forward` — 2D forward FFT

[0.3.0]: https://github.com/sjbeskur/freqshow/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/sjbeskur/freqshow/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/sjbeskur/freqshow/compare/v0.1.2...v0.2.1
[0.1.2]: https://github.com/sjbeskur/freqshow/compare/v0.1.0...v0.1.2
[0.1.0]: https://github.com/sjbeskur/freqshow/releases/tag/v0.1.0
