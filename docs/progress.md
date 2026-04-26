# Development Progress

## Current Status
**Phase 1 — Core ZK Solvency** ✅ Complete

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
- 4 passing unit tests (mock verifier, including one using real script-generated public values)

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

### Step 5 — Inclusion Proof CLI ✅
- `crates/inclusion` — binary in the root workspace
- CLI: `cargo run -p inclusion -- --user-id <ID>`
- Flags: `-u/--user-id` (required), `--users-file` (default `data/users.json`), `--proof-file` (default `proof.json`)
- Rebuilds Merkle tree identically to the SP1 program, generates sibling-path proof, verifies against committed root from `proof.json`
- Prints leaf hash, recomputed root, proof depth, full sibling path, and verification result
- If `proof.json` is absent, prints proof and exits 0 (skips verification)
- Exit 1 on: user ID not found, I/O error, parse error, or failed verification

### Step 6 — Benchmark Suite ✅
- `crates/bench` — Merkle tree operations benchmark (root workspace, release build)
  - Times `MerkleTree::build`, `tree.prove`, `proof.verify`, and full inclusion round-trip
  - Run: `cargo run -p bench --release`
- `script/src/bench.rs` — SP1 mock proof timing (script workspace)
  - Times full zkVM execution for each N
  - Run: `SP1_PROVER=mock cargo run --manifest-path script/Cargo.toml --bin bench`
- Results recorded in `docs/benchmarks.md`
- Key findings: build is O(N), prove/verify are O(log N) ≈ constant; SP1 mock grows ~linearly with N (27s at N=5000)

---

### Step 7 — Next.js Frontend ✅
- `web/` — Next.js 16 App Router, TypeScript, Tailwind CSS
- Server component reads `proof.json`, decodes ABI-encoded public_values, renders attestation card
- Client component (`InclusionChecker`) handles form state and fetches `/api/verify`
- API route (`app/api/verify/route.ts`) spawns the `inclusion` Rust binary as a subprocess, parses stdout
- Verified end-to-end: attestation card shows correct merkle root/totals; user ID 42 → ✓ included; user ID 9999 → ✗ not found
- Start with: `cd web && npm run dev` → http://localhost:3000

---

## In Progress

*(nothing — all seven steps done)*

---

## Up Next

*(Phase 1 complete)*

---

## Key Decisions & Notes
- Using SHA256 (not Keccak256) for Merkle tree — SP1 has a SHA256 precompile (cheaper cycles)
- Merkle tree pads to next power of two by repeating the last leaf hash
- Public values are ABI-encoded (`abi.encode(bytes32, uint64, uint64)`) for Solidity compatibility
- Proving mode: `SP1_PROVER=mock` for dev (instant), `SP1_PROVER=network` for real proof (cloud GPU)
- `data/` is gitignored (generated files); `contracts/lib/` is gitignored (reinstall with `forge install`)
- No `Co-Authored-By` in commits (user preference)
