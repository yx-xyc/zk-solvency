# Development Progress

## Current Status
**Phase 1 — Core ZK Solvency** (in progress)

---

## Completed

### Environment Setup
- Rust 1.95.0 + Cargo installed at `~/.cargo/bin`
- SP1 toolchain installed at `~/.sp1/bin` (cargo-prove v4.2.1)
- Foundry installed (`forge`, `cast`, `anvil`)
- PATH configured in `.claude/settings.json` (project) and `~/.zshrc` (terminal)
- Git initialized, repo live at https://github.com/yx-xyc/zk-solvency

### Step 1 — Data Layer ✅
- Workspace `Cargo.toml` with `resolver = "2"`
- `crates/types` — shared library crate:
  - `UserBalance { id: u64, balance: u64 }` and `ReserveBalance { id: u64, balance: u64 }`
  - `merkle.rs` — SHA256-based Merkle tree with `build()`, `prove()`, inclusion proof `verify()`
  - 4 passing unit tests
- `crates/data-gen` — binary crate:
  - CLI with flags: `--users`, `--reserves`, `--seed`, `--surplus`, `--output`
  - Generates `data/users.json` and `data/reserves.json`
  - Guarantees `total_assets >= total_liabilities * (1 + surplus)`

### Step 2 — SP1 zkVM Program ✅
- `crates/program` targeting `riscv32im-succinct-zkvm-elf`
- Reads `Vec<UserBalance>` and `Vec<ReserveBalance>` as private inputs via `sp1_zkvm::io::read()`
- Rebuilds Merkle root, sums liabilities and assets, asserts solvency
- Commits ABI-encoded public outputs: `(merkle_root, total_liabilities, total_assets)`
- ELF at `target/elf-compilation/riscv64im-succinct-zkvm-elf/release/solvency-program`
- `programVKey = 0x00680f24d7f1c5c844c2852e84244b6a34215092dc492599792cee4304fd15dd`

### Step 3 — Solidity Smart Contract ✅
- Foundry project in `contracts/`
- `SolvencyAttestation.sol`:
  - Constructor takes SP1 Groth16 gateway address + `programVKey`
  - `submitProof(proofBytes, publicValues)` — verifies proof via SP1 gateway, stores attestation, emits `SolvencyProven` event
  - SP1 Groth16 gateway on Sepolia: `0x397A5f7f3dBd538f23DE225B51f532c34448dA9B`
- 3 passing unit tests (mock verifier)

### Step 4 — Integration Script ✅
- `script/` — standalone Cargo workspace (separate from root to avoid serde_core conflict with sp1-sdk)
- `script/src/main.rs`:
  - Loads `data/users.json` + `data/reserves.json`
  - Generates proof via `ProverClient::from_env()` — respects `SP1_PROVER` env var
  - Saves `proof.json` with proof bytes, public values, and programVKey
  - Optionally submits on-chain if `CONTRACT_ADDRESS` + `PRIVATE_KEY` + `RPC_URL` are set
- sp1-sdk 6.1.0 with `default-features = false` (avoids serde conflict)
- ELF embedded via `include_bytes!` pointing to `target/elf-compilation/riscv64im-succinct-zkvm-elf/release/solvency-program`
- Mock mode end-to-end verified: 96-byte public_values, correct Merkle root

---

## In Progress

*(nothing — all four steps done)*

---

## Up Next

### Step 5 — Inclusion Proof CLI
- CLI tool: given a user ID, outputs a Merkle inclusion proof for the latest attestation

### Step 6 — Benchmark Suite
- Measure proof generation time for N = 10 / 100 / 500 / 1000 / 5000 users
- Fill in `docs/benchmarks.md`

---

## Phase 2 — Inclusion Proofs + Benchmarking
- Step 5: CLI tool to generate Merkle inclusion proof for a given user ID
- Step 6: Benchmark suite (N = 10 / 100 / 500 / 1000 / 5000), fill in `docs/benchmarks.md`

## Phase 3 — Frontend Website
- Step 7: Next.js app with inclusion checker UI

---

## Key Decisions & Notes
- Using SHA256 (not Keccak256) for Merkle tree — SP1 has a SHA256 precompile (cheaper cycles)
- Merkle tree pads to next power of two by repeating the last leaf hash
- Public values are ABI-encoded (`abi.encode(bytes32, uint64, uint64)`) for Solidity compatibility
- Proving mode: `SP1_PROVER=mock` for dev (instant), `SP1_PROVER=network` for real proof (cloud GPU)
- `data/` is gitignored (generated files); `contracts/lib/` is gitignored (reinstall with `forge install`)
- No `Co-Authored-By` in commits (user preference)
