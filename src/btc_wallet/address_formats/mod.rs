use bitcoin::{util::bip32::ExtendedPubKey, Address};

use super::wallet_methods::ClientWallet;

pub mod p2tr_addr_fmt;
pub mod p2wpkh_addr_fmt;

pub trait AddressSchema {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> Address;

    fn wallet_purpose(&self) -> u32;

    fn to_wallet(&self) -> ClientWallet;

    fn get_ext_pub_key(&self) -> ExtendedPubKey {
        let cw = self.to_wallet();
        return self
            .to_wallet()
            .create_wallet(self.wallet_purpose(), cw.recieve, cw.change)
            .0;
    }
}
