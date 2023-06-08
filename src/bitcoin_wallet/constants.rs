use bitcoin::secp256k1::{All, Secp256k1};

use super::input_data::regtest_rpc::RegtestRpc;

pub const NETWORK: bitcoin::Network = bitcoin::Network::Regtest;
pub const TIP: u64 = 2000;
pub const MINE: u8 = 0;
pub const SEED: &str = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";
pub const LOG: bool = true;
pub fn secp() -> Secp256k1<All> {
    return Secp256k1::new();
}
