use std::{collections::BTreeMap, str::FromStr};

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    hashes::{sha256, Hash, hex::FromHex},
    psbt::{Input, Output, PartiallySignedTransaction, TapTree},
    schnorr::TapTweak,
    secp256k1::{ecdh::SharedSecret, All, Parity, Secp256k1, SecretKey, Message},
    util::{
        bip32::{DerivationPath, Fingerprint},
        taproot::{
            ControlBlock, LeafVersion, TapBranchHash, TapLeafHash, TapLeafTag, TaprootBuilder,
            TaprootMerkleBranch, TaprootSpendInfo,
        }, bip143::SigHashCache, sighash::{SighashCache, Prevouts},
    },
    KeyPair, Script, Transaction, TxIn, TxOut, XOnlyPublicKey, SchnorrSig, Address,
};
use miniscript::ToPublicKey;

use crate::btc_wallet::{
    address_formats::{p2tr_addr_fmt::P2TR, AddressSchema},
    constants::{Seed, TIP},
};

use super::Vault;

pub struct MultiSigPath<'a, 'b, 's> {
    pub p2tr: &'a P2TR,
    optional_psbt: Option<&'b PartiallySignedTransaction>,
    script: &'s Script,
    shared_secret: Option<&'b String>,
}

impl<'a, 'b, 's> Vault for MultiSigPath<'a, 'b, 's> {
    fn create_tx(
        &self,
        output_list: &Vec<Output>,
        tx_in: Vec<TxIn>,
        total: u64,
    ) -> bitcoin::Transaction {
        // let outputs=self.output_list.map(|o|o.outputs.clone()).unwrap_or(output_list.to_vec());
        // TODO fix filter
        let value = self
            .optional_psbt
            .map(|f| {
                f.clone()
                    .extract_tx()
                    .output
                    .iter()
                    .map(|a| a.value)
                    .sum::<u64>()
            })
            .unwrap_or_else(|| total - TIP);

        let tweak_pub_k = output_list
            .iter()
            .map(|out_put| {
                if (output_list[0].tap_key_origins.len() == 1) {
                    return self.script.clone();
                }
                let secp = self.p2tr.to_wallet().secp;
                let tap_leaf_hash: Vec<TapLeafHash> = out_put
                    .tap_key_origins
                    .values()
                    .into_iter()
                    .flat_map(|f| f.0.clone())
                    .collect();

                let branch = TapBranchHash::from_node_hashes(
                    sha256::Hash::from_inner(tap_leaf_hash[0].into_inner()),
                    sha256::Hash::from_inner(tap_leaf_hash[1].into_inner()),
                );
                let script =
                    Script::new_v1_p2tr(&secp, out_put.tap_internal_key.unwrap(), Some(branch));

                dbg!(script.clone());
                return script;
            })
            .collect::<Vec<Script>>();
        return Transaction {
            version: 2,
            lock_time: 0,
            input: tx_in,
            output: vec![TxOut {
                value,
                script_pubkey: tweak_pub_k[0].clone(),
            }],
        };
    }

    fn lock_key(&self) -> Vec<Output> {
        let secp = self.p2tr.to_wallet().secp;
        let output_list = self
            .optional_psbt
            .map(|f| f.outputs.clone())
            .unwrap_or_else(|| vec![Output::default()]);

        return output_list
            .iter()
            .map(|output| {
                let mut tap_key_origins = output.tap_key_origins.clone();

                let tap_leaf_hash =
                    vec![(TapLeafHash::from_script(self.script, LeafVersion::TapScript))];

                tap_key_origins.insert(
                    self.p2tr.get_ext_pub_key().to_x_only_pub(),
                    (
                        tap_leaf_hash,
                        (Fingerprint::default(), DerivationPath::default()),
                    ),
                );
                let mut result = output.clone();

                let shared_secret = self
                    .shared_secret
                    .map(|sec| SecretKey::from_str(sec).unwrap())
                    .unwrap_or(self.shared_secret(result.tap_key_origins.into_keys()));
                result.tap_internal_key =
                    Some(KeyPair::from_secret_key(&secp, shared_secret).public_key());

                dbg!(tap_key_origins.clone());

                result.tap_key_origins = tap_key_origins;
                return result;
            })
            .collect();

        // todo!()
    }

