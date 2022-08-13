use std::sync::Arc;

use bitcoin::{
    psbt::{Output, TapTree},
    schnorr::TweakedPublicKey,
    secp256k1::{ecdh::SharedSecret, SecretKey},
    util::{
        bip32::{DerivationPath, Fingerprint},
        taproot::{LeafVersion, TapLeafHash, TaprootBuilder},
    },
    Address, KeyPair, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};
use miniscript::ToPublicKey;

use crate::btc_wallet::{
    address_formats::{p2tr_addr_fmt::P2TR, AddressSchema},
    constants::{NETWORK, TIP},
    input_data::RpcCall,
};
pub trait ILock {
    fn create_tx(&self) -> dyn Fn(Vec<Output>, Vec<TxIn>, u64) -> Transaction;
}

pub struct OutputService(pub P2TR);
impl OutputService {
    pub fn insert_tap_key_origin<'a>(
        &'a self,
        scripts: &'a Vec<(u32, Script)>,
    ) -> Box<impl FnMut(&mut Output) + 'a> {
        return Box::new(move |output: &mut Output| {
            let value = scripts
                .clone()
                .iter()
                .map(|(_, s)| TapLeafHash::from_script(&s, LeafVersion::TapScript))
                .collect();
            output.tap_key_origins.insert(
                self.0.get_ext_pub_key().to_x_only_pub(),
                (value, (Fingerprint::default(), DerivationPath::default())),
            );
        });
    }

    pub fn new_tap_internal_key<'a>(
        &'a self,
        key: &'a SecretKey,
    ) -> Box<impl FnMut(&mut Output) + 'a> {
        Box::new(|output: &mut Output| {
            output.tap_internal_key =
                Some(KeyPair::from_secret_key(&self.0.to_wallet().secp, key.clone()).public_key())
        })
    }

    pub fn insert_tap_tree<'a>(
        scripts: &'a Vec<(u32, Script)>,
    ) -> Box<impl FnMut(&mut Output) + 'a> {
        return Box::new(move |output: &mut Output| {
            let internal = output
                .tap_internal_key
                .expect("missing expected internal key ");
            let tap_tweak = Address::p2tr_tweaked(
                TweakedPublicKey::dangerous_assume_tweaked(internal.clone()),
                NETWORK,
            )
            .script_pubkey();
            scripts.clone().push((0, tap_tweak));
            let builder = TaprootBuilder::with_huffman_tree(scripts.clone()).unwrap();
            output.tap_tree = Some(TapTree::from_builder(builder).unwrap());
        });
    }

    pub fn insert_witness<'a>(&'a self) -> Box<impl FnMut(&mut Output) + 'a> {
        return Box::new(move |output: &mut Output| {
            let internal_key = output
                .tap_internal_key
                .expect("missing expected internal key");
            let tap_tree = output.tap_tree.as_ref().expect("missing expected tap tree");
            let branch = tap_tree
                .to_builder()
                .finalize(&self.0.to_wallet().secp, internal_key)
                .unwrap()
                .merkle_root()
                .unwrap();
            output.witness_script = Some(Script::new_v1_p2tr(
                &self.0.get_client_wallet().secp,
                internal_key,
                Some(branch),
            ));
        });
    }

    pub fn new_shared_secret<'a>(
        &self,
        mut iter: impl Iterator<Item = &'a XOnlyPublicKey>,
    ) -> SecretKey {
        {
            match iter.next() {
                Some(x_only) => {
                    return SecretKey::from_slice(
                        &SharedSecret::new(
                            &x_only.to_public_key().inner,
                            &self.new_shared_secret(iter),
                        )
                        .secret_bytes(),
                    )
                    .unwrap()
                }

                None => SecretKey::from_slice(&self.0.get_client_wallet().seed).unwrap(),
            }
        }
    }

    fn lock_key<'a>(&self, func_list_list: Vec<Vec<Box<dyn FnMut(&mut Output)>>>) -> Vec<Output> {
        let mut output_vec = Vec::<Output>::new();
        for func_list in func_list_list {
            let mut output = Output::default();
            for mut func in func_list {
                func(&mut output);
            }
            output_vec.push(output);
        }
        return output_vec;
    }
}
