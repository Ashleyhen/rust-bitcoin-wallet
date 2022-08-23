use std::{collections::BTreeMap, ops::Add};

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    hashes::hex::FromHex,
    psbt::{Input, Output, PartiallySignedTransaction, TapTree},
    schnorr::{TapTweak, TweakedPublicKey},
    secp256k1::{ecdh::SharedSecret, All, Message, Secp256k1, SecretKey},
    util::{
        bip32::{DerivationPath, ExtendedPrivKey, Fingerprint, KeySource},
        sighash::{Prevouts, SighashCache},
        taproot::{
            ControlBlock, LeafVersion, TapBranchHash, TapLeafHash, TaprootBuilder,
            TaprootMerkleBranch, TaprootSpendInfo,
        },
    },
    Address, KeyPair, SchnorrSig, SchnorrSighashType, Script, Transaction, TxIn, WitnessMerkleNode,
    XOnlyPublicKey,
};
use miniscript::ToPublicKey;

pub mod input_service;
pub mod output_service;
pub mod psbt_factory;

use super::{
    constants::{Seed, NETWORK},
    spending_path::p2tr_key_path::P2tr,
};

impl P2tr {
    pub fn insert_tap_key_origin<'a>(
        &'a self,
        xonly: XOnlyPublicKey,
        scripts: &'a Vec<(u32, Script)>,
    ) -> Box<impl FnMut(&mut Output) + 'a> {
        return Box::new(move |output: &mut Output| {
            let value = scripts
                .clone()
                .iter()
                .map(|(_, s)| TapLeafHash::from_script(&s, LeafVersion::TapScript))
                .collect();
            output.tap_key_origins.insert(
                xonly,
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
                Some(KeyPair::from_secret_key(&self.secp, key.clone()).public_key())
        })
    }

    pub fn insert_tap_tree<'a>(scripts: Vec<(u32, Script)>) -> Box<impl FnMut(&mut Output) + 'a> {
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
                .finalize(&self.secp, internal_key)
                .unwrap()
                .merkle_root()
                .unwrap();
            output.witness_script =
                Some(Script::new_v1_p2tr(&self.secp, internal_key, Some(branch)));
        });
    }

    pub fn new_shared_secret<'a>(
        &self,
        mut iter: impl Iterator<Item = &'a XOnlyPublicKey>,
        seed: Seed,
    ) -> SecretKey {
        {
            match iter.next() {
                Some(x_only) => {
                    return SecretKey::from_slice(
                        &SharedSecret::new(
                            &x_only.to_public_key().inner,
                            &self.new_shared_secret(iter, seed),
                        )
                        .secret_bytes(),
                    )
                    .unwrap()
                }

                None => SecretKey::from_slice(&seed).unwrap(),
            }
        }
    }

    pub fn insert_givens<'a>(&'a self) -> Box<impl FnOnce(&Output, &mut Input) + 'a> {
        return Box::new(move |output: &Output, input: &mut Input| {
            let out = output.clone();
            input.witness_script = out.witness_script;
            input.tap_internal_key = out.tap_internal_key;
            input.tap_key_origins = out.tap_key_origins;
        });
    }

    pub fn insert_control_block<'a>(
        &'a self,
        script: &'a Script,
        x_only: XOnlyPublicKey,
    ) -> Box<impl FnOnce(&Output, &mut Input) + 'a> {
        let mut err_msg = "missing expected scripts for x_only ".to_string();
        err_msg.push_str(&x_only.to_string());

        return Box::new(move |output: &Output, input: &mut Input| {
            let internal_key = input.tap_internal_key.expect("msg");
            let spending_info = TaprootSpendInfo::with_huffman_tree(
                &self.secp,
                internal_key,
                output
                    .tap_tree
                    .as_ref()
                    .unwrap()
                    .script_leaves()
                    .map(|s| (u32::from(s.depth()), s.script().clone()))
                    .collect::<Vec<(u32, Script)>>(),
            )
            .unwrap();

            let control = spending_info.control_block(&(script.clone(), LeafVersion::TapScript));
            control.as_ref().unwrap().verify_taproot_commitment(
                &self.secp,
                spending_info.output_key().to_inner(),
                script,
            );
            input
                .tap_scripts
                .insert(control.unwrap(), (script.clone(), LeafVersion::TapScript));
        });
    }

    pub fn sign_tapleaf<'a>(
        &'a self,
        tx: &'a Transaction,
        input_index: usize,
        xonly: XOnlyPublicKey,
        prv: ExtendedPrivKey,
    ) -> Box<impl FnOnce(&Output, &mut Input) + 'a> {
        return Box::new(move |output: &Output, input: &mut Input| {
            let tap_leaf_hash = output.tap_key_origins.get(&xonly).unwrap().0.clone();
            let tap_sig_hash = SighashCache::new(tx)
                .taproot_script_spend_signature_hash(
                    input_index,
                    &Prevouts::All(&tx.output),
                    tap_leaf_hash[0],
                    SchnorrSighashType::AllPlusAnyoneCanPay,
                )
                .unwrap();
            let tweaked_pair = prv
                .to_keypair(&self.secp)
                .tap_tweak(&self.secp, input.tap_merkle_root);
            let sig = self.secp.sign_schnorr(
                &Message::from_slice(&tap_sig_hash).unwrap(),
                &tweaked_pair.into_inner(),
            );
            let schnorrsig = SchnorrSig {
                sig,
                hash_ty: SchnorrSighashType::AllPlusAnyoneCanPay,
            };
            input
                .tap_script_sigs
                .insert((xonly, tap_leaf_hash[0]), schnorrsig);
        });
    }
}