    fn unlock_key(&self, previous: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        let secp=self.p2tr.to_wallet().secp;
        let value: Vec<Input> = self
            .optional_psbt
            .map(|psbt| {
                dbg!(self.p2tr.get_ext_pub_key().public_key.to_x_only_pubkey());
                let output = psbt.outputs[0].clone();
                let leaf_list: Vec<TapLeafHash> = output
                    .tap_key_origins
                    .iter()
                    .filter(|p| {
                        p.0.ne(&self.p2tr.get_ext_pub_key().public_key.to_x_only_pubkey())
                    })
                    .flat_map(|(_, (tap_leaf, _))| tap_leaf.clone())
                    .collect();

                if psbt.outputs.is_empty() {
                    return psbt.inputs.clone();
                }
                if psbt.outputs[0].tap_key_origins.len() < 2 {
                    return psbt.inputs.clone();
                }

                let actual_control = ControlBlock {
                    leaf_version: LeafVersion::TapScript,
                    output_key_parity: Parity::Odd,
                    internal_key: output.tap_internal_key.unwrap().to_x_only_pubkey(),
                    merkle_branch: TaprootMerkleBranch::from_slice(&leaf_list[0]).unwrap(),
                };
                let preimage =
                Vec::from_hex("107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f").unwrap();
            dbg!(actual_control.clone());
            // ControlBlock {
            //     leaf_version: TapScript,
            //     output_key_parity: Odd,
            //     internal_key: XOnlyPublicKey(
            //         1cdd74b3a5ccb19d471c16556203caa3b144e8b230d0f5948d8d9c00d64405f3060dafe3d713cd37d877feee9add5f39874a214b19278d25e9534cd20014db1c,
            //     ),
            //     merkle_branch: TaprootMerkleBranch(
            //         [
            //             c81451874bd9ebd4b6fd4bba1f84cdfb533c532365d22a0a702205ff658b17c9,
            //         ],
            //     ),
            // }
                // let mut b_tree_map = BTreeMap::<ControlBlock, (Script, LeafVersion)>::default();

                



                let mut input = Input::default();
                // input.tap_scripts = b_tree_map;
dbg!(actual_control.clone());
// dbg!(self.script.clone().script_hash());
let actual_script="a8206c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd533388204edfcf9dfe6c0b5c83d1ab3f78d1b39a46ebac6798e08e19761f5ed89ec83c10ac";
;
                input.tap_scripts.insert(
                    actual_control.clone(),
                    (Script::from_hex(actual_script).unwrap(), LeafVersion::TapScript),
                );
                // input.sha256_preimages.insert(sha256::Hash::hash(&preimage), preimage);

                   

// Address::p2tr(&secp,output.tap_internal_key.unwrap(), merkle_root, bitcoin::Network::Testnet);

// Address::p2tr_tweaked(output_key, bitcoin::Network::Testnet);
let filter_script=Address::from_str("bcrt1p5kaqsuted66fldx256lh3en4h9z4uttxuagkwepqlqup6hw639gsm28t6c").unwrap().script_pubkey();
let tap_key_origin=psbt.outputs[0].tap_key_origins.clone();
let (tap_hash,_)=tap_key_origin.get(&self.p2tr.get_ext_pub_key().to_x_only_pub()).unwrap();
        let sig_hash = SighashCache::new(&mut current_tx.clone())
        .taproot_script_spend_signature_hash(0, &Prevouts::All(
            &previous[0].output.iter().filter(|p|p.script_pubkey.eq(&filter_script)).map(|f|f.clone()).collect::<Vec<TxOut>>()
        ), tap_hash[0], bitcoin::SchnorrSighashType::AllPlusAnyoneCanPay).unwrap();

        let msg=Message::from_slice(&sig_hash).unwrap();
let key_pair=KeyPair::from_secret_key(&secp,SecretKey::from_slice(&self.p2tr.get_client_wallet().seed).unwrap());
dbg!(key_pair.public_key());

        let tweaked_key_pair = key_pair 
        // .to_keypair(&secp)
            .tap_tweak(&secp, None)
            .into_inner();
            let sig =secp.sign_schnorr(&msg,&tweaked_key_pair);
            
            let schnorr_sig=SchnorrSig{ sig, hash_ty: bitcoin::SchnorrSighashType::AllPlusAnyoneCanPay };

            dbg!(tap_hash[0]);
                    // input.tap_script_sigs.insert((key_pair.public_key(),tap_hash[0]),schnorr_sig);
                // [src/wallet_test/tapscript_example_with_tap.rs:153] bob_script.clone() = Script(OP_SHA256 OP_PUSHBYTES_32 6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5333 OP_EQUALVERIFY OP_PUSHBYTES_32 4edfcf9dfe6c0b5c83d1ab3f78d1b39a46ebac6798e08e19761f5ed89ec83c10 OP_CHECKSIG)
// [src/wallet_test/tapscript_example_with_tap.rs:159] res = true
                let x_only=XOnlyPublicKey::from_slice(&previous[0].output[0].clone().script_pubkey[2..]).unwrap();
                dbg!(x_only);
                dbg!(self.script);
                let res=actual_control.verify_taproot_commitment(&secp, x_only, self.script);
// dbg!(self.script);
            // dbg!(previous[0].output[0]);
                // script = Script(OP_PUSHNUM_1 OP_PUSHBYTES_32 a5ba0871796eb49fb4caa6bf78e675b9455e2d66e751676420f8381d5dda8951)
                input.witness_utxo = Some(previous[0].output[0].clone());


                dbg!(TapLeafHash::from_script(
                    self.script,
                    LeafVersion::TapScript
                ));
                let res = actual_control.verify_taproot_commitment(
                    &self.p2tr.to_wallet().secp,
                    output.tap_internal_key.unwrap().to_x_only_pubkey(),
                    &self.script,
                );
                // dbg!(res);
                return vec![input];
            })
            .unwrap_or_else(|| vec![]);

        return value;
    }
}
impl<'a, 'b, 's> MultiSigPath<'a, 'b, 's> {
    pub fn new(
        p2tr: &'a P2TR,
        optional_psbt: Option<&'b PartiallySignedTransaction>,
        shared_secret: Option<&'b String>,
        script: &'s Script,
    ) -> Self {
        return MultiSigPath {
            p2tr,
            optional_psbt,
            shared_secret,
            script,
        };
    }

    pub fn shared_secret(&self, mut iter: impl Iterator<Item = XOnlyPublicKey>) -> SecretKey {
        {
            match iter.next() {
                Some(x_only) => {
                    return SecretKey::from_slice(
                        &SharedSecret::new(
                            &x_only.to_public_key().inner,
                            &self.shared_secret(iter),
                        )
                        .secret_bytes(),
                    )
                    .unwrap()
                }

                None => SecretKey::from_slice(&self.p2tr.get_client_wallet().seed).unwrap(),
            }
        }
    }

    pub fn bob_script(public_key: XOnlyPublicKey, preimage_hash: &[u8]) -> Script {
        return Builder::new()
            .push_opcode(all::OP_SHA256)
            .push_slice(&preimage_hash)
            .push_opcode(all::OP_EQUALVERIFY)
            .push_x_only_key(&public_key)
            .push_opcode(all::OP_CHECKSIG)
            .into_script();
    }
}
