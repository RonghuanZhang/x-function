# Data Model: WASM Hypervisor with Secure Execution

**Date**: 2025-11-11  
**Feature**: WASM Hypervisor with Secure Execution

## Core Entities

### ExecutionRequest
**Purpose**: Represents an incoming request to execute a WASM program
**Attributes**:
- `id`: Unique identifier for the execution request
- `encrypted_wasm`: Encrypted WASM program binary data
- `function_name`: Name of the WASM function to execute
- `arguments`: JSON array of function arguments
- `public_key`: Public key for result encryption
- `request_hash`: SHA-256 hash commitment of the request
- `timestamp`: Request creation time
- `client_metadata`: Optional client information for logging

**Validation Rules**:
- `encrypted_wasm` MUST be valid base64 and decryptable with corresponding private key
- `function_name` MUST be a valid WASM function identifier
- `arguments` MUST be valid JSON array
- `public_key` MUST be valid ECC public key format
- `request_hash` MUST be exactly 64 hex characters (SHA-256)

### CryptographicKeyPair
**Purpose**: Generated per-request for secure communication
**Attributes**:
- `request_id`: Associated execution request ID
- `public_key`: P-256 public key in uncompressed format (64 bytes)
- `private_key`: P-256 private key (32 bytes) - stored securely
- `algorithm`: Cryptographic algorithm used (P-256)
- `key_derivation_salt`: Random salt for key derivation
- `created_at`: Key generation timestamp
- `expires_at`: Key expiration (typically 1 hour after creation)

**Validation Rules**:
- Keys MUST be cryptographically valid P-256 key pairs
- Private key MUST NOT be exposed after request processing
- Keys MUST be deleted after request completion for security

### ExecutionResult
**Purpose**: Contains the results of WASM program execution
**Attributes**:
- `request_id`: Associated execution request ID
- `encrypted_output`: AES-GCM encrypted result data
- `result_hash`: SHA-256 hash commitment of unencrypted result
- `execution_status`: Success, timeout, error, or resource_exhausted
- `execution_time_ms`: Actual execution time in milliseconds
- `memory_used_bytes`: Peak memory consumption during execution
- `gas_consumed`: Computational units consumed
- `error_message`: Error details if execution failed
- `resource_limits_hit`: Array of exceeded limits (memory, time, gas)

**Validation Rules**:
- `encrypted_output` MUST be decryptable with the request's public key
- `execution_time_ms` MUST be non-negative and within configured limits
- `result_hash` MUST match SHA-256 of unencrypted output

### CommitmentRecord
**Purpose**: Immutable audit trail of all requests and results
**Attributes**:
- `record_id`: Unique identifier for the commitment record
- `request_id`: Associated execution request ID
- `request_commitment`: SHA-256 hash of original request
- `result_commitment`: SHA-256 hash of execution result
- `commitment_type`: Request, result, or combined commitment
- `timestamp`: When commitment was created
- `merkle_proof`: Merkle tree proof for audit verification
- `validator_signature`: Digital signature of commitment validity

**Validation Rules**:
- Commitments MUST be immutable and tamper-evident
- Each request/result MUST have exactly one commitment record
- Merkle proofs MUST allow verification without exposing sensitive data

### ExecutionSession
**Purpose**: Manages isolated execution environment per request
**Attributes**:
- `session_id`: Unique session identifier
- `request_id`: Associated execution request ID
- `wasm_engine`: Wasmtime engine instance for this session
- `memory_limit_bytes`: Maximum memory allocation for this session
- `time_limit_ms`: Maximum execution time in milliseconds
- `gas_limit`: Computational budget for execution
- `created_at`: Session creation time
- `resource_tracking`: Memory, CPU, and I/O usage metrics
- `termination_reason`: How the session ended (completed, timeout, error)

**Validation Rules**:
- Session MUST be completely isolated from other sessions
- Resource limits MUST be enforced at the engine level
- Session MUST be cleaned up after completion or termination

## Entity Relationships

```
ExecutionRequest (1) ←→ (1) CryptographicKeyPair
ExecutionRequest (1) ←→ (1) ExecutionResult
ExecutionRequest (1) ←→ (1) ExecutionSession
ExecutionRequest (1) ←→ (1..n) CommitmentRecord
ExecutionResult (1) ←→ (1) CommitmentRecord
```

## State Transitions

### ExecutionRequest States
```
CREATED → KEY_GENERATED → DECRYPTING → EXECUTING → COMPLETED
    ↓           ↓              ↓            ↓           ↓
         INVALID_WASM     EXEC_ERROR    TIMEOUT     ERROR
```

### ExecutionSession States
```
INITIALIZING → READY → RUNNING → TERMINATED → CLEANED_UP
                                   ↓
                              INTERRUPTED
```

## Data Validation Rules

### Cryptographic Validation
- All encrypted data MUST use authenticated encryption (AES-GCM)
- Hash commitments MUST be SHA-256 with no truncation
- Key pairs MUST pass cryptographic validation
- All random values MUST use cryptographically secure RNG

### Security Validation
- WASM modules MUST be validated against security policy
- No host function imports allowed in WASM modules
- Resource limits MUST be enforced at engine initialization
- Session isolation MUST be verified before execution

### Performance Validation
- Execution time MUST be measured with sub-millisecond precision
- Memory usage MUST be tracked in real-time during execution
- Gas consumption MUST be calculated per WASM instruction
- Response time MUST be under 30 seconds for 1MB programs

## Audit Requirements

All entities MUST support:
- Immutable logging with timestamps
- Cryptographic integrity verification
- Tamper-evident commit chains
- Non-repudiation through digital signatures
- Compliance with security audit requirements

## Data Retention

- **ExecutionSession**: Deleted immediately after completion
- **CryptographicKeyPair**: Deleted 1 hour after creation
- **ExecutionRequest/Result**: Deleted after 7 days (configurable)
- **CommitmentRecord**: Retained indefinitely for audit trail
- **Audit Logs**: Rotated monthly, retained for 1 year minimum
