use clap::Parser;
use std::path::PathBuf;
use types::{
    UserBalance,
    merkle::{hash_leaf, MerkleTree},
};

#[derive(Parser)]
#[command(about = "Generate and verify a Merkle inclusion proof for a user balance")]
struct Args {
    /// User ID to prove inclusion for
    #[arg(short = 'u', long = "user-id")]
    user_id: u64,

    #[arg(long = "users-file", default_value = "data/users.json")]
    users_file: PathBuf,

    /// If present and exists, extract committed root and verify; otherwise just print proof
    #[arg(long = "proof-file", default_value = "proof.json")]
    proof_file: PathBuf,
}

#[derive(serde::Deserialize)]
struct ProofArtifacts {
    public_values: String,
}

fn extract_merkle_root(pv_hex: &str) -> Result<[u8; 32], String> {
    let stripped = pv_hex
        .strip_prefix("0x")
        .or_else(|| pv_hex.strip_prefix("0X"))
        .unwrap_or(pv_hex);
    let bytes = hex::decode(stripped).map_err(|e| format!("hex decode failed: {e}"))?;
    if bytes.len() < 32 {
        return Err(format!("public_values too short: {} bytes (need at least 32)", bytes.len()));
    }
    // ABI layout: bytes32 merkleRoot || bytes32 assetsCommitment || uint64 liabilities || uint64 assets
    // merkleRoot occupies the first 32 bytes directly — no offset needed.
    Ok(bytes[..32].try_into().unwrap())
}

fn fmt_hash(h: &[u8; 32]) -> String {
    format!("0x{}", hex::encode(h))
}

fn main() {
    let args = Args::parse();

    let users_str = std::fs::read_to_string(&args.users_file).unwrap_or_else(|e| {
        eprintln!("error: cannot read {}: {e}", args.users_file.display());
        std::process::exit(1);
    });
    let users: Vec<UserBalance> = serde_json::from_str(&users_str).unwrap_or_else(|e| {
        eprintln!("error: cannot parse {}: {e}", args.users_file.display());
        std::process::exit(1);
    });

    // Find by ID — do not assume id == array index
    let (leaf_index, user) = users
        .iter()
        .enumerate()
        .find(|(_, u)| u.id == args.user_id)
        .unwrap_or_else(|| {
            eprintln!("error: user ID {} not found in {}", args.user_id, args.users_file.display());
            std::process::exit(1);
        });

    // Build tree identically to the SP1 program (same padding, same root)
    let tree = MerkleTree::build(&users);
    let leaf_hash = hash_leaf(user);
    let proof = tree.prove(leaf_index);

    println!("Inclusion Proof");
    println!("  user_id  : {}", user.id);
    println!("  balance  : {}", user.balance);
    println!("  leaf_hash: {}", fmt_hash(&leaf_hash));
    println!("  merkle_root (recomputed): {}", fmt_hash(&tree.root));
    println!("  proof_depth: {}", proof.siblings.len());
    println!("  sibling path:");
    for (i, (sibling, is_left)) in proof.siblings.iter().zip(proof.path_bits.iter()).enumerate() {
        let side = if *is_left { "left child " } else { "right child" };
        println!("    layer {:2}: {} | sibling = {}", i, side, fmt_hash(sibling));
    }

    if !args.proof_file.exists() {
        println!();
        println!("note: {} not found — skipping committed-root verification", args.proof_file.display());
        println!("verification: skipped");
        return;
    }

    let proof_str = std::fs::read_to_string(&args.proof_file).unwrap_or_else(|e| {
        eprintln!("error: cannot read {}: {e}", args.proof_file.display());
        std::process::exit(1);
    });
    let artifacts: ProofArtifacts = serde_json::from_str(&proof_str).unwrap_or_else(|e| {
        eprintln!("error: cannot parse {}: {e}", args.proof_file.display());
        std::process::exit(1);
    });

    let committed_root = extract_merkle_root(&artifacts.public_values).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        std::process::exit(1);
    });

    println!("  committed_root (from proof.json): {}", fmt_hash(&committed_root));
    println!();

    if proof.verify(leaf_hash, committed_root) {
        println!("verification: OK — user {} is included in the committed Merkle root", user.id);
    } else {
        eprintln!("verification: FAILED — proof does not match committed root");
        std::process::exit(1);
    }
}
