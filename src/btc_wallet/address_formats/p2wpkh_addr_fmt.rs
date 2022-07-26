use bitcoin::{util::bip32::ExtendedPubKey, Address};
use miniscript::ToPublicKey;

use crate::btc_wallet::{constants::NETWORK, wallet_methods::ClientWallet};

use super::AddressSchema;

#[derive(Clone)]
pub struct P2WPKH(ClientWallet);

impl AddressSchema for P2WPKH {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> Address {
        return Address::p2wpkh(&recieve.public_key.to_public_key(), NETWORK).unwrap();
    }

    fn to_wallet(&self) -> ClientWallet {
        return self.0.clone();
    }

    fn wallet_purpose(&self) -> u32 {
        return 84;
    }
}
impl P2WPKH {
    pub fn get_client_wallet(&self) -> ClientWallet {
        return self.0.clone();
    }

    pub fn new(secret_seed: Option<String>, recieve: u32, change: u32) -> Self {
        return P2WPKH(ClientWallet::new(secret_seed, recieve, change));
    }
}
