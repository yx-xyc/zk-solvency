use std::time::{Duration, Instant};
use types::{UserBalance, merkle::{hash_leaf, MerkleTree}};

const NS: &[usize] = &[10, 100, 500, 1000, 5000];

fn make_users(n: usize) -> Vec<UserBalance> {
    (0..n as u64).map(|i| UserBalance { id: i, balance: (i + 1) * 100 }).collect()
}

fn repeat<F: FnMut()>(iters: u32, mut f: F) -> Duration {
    let start = Instant::now();
    for _ in 0..iters { f(); }
    start.elapsed() / iters
}

fn main() {
    println!("=== Merkle Tree Operations (averaged over multiple iterations) ===\n");
    println!("{:<8} {:>14} {:>14} {:>14} {:>18}",
        "N", "build (ms)", "prove (µs)", "verify (µs)", "inclusion (ms)");
    println!("{}", "-".repeat(72));

    for &n in NS {
        let users = make_users(n);
        let iters = if n <= 100 { 200 } else if n <= 1000 { 50 } else { 10 };

        let build_avg = repeat(iters, || { MerkleTree::build(&users); });

        let tree  = MerkleTree::build(&users);
        let prove_avg = repeat(500, || { tree.prove(0); });

        let proof = tree.prove(0);
        let leaf  = hash_leaf(&users[0]);
        let root  = tree.root;
        let verify_avg = repeat(500, || { proof.verify(leaf, root); });

        let inclusion_avg = repeat(iters, || {
            let t = MerkleTree::build(&users);
            let p = t.prove(0);
            let l = hash_leaf(&users[0]);
            p.verify(l, t.root);
        });

        println!("{:<8} {:>14.3} {:>14.3} {:>14.3} {:>18.3}",
            n,
            build_avg.as_secs_f64()     * 1_000.0,
            prove_avg.as_secs_f64()     * 1_000_000.0,
            verify_avg.as_secs_f64()    * 1_000_000.0,
            inclusion_avg.as_secs_f64() * 1_000.0,
        );
    }

    println!("\nNote: 'inclusion' = build + prove + verify (full round-trip)");
}
