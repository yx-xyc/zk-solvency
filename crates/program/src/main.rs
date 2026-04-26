#![no_main]
sp1_zkvm::entrypoint!(main);

use types::{ReserveBalance, UserBalance, merkle::MerkleTree};

pub fn main() {
    let users: Vec<UserBalance>       = sp1_zkvm::io::read();
    let reserves: Vec<ReserveBalance> = sp1_zkvm::io::read();

    let tree = MerkleTree::build(&users);
    let merkle_root = tree.root;

    let total_liabilities: u64 = users.iter().map(|u| u.balance).sum();
    let total_assets: u64      = reserves.iter().map(|r| r.balance).sum();

    assert!(total_assets >= total_liabilities, "insolvent");

    sp1_zkvm::io::commit(&merkle_root);
    sp1_zkvm::io::commit(&total_liabilities);
    sp1_zkvm::io::commit(&total_assets);
}
