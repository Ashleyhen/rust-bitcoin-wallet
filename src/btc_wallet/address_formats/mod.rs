use bitcoin::{util::bip32::{ExtendedPubKey, ExtendedPrivKey, DerivationPath}, Address};

use super::wallet_methods::ClientWallet;

pub mod p2tr_addr_fmt;
pub mod p2wpkh_addr_fmt;

pub trait AddressSchema {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> Address;

    fn wallet_purpose(&self) -> u32;

    fn to_wallet(&self) -> ClientWallet;

    fn get_ext_pub_key(&self) -> ExtendedPubKey {
        return self.to_wallet().derive_pub_k(self.get_ext_prv_k());
    }

    fn get_derivation_p(&self)->DerivationPath{
        let cw = self.to_wallet();
        return self.to_wallet().derive_derivation_path(self.wallet_purpose(), cw.recieve,cw.change);
    }

    fn get_ext_prv_k(&self)->ExtendedPrivKey{
        return self.to_wallet().derive_ext_priv_k(&self.get_derivation_p());
    }

}
