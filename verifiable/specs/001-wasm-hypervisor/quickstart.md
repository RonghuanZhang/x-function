# WASM Hypervisor Quick Start Guide

**Date**: 2025-11-11  
**Version**: 1.0.0  
**Feature**: WASM Hypervisor with Secure Execution

## Overview

The WASM Hypervisor provides secure, isolated execution of WebAssembly programs with cryptographic protection. Each request generates unique key pairs for encryption/decryption and creates hash commitments for audit verification.

## Prerequisites

- Rust 1.75+ with WASM target support
- OpenSSL development libraries
- System with at least 4GB RAM for concurrent execution
- Network access for API endpoints

## Installation

```bash
# Clone and setup
git clone <repository>
cd wasm-hypervisor

# Install Rust targets
rustup target add wasm32-unknown-unknown

# Install dependencies
cargo build --release

# Run tests
cargo test

# Start development server
cargo run --bin hypervisor
```

## Basic Usage

### 1. Generate Key Pair for Request

```rust
use p256::{PublicKey, SecretKey};
use rand::rngs::OsRng;
use elliptic_curves::SecretKey as EC;

fn generate_keypair() -> (PublicKey, SecretKey) {
    let mut rng = OsRng;
    let secret_key = SecretKey::random(&mut rng);
    let public_key = PublicKey::from_secret_key(&secret_key);
    (public_key, secret_key)
}
```

### 2. Prepare WASM Program

```rust
// Example WASM module (binary format)
let wasm_binary = vec![
    0x00, 0x61, 0x73, 0x6d, // WASM magic number
    0x01, 0x00, 0x00, 0x00, // WASM version
    // ... compiled WASM binary data
];

// Encrypt with public key
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::Aead;

let key = Key::from_slice(b"your-32-byte-key-here");
let cipher = Aes256Gcm::new(key);
let nonce = Nonce::from_slice(b"unique-nonce-12");
let encrypted_wasm = cipher.encrypt(nonce, wasm_binary.as_ref()).unwrap();
```

### 3. Execute WASM Program

```bash
curl -X POST https://api.hypervisor.example.com/v1/execute \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "encrypted_wasm": "base64_encoded_wasm_binary",
    "function_name": "main",
    "arguments": [{"value": 42}],
    "public_key": "base64_encoded_public_key",
    "request_metadata": {
      "client_id": "my-app",
      "version": "1.0.0"
    },
    "timeout_seconds": 30
  }'
```

### 4. Decrypt Result

```rust
// Decrypt response
let encrypted_result: Vec<u8> = base64::decode(response.encrypted_result).unwrap();
let nonce = Nonce::from_slice(b"unique-nonce-12");
let result = cipher.decrypt(nonce, encrypted_result.as_ref()).unwrap();
println!("Execution result: {}", String::from_utf8(result).unwrap());
```

## API Reference

### POST /execute
Execute a WASM program with cryptographic protection.

**Request Body**:
- `encrypted_wasm` (string, base64): Encrypted WASM binary
- `function_name` (string): WASM function to execute
- `arguments` (array): Function arguments
- `public_key` (string, base64): P-256 public key for encryption
- `request_metadata` (object): Client information
- `timeout_seconds` (integer): Execution timeout (1-300s)

**Response**:
- `request_id` (string, uuid): Unique request identifier
- `encrypted_result` (string, base64): Encrypted execution result
- `result_commitment` (string, hex): SHA-256 hash of result
- `execution_summary` (object): Execution metrics

### GET /verify/{request_id}
Verify cryptographic commitment for audit purposes.

**Parameters**:
- `request_id` (uuid): Request to verify
- `commitment_type` (string): Type of commitment
- `hash_value` (string, hex): Expected hash value

### GET /health
Get system health status and capacity metrics.

### GET /metrics
Get detailed performance and usage metrics (authenticated).

## Security Model

### Cryptographic Protection
- **Key Generation**: Unique P-256 key pairs per request
- **Encryption**: AES-GCM for WASM programs and results
- **Hash Commitments**: SHA-256 commitments for audit trail
- **Session Isolation**: Complete isolation between executions

### Resource Limits
- **Memory**: 64MB per execution
- **Time**: 10 seconds maximum execution time
- **Gas**: Computational budget enforcement
- **Instances**: Maximum 100 concurrent executions

### WASM Security
- No filesystem access
- No network access
- No host function imports
- Sandboxed execution environment
- Deterministic resource tracking

## Performance Targets

- **Response Time**: < 30 seconds for 1MB programs
- **Concurrent Capacity**: 100 simultaneous executions
- **Memory Usage**: < 1GB total system memory
- **Throughput**: 10+ executions per second
- **Availability**: 99.9% uptime target

## Error Handling

### Common Error Codes
- `INVALID_WASM`: WASM module validation failed
- `EXECUTION_TIMEOUT`: Program exceeded time limit
- `MEMORY_EXCEEDED`: Resource limit exceeded
- `ENCRYPTION_ERROR`: Cryptographic operation failed
- `RATE_LIMITED`: Too many requests

### Error Response Format
```json
{
  "error_code": "EXECUTION_TIMEOUT",
  "message": "Program execution exceeded timeout limit",
  "timestamp": "2025-11-11T10:00:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Monitoring and Observability

### Metrics Tracked
- Execution count and success rate
- Average execution time
- Memory and CPU usage
- Error rates by type
- Resource utilization

### Logging
- All requests logged with request_id
- Cryptographic operations logged
- Security events flagged
- Performance metrics collected

### Health Checks
- `/health` endpoint for basic status
- `/metrics` for detailed metrics
- Automated health monitoring
- Capacity alerting

## Development

### Testing
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# Cryptographic tests
cargo test crypto

# Performance benchmarks
cargo bench
```

### Debugging
```bash
# Enable debug logging
RUST_LOG=hypervisor=debug cargo run

# Memory usage tracking
RUST_BACKTRACE=1 cargo run

# Profile performance
cargo install flamegraph
cargo flamegraph --bin hypervisor
```

### Configuration
```yaml
# config.yaml
server:
  host: "0.0.0.0"
  port: 3000
  workers: 4

wasm:
  max_memory_mb: 64
  max_execution_time_seconds: 10
  max_concurrent_executions: 100

crypto:
  key_rotation_hours: 1
  commitment_retention_days: 7

logging:
  level: "info"
  file: "/var/log/hypervisor.log"
```

## Troubleshooting

### Common Issues

**Execution Timeout**
- Check WASM program for infinite loops
- Reduce complexity of program
- Increase timeout if needed

**Memory Exceeded**
- Optimize WASM program memory usage
- Check for memory leaks in program
- Reduce memory limits if too strict

**Encryption Errors**
- Verify public key format (P-256)
- Check base64 encoding
- Ensure key pair consistency

**High Latency**
- Check concurrent execution load
- Monitor system resources
- Review WASM compilation time

### Getting Help
- Check `/health` endpoint for system status
- Review logs for detailed error messages
- Use `/metrics` to identify performance issues
- Contact support with request_id for investigation

## Best Practices

1. **Key Management**: Generate new key pairs for each request
2. **WASM Validation**: Test programs in development environment first
3. **Resource Limits**: Set appropriate timeouts and memory limits
4. **Error Handling**: Implement proper retry logic with exponential backoff
5. **Monitoring**: Set up alerts for error rates and resource usage
6. **Security**: Rotate API keys regularly and monitor access patterns
7. **Performance**: Profile WASM programs for optimization opportunities
8. **Compliance**: Maintain audit logs for regulatory requirements
