# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

toychacha-rs is an educational implementation of the ChaCha20-Poly1305 AEAD (Authenticated Encryption with Associated Data) cipher in Rust. This is a toy implementation for learning purposes and should not be used in production.

## Commands

### Build and Test
```bash
# Run all tests
cargo test --all-features

# Run tests with output
cargo test -- --nocapture

# Build the project
cargo build

# Build with optimizations
cargo build --release

# Run a specific test
cargo test test_name

# Run tests in a specific module
cargo test chacha::  # for ChaCha20 tests
cargo test poly::    # for Poly1305 tests
cargo test aead::    # for AEAD tests
```

### Performance Testing
```bash
# Run benchmarks
cargo bench

# Generate HTML benchmark reports
cargo bench -- --save-baseline baseline_name
```

### Development Tools
```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

## Architecture

### Core Module Structure

The library consists of three main cryptographic components that work together:

1. **ChaCha20 Stream Cipher (`src/chacha.rs`)**
   - Implements the ChaCha20 stream cipher as per RFC 7539
   - `State` struct manages the 16-word internal state
   - `ChaCha20` struct provides the main encryption/decryption interface
   - Uses quarter-round operations as the core mixing function

2. **Poly1305 MAC (`src/poly.rs`)** 
   - Private module implementing Poly1305 message authentication
   - Uses `num::BigUint` for large integer arithmetic
   - `generate_key` function derives Poly1305 keys from ChaCha20
   - `mac` function computes authentication tags

3. **AEAD Integration (`src/aead.rs`)**
   - `ToyAEAD` struct combines ChaCha20 and Poly1305
   - `seal` method performs authenticated encryption
   - Handles padding and MAC computation according to RFC 8439

### Data Flow

```
Encryption:
plaintext → ChaCha20 encrypt → ciphertext
                                    ↓
                              Poly1305 MAC → authentication tag
```

### Key Implementation Details

- Uses safe Rust standard library methods for byte array conversions (`to_le_bytes`, `from_le_bytes`)
- Comprehensive RFC test vectors validate correctness
- All cryptographic constants follow RFC specifications exactly
- Endianness is handled using standard library little-endian conversion methods

## Testing Strategy

The codebase includes extensive testing against official test vectors from:
- RFC 7539 (ChaCha20 and Poly1305)
- RFC 8439 (ChaCha20-Poly1305 AEAD)

Test files are embedded directly in each module using `#[cfg(test)]` blocks.

## Security Notice

This is an educational implementation. It lacks:
- Constant-time operations (vulnerable to timing attacks)
- Secure memory handling
- Production-ready error handling

Never use this implementation for real cryptographic needs.