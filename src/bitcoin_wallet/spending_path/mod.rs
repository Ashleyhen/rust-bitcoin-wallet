use bitcoin::{psbt::Output, Address, Script, Transaction, TxIn, TxOut};

use super::{
    constants::{NETWORK, TIP},
    script_services::{output_service::new_witness_pub_k, psbt_factory::LockFn},
};

pub mod p2tr_key_path;
pub mod p2wpkh_script_path;
pub mod p2wsh_path;
pub mod tap_script_spending_ex;

pub fn single_create_tx() -> Box<dyn Fn(Vec<Output>, Vec<TxIn>, u64) -> Transaction> {
    return Box::new(move |outputs: Vec<Output>, tx_in: Vec<TxIn>, total: u64| {
        let tx_out_vec = vec![TxOut {
            value: total - TIP,
            script_pubkey: outputs[0].clone().witness_script.unwrap(),
        }];
        return Transaction {
            version: 2,
            lock_time: bitcoin::PackedLockTime(0),
            input: tx_in,
            output: tx_out_vec,
        };
    });
}
pub fn single_output<'a>(send: Script) -> Vec<LockFn<'a>> {
    return vec![new_witness_pub_k(send)];
}

pub fn get_script_addresses(output_list: Vec<Output>) -> Vec<Address> {
    return output_list
        .iter()
        .map(|f| Address::from_script(&f.witness_script.as_ref().unwrap(), NETWORK).unwrap())
        .collect::<Vec<Address>>();
}
