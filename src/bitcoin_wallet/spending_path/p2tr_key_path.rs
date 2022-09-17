use bitcoin::{
    psbt::Output,
    secp256k1::{All, Secp256k1},
    Address, KeyPair, Script, Transaction, TxIn, TxOut,
};

use crate::bitcoin_wallet::{
    constants::{NETWORK, TIP},
    script_services::{
        input_service::{insert_witness_tx, sign_key_sig},
        output_service::new_witness_pub_k,
        psbt_factory::{LockFn, UnlockFn},
    },
};

pub struct P2tr {
    pub secp: Secp256k1<All>,
}

impl P2tr {
    pub fn new(secp: &Secp256k1<All>) -> Self {
        return P2tr { secp: secp.clone() };
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

    pub fn single_create_tx() -> Box<dyn Fn(Vec<Output>, Vec<TxIn>, u64) -> Transaction> {
        return Box::new(move |outputs: Vec<Output>, tx_in: Vec<TxIn>, total: u64| {
            let mut tx_out_vec = vec![TxOut {
                value: total-TIP,
                script_pubkey: outputs[0].clone().witness_script.unwrap(),
            }];
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

    pub fn single_output<'a>(&'a self, send: Script) -> Vec<LockFn<'a>> {
        return vec![
            new_witness_pub_k(send),
            ];
    }

    pub fn input_factory<'a>(
        &'a self,
        keypair: &'a KeyPair,
        script_pubkey:Script
    ) -> Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<UnlockFn<'a>> + 'a> {
        return Box::new(
            move |previous_list: Vec<Transaction>, current_tx: Transaction| {
                let prev_output_list=previous_list.iter().flat_map(|tx|tx.output.clone()).collect::<Vec<TxOut>>();
                let mut unlock_vec: Vec<UnlockFn> = vec![];
                for (size, prev) in previous_list.iter().enumerate() {
                    let tx_out = prev
                        .output
                        .iter()
                        .find(|t| {
                            t.script_pubkey.eq(&script_pubkey)
                        })
                        .unwrap();
                    unlock_vec.push(insert_witness_tx(tx_out.clone()));
                    unlock_vec.push(sign_key_sig(
                        &self.secp,
                        &keypair,
                        current_tx.clone(),
                        prev_output_list.clone(),
                        size,
                    ));
                }
                return unlock_vec;
            },
        );
    }
}
