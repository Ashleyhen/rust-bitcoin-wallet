use std::str::FromStr;

use bitcoin::{
    psbt::{Output, TapTree},
    secp256k1::{ecdh::SharedSecret, All, Parity, Secp256k1, SecretKey},
    util::{
        bip32::{DerivationPath, Fingerprint},
        taproot::{LeafVersion, TapLeafHash, TaprootBuilder},
    },
    Script, XOnlyPublicKey,
};

use crate::bitcoin_wallet::spending_path::p2tr_key_path::P2tr;

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
