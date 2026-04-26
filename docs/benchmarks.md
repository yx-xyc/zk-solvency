# Benchmarks

---

## Proof Generation Time vs. Dataset Size

SP1 mock mode — times the full zkVM execution (Merkle tree + solvency assertion) for each N.
Gas cost is constant regardless of N: the on-chain Groth16 verifier always verifies the same fixed-size proof.

| N (users) | SP1 Mock Time (s) | Peak Memory | Gas Cost (submitProof) |
|---|---|---|---|
| 10 | 0.074 | N/A | ~97k (mock) / ~250k (real Groth16) |
| 100 | 0.469 | N/A | ~97k (mock) / ~250k (real Groth16) |
| 500 | 1.947 | N/A | ~97k (mock) / ~250k (real Groth16) |
| 1000 | 4.031 | N/A | ~97k (mock) / ~250k (real Groth16) |
| 5000 | 27.152 | N/A | ~97k (mock) / ~250k (real Groth16) |

Gas note: mock verifier costs 857 gas for `verifyProof`; real SP1 Groth16 gateway on Sepolia costs ~250k gas (fixed, independent of N).

---

## Merkle Tree Operations

Native Rust benchmark (release build, averaged over multiple iterations).

| N (users) | build (ms) | prove (µs) | verify (µs) |
|---|---|---|---|
| 10 | 0.016 | 0.121 | 2.575 |
| 100 | 0.088 | 0.137 | 3.124 |
| 500 | 0.271 | 0.167 | 3.073 |
| 1000 | 0.458 | 0.142 | 2.905 |
| 5000 | 3.116 | 0.149 | 3.780 |

---

## Inclusion Proof Generation Time

Full round-trip: `MerkleTree::build` + `tree.prove` + `proof.verify` (release build).

| N (users) | Inclusion Proof Gen Time (ms) |
|---|---|
| 10 | 0.008 |
| 100 | 0.076 |
| 500 | 0.248 |
| 1000 | 0.443 |
| 5000 | 3.104 |

---

## Notes

- Hardware: Apple Silicon (arm64), macOS Darwin
- SP1 version: 6.1.0
- Proving mode: mock (`SP1_PROVER=mock`) — executes program logic without generating a real ZK proof
- Merkle benchmarks: release build (`cargo run -p bench --release`), averaged over 10–200 iterations depending on N
- SP1 benchmarks: debug build (`cargo run --manifest-path script/Cargo.toml --bin bench`), single run per N
- Tree depth: ⌈log₂(N)⌉ levels after padding to next power of two (e.g. N=100 → 128 leaves → depth 7)
- `prove` time is O(log N) — nearly constant because it's just array index lookups
- `build` time is O(N) — dominated by N SHA256 hashes for the leaf layer
- SP1 mock time grows roughly linearly with N (program executes more SHA256 precompile calls)
