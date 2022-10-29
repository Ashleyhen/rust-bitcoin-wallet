use bitcoin::secp256k1::constants;

pub(crate) type Seed = [u8; constants::SECRET_KEY_SIZE];
pub const NETWORK: bitcoin::Network = bitcoin::Network::Regtest;
pub const TIP: u64 = 2000;
pub const SEED: &str = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";
