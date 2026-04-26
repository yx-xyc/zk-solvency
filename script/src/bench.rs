use sp1_sdk::{Elf, ProveRequest, Prover, ProverClient, SP1Stdin};
use types::{ReserveBalance, UserBalance};

const SOLVENCY_ELF: &[u8] = include_bytes!(
    "../../target/elf-compilation/riscv64im-succinct-zkvm-elf/release/solvency-program"
);

const NS: &[usize] = &[10, 100, 500, 1000, 5000];

fn make_inputs(n: usize) -> (Vec<UserBalance>, Vec<ReserveBalance>) {
    let users: Vec<UserBalance> =
        (0..n as u64).map(|i| UserBalance { id: i, balance: (i + 1) * 100 }).collect();
    let total: u64 = users.iter().map(|u| u.balance).sum();
    let reserves = vec![ReserveBalance { id: 0, balance: total + 1 }];
    (users, reserves)
}

#[tokio::main]
async fn main() {
    let client = ProverClient::from_env().await;
    let pk = client.setup(Elf::Static(SOLVENCY_ELF)).await.unwrap();

    println!("SP1 mock proof generation time (SP1_PROVER=mock)\n");
    println!("{:<8} {:>20}", "N", "time (s)");
    println!("{}", "-".repeat(30));

    for &n in NS {
        let (users, reserves) = make_inputs(n);

        let mut stdin = SP1Stdin::new();
        stdin.write(&users);
        stdin.write(&reserves);

        let start = std::time::Instant::now();
        client.prove(&pk, stdin).groth16().await.unwrap();
        let elapsed = start.elapsed();

        println!("{:<8} {:>20.3}", n, elapsed.as_secs_f64());
    }
}
