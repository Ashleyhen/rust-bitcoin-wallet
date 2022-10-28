use std::str::FromStr;

use bitcoin::{
    secp256k1::{rand::rngs::OsRng, All, Scalar, Secp256k1, SecretKey},
    util::{
        bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey},
        taproot::TapBranchHash,
    },
    Address, KeyPair, XOnlyPublicKey,
};

use super::constants::{Seed, NETWORK};

type AddressMapping = Box<(dyn Fn(&Secp256k1<All>, ExtendedPubKey) -> Address)>;
type DeriveKeyMapping = Box<(dyn Fn(u32, u32) -> ExtendedPrivKey)>;

pub fn generate_key_pair(seed: Option<String>) -> ExtendedPrivKey {
    return ExtendedPrivKey::new_master(
        NETWORK,
        &seed
            .map(|f| SecretKey::from_str(&f).unwrap())
            .unwrap_or(SecretKey::from_slice(&Scalar::random().to_be_bytes()).unwrap())
            .secret_bytes(),
    )
    .unwrap();
}

pub fn map_tr_address(merkle_root: Option<TapBranchHash>) -> AddressMapping {
    return Box::new(move |secp: &Secp256k1<All>, extended_pub: ExtendedPubKey| {
        return Address::p2tr(secp, extended_pub.to_x_only_pub(), merkle_root, NETWORK);
    });
}

pub fn map_seeds_to_scripts<'a>(
    seed: Option<String>,
    secp: &'a Secp256k1<All>,
    purpose: u32,
    map_to_addr: AddressMapping,
) -> Box<dyn Fn(u32, u32) -> Address + 'a> {
    let extended_priv_key = generate_key_pair(seed);
    return Box::new(move |recieve, index| {
        return map_to_addr(
            secp,
            ExtendedPubKey::from_priv(
                &secp,
                &extended_priv_key
                    .derive_priv(&secp, &get_derivation_p(purpose, recieve, index))
                    .unwrap(),
            ),
        );
    });
}

pub fn get_derivation_p(purpose: u32, recieve: u32, index: u32) -> DerivationPath {
    let keychain = 1;
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
        return extended_priv_key
            .derive_priv(&secp, &get_derivation_p(purpose, recieve, index))
            .unwrap();
    });
}
