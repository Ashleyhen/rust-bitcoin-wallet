use bitcoin::secp256k1::constants;

pub(crate) type Seed = [u8; constants::SECRET_KEY_SIZE];
pub const NETWORK: bitcoin::Network = bitcoin::Network::Regtest;
pub const TIP: u64 = 400;
