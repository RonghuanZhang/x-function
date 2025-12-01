# WASM Hypervisor

A secure, isolated execution environment designed for Trusted Execution Environment (TEE) guest VMs.

> **⚠️ ALPHA SOFTWARE WARNING**
> This codebase is of **alpha quality** and for **demonstration purposes only**. It is **NOT** production-ready. It is intended solely for testing secure agent execution within TEE environments.

## Key Components

1.  **TEE Hypervisor**: Leverages hardware attestation and secure enclaves (e.g., TDX/SEV/SGX) to provide a trusted runtime.
2.  **Secure Private Agent Execution**: Ensures that agent code, inputs, and outputs remain encrypted and invisible to the host.
3.  **Agent-2-Agent Protocol**: Built-in support for secure, verifiable communication and payments between autonomous agents (via X402).

## Multi-Architecture Support

While the current implementation focuses on **WebAssembly (WASM)** for its isolation and portability properties, the hypervisor is designed to be architecture-agnostic.

*   **Current**: WASM (via Wasmtime)
*   **Future**: RISC-V VM

## Architecture

The hypervisor is designed to run inside a TEE Guest VM, acting as a secure gateway for executing sensitive workloads.

```
┌─────────────────┐    ┌──────────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client        │────│  TEE Guest VM        │────│  Hypervisor     │────│  WASM / RISC-V  │
│                 │    │  (Trusted Zone)      │    │                 │    │                 │
│ • Key Generation│    │  🔒 Fully Encrypted  │    │ • Encryption    │    │ • Isolated Exec │
│ • Encrypted Args│    │  🏛️  TEE Protected   │    │ • Decryption    │    │ • No Data Leak  │
│ • Decrypted Res │    │                      │    │ • Commitments   │    │ • Resource Ctrl │
└─────────────────┘    └──────────────────────┘    └─────────────────┘    └─────────────────┘
```

## Quick Start

### Prerequisites
*   [devenv](https://devenv.sh/)
*   [Nix](https://nixos.org/) (2.4+)

### Development Flow

The project uses `devenv` to automatically manage the Rust toolchain, Python, and system dependencies.

```bash
# 1. Enter the environment
devenv shell

# 2. Build and Test
cargo build --release
cargo test

# 3. Run the Server
# Create a local config
cat > hypervisor.toml << EOF
executor_path = "./data/executor"
app_path = "./data/apps"
listening = "0.0.0.0:3000"
EOF

cargo run --bin hypervisor -- --config hypervisor.toml
```

## API Overview

The server listens on port `3000` by default.

### 1. Establish Secure Session
**Endpoint**: `POST /verifiable/encrypt/create_keypair`
Exchanges keys and returns an attestation quote verifying the TEE environment.

### 2. Execute Agent (WASM)
**Endpoint**: `POST /x402_execute/verifiable/wasm`
Executes an encrypted WASM binary.
*   **Headers**: Requires `X-Payment` headers (X402).
*   **Input**: Encrypted WASM binary, Encrypted arguments.
*   **Output**: Encrypted result, Result commitment, Attestation Quote.

### 3. Execute Policy (Python)
**Endpoint**: `POST /x402_policy/unsafe/python`
Executes a Python policy script (NOTE: test purpose, all code must run inside vm)

### 4. Discover Agent
**Endpoint**: `POST /search`
Search for available agents by description (hardcode one for demo).
*   **Input**: Description string.
*   **Output**: Agent found message.

### 5. Deploy Agent
**Endpoint**: `POST /agent/deploy`
Deploys a specific agent to the runtime (hardcode one for demo, must run inside vm).
*   **Input**: Agent name (e.g., "arxiv").
*   **Output**: Deployment status.

## Project Structure

*   `binaries/hypervisor`: Main server implementation (Axum).
*   `binaries/hypervisor/src/api`: API route definitions.
*   `crates/attest`: TEE attestation logic and hardware integration.
*   `tests/integration`: Integration tests and example WASM/Python payloads.

## License
MIT
