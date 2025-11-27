# WASM Hypervisor

A secure, isolated WebAssembly execution environment designed for Trusted Execution Environment (TEE) guest VMs with cryptographic protection and attestation capabilities.

## ⚠️ Important Disclaimers

- **Demo/Alpha Quality**: This code is for demonstration and testing purposes only
- **Not Production Ready**: Do not deploy in production environments
- **TEE-Only Deployment**: Intended for deployment inside Trusted Execution Environment guest VMs
- **Privacy-First Design**: All executed programs and their outputs are kept completely secret

## Overview

The WASM Hypervisor provides secure, isolated execution of WebAssembly programs with cryptographic protection, designed specifically for TEE guest VM deployments where privacy is paramount. Each request generates unique key pairs for encryption/decryption and creates hash commitments for audit verification. The system supports concurrent execution, hardware attestation, and integrates with X402 payment protocols.

### Privacy & Security Focus

🔒 **Absolute Privacy**: All executed programs, inputs, and outputs are encrypted and kept secret
🛡️ **TEE Integration**: Leverages hardware attestation and secure enclaves
🔐 **Cryptographic Protection**: Multiple layers of encryption and commitment verification
🏛️ **Zero Trust**: No data leaves the execution environment in plaintext

## Features

- 🔒 **Secure Execution**: Isolated WASM runtime with resource limits
- 🔐 **Cryptographic Protection**: Unique P-256 key pairs per request
- 🏛️ **TEE Attestation**: Hardware attestation for TEE verification
- 💰 **X402 Integration**: Payment support for execution requests
- 🐍 **Multi-Language Support**: WASM and Python execution
- 📊 **Audit Trail**: Cryptographic hash commitments for privacy-preserving verification
- 🚀 **High Performance**: Concurrent execution support

## Architecture

**TEE Guest VM Deployment Context**

```
┌─────────────────┐    ┌──────────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client        │────│  TEE Guest VM        │────│  Hypervisor     │────│  WASM Runtime   │
│                 │    │  (Trusted Zone)      │    │                 │    │                 │
│ • Key Generation│    │  🔒 Fully Encrypted  │    │ • Encryption    │    │ • Isolated Exec │
│ • Encrypted WASM│    │  🏛️  TEE Protected   │    │ • Decryption    │    │ • No Data Leak  │
│ • Encrypted Args│    │                      │    │ • Commitments   │    │ • Resource Ctrl │
│ • Decrypted Res │    │                      │    │ • TEE Attest    │    │ • WASI Support  │
└─────────────────┘    └──────────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                              │                       │
         │              ┌──────────────────┐                    │              ┌────────────────────┐
         └──────────────│  X402 Payments   │────────────────────└──────────────│  TEE Hardware      │
                        │  🔒 Encrypted    │                                   │  🏛️  Attestation   │
                        │  • USDC Support  │                                   │  🔐 Secure Enclave │
                        │  • Base Sepolia  │                                   └────────────────────┘
                        └──────────────────┘
```

### TEE Deployment Context

This hypervisor is designed specifically for deployment **inside Trusted Execution Environment (TEE) guest VMs**:

- **Fully Encrypted Execution**: All programs, inputs, and outputs remain encrypted throughout execution
- **Hardware-Level Privacy**: Leverages TEE hardware for memory protection and isolation  
- **Attestation Integration**: Provides cryptographic proof of TEE execution context
- **Secure Communication**: All data exchange uses end-to-end encryption with TEE-derived keys

### Privacy-First Design Principles

1. **Encrypted Input/Output**: All WASM programs and results are encrypted before entering the TEE
2. **Secure Key Management**: Keys are derived within the TEE using hardware attestation
3. **Memory Protection**: TEE hardware ensures no plaintext data can escape the execution environment
4. **Audit Without Disclosure**: Hash commitments enable verification without exposing sensitive data
5. **Isolated Execution**: Each program runs in complete isolation within the TEE boundary

## Quick Start

### Prerequisites

