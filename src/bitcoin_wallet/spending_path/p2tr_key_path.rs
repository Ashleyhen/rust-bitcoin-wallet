use std::{str::FromStr, sync::Arc};

use bitcoin::{
    psbt::{Input, Output},
    schnorr::TapTweak,
    secp256k1::{All, Message, Secp256k1},
    util::{
        bip32::ExtendedPrivKey,
        sighash::{Prevouts, SighashCache},
    },
    Address, KeyPair, SchnorrSig, SchnorrSighashType, Script, Transaction, TxIn, TxOut,
    XOnlyPublicKey,
};

use crate::bitcoin_wallet::{
    constants::{NETWORK, TIP},
    script_services::{
        input_service::{insert_witness_tx, sign_key_sig},
        output_service::{insert_tree_witness, new_tap_internal_key, new_witness_pub_k},
        psbt_factory::{LockFn, UnlockFn},
    },
};

pub struct P2TR_K {
    secp: Secp256k1<All>,
}

impl P2TR_K {
    pub fn new(secp: &Secp256k1<All>) -> Self {
        return P2TR_K { secp: secp.clone() };
    }
    pub fn create_tx(amount: u64) -> Box<dyn Fn(Vec<Output>, Vec<TxIn>, u64) -> Transaction> {
        return Box::new(move |outputs: Vec<Output>, tx_in: Vec<TxIn>, total: u64| {
            let mut tx_out_vec = vec![TxOut {
                value: amount,
                script_pubkey: outputs[0].clone().witness_script.unwrap(),
            }];

            if (total - amount) > TIP {
                tx_out_vec.push(TxOut {
                    value: total - (amount + TIP),
                    script_pubkey: outputs[1].clone().witness_script.unwrap(),
                });
            }
            return Transaction {
                version: 2,
                lock_time: 0,
                input: tx_in,
                output: tx_out_vec,
            };
        });
    }

    pub fn output_factory<'a>(&'a self, change: Script, send: Script) -> Vec<Vec<LockFn<'a>>> {
        return vec![
            vec![new_witness_pub_k(change)],
            vec![new_witness_pub_k(send)],
        ];
    }
    pub fn single_output<'a>(&'a self, send: Script) -> LockFn<'a> {
        return new_witness_pub_k(send);
    }
    pub fn input_factory<'a>(
        &'a self,
        keypair: &'a KeyPair,
    ) -> Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<UnlockFn<'a>> + 'a> {
        return Box::new(
            move |previous_list: Vec<Transaction>, current_tx: Transaction| {
                let mut unlock_vec: Vec<UnlockFn> = vec![];
                for (size, prev) in previous_list.iter().enumerate() {
                    let tx_out = prev
                        .output
                        .iter()
                        .find(|t| {
                            t.script_pubkey.eq(&Address::p2tr(
                                &self.secp,
                                keypair.public_key(),
                                None,
                                NETWORK,
                            )
                            .script_pubkey())
                        })
                        .unwrap();
                    unlock_vec.push(insert_witness_tx(tx_out.clone()));
                    unlock_vec.push(sign_key_sig(
                        &self.secp,
                        &keypair,
                        current_tx.clone(),
                        prev.clone().output,
                        size,
                    ));
                }
                return unlock_vec;
            },
        );
    }
}