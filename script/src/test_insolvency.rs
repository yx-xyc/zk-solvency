// Execution-only test for the insolvency rejection path.
//
// SP1's execute() returns Ok even when the guest panics — the execution is
// "complete" from the SDK's perspective, but no public values are committed
// because the guest panics before reaching commit_slice. We verify this by
// checking that insolvent inputs produce zero committed bytes.
//
// Usage: SP1_PROVER=mock cargo run --manifest-path script/Cargo.toml --bin test-insolvency

use sp1_sdk::{Elf, Prover, ProverClient, SP1Stdin};
use types::{ReserveBalance, UserBalance};

const SOLVENCY_ELF: &[u8] = include_bytes!(
    "../../target/elf-compilation/riscv64im-succinct-zkvm-elf/release/solvency-program"
);

fn stdin_for(users: &Vec<UserBalance>, reserves: &Vec<ReserveBalance>) -> SP1Stdin {
    let mut stdin = SP1Stdin::new();
    stdin.write(users);
    stdin.write(reserves);
    stdin
}

#[tokio::main]
async fn main() {
    std::env::var("SP1_PROVER")
        .expect("SP1_PROVER must be set — use 'mock' for dev or 'network' for production");
    let client = ProverClient::from_env().await;

    // ── Solvent: assets > liabilities ────────────────────────────────────────
    let users    = vec![UserBalance    { id: 0, balance: 1_000 }];
    let reserves = vec![ReserveBalance { id: 0, balance: 1_200 }];

    let (pv, _) = client
        .execute(Elf::Static(SOLVENCY_ELF), stdin_for(&users, &reserves))
        .await
        .expect("solvent execution should succeed");

    assert!(!pv.as_slice().is_empty(), "solvent program must commit public values");
    assert_eq!(pv.as_slice().len(), 128, "public values must be 128 bytes (4 ABI slots)");
    println!("[PASS] solvent case — committed {} bytes of public values", pv.as_slice().len());

    // ── Insolvent: assets < liabilities ──────────────────────────────────────
    // The guest panics at `assert!(total_assets >= total_liabilities, "insolvent")`
    // before reaching commit_slice, so no public values are emitted.
    let users_insolvent    = vec![UserBalance    { id: 0, balance: 1_000 }];
    let reserves_insolvent = vec![ReserveBalance { id: 0, balance: 500  }];

    let (pv_bad, _) = client
        .execute(Elf::Static(SOLVENCY_ELF), stdin_for(&users_insolvent, &reserves_insolvent))
        .await
        .expect("execute completes at SDK level even on guest panic");

    assert!(
        pv_bad.as_slice().is_empty(),
        "insolvent program must commit no public values; got {} bytes",
        pv_bad.as_slice().len()
    );
    println!("[PASS] insolvent case — zero bytes committed (guest panicked before commit_slice)");

    println!("\nAll insolvency tests passed.");
}
