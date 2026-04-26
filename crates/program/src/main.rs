#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use alloy_sol_types::sol_data::{FixedBytes, Uint};
use types::{ReserveBalance, UserBalance, merkle::MerkleTree};

type PublicValuesTuple = (FixedBytes<32>, Uint<64>, Uint<64>);

pub fn main() {
    let users: Vec<UserBalance>       = sp1_zkvm::io::read();
    let reserves: Vec<ReserveBalance> = sp1_zkvm::io::read();

    let tree = MerkleTree::build(&users);
    let merkle_root = tree.root;

    let total_liabilities: u64 = users.iter().map(|u| u.balance).sum();
    let total_assets: u64      = reserves.iter().map(|r| r.balance).sum();

    assert!(total_assets >= total_liabilities, "insolvent");

    let encoded = PublicValuesTuple::abi_encode(&(merkle_root, total_liabilities, total_assets));
    sp1_zkvm::io::commit_slice(&encoded);
}
