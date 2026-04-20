use clap::Parser;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::fs;
use types::{ReserveBalance, UserBalance};

#[derive(Parser)]
#[command(about = "Generate mock exchange data for the zk-solvency protocol")]
struct Args {
    /// Number of user accounts (liabilities)
    #[arg(short, long, default_value_t = 100)]
    users: u64,

    /// Number of reserve accounts (assets)
    #[arg(short, long, default_value_t = 5)]
    reserves: u64,

    /// Random seed for reproducibility
    #[arg(short, long, default_value_t = 42)]
    seed: u64,

    /// Surplus ratio: assets = liabilities * (1 + surplus). E.g. 0.2 = 20% overcollateralized
    #[arg(long, default_value_t = 0.2)]
    surplus: f64,

    /// Output directory for generated JSON files
    #[arg(short, long, default_value = "data")]
    output: String,
}

fn main() {
    let args = Args::parse();
    let mut rng = StdRng::seed_from_u64(args.seed);

    // Generate user balances (liabilities)
    let users: Vec<UserBalance> = (0..args.users)
        .map(|id| UserBalance {
            id,
            balance: rng.gen_range(100..10_000),
        })
        .collect();

    let total_liabilities: u64 = users.iter().map(|u| u.balance).sum();

    // Generate reserve balances such that total assets >= total liabilities * (1 + surplus)
    let total_assets_target = (total_liabilities as f64 * (1.0 + args.surplus)) as u64;
    let base_reserve = total_assets_target / args.reserves;
    let remainder = total_assets_target - base_reserve * args.reserves;

    let reserves: Vec<ReserveBalance> = (0..args.reserves)
        .map(|id| ReserveBalance {
            id,
            balance: base_reserve + if id == 0 { remainder } else { 0 },
        })
        .collect();

    let total_assets: u64 = reserves.iter().map(|r| r.balance).sum();

    fs::create_dir_all(&args.output).expect("failed to create output directory");

    let users_path = format!("{}/users.json", args.output);
    let reserves_path = format!("{}/reserves.json", args.output);

    fs::write(&users_path, serde_json::to_string_pretty(&users).unwrap())
        .expect("failed to write users.json");
    fs::write(&reserves_path, serde_json::to_string_pretty(&reserves).unwrap())
        .expect("failed to write reserves.json");

    println!("Generated {} users  → {}", args.users, users_path);
    println!("Generated {} reserves → {}", args.reserves, reserves_path);
    println!("Total liabilities : {total_liabilities}");
    println!("Total assets      : {total_assets}");
    println!("Surplus           : {}", total_assets - total_liabilities);
}
