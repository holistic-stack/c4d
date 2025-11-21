# Use a pinned Rust version for reproducibility
FROM rust:1.83

# Install the WASM target
RUN rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli
# Note: The version must match the one in Cargo.toml. 
# We'll assume 0.2.99 for now based on typical usage, but ideally we'd check Cargo.lock.
# For now, we install a recent version.
RUN cargo install -f wasm-bindgen-cli --version 0.2.105

# Set working directory
WORKDIR /app

# Copy the entire workspace
# (In a real optimized build, we'd copy Cargo.toml/Lock first to cache dependencies,
# but for this initial implementation, copying everything is simpler and sufficient)
COPY . .

# Create output directory
RUN mkdir -p /out

# Build the WASM crate
# We use --release for optimized WASM
RUN cargo build --release -p wasm --target wasm32-unknown-unknown

# Generate JS bindings
# We output to /out so we can easily extract it later
RUN wasm-bindgen \
    --target web \
    --out-dir /out \
    --out-name wasm \
    target/wasm32-unknown-unknown/release/wasm.wasm

# Command to keep the container running if needed, or just exit
CMD ["ls", "-l", "/out"]
