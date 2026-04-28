# ZK Solvency Protocol — Project Overview

**Author**: Vincent Xu (yx2021 | N13337595)
**Course**: Crypto and Blockchain

---

## Objective

Build a simplified proof-of-solvency protocol using the SP1 zkVM with on-chain verification via a Solidity smart contract. An exchange cryptographically proves `total_assets ≥ total_liabilities` without revealing individual user balances.

---

## Architecture

```
Private Inputs                  SP1 zkVM Program              Public Outputs (128 bytes)
─────────────────               ─────────────────             ──────────────────────────
User balances (liabilities) ──► Build SHA-256 Merkle tree ──► merkleRoot       (bytes32)
Reserve balances (assets)   ──► Compute assets commitment ──► assetsCommitment (bytes32)
                                Sum liabilities & assets  ──► totalLiabilities (uint64)
                                Assert assets ≥ liabilities ► totalAssets      (uint64)
                                        │
                                        ▼
                               ZK Proof (PLONK, 964 bytes)
                                        │
                                        ▼
                            SolvencyAttestation.sol
                            - Verifies SP1 PLONK proof on-chain
                            - Stores attestation (merkleRoot,
                              assetsCommitment, totals, timestamp)
                            - Emits SolvencyProven event
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| zkVM | SP1 v6.1.0 (Succinct) |
| ZK program | Rust |
| Proof system | PLONK (Gnark BN254) |
| Smart contract | Solidity + Foundry |
| Integration scripts | Rust |
| Frontend | Next.js (TypeScript) |

---

## Directory Structure

```
zk-solvency/
├── docs/                  # Project documentation
├── crates/
│   ├── types/             # Shared types: UserBalance, ReserveBalance, MerkleTree
│   ├── data-gen/          # CLI — generates data/users.json and data/reserves.json
│   ├── program/           # SP1 zkVM guest program (solvency logic, compiled to RISC-V ELF)
│   ├── inclusion/         # CLI — Merkle inclusion proof generator and verifier
│   └── bench/             # Merkle tree operation benchmarks
├── script/                # Standalone workspace — proof generation via sp1-sdk
├── contracts/             # Foundry project — SolvencyAttestation.sol + tests
├── web/                   # Next.js frontend — inclusion checker UI
├── proof.json             # Latest proof artifacts (proof_bytes, public_values, program_vkey)
├── deployment.json        # Sepolia deployment info (contract address, submit tx hash)
└── data/                  # Generated (gitignored) — users.json, reserves.json
```

---

## Live Deployment (Sepolia)

The full end-to-end pipeline has been executed and verified on Ethereum Sepolia testnet.

| Item | Value |
|---|---|
| Contract | `0x97d55Ff73f7592F85AafF025a94963d02266cC78` |
| SP1 PLONK gateway | `0xd685a80aF2d1761648e56716af4868d850Dae49B` |
| Program vkey | `0x0098ee1f091411258d9318cb9a146c4e48145cee16b45a774d0445772cbfca4f` |
| Submit tx | [`0xb952c483...`](https://sepolia.etherscan.io/tx/0xb952c483e839b5cfbe3694ac4e3a3ace9a643d2dea2273d867dd5c5ea8f43ea3) |
| `merkleRoot` | `0xc62b97ef52f4d1c1139f3d829235bfa7510b43beb1da0bf0d1b2f961452bb41b` |
| `totalLiabilities` | 501,258 |
| `totalAssets` | 601,509 |
| Surplus | +100,251 (~20%) |

---

## Professor Feedback (incorporated)

Original proposal had user-level inclusion proofs explicitly out of scope. Based on feedback, two additions are now in scope:

1. **Benchmark user inclusion proofs** — measure generation time as N (number of users) scales
2. **User-facing website** — simple UI where a user enters their ID and sees whether their balance is included in the latest solvency proof, with a link to the on-chain attestation

---

## Workspace Layout Note

`script/` is intentionally a **separate Cargo workspace** from the root workspace. This is the idiomatic layout recommended by Succinct for SP1 projects — `sp1-sdk` carries a forked `serde_core` crate that conflicts with `alloy`'s derive macros when both live in the same dependency graph. There is no clean in-tree fix: `[patch.crates-io]` is fragile across sdk updates, and Cargo's feature resolver cannot resolve crate identity conflicts. Keeping `script/` isolated is the upstream-recommended solution and what Succinct's own example repos do.

If a future sp1-sdk release drops the serde fork, `script/` can be folded back into the root workspace.

---

## Proving Modes

The integration script supports two proving modes, switched via a single environment variable:

| Mode | `SP1_PROVER` | How | When to use |
|---|---|---|---|
| Mock | `mock` | Runs program logic in SP1's simulator; no real ZK proof generated | Development — instant, no API key needed |
| Network | `network` | Real PLONK proof via Succinct's cloud GPU network (~5–15 min) | Production / submission |

Switching modes requires no code changes — just set the env var before running the script:
```bash
SP1_PROVER=mock    cargo run --manifest-path script/Cargo.toml --bin script   # dev
SP1_PROVER=network cargo run --manifest-path script/Cargo.toml --bin script   # production
```

The program vkey is derived automatically from `pk.vk.bytes32()` and written into `proof.json`. `Deploy.s.sol` reads it back at deploy time via `PROGRAM_VKEY=$(jq -r '.program_vkey' proof.json)` — nothing is hardcoded.

Mock proofs cannot be verified on-chain. On-chain submission is a separate step via the Forge script:
```bash
CONTRACT_ADDRESS=0x... PRIVATE_KEY=0x... \
PROOF_BYTES=$(jq -r '.proof_bytes' proof.json) \
PUBLIC_VALUES=$(jq -r '.public_values' proof.json) \
forge script contracts/script/Submit.s.sol:Submit \
  --root contracts --rpc-url <RPC_URL> --broadcast
```

---

## What Gives Users Confidence

- The Merkle root is committed on-chain — tamper-evident and permanent
- The ZK proof guarantees `assets ≥ liabilities` without revealing any individual balance
- Each user can independently verify their own inclusion using only their own data
- The proof is publicly verifiable by anyone with the on-chain transaction hash
