# List available recipes
default:
    @just --list

# Build the library
build:
    cargo build

# Build in release mode
release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run a single test by name
test-one name:
    cargo test {{name}}

# Run clippy lints
lint:
    cargo clippy

# Check without building
check:
    cargo check

# Run the freq_out example
ex-freq-out file="data/mandrill.jpg":
    cargo run --example freq_out -- {{file}}

# Run the edge detection example
ex-edge-detect file="data/mandrill.jpg":
    cargo run --example edge_detect -- {{file}}

# Run the sharpening example
ex-sharpen file="data/mandrill.jpg":
    cargo run --example sharpen -- {{file}}

# Run the denoising example
ex-denoise file="data/mandrill.jpg":
    cargo run --example denoise -- {{file}}

# Run all examples
examples file="data/mandrill.jpg":
    cargo run --example freq_out -- {{file}}
    cargo run --example edge_detect -- {{file}}
    cargo run --example sharpen -- {{file}}
    cargo run --example denoise -- {{file}}

# Run benchmarks
bench:
    cargo bench --bench fft_bench

# Build, lint, and test
ci: check lint test
