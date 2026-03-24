# Atrium workspace justfile

root := justfile_directory()

# Format all code
format:
    cargo fmt

# Check formatting without modifying
format-check:
    cargo fmt -- --check

# Run clippy with deny warnings
lint:
    cargo clippy --all-targets --workspace -- -D warnings

# Run all tests
test:
    cargo test --workspace

# Run tests matching a filter
stest filter:
    cargo test --workspace -- {{filter}}

# Full CI suite: format, lint, test
ci:
    just format
    just lint
    just test

# Build the GUI binary
build:
    cargo build -p atrium-gui

# Build in release mode
release:
    cargo build --release -p atrium-gui

# Run the GUI
run:
    cargo run -p atrium-gui

# Check all crates compile
check:
    cargo check --workspace
