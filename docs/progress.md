# Development Progress

## Current Status
**Phase 1 — Core ZK Solvency** (in progress)

---

## Completed

### Environment Setup
- Rust 1.95.0 + Cargo installed at `~/.cargo/bin`
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
- **Not yet run**: data generator not executed yet — do this at the start of next session

---

## Up Next

### Step 2 — SP1 zkVM Program
- Install SP1 toolchain (`sp1up`) if not already available (check `~/.sp1/bin`)
- Create `crates/program` crate targeting SP1
- Private inputs via `sp1_zkvm::io::read()`: user list + reserve list
- Program logic:
  1. Rebuild Merkle root from user list
  2. Sum liabilities and assets
  3. `assert!(total_assets >= total_liabilities)`
- Public outputs via `sp1_zkvm::io::commit()`: `merkle_root`, `total_liabilities`, `total_assets`

### Step 3 — Solidity Smart Contract
- Foundry project in `contracts/`
- `SolvencyAttestation.sol` wraps SP1 verifier, stores attestation on-chain

### Step 4 — Integration Script
- Rust binary in `crates/script` using `sp1-sdk`
- Generates proof → submits to contract via RPC

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
- `data/` is gitignored (generated files)
- No `Co-Authored-By` in commits (user preference)
