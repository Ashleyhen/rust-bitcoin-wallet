use bitcoin::{util::bip32::ExtendedPubKey, Address};

use super::{address_formats, wallet_methods::ClientWallet};

pub mod p2tr_addr_fmt;
pub mod p2wpkh_addr_fmt;

pub trait AddressSchema {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> Address;
    fn wallet_purpose(&self) -> u32;
    fn to_wallet(&self) -> ClientWallet;
}
