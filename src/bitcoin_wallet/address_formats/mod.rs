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
type DeriveKeyMapping = Box<(dyn Fn(u32, u32) -> ExtendedPrivKey)>;
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

// pub fn map_seeds_to_scripts<'a>(
//     seed:Option<String>,
//     secp: &'a Secp256k1<All>,
//     merkle_root: Option<TapBranchHash>,
//     purpose: u32
//     )->(){
// let prv=generate_key_pair(seed);
// let derive=derive_derivation_path(prv,purpose);
//         let a= Box::new(move |derive_key_fn:ExtendedPrivKey| {
//             Address::p2tr(secp, derive_key_fn.to_keypair(&secp).public_key(), merkle_root, NETWORK);
//         });
// }

// pub fn map_wpkh_address<'a>() -> AddressMapping<'a> {
//     return Box::new(move |extended_pub: ExtendedPubKey| {
//         return Address::p2wpkh(&extended_pub.to_pub(), NETWORK).unwrap();
//     });
// }

pub fn map_seeds_to_scripts<'a>(
    seed: Option<String>,
    secp:&'a Secp256k1<All>,
    purpose: u32, 
    map_to_addr:AddressMapping<'a>
) -> Box<dyn Fn(u32, u32) -> Address + 'a> {
    let extended_priv_key = generate_key_pair(seed);
    return Box::new(move |recieve, index| {
        return map_to_addr(ExtendedPubKey::from_priv(&secp,&extended_priv_key)
        .derive_pub(&secp, &get_derivation_p(purpose,recieve,index)).unwrap());
    });
}

pub fn get_derivation_p(purpose: u32,recieve:u32,index:u32)->DerivationPath{
      let keychain = KeychainKind::External;
    let path = DerivationPath::from(vec![
        ChildNumber::from_hardened_idx(purpose).unwrap(), // purpose
        ChildNumber::from_hardened_idx(recieve).unwrap(), // first recieve
        ChildNumber::from_hardened_idx(0).unwrap(),       // second recieve
        ChildNumber::from_normal_idx(keychain as u32).unwrap(),
        ChildNumber::from_normal_idx(index).unwrap(),
    ]);
    return path; 
}

pub fn derive_derivation_path(
    extended_priv_key: ExtendedPrivKey,
    purpose: u32,
) -> DeriveKeyMapping {
    let secp = Secp256k1::new();
    return Box::new(move |recieve, index| {
        let keychain = KeychainKind::External;
        let path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(purpose).unwrap(), // purpose
            ChildNumber::from_hardened_idx(recieve).unwrap(), // first recieve
            ChildNumber::from_hardened_idx(0).unwrap(),       // second recieve
            ChildNumber::from_normal_idx(keychain as u32).unwrap(),
            ChildNumber::from_normal_idx(index).unwrap(),
        ]);
        return extended_priv_key.derive_priv(&secp, &path).unwrap();
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
