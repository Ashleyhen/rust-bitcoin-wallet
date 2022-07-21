use bitcoin::{util::bip32::ExtendedPubKey, Address};

use crate::btc_wallet::{
    constants::NETWORK,
    spending_path::{p2tr_key_path::P2TR, p2tr_multisig_path::P2trMultisig},
    wallet_methods::ClientWallet,
};

use super::AddressSchema;

impl AddressSchema for P2TR {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> Address {
        return Address::p2tr(
            &self.get_client_wallet().secp,
            recieve.to_x_only_pub(),
            None,
            NETWORK,
        );
    }

    fn wallet_purpose(&self) -> u32 {
        return 341;
    }

    fn to_wallet(&self) -> ClientWallet {
        return self.client_wallet.clone();
    }
}

impl AddressSchema for P2trMultisig {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> Address {
        return Address::p2tr(
            &self.get_client_wallet().secp,
            recieve.to_x_only_pub(),
            None,
            NETWORK,
        );
    }

    fn wallet_purpose(&self) -> u32 {
        return 341;
    }

    fn to_wallet(&self) -> ClientWallet {
        return self.client_wallet.clone();
    }
}
