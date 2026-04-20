pub mod merkle;

use serde::{Deserialize, Serialize};

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
