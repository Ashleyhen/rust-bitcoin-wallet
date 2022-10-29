use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    hashes::hex::FromHex,
    psbt::{Output, PartiallySignedTransaction},
    secp256k1::{All, Secp256k1},
    util::taproot::TaprootBuilder,
    Address, KeyPair, Script, Transaction, TxIn, TxOut, Witness, XOnlyPublicKey,
};
use bitcoin_hashes::Hash;

use crate::bitcoin_wallet::{
    constants::{NETWORK, TIP},
    script_services::{
        input_service::{insert_control_block, sign_tapleaf},
        output_service::{
            insert_tap_key_origin, insert_tap_tree, insert_tree_witness, new_tap_internal_key,
        },
        psbt_factory::{CreateTxFn, LockFn, UnlockFn},
    },
};

use super::scripts::TapScripts;

pub struct TapScriptSendEx<'a> {
    pub secp: &'a Secp256k1<All>,
}

pub fn get_preimage() -> Vec<u8> {
    return Vec::from_hex("107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f")
        .unwrap();
}

pub fn bob_scripts(x_only: &XOnlyPublicKey) -> Script {
    let preimage_hash = bitcoin_hashes::sha256::Hash::hash(&get_preimage());
    let bob_script = Builder::new()
        .push_opcode(all::OP_SHA256)
        .push_slice(&preimage_hash)
        .push_opcode(all::OP_EQUALVERIFY)
        .push_x_only_key(&x_only)
        .push_opcode(all::OP_CHECKSIG)
        .into_script();

    return bob_script;
}

impl<'a> TapScriptSendEx<'a> {
    pub fn new(secp: &'a Secp256k1<All>) -> Self {
        return TapScriptSendEx { secp };
    }

    pub fn alice_script() -> Script {
        let script = Script::from_hex(
            "029000b275209997a497d964fc1a62885b05a51166a65a90df00492c8d7cf61d6accf54803beac",
        )
        .unwrap();
        return script;
    }

    pub fn output_factory(
        &'a self,
        xinternal: &'a XOnlyPublicKey,
        xalice: &'a XOnlyPublicKey,
        xbob: &'a XOnlyPublicKey,
    ) -> Vec<LockFn<'a>> {
        let bob_script = bob_scripts(&xbob);
        let alice_script = TapScriptSendEx::alice_script();
        let combined_script = vec![(1, bob_script.clone()), (1, alice_script.clone())];

        return vec![
            new_tap_internal_key(xinternal),
            insert_tap_key_origin(vec![(1, alice_script)], xalice),
            insert_tap_key_origin(vec![(1, bob_script)], xbob),
            insert_tap_tree(combined_script),
            insert_tree_witness(&self.secp),
        ];
    }

    pub fn adaptor_sig(
        &'a self,
        xinternal:&'a XOnlyPublicKey,
        primary_xonly:&'a XOnlyPublicKey,
        secondary_xonly:&'a XOnlyPublicKey
    )->Vec<LockFn<'a>>{
        let delay=TapScripts::delay(primary_xonly);
        let multi_sig=TapScripts::multi_2_of_2_script(primary_xonly, secondary_xonly);
        let combined_script = vec![(1, delay.get_script()), (1, multi_sig.get_script())];
        return vec![
            new_tap_internal_key(xinternal),
            insert_tap_tree(combined_script.clone()),
            insert_tap_key_origin(combined_script, primary_xonly)
            ];
    }

    pub fn input_factory(
        &'a self,
        bob_keypair: &'a KeyPair,
        internal_key: XOnlyPublicKey,
    ) -> Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<Vec<UnlockFn<'a>>> + 'a> {
        let xbob = bob_keypair.x_only_public_key().0;
        let bob_script = bob_scripts(&xbob);
        let alice_script = TapScriptSendEx::alice_script();

        let script_weights = vec![(1, bob_script.clone()), (1, alice_script.clone())];
        let tap_builder = TaprootBuilder::with_huffman_tree(script_weights.clone()).unwrap();
        let tap_spending_info = tap_builder.finalize(&self.secp, internal_key).unwrap();
        let witness = Script::new_v1_p2tr_tweaked(tap_spending_info.output_key());

        return Box::new(
            move |previous_list: Vec<Transaction>, current_tx: Transaction| {
                let mut unlock_vec_vec: Vec<Vec<UnlockFn>> = vec![];
                for (size, prev) in previous_list.iter().enumerate() {
                    let mut unlock_vec: Vec<UnlockFn> = vec![];
                    unlock_vec.push(insert_control_block(
                        &self.secp,
                        bob_script.clone(),
                        tap_spending_info.clone(),
                    ));
                    unlock_vec.push(sign_tapleaf(
                        &self.secp,
                        &bob_keypair,
                        current_tx.clone(),
                        prev.clone().output,
                        size,
                        witness.clone(),
                        bob_script.clone(),
                    ));
                    unlock_vec_vec.push(unlock_vec);
                }
                return unlock_vec_vec;
            },
        );
    }

    pub fn create_tx() -> CreateTxFn<'a> {
        return Box::new(move |output_list, tx_in, total| {
            let addr =
                Address::from_script(&output_list[0].clone().witness_script.unwrap(), NETWORK)
                    .unwrap();
            dbg!(addr.to_string());
            let tx_out = vec![TxOut {
                value: total - TIP,
                script_pubkey: output_list[0].clone().witness_script.unwrap(),
            }];
            return Transaction {
                version: 2,
                lock_time: bitcoin::PackedLockTime(0),
                input: tx_in,
                output: tx_out,
            };
        });
    }

    pub fn get_script_addresses(output_list: Vec<Output>) -> Vec<Address> {
        return output_list
            .iter()
            .map(|f| Address::from_script(&f.witness_script.as_ref().unwrap(), NETWORK).unwrap())
            .collect::<Vec<Address>>();
    }

    pub fn finialize_script(
        psbt: PartiallySignedTransaction,
        x_only: &XOnlyPublicKey,
    ) -> Transaction {
        let mut witness = Witness::new();

        for sig in &psbt.inputs[0].tap_script_sigs {
            let shnor = sig.1;
            witness.push(shnor.to_vec());
        }
        witness.push(get_preimage());
        let bob_script = bob_scripts(x_only);
        witness.push(bob_script.as_bytes());
        for control in &psbt.inputs[0].tap_scripts {
            // let control_hash = control.0.merkle_branch.as_inner();
            witness.push(control.0.serialize());
        }

        let mut tx = psbt.extract_tx();
        tx.input[0].witness = witness;
        return tx;
    }
}
