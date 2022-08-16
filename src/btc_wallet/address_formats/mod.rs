use std::str::FromStr;

use bdk::{keys::GeneratableKey, KeychainKind};
use bitcoin::{
    secp256k1::{rand::rngs::OsRng, All, Secp256k1, SecretKey},
    util::{
        bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey},
        taproot::TapBranchHash,
    },
    Address, KeyPair, XOnlyPublicKey,
};

use super::{
    constants::{Seed, NETWORK},
    wallet_methods::ClientWallet,
};

pub mod p2tr_addr_fmt;
pub mod p2wpkh_addr_fmt;

type AddressMapping<'a> = Box<(dyn Fn(ExtendedPubKey) -> Address + 'a)>;

pub fn generate_key_pair(seed: Option<String>) -> ExtendedPrivKey {
    return ExtendedPrivKey::new_master(
        NETWORK,
        &seed
            .map(|f| SecretKey::from_str(&f).unwrap())
            .unwrap_or(SecretKey::new(&mut OsRng::new().unwrap()))
            .secret_bytes(),
    )
    .unwrap();
}

pub fn map_tr_address<'a>(
    secp: &'a Secp256k1<All>,
    merkle_root: Option<TapBranchHash>,
) -> AddressMapping {
    return Box::new(move |extended_pub: ExtendedPubKey| {
        return Address::p2tr(secp, extended_pub.to_x_only_pub(), merkle_root, NETWORK);
    });
}

pub fn map_wpkh_address<'a>() -> AddressMapping<'a> {
    return Box::new(move |extended_pub: ExtendedPubKey| {
        return Address::p2wpkh(&extended_pub.to_pub(), NETWORK).unwrap();
    });
}

pub fn derive_derivation_path(
    recieve: u32,
    index: u32,
    addr:AddressMapping
) -> Box<dyn Fn(ExtendedPrivKey, u32) -> ExtendedPrivKey> {
    let secp = Secp256k1::new();
    return Box::new(move |extended_priv_key, purpose| {
        let keychain = KeychainKind::External;
        let path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(purpose).unwrap(), // purpose
            ChildNumber::from_hardened_idx(recieve).unwrap(), // first recieve
            ChildNumber::from_hardened_idx(0).unwrap(),       // second recieve
            ChildNumber::from_normal_idx(keychain as u32).unwrap(),
            ChildNumber::from_normal_idx(index).unwrap(),
        ]);
        return addr(extended_priv_key.derive_priv(&secp, &path).unwrap());
        
    });
}

pub trait AddressSchema {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> Address;

    fn wallet_purpose(&self) -> u32;

    fn to_wallet(&self) -> ClientWallet;

    fn get_ext_pub_key(&self) -> ExtendedPubKey {
        return self.to_wallet().derive_pub_k(self.get_ext_prv_k());
    }

    fn get_derivation_p(&self) -> DerivationPath {
        let cw = self.to_wallet();
        return self.to_wallet().derive_derivation_path(
            self.wallet_purpose(),
            cw.recieve,
            cw.change,
        );
    }

    fn get_ext_prv_k(&self) -> ExtendedPrivKey {
        return self.to_wallet().derive_ext_priv_k(&self.get_derivation_p());
    }
}
