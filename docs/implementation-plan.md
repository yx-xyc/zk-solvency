# Implementation Plan

---

## Phase 1 — Core ZK Solvency ✅ Complete

### Step 1: Data Layer ✅
- `UserBalance { id: u64, balance: u64 }` and `ReserveBalance { id: u64, balance: u64 }` in `crates/types`
- SHA256 Merkle tree: `build()`, `prove()`, `verify()` with 4 passing unit tests
- `crates/data-gen` CLI: generates `data/users.json` and `data/reserves.json` with guaranteed solvency surplus

### Step 2: SP1 zkVM Program ✅
- **Private inputs**: `Vec<UserBalance>`, `Vec<ReserveBalance>`
- **Program logic**:
  1. Build Merkle tree from user balances → compute `merkle_root`
  2. Sum all user balances → `total_liabilities`
  3. Sum all reserve balances → `total_assets`
  4. Assert `total_assets ≥ total_liabilities`
- **Public outputs**: ABI-encoded `(bytes32 merkleRoot, uint64 totalLiabilities, uint64 totalAssets)`
- `programVKey = 0x00680f24d7f1c5c844c2852e84244b6a34215092dc492599792cee4304fd15dd`

### Step 3: Solidity Smart Contract ✅
- `SolvencyAttestation.sol` in `contracts/`:
  - Stores latest attestation: `(merkleRoot, totalLiabilities, totalAssets, timestamp)`
  - `submitProof(bytes proofBytes, bytes publicValues)` — verifies via SP1 gateway, records on-chain
  - Emits `SolvencyProven(bytes32 merkleRoot, uint64 totalLiabilities, uint64 totalAssets, uint256 timestamp)`
  - SP1 Groth16 gateway on Sepolia: `0x397A5f7f3dBd538f23DE225B51f532c34448dA9B`
- 4 passing Forge tests (mock verifier + real public values from script)

### Step 4: Integration Script ✅
- `script/src/main.rs` — loads data, runs SP1 prover, saves `proof.json`
- `SP1_PROVER=mock` for dev (instant); `SP1_PROVER=network` for real Groth16 proof via Succinct cloud GPU
- Separate Cargo workspace (avoids `serde_core` conflict with sp1-sdk)

---

## Phase 2 — Inclusion Proofs & Benchmarking ✅ Complete

### Step 5: Merkle Inclusion Proof CLI ✅
- `crates/inclusion` — `cargo run -p inclusion -- --user-id <ID>`
- Rebuilds Merkle tree identically to the SP1 program, generates sibling-path proof, verifies against committed root in `proof.json`
- Exit 0 on success, exit 1 on not found / verification failure

### Step 6: Benchmark Suite ✅
Results across N = 10 / 100 / 500 / 1000 / 5000 users — see `docs/benchmarks.md`

| Metric | Tool | Finding |
|---|---|---|
| Merkle build time | `crates/bench` (release) | O(N) — 0.016 ms → 3.1 ms |
| Inclusion proof time | `crates/bench` (release) | O(N) — dominated by build |
| prove / verify | `crates/bench` (release) | O(log N) ≈ constant |
| SP1 mock execution time | `script/src/bench.rs` | ~linear — 0.07s → 27s |
| On-chain gas cost | `forge test --gas-report` | ~97k gas (mock) / ~250k (real Groth16), constant |

---

## Phase 3 — User-Facing Website ✅ Complete

### Step 7: Frontend (Next.js) ✅
- `web/` — Next.js 16 App Router, TypeScript, Tailwind CSS
- Home page (`/`):
  - Attestation card: merkle root, total liabilities, total assets, surplus — decoded from `proof.json`
  - Etherscan link to SP1 Groth16 verifier gateway on Sepolia
- Inclusion checker:
  - User enters ID → `POST /api/verify` → API route spawns `inclusion` binary → returns JSON
  - Green ✓ panel: balance, leaf hash, merkle root, proof depth
  - Red ✗ panel: user not found or verification failed

---

## Progress Tracker

| Step | Description | Status |
|---|---|---|
| 1 | Data layer + Merkle tree | ✅ Done |
| 2 | SP1 zkVM program | ✅ Done |
| 3 | Solidity contract | ✅ Done |
| 4 | Integration script | ✅ Done |
| 5 | Inclusion proof CLI | ✅ Done |
| 6 | Benchmark suite | ✅ Done |
| 7 | Frontend website | ✅ Done |
