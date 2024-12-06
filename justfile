# Run linting
lint:
    cargo clippy --all-targets -- -D clippy::all -D clippy::nursery

# Check formatting
fmt:
    cargo fmt --check

# Check docs
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc
    
# Verify all compiles
check:
    cargo check
    
# Run unit tests
test-unit:
    cargo test --lib

# Run integration tests
test-integration:
    RUST_TEST_THREADS=1 cargo test --test '*'

# Build the project
build:
    cargo build