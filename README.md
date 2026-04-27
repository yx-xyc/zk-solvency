# ZK Solvency Protocol

A zero-knowledge proof system for cryptocurrency exchange solvency. An exchange proves `total_assets ≥ total_liabilities` without revealing individual user balances. Users can verify their balance is included via a Merkle proof — entirely in the browser.

## How It Works

1. Exchange provides user balances (liabilities) and reserve balances (assets) as private inputs to the SP1 zkVM program.
2. The program builds a SHA-256 Merkle tree over user balances, sums both sides, asserts solvency, and commits four public values: `merkleRoot`, `assetsCommitment`, `totalLiabilities`, `totalAssets`.
3. A ZK proof is generated and submitted to `SolvencyAttestation.sol` on-chain.
4. Any user can verify their balance is in the Merkle root using the web UI — no Rust binary required.

## Architecture

```
Private Inputs              SP1 zkVM Program                 Public Outputs (128 bytes)
──────────────              ─────────────────                ──────────────────────────
User balances (N)  ──────►  Build SHA-256 Merkle tree   ──►  merkleRoot       (bytes32)
Reserve balances   ──────►  Compute assets commitment   ──►  assetsCommitment (bytes32)
                            Sum liabilities & assets    ──►  totalLiabilities (uint64)
                            Assert assets ≥ liabilities ──►  totalAssets      (uint64)
                                        │
                                        ▼
                               ZK Proof (Groth16)
                                        │
                                        ▼
                            SolvencyAttestation.sol
                            Verifies proof, stores attestation on-chain
```

---

## Prerequisites

