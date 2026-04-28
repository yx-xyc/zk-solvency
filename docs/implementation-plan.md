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
  1. Build Merkle tree from user balances → compute `merkleRoot`
  2. Compute `assetsCommitment = SHA-256(reserve[0].id || reserve[0].balance || ...)`
  3. Sum all user balances → `totalLiabilities`
  4. Sum all reserve balances → `totalAssets`
  5. Assert `totalAssets ≥ totalLiabilities`
- **Public outputs** (128 bytes): ABI-encoded `(bytes32 merkleRoot, bytes32 assetsCommitment, uint64 totalLiabilities, uint64 totalAssets)`
- `programVKey` is derived at runtime via `pk.vk.bytes32()` — not hardcoded. Current value: `0x0098ee1f091411258d9318cb9a146c4e48145cee16b45a774d0445772cbfca4f`

### Step 3: Solidity Smart Contract ✅
- `SolvencyAttestation.sol` in `contracts/`:
  - Stores latest attestation: `(merkleRoot, assetsCommitment, totalLiabilities, totalAssets, timestamp)`
  - `submitProof(bytes proofBytes, bytes publicValues)` — verifies via SP1 PLONK gateway, records on-chain
  - Emits `SolvencyProven(bytes32 indexed merkleRoot, bytes32 assetsCommitment, uint64 totalLiabilities, uint64 totalAssets, uint256 timestamp)`
  - SP1 PLONK gateway on Sepolia: `0xd685a80aF2d1761648e56716af4868d850Dae49B`
- 7 passing Forge tests: 4 happy-path + 3 negative (verifier rejection, malformed public values, zero initial state)
- `Deploy.s.sol` reads `PROGRAM_VKEY` from env (`jq -r '.program_vkey' proof.json`) — no hardcoded vkey

### Step 4: Integration Script ✅
- `script/src/main.rs` — loads data, runs SP1 prover, saves `proof.json`
- `SP1_PROVER=mock` for dev (instant); `SP1_PROVER=network` for real PLONK proof via Succinct cloud GPU
- `program_vkey` derived live from `pk.vk.bytes32()` and written into `proof.json`
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
| On-chain gas cost | `forge test --gas-report` | ~97k gas (mock) / ~250k (real PLONK), constant |

---

## Phase 3 — User-Facing Website ✅ Complete

### Step 7: Frontend (Next.js) ✅
- `web/` — Next.js 16 App Router, TypeScript, Tailwind CSS
- Home page (`/`):
  - Attestation card: merkle root, assets commitment, total liabilities, total assets — decoded from `proof.json` (128-byte public values)
  - Live Etherscan link to `SolvencyProven` tx when `deployment.json` is present; falls back to SP1 PLONK gateway link
- Inclusion checker (browser-native, no Rust binary):
  - User enters ID → `POST /api/verify` → API generates Merkle proof server-side, returns single user's proof material only
  - Client re-derives leaf hash from server-reported balance to detect tampering, then verifies sibling path against on-chain `merkleRoot` using `_lib/merkle.ts` + Web Crypto API
  - Green ✓ panel: balance, leaf hash, merkle root, proof depth
  - Red ✗ panel: user not found or verification failed

---

## Phase 4 — Sepolia Deployment ✅ Complete

### Step 8: On-Chain Deployment ✅
- Real PLONK proof generated via Succinct Prover Network (`SP1_PROVER=network`)
- `SolvencyAttestation` deployed to Sepolia: `0x97d55Ff73f7592F85AafF025a94963d02266cC78`
- Proof submitted on-chain — tx `0xb952c483e839b5cfbe3694ac4e3a3ace9a643d2dea2273d867dd5c5ea8f43ea3`
- `SolvencyProven` event emitted with `merkleRoot = 0xc62b97ef...`, `totalLiabilities = 501,258`, `totalAssets = 601,509`
- `deployment.json` created; web UI shows live Etherscan link

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
| 8 | Sepolia deployment + on-chain proof | ✅ Done |
