use std::{hash::BuildHasher, str::FromStr};

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    psbt::{Output, TapTree},
    secp256k1::{ecdh::SharedSecret, All, Parity, PublicKey, Secp256k1, SecretKey},
    util::{
        bip32::{DerivationPath, Fingerprint, KeySource},
        taproot::{LeafVersion, TapLeafHash, TaprootBuilder},
    },
    Address, Script, XOnlyPublicKey,
};
use bitcoin_hashes::{sha256, Hash, HashEngine};

use crate::bitcoin_wallet::{
    constants::NETWORK,
    scripts::p2wsh_multi_sig,
    spending_path::{p2tr_key_path::P2tr, p2wpkh_script_path::P2wpkh, p2wsh_path::P2wsh},
};

pub struct OutputService(pub P2tr);

pub fn insert_tap_key_origin<'a>(
    scripts: Vec<(u32, Script)>,
    xonly: &'a XOnlyPublicKey,
) -> Box<impl FnMut(&mut Output) + 'a> {
    return Box::new(move |output: &mut Output| {
        let value = scripts
            .clone()
            .iter()
            .map(|(_, s)| TapLeafHash::from_script(&s, LeafVersion::TapScript))
            .collect();
        output.tap_key_origins.insert(
            xonly.clone(),
            (value, (Fingerprint::default(), DerivationPath::default())),
        );
    });
}

pub fn new_tap_internal_key<'a>(xinternal: &'a XOnlyPublicKey) -> Box<impl Fn(&mut Output) + 'a> {
    return Box::new(move |output: &mut Output| output.tap_internal_key = Some(xinternal.clone()));
}

pub fn insert_tap_tree<'a>(scripts: Vec<(u32, Script)>) -> Box<impl Fn(&mut Output) + 'a> {
    return Box::new(move |output: &mut Output| {
        let builder = TaprootBuilder::with_huffman_tree(scripts.clone()).unwrap();
        output.tap_tree = Some(TapTree::try_from(builder).unwrap());
    });
}

pub fn new_witness_pub_k<'a>(witness: Script) -> Box<impl Fn(&mut Output) + 'a> {
    return Box::new(move |output: &mut Output| output.witness_script = Some(witness.clone()));
}

pub fn insert_tree_witness<'a>(secp: &'a Secp256k1<All>) -> Box<impl Fn(&mut Output) + 'a> {
    return Box::new(move |output: &mut Output| {
        let internal_key = output
            .tap_internal_key
            .expect("missing expected internal key");
        let tap_tree = output.tap_tree.as_ref().expect("missing expected tap tree");
        let branch = tap_tree
            .to_builder()
            .finalize(&secp, internal_key)
            .unwrap()
            .merkle_root()
            .unwrap();
        output.witness_script = Some(Script::new_v1_p2tr(&secp, internal_key, Some(branch)));
    });
}

pub fn segwit_v0_add_key<'a>(
    key: &'a PublicKey,
    source: &'a KeySource,
) -> Box<impl Fn(&mut Output) + 'a> {
    return Box::new(move |output: &mut Output| {
        output.bip32_derivation.insert(key.clone(), source.clone());
    });
}

pub fn segwit_v0_agg_witness<'a>() -> Box<impl Fn(&mut Output) + 'a> {
    return Box::new(move |output: &mut Output| {
        let pub_keys = output
            .bip32_derivation
            .iter()
            .map(|(pub_k, _)| pub_k.clone())
            .collect::<Vec<PublicKey>>();
        let script = P2wsh::witness_program_fmt(p2wsh_multi_sig(&pub_keys));
        output.witness_script = Some(script);
    });
}

fn new_shared_secret<'a>(
    mut iter: impl Iterator<Item = &'a XOnlyPublicKey>,
    secret: String,
) -> SecretKey {
    match iter.next() {
        Some(x_only) => {
            return SecretKey::from_slice(
                &SharedSecret::new(
                    &x_only.public_key(Parity::Even),
                    &new_shared_secret(iter, secret),
                )
                .secret_bytes(),
            )
            .unwrap()
        }

        None => SecretKey::from_str(&secret).unwrap(),
    }
}
