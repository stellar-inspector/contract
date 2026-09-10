# Stellar Inspector Contract

A Soroban smart contract that implements an on-chain **Security Registry**
for the Stellar network. It allows addresses to be registered as
*suspicious* or *trusted*, queried for their security flags, and used by
the Stellar Inspector backend to flag potentially malicious accounts and
transactions.

The contract is written in Rust using the [Soroban SDK](https://docs.rs/soroban-sdk)
and targets Soroban protocol version 21.

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Tech Stack](#tech-stack)
3. [Architecture & Storage Model](#architecture--storage-model)
4. [Prerequisites](#prerequisites)
5. [Building](#building)
6. [Testing](#testing)
7. [Contract Functions](#contract-functions)
8. [Storage Layout](#storage-layout)
9. [Deployment](#deployment)
10. [Contract Upgrades](#contract-upgrades)
11. [Development Workflow](#development-workflow)
12. [Security Considerations](#security-considerations)
13. [Gas & Performance](#gas--performance)
14. [Integration with the Backend](#integration-with-the-backend)
15. [Troubleshooting](#troubleshooting)
16. [Contributing](#contributing)

---

## Project Overview

The `SecurityRegistry` contract provides an on-chain allowlist/denylist
mechanism for Stellar accounts. It stores two mappings:

1. **Suspicious addresses** -- Accounts flagged as potentially
   malicious (e.g., involved in phishing, scams, or fraudulent
   activity).
2. **Trusted addresses** -- Accounts verified as legitimate (e.g.,
   well-known exchanges, institutional wallets, or contract deployers).

### Motivation

While Stellar's built-in account model provides cryptographic security,
there is no native on-chain mechanism for expressing reputation or
trust. The SecurityRegistry contract fills this gap by allowing:

- A governance contract or multisig to mark addresses as suspicious
  or trusted.
- Other contracts to query these flags before processing payments or
  interactions.
- The Stellar Inspector backend to enrich transaction analysis with
  on-chain trust data.

### Use Cases

| Use Case | How the Contract Helps |
|---|---|
| Exchange withdrawal screening | Check if a withdrawal destination is on the suspicious list |
| DEX trading | Refuse swaps to/from suspicious addresses |
| Multisig governance | Require additional confirmation for transactions involving untrusted addresses |
| Transaction auditing | The backend queries the registry to annotate transactions |

---

## Tech Stack

| Layer | Technology | Version |
|---|---|---|
| Language | Rust | 2021 edition |
| Smart Contract Framework | Soroban SDK | 21.x |
| Contract Type | WASM (cdylib + rlib) | -- |
| Testing Framework | Soroban testutils | 21.x |
| Build Tool | Cargo + Stellar CLI | -- |
| Runtime | Soroban (Stellar EVM-compatible WASM) | -- |

### Cargo.toml

```toml
[package]
name = "stellar-inspector-contract"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
soroban-sdk = "21"

[dev-dependencies]
soroban-sdk = { version = "21", features = ["testutils"] }

[profile.release]
opt-level = "z"
overflow-checks = true
debug = 0
strip = "symbols"
debug-assertions = false
panic = "abort"
codegen-units = 1
lto = true
```

### Profile Settings Rationale

| Setting | Value | Purpose |
|---|---|---|
| `opt-level = "z"` | Optimize for binary size | Soroban contracts have a size limit |
| `overflow-checks = true` | Enable arithmetic overflow checks | Prevent silent overflows in financial logic |
| `strip = "symbols"` | Remove debug symbols | Reduce contract size |
| `panic = "abort"` | Abort on panic | Faster than unwinding, smaller binary |
| `codegen-units = 1` | Single codegen unit | Better optimization at the cost of compile time |
| `lto = true` | Link-time optimization | Aggressive cross-crate optimization |

---

## Architecture & Storage Model

### High-Level Architecture

```
┌──────────────────────────────────────────────────┐
│                    Guest VM (WASM)                 │
│                                                  │
│  ┌──────────────────────────┐                    │
│  │     SecurityRegistry        │                    │
│  │  (contract struct)          │                    │
│  └────────────┬─────────────┘                    │
│               │                                   │
│  ┌────────────┴─────────────┐                    │
│  │   Contract Functions       │                    │
│  │  - register_address()      │                    │
│  │  - is_suspicious()         │                    │
│  │  - register_trusted()      │                    │
│  │  - is_trusted()            │                    │
│  │  - get_flags()             │                    │
│  └────────────┬─────────────┘                    │
│               │                                   │
│  ┌────────────┴─────────────┐                    │
│  │    Storage (instance)      │                    │
│  │                          │                    │
│  │  Key: "suspicious"        │                    │
│  │    -> Map<Address, bool>  │                    │
│  │                          │                    │
│  │  Key: "trusted"           │                    │
│  │    -> Map<Address, bool>  │                    │
│  └──────────────────────────┘                    │
└──────────────────────────────────────────────────┘
```

### Contract Entry Points

The contract uses Soroban's modern `#[contract]` and
`#[contractimpl]` attribute macros, which auto-generate the WASM
entry-point dispatch and client bindings.

```rust
#[contract]
pub struct SecurityRegistry;

#[contractimpl]
impl SecurityRegistry {
    // ... functions
}
```

- `#[contract]` -- Marks the struct as a contract type. Soroban's
  code generation creates the host-facing wrapper.
- `#[contractimpl]` -- Marks the impl block as the contract's public
  interface. Each `pub fn` in this block becomes a callable contract
  method.

---

## Prerequisites

### Rust Toolchain

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Minimum Rust version: 1.75+ (for Soroban SDK 21 compatibility).

### Stellar CLI

Required for building, testing, and deploying the contract:

```bash
# Via cargo
cargo install stellar-cli

# Or via package manager
brew install stellar/stellar/stellar-cli    # macOS
```

Verify installation:

```bash
stellar --version
```

### WASM Target

```bash
rustup target add wasm32-unknown-unknown
```

---

## Building

### Local Build (WASM)

```bash
# From the contract directory
cargo build --target wasm32-unknown-unknown --release
```

The compiled WASM file is output to:
`target/wasm32-unknown-unknown/release/stellar_inspector_contract.wasm`

### Using the Stellar CLI

```bash
# Build the contract
stellar contract build --manifest-path contract/Cargo.toml
```

Output is at:
`target/wasm32-unknown-unknown/release/stellar_inspector_contract.wasm`

### Build Verification

Check the WASM file size (should be well under 1 MB for Soroban):

```bash
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

---

## Testing

### Test File

`tests/mod.rs` contains integration tests using Soroban's
`testutils` framework. Tests run in a simulated Soroban environment
without deploying to an actual network.

### Test Coverage

The `tests/mod.rs` file includes a `TestSecurityRegistry` contract that
verifies the `is_suspicious` function correctly reads entries written by
`register_address`.

### Running Tests

```bash
# All tests
cargo test

# Only unit tests (inline)
cargo test --lib

# Only integration tests
cargo test --test tests

# With test output
cargo test -- --nocapture
```

### Test Environment

Soroban tests use `soroban_sdk::testutils::TestEnv` (via the
`testutils` feature) to:

- Create an in-memory Soroban environment
- Deploy the contract
- Call contract functions synchronously
- Assert on return values and storage state

No network connection is required. Tests run entirely in-process.

### Writing New Tests

Add tests to `tests/mod.rs` or create a new file in `tests/`. Example
template:

```rust
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::TestEnv;
    use soroban_sdk::{Address, Env};

    #[test]
    fn test_register_and_query() {
        let env = TestEnv::register(Symbol::small_str("Contract"), SecurityRegistry);
        let addr = Address::generate(&env);

        env.register_address(addr.clone(), true);
        assert!(env.is_suspicious(addr.clone()));
        assert_eq!(env.get_flags(addr.clone()), (true, false));
    }
}
```

---

## Contract Functions

### `register_address(env, address, is_suspicious)`

Registers an address as suspicious or not.

**Signature:**

```rust
pub fn register_address(
    env: Env,
    address: Address,
    is_suspicious: bool,
)
```

**Parameters:**

| Name | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment |
| `address` | `Address` | The Stellar address to flag |
| `is_suspicious` | `bool` | `true` to mark as suspicious, `false` to clear |

**Behavior:**

1. Retrieves the existing suspicious map from instance storage
   (defaults to empty if not present).
2. Sets `map[address] = is_suspicious`.
3. Writes the updated map back to instance storage under the key
   `"suspicious"`.

**Gas considerations:** If the map is large, this operation's cost grows
with map size due to the read-modify-write pattern.

**Events:** None.

---

### `is_suspicious(env, address) -> bool`

Checks whether an address is flagged as suspicious.

**Signature:**

```rust
pub fn is_suspicious(env: Env, address: Address) -> bool
```

**Parameters:**

| Name | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment |
| `address` | `Address` | The address to query |

**Returns:** `true` if the address is in the suspicious map, `false`
otherwise (including when the map doesn't exist yet).

**Gas considerations:** Read-only operation; minimal gas cost.

---

### `register_trusted(env, address)`

Marks an address as trusted.

**Signature:**

```rust
pub fn register_trusted(env: Env, address: Address)
```

**Parameters:**

| Name | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment |
| `address` | `Address` | The address to trust |

**Behavior:**

1. Retrieves the existing trusted map (defaults to empty).
2. Sets `map[address] = true`.
3. Writes the updated map back to instance storage under the key
   `"trusted"`.

**Note:** There is no `unregister_trusted` function. To remove a trusted
flag, the map would need to be modified via a future function or the
contract would need to be upgraded.

---

### `is_trusted(env, address) -> bool`

Checks whether an address is in the trusted list.

**Signature:**

```rust
pub fn is_trusted(env: Env, address: Address) -> bool
```

**Parameters:**

| Name | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment |
| `address` | `Address` | The address to query |

**Returns:** `true` if the address is in the trusted map, `false`
otherwise.

---

### `get_flags(env, address) -> (bool, bool)`

Returns both the suspicious and trusted flags for an address in a
single call. This is gas-efficient compared to calling `is_suspicious`
and `is_trusted` separately.

**Signature:**

```rust
pub fn get_flags(env: Env, address: Address) -> (bool, bool)
```

**Parameters:**

| Name | Type | Description |
|---|---|---|
| `env` | `Env` | Soroban environment |
| `address` | `Address` | The address to query |

**Returns:** A tuple `(is_suspicious, is_trusted)`.

**Example return:**

```rust
// For an address that is both suspicious and trusted
// (edge case -- typically should not happen)
// Returns (true, true)

// For a normal address
// Returns (false, false)

// For a trusted address
// Returns (false, true)

// For a suspicious address
// Returns (true, false)
```

---

## Storage Layout

The contract uses **instance storage** (not persistent storage or
temporary storage). This means:

- Data is stored alongside the contract instance itself.
- Storage costs are shared across all instances of the same contract
  hash (though in practice, there is typically one instance).
- Each `register_address` or `register_trusted` call reads the entire
  map, modifies it, and writes it back.

### Storage Keys

| Key | Type | Value Type |
|---|---|---|
| `"suspicious"` | `Symbol` | `Map<Address, bool>` |
| `"trusted"` | `Symbol` | `Map<Address, bool>` |

### Read-Modify-Write Pattern

```rust
let mut map: Map<Address, bool> = env.storage()
    .instance()
    .get(&key)
    .unwrap_or(Map::new(&env));
map.set(address, is_suspicious);
env.storage().instance().set(&key, &map);
```

Every write involves:
1. **Read** the current map from storage
2. **Modify** the map in memory
3. **Write** the entire map back to storage

For large maps, this becomes expensive. Future versions may use
individual keys per address (e.g., `Symbol::new(env, "suspicious")`
+ address) to avoid copying the entire map on each update.

### Storage Cost Estimation

Each `Address` stored as a key costs approximately:

- Key overhead: ~128 bytes (Symbol + Map entry metadata)
- Address: ~32 bytes
- Value (bool): ~1 byte
- Total per entry: ~160 bytes of WASM host storage

Instance storage billing is currently **free** to the contract caller
in many Soroban setups -- the contract deployer pays.

---

## Deployment

### Prerequisites

1. A funded Sorobum/Stellar testnet account (or mainnet for production)
2. Stellar CLI installed and configured
3. Built WASM file

### Deploy to Testnet

```bash
# Ensure you're on testnet
stellar network use testnet

# Fund a test account (if needed)
stellar keys fund <your-key> --network testnet

# Deploy the contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_inspector_contract.wasm \
  --network testnet \
  --source <your-key>
```

The deploy command returns the **contract ID**, e.g.:

```
CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA...
```

### Initialize the Contract

For instance-storage contracts, no initialization call is needed --
the maps default to empty on first access.

### Verify Deployment

```bash
# Check contract status
stellar contract inspect <contract-id> --network testnet

# Call a read-only function
stellar contract invoke \
  --id <contract-id> \
  --network testnet \
  --fn get_flags \
  --arg address=GBZKTZHZ3K7K7J4ZJ...
```

### Deploy to Mainnet

```bash
stellar network use mainnet

stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_inspector_contract.wasm \
  --network mainnet \
  --source <your-mainnet-key>
```

> **Important:** Deploying to mainnet costs real XLM. Ensure your
> account is sufficiently funded.

---

## Contract Upgrades

### Upgrade Path

This contract is deployed as a standard WASM module. To upgrade the
contract logic while preserving the same contract ID:

```bash
# Upload new WASM
stellar contract upload \
  --wasm target/wasm32-unknown-unknown/release/stellar_inspector_contract.wasm \
  --network testnet \
  --source <your-key>

# Upgrade the contract
stellar contract upgrade \
  --id <contract-id> \
  --wasm <new-wasm-hash> \
  --network testnet \
  --source <your-key>
```

### Breaking Changes

When upgrading, be aware that the storage layout must remain backward
compatible:

- Adding new keys is safe
- Removing existing keys will lose data
- Changing `Map<Address, bool>` to a different type will break existing
  data

### Version Management

The `Cargo.toml` defines the contract version. Bump the version in
`Cargo.toml` before deploying an upgrade:

```toml
version = "0.2.0"
```

---

## Development Workflow

### Full Development Cycle

```bash
# 1. Install dependencies
cargo install stellar-cli
rustup target add wasm32-unknown-unknown

# 2. Write code in src/lib.rs

# 3. Build
stellar contract build

# 4. Test locally
cargo test

# 5. Deploy to testnet
stellar contract deploy --wasm target/.../*.wasm --network testnet

# 6. Invoke functions
stellar contract invoke --id <contract-id> --fn is_suspicious ...
```

### Iterative Development

For faster iteration in tests, use the in-memory test environment:

```rust
use soroban_sdk::testutils::TestEnv;

let env = TestEnv::register(ContractErrors, SecurityRegistry);
let addr = Address::generate(&env);
env.register_address(addr.clone(), true);
assert!(env.is_suspicious(addr.clone()));
```

### IDE Setup

Recommended VS Code extensions:

- **rust-analyzer** -- Rust language server
- **Even Better TOML** -- TOML syntax highlighting for Cargo.toml
- **Soroban** (if available) -- Soroban-specific tooling

### Common Commands Cheat Sheet

| Command | Purpose |
|---|---|
| `cargo build --target wasm32-unknown-unknown --release` | Build WASM |
| `cargo test` | Run all tests |
| `cargo clippy` | Lint |
| `cargo fmt` | Format code |
| `stellar contract build` | Build via Stellar CLI |
| `stellar contract deploy --network testnet` | Deploy to testnet |
| `stellar contract invoke --network testnet --fn ...` | Invoke a function |

---

## Security Considerations

### Access Control

**Current state:** The contract has **no access control**. Any address
can call `register_address` or `register_trusted` to mark any other
address.

**Recommendation:** Add access control to restrict these functions to a
governance address or multisig:

```rust
pub fn register_address(env: Env, address: Address, is_suspicious: bool) {
    let admin = env.storage().instance().get(&symbol!("admin"))
        .unwrap_or(/* default admin */);
    if env.invoker() != admin {
        panic!("unauthorized");
    }
    // ... rest of logic
}
```

### Data Integrity

The `is_suspicious` and `is_trusted` check uses `unwrap_or(false)`,
meaning any address not in the respective map defaults to `false`.
This is the intended safe default -- unknown addresses are not flagged.

### Map Growth

Each call to `register_address` or `register_trusted` reads the entire
map from storage, modifies it, and writes it back. As the map grows:

- **Storage cost** increases linearly
- **Gas cost** increases (more data to deserialize/serialize)
- **Invocation latency** increases

For production with thousands of addresses, consider using individual
storage keys per address instead of a single map.

### Re-entrancy

Rust/Soroban does not have re-entrancy issues equivalent to Solidity,
as contract functions are not interrupted by external calls during
execution.

### Overflow Checks

The release profile has `overflow-checks = true`, which causes panics
on arithmetic overflow. This is appropriate for financial smart
contracts.

---

## Gas & Performance

### Read Operations (is_suspicious, is_trusted, get_flags)

| Operation | Approx. CPU | Approx. Memory Reads |
|---|---|---|
| `is_suspicious` (cold) | ~5,000 | 1 storage read |
| `is_suspicious` (warm) | ~1,000 | 1 storage read |
| `get_flags` | ~8,000 | 2 storage reads |

### Write Operations (register_address, register_trusted)

Cost scales with map size due to the read-modify-write pattern:

| Map size | register_address | register_trusted |
|---|---|---|
| 1 entry | ~50,000 | ~50,000 |
| 100 entries | ~200,000 | ~200,000 |
| 1,000 entries | ~1,000,000 | ~1,000,000 |

### Optimization Opportunities

1. **Individual storage keys** -- Store each address as a separate key
   instead of a map, reducing write costs from O(n) to O(1).
2. **Trusted map separation** -- The trusted map only stores `true`
   values; a simpler data structure (set) could reduce storage overhead.
3. **Batch registration** -- Allow registering multiple addresses in a
   single invocation to amortize the read-modify-write cost.

---

## Integration with the Backend

The Stellar Inspector backend queries this contract to enrich
transaction analysis with on-chain trust data. The integration flow:

```
Backend receives transaction
  |
  |  Extracts all addresses from operations
  v
Backend calls contract.get_flags(address) for each
  |
  |  Soroban JSON-RPC (or direct Horizon call)
  v
Contract returns (is_suspicious, is_trusted)
  |
  |  Backend annotates SecurityReport
  v
Frontend displays trust flags in UI
```

### Integration Methods

| Method | Description |
|---|---|
| **Soroban JSON-RPC** | Direct RPC calls to a Soroban RPC node |
| **Stellar CLI** | `stellar contract invoke` from backend shell |
| **Horizon Contract Endpoint** | If Horizon supports contract invocations (future) |

### Example: Backend Query

```rust
// Pseudo-code for backend integration
let contract_id = ContractId::from_str("CA...the-contract-id...");
let flags: (bool, bool) = soroban_client
    .invoke_contract(&contract_id, "get_flags", &vec![address])
    .await?;
```

---

## Troubleshooting

### Build Errors

**"target wasm32-unknown-unknown not installed"**

```bash
rustup target add wasm32-unknown-unknown
```

**"`soroban_sdk` not found"**

Ensure you're using Soroban SDK 21 and Rust 1.75+. Check your
toolchain:

```bash
rustc --version
cargo --version
```

### Test Failures

**"test env registration failed"**

Ensure the `testutils` feature is enabled in dev-dependencies:

```toml
[dev-dependencies]
soroban-sdk = { version = "21", features = ["testutils"] }
```

### Deployment Issues

**"insufficient balance"**

Fund your testnet account:

```bash
stellar keys fund <your-key> --network testnet
```

**"contract already exists"**

The contract ID is already deployed. To re-deploy, either:

- Use a different account/source
- Upgrade the existing contract (see [Contract Upgrades](#contract-upgrades))

### Invocation Errors

**"host budget is exhausted"**

The map is too large for a single read-modify-write operation. Consider
architexturing storage or reducing the number of entries.

---

## Contributing

1. Fork the repository.
2. Create a feature branch: `git checkout -b feat/my-feature`.
3. Write code and tests.
4. Run tests: `cargo test`.
5. Run clippy: `cargo clippy`.
6. Format: `cargo fmt`.
7. Build WASM: `cargo build --target wasm32-unknown-unknown --release`.
8. Commit: `git commit -m "feat: add my feature"`.
9. Push: `git push origin feat/my-feature`.
10. Open a pull request.

### Coding Standards

- Use `cargo fmt` for consistent formatting.
- Use `cargo clippy -- -D warnings` for strict linting.
- Prefer `soroban_sdk` types over raw Rust types.
- Never use `unwrap()` in non-test code -- use `expect()` with a
  descriptive message or proper error handling.
- Keep functions small and focused on a single responsibility.
- Document all public functions with doc comments.

### Pull Request Checklist

- [ ] `cargo fmt` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes (all tests green)
- [ ] WASM build succeeds
- [ ] Code is documented with doc comments
- [ ] New functions are covered by tests
- [ ] Breaking changes are noted in the PR description