| Tool | Purpose | Install |
|------|---------|---------|
| **Rust** (stable) | Build all crates | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **SP1 toolchain** | Compile zkVM program to RISC-V ELF | `curl -L https://sp1up.succinct.xyz \| bash && sp1up` |
| **Foundry** | Compile and test Solidity contracts | `curl -L https://foundry.paradigm.xyz \| bash && foundryup` |
| **Node.js ≥ 18** | Run the Next.js web frontend | [nodejs.org](https://nodejs.org) or via `nvm` |

Verify your SP1 install:
```bash
cargo prove --version   # should print sp1-cli ...
```

---

## Quickstart

### 1. Generate sample data

```bash
cargo run -p data-gen
```

This writes `data/users.json` (100 users) and `data/reserves.json` to the repo root.

You can customise the dataset size:
```bash
cargo run -p data-gen -- --users 500 --surplus 0.2
```

Validation: `--users` and `--reserves` must be ≥ 1; `--surplus` must be ≥ 0 (negative means insolvent).

### 2. Build the zkVM program (ELF)

Run this whenever `crates/program/src/main.rs` changes:

```bash
cd crates/program
cargo prove build
cd ../..
```

The ELF is compiled to `target/elf-compilation/riscv64im-succinct-zkvm-elf/release/solvency-program` and baked into the script binary via `include_bytes!`.

### 3. Generate a ZK proof

**Mock mode** (instant, for development — no real proof):
```bash
SP1_PROVER=mock cargo run --manifest-path script/Cargo.toml --bin script
```

**Network mode** (real Groth16 proof via Succinct Prover Network, ~30–60 s):
```bash
SP1_PROVER=network \
NETWORK_PRIVATE_KEY=<your_key> \
cargo run --manifest-path script/Cargo.toml --bin script
```

Both modes write `proof.json` to the repo root:
```json
{
  "proof_bytes":   "0x...",
  "public_values": "0x<merkleRoot_32><assetsCommitment_32><liabilities_32><assets_32>",
  "program_vkey":  "0x..."
}
```

`public_values` is 128 bytes (256 hex chars) in Solidity ABI encoding:

| Bytes | Field |
|-------|-------|
| 0–31  | `merkleRoot` — SHA-256 Merkle root of all user balances |
| 32–63 | `assetsCommitment` — SHA-256 hash of all reserve balances |
| 64–95 | `totalLiabilities` — sum of user balances (uint64, 32-byte slot) |
| 96–127 | `totalAssets` — sum of reserve balances (uint64, 32-byte slot) |

### 4. Run Solidity tests

```bash
cd contracts
forge test
```

Expected: **7 tests pass** (4 happy-path + 3 negative/rejection tests).

To also test on a local fork with a real SP1 verifier, see `contracts/README.md` (if present) or deploy with `forge script`.

### 5. Start the web frontend

```bash
cd web
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000). The page reads `proof.json` from the repo root and displays the latest attestation. Users can enter any user ID to verify their balance inclusion — Merkle path verification runs in the browser using the Web Crypto API (no Rust binary needed).

> **Demo scope note:** The inclusion checker is a demo UX. The server generates Merkle proof material per request and only returns the queried user's data; it is not a production-authenticated lookup service.

---

## Project Structure

```
zk-solvency/
├── crates/
│   ├── types/         # Shared types: UserBalance, ReserveBalance, MerkleTree, assets_commitment
│   ├── data-gen/      # CLI: generates data/users.json and data/reserves.json
│   ├── program/       # SP1 zkVM guest program — the ZK circuit
│   ├── inclusion/     # CLI: Merkle inclusion proof generator and verifier (Rust)
│   └── bench/         # Merkle tree operation benchmarks
├── script/            # Separate Cargo workspace — proof generation via sp1-sdk
│   ├── src/main.rs          # Proof generation script (SP1_PROVER=mock|network)
│   ├── src/bench.rs         # SP1 mock timing benchmark across dataset sizes
│   └── src/test_insolvency.rs  # Verifies guest panic on insolvent inputs → zero public values
├── contracts/
│   ├── src/SolvencyAttestation.sol   # On-chain proof verifier and attestation store
│   └── test/SolvencyAttestation.t.sol
├── web/
│   ├── app/page.tsx                  # Server component: reads proof.json, shows attestation
│   ├── app/api/verify/route.ts       # POST endpoint: generates one user's Merkle proof server-side
│   ├── _components/InclusionChecker.tsx  # Client component: submits ID, re-derives leaf, verifies proof in browser
│   └── _lib/merkle.ts                # TypeScript Merkle tree (mirrors crates/types/src/merkle.rs)
├── docs/              # Architecture, benchmarks, progress notes
└── data/              # Generated (gitignored): users.json, reserves.json
```

**Why is `script/` a separate Cargo workspace?** `sp1-sdk` carries a forked `serde_core` that conflicts with `alloy`'s derive macros in a shared dependency graph. The separate workspace is the upstream-recommended pattern for SP1 projects.

---

## Cryptographic Details

### Merkle tree

- **Leaf**: `SHA-256(user.id_le_8 || user.balance_le_8)` — 16-byte input, little-endian u64s
- **Node**: `SHA-256(left_32 || right_32)`
- **Padding**: repeated last leaf to next power of two (e.g. 100 users → 128 leaves → depth 7)
- **Inclusion proof**: sibling path from leaf to root, O(log N)

### Assets commitment

`SHA-256(reserve[0].id_le_8 || reserve[0].balance_le_8 || reserve[1].id_le_8 || reserve[1].balance_le_8 || ...)` — binds the prover to the exact asset set (identity + amount) in order. Two reserve sets that sum to the same total but differ in individual allocations or IDs produce different commitments.

### Public value encoding

Solidity ABI encoding (`abi.encode(bytes32, bytes32, uint64, uint64)`): each field occupies a 32-byte slot. `uint64` values are right-aligned with leading zero padding.

---

## Running Benchmarks

**Merkle tree operations** (native Rust, release build):
```bash
cargo run -p bench --release
```

**SP1 proof generation timing** across dataset sizes:
```bash
SP1_PROVER=mock cargo run --manifest-path script/Cargo.toml --bin bench
```

See `docs/benchmarks.md` for reference numbers on Apple Silicon.

---

## Deployment

After generating a real Groth16 proof (network mode), submit it on-chain:

1. Deploy `SolvencyAttestation.sol` with the SP1 verifier gateway address and `PROGRAM_VKEY` from `proof.json`.
2. Run the Forge submit script:

```bash
CONTRACT_ADDRESS=0x... PRIVATE_KEY=0x... \
PROOF_BYTES=$(jq -r '.proof_bytes' proof.json) \
PUBLIC_VALUES=$(jq -r '.public_values' proof.json) \
forge script contracts/script/Submit.s.sol \
  --rpc-url <RPC_URL> --broadcast
```

SP1 verifier gateway on Sepolia: `0x397A5f7f3dBd538f23DE225B51f532c34448dA9B`

The contract records the attestation (merkle root, assets commitment, totals, timestamp) and emits a `SolvencyProven` event.
