use sha2::{Digest, Sha256};
use crate::UserBalance;

pub type Hash = [u8; 32];

pub struct MerkleTree {
    pub leaves: Vec<Hash>,
    pub layers: Vec<Vec<Hash>>,
    pub root: Hash,
}

/// Hash a single user balance leaf: SHA256(id_bytes || balance_bytes)
pub fn hash_leaf(user: &UserBalance) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(user.id.to_le_bytes());
    hasher.update(user.balance.to_le_bytes());
    hasher.finalize().into()
}

/// Hash two child nodes into a parent: SHA256(left || right)
pub fn hash_pair(left: &Hash, right: &Hash) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

impl MerkleTree {
    /// Build a Merkle tree from a list of users.
    /// The leaf list is padded to the next power of two by repeating the last leaf.
    pub fn build(users: &[UserBalance]) -> Self {
        assert!(!users.is_empty(), "cannot build Merkle tree from empty list");

        let mut leaves: Vec<Hash> = users.iter().map(hash_leaf).collect();

        // Pad to next power of two
        let target = leaves.len().next_power_of_two();
        let last = *leaves.last().unwrap();
        leaves.resize(target, last);

        let mut layers: Vec<Vec<Hash>> = vec![leaves.clone()];

        let mut current = leaves.clone();
        while current.len() > 1 {
            let next = current
                .chunks(2)
                .map(|pair| hash_pair(&pair[0], &pair[1]))
                .collect::<Vec<_>>();
            layers.push(next.clone());
            current = next;
        }

        let root = current[0];
        Self { leaves, layers, root }
    }

    /// Generate an inclusion proof for the leaf at `index`.
    /// Returns the sibling hashes from leaf to root and the index path bits.
    pub fn prove(&self, index: usize) -> MerkleProof {
        let mut siblings = Vec::new();
        let mut path_bits = Vec::new();
        let mut idx = index;

        for layer in &self.layers[..self.layers.len() - 1] {
            let sibling_idx = if idx % 2 == 0 { idx + 1 } else { idx - 1 };
            siblings.push(layer[sibling_idx]);
            path_bits.push(idx % 2 == 0); // true = we are the left child
            idx /= 2;
        }

        MerkleProof { siblings, path_bits }
    }
}

#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub siblings: Vec<Hash>,
    pub path_bits: Vec<bool>,
}

impl MerkleProof {
    /// Recompute the root from a leaf hash and this proof.
    pub fn verify(&self, leaf: Hash, expected_root: Hash) -> bool {
        let mut current = leaf;
        for (sibling, is_left) in self.siblings.iter().zip(self.path_bits.iter()) {
            current = if *is_left {
                hash_pair(&current, sibling)
            } else {
                hash_pair(sibling, &current)
            };
        }
        current == expected_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UserBalance;

    fn users(n: u64) -> Vec<UserBalance> {
        (0..n).map(|i| UserBalance { id: i, balance: (i + 1) * 100 }).collect()
    }

    #[test]
    fn single_leaf_inclusion_proof_verifies() {
        let u = vec![UserBalance { id: 0, balance: 500 }];
        let tree = MerkleTree::build(&u);
        let proof = tree.prove(0);
        assert!(proof.verify(hash_leaf(&u[0]), tree.root));
    }

    #[test]
    fn inclusion_proof_verifies_for_all_leaves() {
        let tree = MerkleTree::build(&users(8));
        for (i, user) in users(8).iter().enumerate() {
            let proof = tree.prove(i);
            let leaf = hash_leaf(user);
            assert!(proof.verify(leaf, tree.root), "proof failed for leaf {i}");
        }
    }

    #[test]
    fn wrong_leaf_fails_verification() {
        let tree = MerkleTree::build(&users(4));
        let proof = tree.prove(0);
        let wrong_leaf = hash_leaf(&UserBalance { id: 99, balance: 99 });
        assert!(!proof.verify(wrong_leaf, tree.root));
    }

    #[test]
    fn non_power_of_two_input() {
        let tree = MerkleTree::build(&users(5));
        for (i, user) in users(5).iter().enumerate() {
            let proof = tree.prove(i);
            assert!(proof.verify(hash_leaf(user), tree.root));
        }
    }
}
