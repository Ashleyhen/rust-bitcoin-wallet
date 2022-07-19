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
    fn unlock_key(&self, previous: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input>;
    fn lock_key<'a, S>(&self, schema: &'a S, total: u64) -> Vec<(Output, u64)>
    where
        S: AddressSchema;
    fn extract_tx(&self, output_list: &Vec<(Output, u64)>, tx_in: Vec<TxIn>) -> Transaction {
        let tx_out = output_list
            .iter()
            .map(|(output, value)| TxOut {
                value: *value,
                script_pubkey: output.witness_script.as_ref().unwrap().clone(),
            })
            .collect();

        return Transaction {
            version: 2,
            lock_time: 0,
            input: tx_in,
            output: tx_out,
        };
    }
}

/*
pub fn pub_key_lock<'a, S>(
    schema: &'a S,
    amount: u64,
    total: u64,
    change_addr: ExtendedPubKey,
    to_addr: &String,
) -> Vec<(Output,u64)>
where
    S: AddressSchema,
{ let tip: u64 = 300;

    let output_fn=|script:&Script|->Output{
    let output=Output::default();
      output.tap_internal_key=Some(XOnlyPublicKey::from_slice(&script[2..]).unwrap());
      return output
    };
    let send_tx =(output_fn(&Address::from_str(&to_addr).unwrap().script_pubkey()), amount);


    if (total <= (amount + tip)) {
        return vec![send_tx];
    }
    let change_tx =(output_fn(&Address::from_str(&to_addr).unwrap().script_pubkey()), total - (amount + tip));

    return vec![send_tx, change_tx];
}
 */
pub fn standard_extraction(output_list: &Vec<(Output, u64)>, tx_in: Vec<TxIn>) -> Transaction {
    let tx_out = output_list
        .iter()
        .map(|(output, value)| TxOut {
            value: *value,
            script_pubkey: output.witness_script.as_ref().unwrap().clone(),
        })
        .collect();

    return Transaction {
        version: 2,
        lock_time: 0,
        input: tx_in,
        output: tx_out,
    };
}

pub fn standard_lock<'a, S>(
    schema: &'a S,
    amount: u64,
    total: u64,
    change_addr: ExtendedPubKey,
    to_addr: &String,
) -> Vec<(Output, u64)>
where
    S: AddressSchema,
{
    let tip: u64 = 300;

    let output_fn = |script: Script| -> Output {
        let mut output = Output::default();
        output.witness_script = Some(script);
        return output;
    };
    schema.map_ext_keys(&change_addr);
    let send_tx = (
        output_fn(Address::from_str(&to_addr).unwrap().script_pubkey()),
        amount,
    );

    if (total <= (amount + tip)) {
        return vec![send_tx];
    }

    let change_tx = (
        output_fn(schema.map_ext_keys(&change_addr).script_pubkey()),
        total - (amount + tip),
    );

    return vec![send_tx, change_tx];
}
