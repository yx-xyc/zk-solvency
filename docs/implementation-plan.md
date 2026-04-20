# Implementation Plan

---

## Phase 1 — Core ZK Solvency

### Step 1: Data Layer
- Define `UserBalance { id: u64, balance: u64 }` struct
- Mock data generator: produce N users with random balances
- Merkle tree builder over the sorted list of `(id, balance)` leaves

### Step 2: SP1 zkVM Program
- **Private inputs**: list of `UserBalance`, list of reserve balances (assets)
- **Program logic**:
  1. Build Merkle tree from user balances → compute `merkle_root`
  2. Sum all user balances → `total_liabilities`
  3. Sum all reserve balances → `total_assets`
  4. Assert `total_assets ≥ total_liabilities`
- **Public outputs committed to proof**: `merkle_root`, `total_liabilities`, `total_assets`, `surplus`

### Step 3: Solidity Smart Contract
- Wrap SP1's auto-generated verifier
- `SolvencyAttestation` contract:
  - Stores latest attestation: `(merkle_root, total_liabilities, total_assets, block_timestamp)`
  - `submitProof(bytes proof, PublicValues pubValues)` — verifies and records
  - Emits `SolvencyProven(merkle_root, surplus, timestamp)` event

### Step 4: Integration Script
- Rust script: generate mock data → run SP1 prover → serialize proof → call contract via RPC

**Status**: [ ] Not started

---

## Phase 2 — Inclusion Proofs & Benchmarking

### Step 5: Merkle Inclusion Proof Module
- Given a `user_id`, walk the Merkle tree and output a `MerkleProof { leaf, siblings, path_bits }`
- Verification function: recompute root from leaf + proof, compare against on-chain root
- CLI tool: `cargo run --bin inclusion -- --user-id <ID> --data-file <PATH>`

### Step 6: Benchmark Suite
Measure the following as N (number of users) scales: 10 / 100 / 500 / 1000 / 5000

| Metric | Tool |
|---|---|
| Proof generation time | `std::time::Instant` in Rust |
| Peak memory during proving | `heaptrack` or `/usr/bin/time -v` |
| On-chain verification gas cost | Foundry `forge test --gas-report` |
| Inclusion proof generation time | `std::time::Instant` |

Results written to `benchmark/results.csv` and summarized in `docs/benchmarks.md`.

**Status**: [ ] Not started

---

## Phase 3 — User-Facing Website

### Step 7: Frontend (Next.js)
Pages:
- `/` — Home: shows latest on-chain attestation (merkle root, surplus, timestamp)
- `/check` — Inclusion checker: user enters their ID → frontend requests Merkle proof from local API → displays result with on-chain proof link

Backend API (Next.js API routes):
- `GET /api/attestation` — fetch latest attestation from chain
- `POST /api/inclusion` — body: `{ user_id }`, returns `{ balance, proof, verified: bool }`

Trust indicators shown to user:
- Green checkmark + balance amount if included
- Link to Etherscan/block explorer for the on-chain attestation tx
- Display of the Merkle root that was verified against

**Status**: [ ] Not started

---

## Progress Tracker

| Step | Description | Status |
|---|---|---|
| 1 | Data layer + Merkle tree | [ ] |
| 2 | SP1 zkVM program | [ ] |
| 3 | Solidity contract | [ ] |
| 4 | Integration script | [ ] |
| 5 | Inclusion proof module | [ ] |
| 6 | Benchmark suite | [ ] |
| 7 | Frontend website | [ ] |
