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

## What Gives Users Confidence

- The Merkle root is committed on-chain — tamper-evident and permanent
- The ZK proof guarantees `assets ≥ liabilities` without revealing any individual balance
- Each user can independently verify their own inclusion using only their own data
- The proof is publicly verifiable by anyone with the on-chain transaction hash
