use sp1_sdk::{Elf, HashableKey, ProveRequest, Prover, ProverClient, ProvingKey, SP1Stdin};
use types::{ReserveBalance, UserBalance};

// Pre-compiled ELF — run `cargo prove build` in crates/program to regenerate.
const SOLVENCY_ELF: &[u8] = include_bytes!(
    "../../target/elf-compilation/riscv64im-succinct-zkvm-elf/release/solvency-program"
);

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

    // Derive vkey directly from the proving key — never hardcoded.
    let program_vkey = pk.verifying_key().bytes32();
    println!("Program vkey : {program_vkey}");

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

    // 5. Save proof artifacts — mock goes to proof.mock.json, network to proof.json.
    //    Keeping them separate prevents a dev mock run from overwriting a real proof.
    let out_file = if mode == "mock" { "proof.mock.json" } else { "proof.json" };
    let artifacts = serde_json::json!({
        "proof_bytes":   format!("0x{}", hex::encode(&proof_bytes)),
        "public_values": format!("0x{}", hex::encode(&public_values)),
        "program_vkey":  program_vkey,
    });
    std::fs::write(out_file, serde_json::to_string_pretty(&artifacts).unwrap()).unwrap();
    println!("Proof artifacts saved to {out_file}");
    if mode != "mock" {
        println!("To submit on-chain:");
        println!("  CONTRACT_ADDRESS=0x... PRIVATE_KEY=0x... \\");
        println!("  PROOF_BYTES=$(jq -r '.proof_bytes' proof.json) \\");
        println!("  PUBLIC_VALUES=$(jq -r '.public_values' proof.json) \\");
        println!("  forge script contracts/script/Submit.s.sol:Submit \\");
        println!("    --root contracts --rpc-url <RPC_URL> --broadcast");
    }
}