- [devenv](https://devenv.sh/) for development environment
- Nix 2.4+ or DevContainer support
- VS Code (optional, for devcontainer development)

### Building

#### Using devenv (Recommended)

```bash
# Enter the devenv development environment
devenv up

# In another terminal, enter the shell
devenv shell

# Build the project
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Run linting
cargo clippy
```

The devenv environment includes:
- Rust 1.91.1 with WASM target (`wasm32-wasip2`)
- Python 3.x with pip
- OpenSSL development libraries
- Git and essential development tools
- All Rust components (clippy, rustfmt, rust-analyzer)

#### Using DevContainer

1. Open the project in VS Code
2. When prompted, "Reopen in Container" or use Command Palette: "Dev Containers: Reopen in Container"
3. The container will automatically set up dependencies via devenv
4. Build and test inside the container:

```bash
cargo build --release
cargo test
```

#### Manual Setup (Not Recommended)

If you cannot use devenv, manual setup requires:

```bash
# Install Rust 1.91.1 and add WASM target
rustup toolchain install 1.91.1
rustup target add wasm32-wasip2 --toolchain 1.91.1

# Install system dependencies
# Ubuntu/Debian:
sudo apt-get install build-essential pkg-config libssl-dev python3 python3-pip

# macOS:
brew install openssl pkg-config python3

# Build
cargo build --release
```

### Running the Server

```bash
# Create a config file
cat > hypervisor.toml << EOF
executor_path = "./data/executor"
app_path = "./data/apps"
listening = "0.0.0.0:3000"
EOF

# Start the hypervisor
cargo run --bin hypervisor -- --config hypervisor.toml
```

The server will start on `http://localhost:3000`.

## API Reference

### Core Endpoints

#### Health Check
```http
GET /ping
```

**Response:**
```text
pong
```

#### Key Pair Creation
```http
POST /encrypt/create_keypair
Content-Type: application/json

{
  "pubkey": "your_hex_encoded_public_key"
}
```

**Response:**
```json
{
  "session_pubkey": "hex_encoded_session_public_key",
  "session_id": "uuid-v7"
}
```

#### Verifiable Key Pair Creation
```http
POST /verifiable/encrypt/create_keypair
Content-Type: application/json

{
  "pubkey": "your_hex_encoded_public_key"
}
```

**Response:**
```json
{
  "session_pubkey": "hex_encoded_session_public_key",
  "session_id": "uuid-v7",
  "quote": "hex_encoded_attestation_quote"
}
```

### WASM Execution

#### Test Execution
```http
POST /test/execute/wasm
Content-Type: application/json

{
  "encrypted_wasm": "hex_encoded_wasm_binary",
  "encrypted_arguments": ["hex_encoded_arg1", "hex_encoded_arg2"],
  "public_key": "your_hex_encoded_public_key"
}
```

**Response:**
```json
{
  "session_id": "uuid-v7",
  "encrypted_result": "hex_encoded_result",
  "result_nonce": "hex_encoded_nonce",
  "result_commitment": "hex_encoded_commitment"
}
```

#### X402 WASM Execution
```http
POST /x402_execute/test/wasm
Content-Type: application/json
X-Payment: <x402_payment_header>

{
  "encrypted_wasm": "hex_encoded_wasm_binary",
  "encrypted_arguments": ["hex_encoded_arg1"],
  "public_key": "your_hex_encoded_public_key"
}
```

#### Verifiable WASM Execution
```http
POST /x402_execute/verifiable/wasm
Content-Type: application/json
X-Payment: <x402_payment_header>

{
  "encrypted_wasm": "hex_encoded_wasm_binary",
  "encrypted_arguments": ["hex_encoded_arg1"],
  "public_key": "your_hex_encoded_public_key"
}
```

**Response:**
```json
{
  "session_id": "uuid-v4",
  "encrypted_result": "hex_encoded_result",
  "result_nonce": "hex_encoded_nonce",
  "result_commitment": "hex_encoded_commitment",
  "result_quote": "hex_encoded_attestation_quote"
}
```

### Python Policy Execution

#### Unsafe Python Execution
```http
POST /test/policy/unsafe/python
Content-Type: application/json

{
  "encrypted_python": "hex_encoded_python_script",
  "encrypted_arguments": ["hex_encoded_arg1"],
  "public_key": "your_hex_encoded_public_key"
}
```

#### Attested Python Execution
```http
POST /test/policy/unsafe/python/attest
Content-Type: application/json

{
  "encrypted_python": "hex_encoded_python_script",
  "encrypted_arguments": ["hex_encoded_arg1"],
  "public_key": "your_hex_encoded_public_key"
}
```

#### X402 Python Execution
```http
POST /x402_policy/unsafe/python
Content-Type: application/json
X-Payment: <x402_payment_header>

{
  "encrypted_python": "hex_encoded_python_script",
  "encrypted_arguments": ["hex_encoded_arg1"],
  "public_key": "your_hex_encoded_public_key"
}
```

## Usage Example

⚠️ **TEE Guest VM Context Required**: This example assumes deployment inside a TEE guest VM with proper hardware attestation.

Here's a complete example of executing a WASM program with privacy protection:

```rust
// DEMO CODE - TEE Guest VM Context Only
// Not for production use - Alpha quality

use k256::ecdsa::SigningKey;
use aes_gcm_siv::{aead::Aead, KeyInit, Aes256GcmSiv};
use uuid::Uuid;
use rand::rngs::OsRng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ⚠️  TEE Context: All operations happen within TEE boundary
    // 🔒 Privacy: WASM binary and results remain encrypted throughout
    
    // 1. Generate client keypair (within TEE)
    let sk = SigningKey::random(&mut OsRng);
    let user_pk = sk.verifying_key();

    // 2. Create TEE-verified session keypair
    let client = reqwest::Client::new();
    let resp = client
        .post("http://localhost:3000/encrypt/create_keypair")
        .json(&serde_json::json!({
            "pubkey": format!("{:x}", user_pk.to_encoded_point(true))
        }))
        .send()
        .await?;
    
    let session_data = resp.json::<SessionKeyPairResponse>().await?;
    
    // 3. Prepare WASM binary (remains encrypted)
    let wasm_binary = include_bytes!("hello.wasm");
    
    // 4. Create encryption key within TEE and encrypt WASM
    let session_pk = parse_public_key(&session_data.session_pubkey)?;
    let cipher = create_cipher(&sk, &session_pk, session_data.session_id)?;
    let nonce = derive_nonce(session_data.session_id);
    let encrypted_wasm = cipher.encrypt(&nonce, wasm_binary)?;
    
    // 5. Execute WASM (inside TEE - no plaintext exposure)
    let response = client
        .post("http://localhost:3000/test/execute/wasm")
        .json(&ExecutionRequest {
            encrypted_wasm: hex::encode(&encrypted_wasm),
            encrypted_arguments: vec![], // All encrypted
            public_key: format!("{:x}", user_pk.to_encoded_point(true))
        })
        .send()
        .await?;
    
    let result = response.json::<ExecutionResponse>().await?;
    
    // 6. Decrypt result (within TEE boundary)
    let result_nonce = parse_nonce(&result.result_nonce)?;
    let decrypted_result = cipher.decrypt(&result_nonce, hex::decode(&result.encrypted_result)?)?;
    
    println!("WASM execution result (within TEE): {}", String::from_utf8(decrypted_result)?);
    
    // 🔒 Privacy guarantee: WASM code and results never exposed outside TEE
    // 🏛️ Attestation: Hardware provides proof of secure execution
    
    Ok(())
}
```

**Privacy & TEE Notes:**
- All WASM binaries remain encrypted throughout execution
- Results are encrypted before leaving the TEE
- TEE hardware ensures no data leakage
- Attestation provides execution integrity proof

## Testing

### Unit Tests
```bash
# Run all unit tests
cargo test

# Run tests for specific module
cargo test api::execute::wasm

# Run tests with logging
RUST_LOG=hypervisor=debug cargo test
```

### Integration Tests
```bash
# Run integration tests
cargo test --test integration

# Run with X402 integration
cargo test --test integration x402_execute_wasm
```

### Test Examples

The project includes example WASM and Python programs:

- `tests/integration/hello.wasm` - Simple WASM program that prints "Hello"
- `binaries/hypervisor/src/api/execute/hello.wasm` - Test WASM binary
- `binaries/hypervisor/src/api/policy/python/hello.py` - Test Python script

## Configuration

### Server Configuration (TOML)

Create a `hypervisor.toml` file:

```toml
executor_path = "./data/executor"
app_path = "./data/apps"
listening = "0.0.0.0:3000"
```

### Environment Variables

```bash
# Logging level
RUST_LOG=hypervisor=debug

# Backtrace on panic
RUST_BACKTRACE=1
```

## Development

### devenv Commands

```bash
# Start the development environment
devenv up

# Enter development shell
devenv shell

# Run development task (if defined)
devenv task myproj:setup

# Run tests in devenv
devenv test

# Check the environment
devenv info
```

### Project Structure

```
├── binaries/
│   ├── apps/examples/hello/    # Example applications
│   └── hypervisor/             # Main hypervisor binary
│       └── src/
│           ├── api/            # API endpoints
│           ├── utils/          # Cryptographic utilities
│           └── types/          # Core types
├── crates/
│   └── attest/                # Attestation provider
├── specs/
│   └── 001-wasm-hypervisor/   # Feature specification
└── tests/
    └── integration/           # Integration tests
```

### Development Workflow

1. **Start development environment:**
   ```bash
   devenv up
   ```

2. **Enter shell and build:**
   ```bash
   devenv shell
   cargo build --release
   ```

3. **Run tests continuously:**
   ```bash
   cargo test --watch  # if installed
   # or
   cargo test
   ```

4. **Code formatting and linting:**
   ```bash
   cargo fmt
   cargo clippy
   ```

5. **Add new features:**
   - Create API module in `binaries/hypervisor/src/api/`
   - Implement `api_register` function
   - Register in `binaries/hypervisor/src/server.rs`
   - Add tests
   - Update documentation

### WASM Program Requirements

WASM programs must:
- Be compiled for `wasm32-wasip2` target (configured in devenv)
- Implement WASI interface (for stdin/stdout)
- Be self-contained (no external imports)
- Handle resource limits appropriately

Example WASM component:
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn main() {
    println!("Hello from WASM!");
}
```

### devenv Configuration

The project uses `devenv.yaml` for environment setup:

```yaml
inputs:
  nixpkgs:
    url: github:cachix/devenv-nixpkgs/rolling
  rust-overlay:
    url: github:oxalica/rust-overlay
```

Environment configuration in `devenv.nix`:
- **Languages**: Rust 1.91.1 with WASM target (`wasm32-wasip2`), Python enabled
- **Packages**: Git, OpenSSL development libraries
- **DevContainer**: Enabled for VS Code integration
- **Tools**: All Rust components pre-installed (clippy, rustfmt, rust-analyzer)

### Adding New APIs

1. Create API module in `binaries/hypervisor/src/api/`
2. Implement `api_register` function
3. Register in `binaries/hypervisor/src/server.rs`
4. Add tests
5. Update documentation

## Security & Privacy Considerations

⚠️ **CRITICAL**: This is demo code for TEE guest VM environments. Do not use in production without extensive security review.

### TEE-Specific Security Model

**Trusted Execution Environment Context:**
- **Hardware Enclaves**: Relies on TEE hardware for ultimate security guarantees
- **Memory Encryption**: All execution happens in encrypted memory regions
- **Secure Boot**: TEE provides hardware-backed secure boot verification
- **Attestation**: Hardware attestation ensures execution integrity

### Cryptographic Security (TEE-Enhanced)
- **P-256 elliptic curve** for key pairs
- **AES-GCM-SIV** for authenticated encryption with TEE-derived keys
- **SHA-256** for privacy-preserving hash commitments
- **Hardware attestation** provides root of trust
- **TEE-secure key derivation** prevents key extraction

### Isolation & Privacy Layers

**Layer 1 - WASM Sandbox:**
- WASM execution with resource limits
- No filesystem or network access from WASM
- Memory isolation between executions

**Layer 2 - TEE Boundary:**
- Complete isolation from host system
- Hardware-enforced memory protection
- No data can exit TEE in plaintext

**Layer 3 - Cryptographic Envelope:**
- End-to-end encryption of all data
- TEE-derived session keys

### Privacy Guarantees

🔒 **Absolute Privacy**: No plaintext data ever leaves the TEE boundary
🏛️ **Hardware-Verified**: Attestation ensures genuine TEE execution
📊 **Auditable Without Disclosure**: Hash commitments enable verification without data exposure

### Development Security Notes

⚠️ **Alpha Code Limitations:**
- Security features are still in development
- API stability not guaranteed
- Attack surface not fully evaluated
- Requires extensive TEE testing before production use

### TEE Deployment Requirements

**Minimum TEE Features Required:**
- Memory encryption (Intel SGX, AMD SEV, or ARM TrustZone)
- Secure boot and attestation capability
- Hardware random number generation
- Protected key storage

**Recommended TEE Configuration:**
- Minimum 2GB dedicated memory
- Hardware-based secure enclaves
- Attestation reporting capability
- Secure time source

## Performance

### Targets
- < 30 seconds response time for 1MB WASM programs
- 100+ concurrent executions
- < 1GB memory usage
- 99.9% availability

### Optimization
- WASM compilation caching
- Connection pooling
- Memory pooling
- Asynchronous I/O

## Monitoring

### Health Checks
- `GET /ping` - Basic connectivity
- System resource monitoring
- Execution queue status

### Metrics
- Execution count and timing
- Success/failure rates
- Resource utilization
- Error categorization

## Troubleshooting

### Common Issues

**devenv environment issues**
```bash
# Check devenv status
devenv status

# Update devenv environment
devenv up

# Clean and rebuild environment
devenv gc

# Check devenv configuration
devenv info
```

**Server won't start**
```bash
# Check port availability
netstat -tlnp | grep 3000

# Check config file
cargo run --bin hypervisor -- --config hypervisor.toml --verbose
```

**WASM execution fails**
```bash
# Verify WASM binary format (in devenv shell)
wasm-objdump -h your_program.wasm

# Check WASI compatibility
wasm2wat your_program.wasm

# Ensure correct target (wasm32-wasip2)
rustup target list | grep wasip2
```

**Rust toolchain issues**
```bash
# Check current toolchain
rustc --version

# Verify WASM target is installed
rustup target list --installed

# Update rust-toolchain if needed
# (should be 1.91.1 with wasm32-wasip2 target)
```

**Cryptographic errors**
```bash
# Enable debug logging
RUST_LOG=hypervisor=debug

# Verify key formats
# P-256 public keys should be 65 bytes (uncompressed)
# Keys should be hex encoded
```

### devenv-Specific Troubleshooting

**Environment not loading**
```bash
# Ensure you're in the project directory
pwd  # Should show the project root

# Check .envrc file exists
ls -la .envrc

# Load environment manually
source .envrc
```

**Missing dependencies in devenv**
```bash
# Add packages to devenv.nix, then:
devenv up

# For Python packages, use in devenv shell:
pip install package-name
```

**DevContainer not working**
```bash
# Ensure Docker is running
# Check VS Code DevContainer extension is installed
# Try "Dev Containers: Rebuild Container" from command palette
```

### TEE-Specific Issues (TEE Guest VMs)

⚠️ **Only applicable when deployed inside TEE guest VMs**

**TEE Attestation Failures**
```bash
# Check TEE is properly initialized
# Verify hardware attestation capability
# Ensure TEE drivers are loaded
dmesg | grep -i tee
```

**Memory Encryption Issues**
```bash
# Verify memory encryption is enabled
# Check TEE memory allocation
# Monitor encrypted memory regions
```

**TEE-Specific Build Issues**
```bash
# Ensure TEE SDK is installed in devenv
# Verify TEE target architecture
# Check TEE-specific compilation flags
```

### Getting Help

1. **Development Environment:**
   - Check devenv status: `devenv status`
   - Check devenv configuration: `devenv info`
   - Review logs: `RUST_LOG=hypervisor=debug`

2. **Project Issues:**
   - Verify WASM binary compatibility
   - Test with simple examples first
   - Review API request format
   - Check integration test: `cargo test --test integration`

3. **TEE-Specific Issues (TEE Guest VMs):**
   - Verify TEE hardware support
   - Check attestation capability
   - Review TEE SDK documentation

4. **Resources:**
   - [devenv documentation](https://devenv.sh/)
   - [TEE/SGX documentation](https://software.intel.com/content/www/us/en/develop/topics/software-guard-extensions.html)
   - [Rust WASM documentation](https://rustwasm.github.io/)
   - Project specifications in `specs/001-wasm-hypervisor/`
   - Integration tests in `tests/integration/`

## ⚠️ Final Production Warning

**THIS CODE IS DEMO/ALPHA QUALITY AND NOT READY FOR PRODUCTION USE**

### Deployment Restrictions

- ❌ **Do NOT deploy in production environments**
- ❌ **Do NOT use for handling sensitive production data**
- ❌ **Do NOT expose to untrusted networks**
- ✅ **Intended ONLY for TEE guest VM testing and demonstration**
- ✅ **Use ONLY in controlled, isolated TEE environments**
- ✅ **Test extensively in TEE context before any deployment**

### TEE Production Readiness

**This code is NOT production-ready even in TEE environments without:**
- Comprehensive security audit
- TEE-specific penetration testing  
- Production-grade attestation verification
- Proper key management implementation
- Comprehensive error handling
- Performance optimization and testing
- Documentation and operational procedures

### Privacy Notice

**By using this software, you acknowledge:**
- This is experimental/demo software
- No warranty or guarantee of security
- Privacy features are still under development
- TEE integration requires proper setup and testing
- Use at your own risk in non-production environments

**For production use, please:**
- Conduct thorough security audits
- Perform TEE-specific testing
- Implement proper monitoring and alerting
- Establish secure operational procedures
- Consider commercial TEE solutions

## Quick Reference

### devenv Commands

```bash
# Essential devenv commands
devenv up           # Start development environment
devenv shell        # Enter development shell
devenv status       # Check environment status
devenv info         # Show environment information
devenv test         # Run environment tests
devenv gc           # Clean and rebuild environment

# Development workflow
cargo build --release    # Build project
cargo test               # Run tests
cargo fmt               # Format code
cargo clippy            # Run linter
cargo run --bin hypervisor -- --config hypervisor.toml  # Run server
```

## Quick Start (For Experienced Users)

If you're familiar with devenv and Rust:

```bash
# 1. Setup environment
devenv up && devenv shell

# 2. Create config and start server
echo 'executor_path = "./data/executor"
app_path = "./data/apps"
listening = "0.0.0.0:3000"' > hypervisor.toml

# 3. Build and run
cargo build --release && cargo run --bin hypervisor -- --config hypervisor.toml

# 4. Test the server (in another terminal)
curl http://localhost:3000/ping
```

**That's it!** The server should be running and responding to requests. See the [API Reference](#api-reference) section for available endpoints.

### Development Tools (Available in devenv)

The devenv environment includes these pre-configured tools:

- **Rust**: 1.91.1 with all components
- **WASM Target**: `wasm32-wasip2`
- **Python**: 3.x with pip
- **Git**: Version control
- **OpenSSL**: Development libraries

### Environment Variables

```bash
# Logging configuration
export RUST_LOG=hypervisor=debug    # Enable debug logging
export RUST_BACKTRACE=1             # Enable backtrace on panic

# Project-specific (add to your shell)
export HYPERVISOR_CONFIG=./hypervisor.toml
export HYPERVISOR_HOST=0.0.0.0:3000
```

## Contributing

⚠️ **IMPORTANT**: This is demo/alpha code for TEE environments. Please consider the security implications of your contributions.

### Development Setup

1. **Setup development environment:**
   ```bash
   devenv up
   devenv shell
   ```

2. **Fork the repository and create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make changes with comprehensive tests:**
   ```bash
   cargo test
   # Add TEE-specific tests if applicable
   ```

4. **Code quality checks:**
   ```bash
   cargo fmt
   cargo clippy
   # Security-focused linting
   ```

5. **Test in devenv environment:**
   ```bash
   devenv test
   ```

### Security Considerations for Contributors

**Privacy & Security Requirements:**
- 🔒 All new features must maintain encryption-first design
- 🏛️ Consider TEE integration implications
- 🛡️ Add tests for security boundaries
- 📊 Include privacy-preserving verification methods

**TEE-Specific Development:**
- Test in TEE environments when possible
- Consider hardware attestation integration
- Validate memory encryption compatibility
- Ensure secure key derivation patterns

### Pull Request Guidelines

**Required for all PRs:**
- Clear description of changes and motivation
- Security impact assessment
- Test coverage for new features
- Privacy protection verification
- Documentation updates
- TEE compatibility notes (if applicable)

**Additional Requirements for Security Changes:**
- Security review checklist completed
- Cryptographic implementation verified
- TEE integration tested (if applicable)
- Privacy guarantee validation

**Security Disclosure:**
- Report security issues privately
- Include detailed reproduction steps
- Provide TEE environment details
- Suggest mitigation strategies

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [devenv.sh](https://devenv.sh/) for reproducible development environments
- [Wasmtime](https://wasmtime.dev/) runtime for WASM execution
- [k256](https://docs.rs/k256/) crate for cryptographic operations
- [axum](https://docs.rs/axum/) for HTTP server framework
- [X402 protocol](https://github.com/x402-rs/x402-rs) for payment integration
- [Nix](https://nixos.org/) and [Nixpkgs](https://github.com/NixOS/nixpkgs) for package management
- [Rust](https://rust-lang.org/) ecosystem and tooling