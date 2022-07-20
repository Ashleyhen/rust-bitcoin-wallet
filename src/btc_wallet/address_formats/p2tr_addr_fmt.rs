use std::str::FromStr;

use bitcoin::{
    psbt::Input, util::bip32::ExtendedPrivKey, Address, Transaction, TxOut, XOnlyPublicKey,
};
use miniscript::ToPublicKey;

use crate::btc_wallet::{
    constants::NETWORK, spending_path::p2tr_key_path::P2TR, wallet_methods::ClientWallet,
};

use super::AddressSchema;

impl AddressSchema for P2TR {
    fn map_ext_keys(&self, recieve: &bitcoin::util::bip32::ExtendedPubKey) -> bitcoin::Address {
        return Address::p2tr(
            &self.get_client_wallet().secp,
            recieve.to_x_only_pub(),
            None,
            NETWORK,
        );
    }

    fn to_wallet(&self) -> ClientWallet {
        return self.get_client_wallet().clone();
    }

    fn wallet_purpose(&self) -> u32 {
        return 341;
    }
}
