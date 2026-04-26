# ZK Solvency Protocol — Project Overview

**Author**: Vincent Xu (yx2021 | N13337595)
**Course**: Crypto and Blockchain

---

## Objective

Build a simplified proof-of-solvency protocol using the SP1 zkVM with on-chain verification via a Solidity smart contract. An exchange cryptographically proves `total_assets ≥ total_liabilities` without revealing individual user balances.

---

## Architecture

```
Private Inputs                  SP1 zkVM Program              Public Outputs
─────────────────               ─────────────────             ─────────────────
User balances (liabilities) ──► Recompute Merkle root    ──► merkle_root
Reserve balances (assets)   ──► Sum liabilities & assets ──► total_liabilities
                                Verify assets ≥ liabilities ► total_assets
                                                         ──► surplus
                                        │
                                        ▼
                               ZK Proof (STARK)
                                        │
                                        ▼
                            Solidity Verifier Contract
                            - Verifies SP1 proof
                            - Records attestation on-chain
                              (merkle_root, timestamp, valid)
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| zkVM | SP1 (Succinct) |
| ZK program | Rust |
| Smart contract | Solidity + Foundry |
| Integration scripts | Rust |
| Frontend | Next.js (TypeScript) |

---

## Directory Structure

```
zk-solvency/
├── docs/             # Project documentation (this folder)
├── program/          # SP1 zkVM Rust program (solvency logic)
├── script/           # Proof generation & on-chain submission scripts
├── contracts/        # Foundry project — Solidity verifier + attestation contract
├── data/             # Mock data generator (user balances, reserve balances)
├── inclusion/        # Merkle inclusion proof module
├── benchmark/        # Benchmark harness
└── web/              # Next.js frontend (user inclusion checker)
```

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
| Mock | `mock` (default) | Runs program logic in SP1's simulator; no real ZK proof generated | Development — instant, no API key needed |
| Network | `network` | Real Groth16 proof via Succinct's cloud GPU network (~30–60s) | Final demo / submission |

Switching modes requires no code changes — just set the env var before running the script:
```bash
SP1_PROVER=mock    cargo run -p script   # dev
SP1_PROVER=network cargo run -p script   # production (also requires NETWORK_PRIVATE_KEY)
```

Mock proofs cannot be verified on-chain. On-chain submission is only performed when `SP1_PROVER=network` and `CONTRACT_ADDRESS` + `PRIVATE_KEY` env vars are set.

---

## What Gives Users Confidence

- The Merkle root is committed on-chain — tamper-evident and permanent
- The ZK proof guarantees `assets ≥ liabilities` without revealing any individual balance
- Each user can independently verify their own inclusion using only their own data
- The proof is publicly verifiable by anyone with the on-chain transaction hash
