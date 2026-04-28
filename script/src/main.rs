use sp1_sdk::{Elf, ProveRequest, Prover, ProverClient, SP1Stdin};
use types::{ReserveBalance, UserBalance};

// Pre-compiled ELF — run `cargo prove build` in crates/program to regenerate.
const SOLVENCY_ELF: &[u8] = include_bytes!(
    "../../target/elf-compilation/riscv64im-succinct-zkvm-elf/release/solvency-program"
);

// Obtained via `cargo prove vkey` — must match SolvencyAttestation constructor.
const PROGRAM_VKEY: &str =
    "0x0098ee1f091411258d9318cb9a146c4e48145cee16b45a774d0445772cbfca4f";

#[tokio::main]
async fn main() {
    // 1. Load data
    let users: Vec<UserBalance> =
        serde_json::from_str(&std::fs::read_to_string("data/users.json").unwrap()).unwrap();
    let reserves: Vec<ReserveBalance> =
        serde_json::from_str(&std::fs::read_to_string("data/reserves.json").unwrap()).unwrap();

    // 2. Set up prover (SP1_PROVER must be set explicitly: "mock" or "network")
    let mode = std::env::var("SP1_PROVER")
        .expect("SP1_PROVER must be set — use 'mock' for dev or 'network' for production");
    let client = ProverClient::from_env().await;
    let pk = client.setup(Elf::Static(SOLVENCY_ELF)).await.unwrap();
    println!("Program vkey : {PROGRAM_VKEY}");

    // 3. Write private inputs
    let mut stdin = SP1Stdin::new();
    stdin.write(&users);
    stdin.write(&reserves);

    // 4. Generate proof
    println!("Generating proof (SP1_PROVER={mode})...");
    let proof = client.prove(&pk, stdin).plonk().await.unwrap();
    println!("Proof generated.");

    let proof_bytes   = proof.bytes();
    let public_values = proof.public_values.to_vec();
    println!("proof_bytes   ({} bytes): 0x{}", proof_bytes.len(),   hex::encode(&proof_bytes));
    println!("public_values ({} bytes): 0x{}", public_values.len(), hex::encode(&public_values));

    // 5. Save proof artifacts for on-chain submission (via forge script)
    let artifacts = serde_json::json!({
        "proof_bytes":   format!("0x{}", hex::encode(&proof_bytes)),
        "public_values": format!("0x{}", hex::encode(&public_values)),
        "program_vkey":  PROGRAM_VKEY,
    });
    std::fs::write("proof.json", serde_json::to_string_pretty(&artifacts).unwrap()).unwrap();
    println!("Proof artifacts saved to proof.json");
    println!("To submit on-chain:");
    println!("  CONTRACT_ADDRESS=0x... PRIVATE_KEY=0x... \\");
    println!("  PROOF_BYTES=$(jq -r '.proof_bytes' proof.json) \\");
    println!("  PUBLIC_VALUES=$(jq -r '.public_values' proof.json) \\");
    println!("  forge script contracts/script/Submit.s.sol --rpc-url <RPC_URL> --broadcast");
}
