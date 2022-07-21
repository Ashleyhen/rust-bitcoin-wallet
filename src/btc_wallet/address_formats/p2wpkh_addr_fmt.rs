use bitcoin::{util::bip32::ExtendedPubKey, Address};
use miniscript::ToPublicKey;

use crate::btc_wallet::{
    constants::NETWORK, spending_path::p2wpkh_script_path::P2PWKh, wallet_methods::ClientWallet,
};

use super::AddressSchema;

impl AddressSchema for P2PWKh {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> Address {
        return Address::p2wpkh(&recieve.public_key.to_public_key(), NETWORK).unwrap();
    }

    fn to_wallet(&self) -> ClientWallet {
        return self.client_wallet.clone();
    }

    fn wallet_purpose(&self) -> u32 {
        return 84;
    }
}
