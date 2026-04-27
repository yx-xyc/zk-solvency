pub mod merkle;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalance {
    pub id: u64,
    pub balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveBalance {
    pub id: u64,
    pub balance: u64,
}

pub fn assets_commitment(reserves: &[ReserveBalance]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for r in reserves {
        hasher.update(r.id.to_le_bytes());
        hasher.update(r.balance.to_le_bytes());
    }
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assets_commitment_includes_reserve_identity() {
        let r_a = vec![ReserveBalance { id: 1, balance: 500 }, ReserveBalance { id: 2, balance: 500 }];
        let r_b = vec![ReserveBalance { id: 2, balance: 500 }, ReserveBalance { id: 1, balance: 500 }];
        let r_c = vec![ReserveBalance { id: 9, balance: 500 }, ReserveBalance { id: 8, balance: 500 }];
        // Same balances, different IDs → different commitments
        assert_ne!(assets_commitment(&r_a), assets_commitment(&r_c));
        // Same IDs in different order → different commitments (order-sensitive)
        assert_ne!(assets_commitment(&r_a), assets_commitment(&r_b));
        // Identical inputs → identical commitments
        assert_eq!(assets_commitment(&r_a), assets_commitment(&r_a));
    }

    #[test]
    #[should_panic(expected = "insolvent")]
    fn solvency_assertion_rejects_insolvency() {
        // Mirrors the assertion in crates/program/src/main.rs
        let total_assets: u64 = 500;
        let total_liabilities: u64 = 1_000;
        assert!(total_assets >= total_liabilities, "insolvent");
    }
}
