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

# Run the example with an image file
example file="data/mandrill.jpg":
    cargo run --example freq_out -- {{file}}

# Build, lint, and test
ci: check lint test
