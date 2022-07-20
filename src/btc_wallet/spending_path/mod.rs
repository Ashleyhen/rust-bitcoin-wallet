use std::{collections::BTreeMap, iter::Map, str::FromStr};

use bitcoin::{
    psbt::{Input, Output},
    util::bip32::ExtendedPubKey,
    Address, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};

use super::{address_formats::AddressSchema, wallet_methods::NETWORK};

pub mod p2tr_key_path;
mod p2tr_multisig_path;
pub mod p2wpkh_script_path;
pub trait Vault {
    fn create_tx(&self, output_list: &Vec<Output>, tx_in: Vec<TxIn>, total: u64) -> Transaction;
    fn lock_key<'a, S>(&self, schema: &'a S) -> Vec<Output>
    where
        S: AddressSchema;
    fn unlock_key(&self, previous: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input>;
}

fn standard_create_tx(amount: u64, output_list: &Vec<Output>, tx_in: Vec<TxIn>, total: u64) -> Transaction {
    let tip = 300;
    let tx_out_list = || {
        let tx_out = TxOut {
            value: amount,
            script_pubkey: output_list[0].clone().witness_script.unwrap(),
        };
        if (total - amount) > tip {
            return vec![ 
                tx_out,
                TxOut {
                    value: total - amount,
                    script_pubkey: output_list[1].clone().witness_script.unwrap(),
                },
            ];
        }
        return vec![tx_out];
    };
    return Transaction {
        version: 2,
        lock_time: 0,
        input: tx_in,
        output: tx_out_list(),
    };
}
pub fn standard_lock<'a, S>(
    schema: &'a S,
    change_addr: ExtendedPubKey,
    to_addr: &String,
) -> Vec<Output>
where
    S: AddressSchema,
{

    let output_fn = |script: Script| -> Output {
        let mut output = Output::default();
        output.witness_script = Some(script);
        return output;
    };

    schema.map_ext_keys(&change_addr);
    let send_tx = output_fn(Address::from_str(&to_addr).unwrap().script_pubkey());
    let change_tx = output_fn(schema.map_ext_keys(&change_addr).script_pubkey());

    return vec![send_tx, change_tx];
}
