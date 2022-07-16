use std::str::FromStr;

use bitcoin::{psbt::Input, util::bip32::ExtendedPubKey, Address, Transaction, TxIn, TxOut};

use super::address_formats::AddressSchema;

pub mod p2tr_key_path;
mod p2tr_multisig_path;
pub mod p2wpkh_script_path;
pub trait Vault {
    fn unlock_key(&self, previous: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input>;
    fn lock_key<'a, S>(&self, schema: &'a S, tx_in: Vec<TxIn>, total: u64) -> Transaction
    where
        S: AddressSchema;
}
pub fn pub_key_lock<'a, S>(
    schema: &'a S,
    amount: u64,
    total: u64,
    change_addr: ExtendedPubKey,
    to_addr: &String,
) -> Vec<TxOut>
where
    S: AddressSchema,
{
    let tip: u64 = 300;

    let send_tx = TxOut {
        value: amount,
        script_pubkey: Address::from_str(&to_addr).unwrap().script_pubkey(),
    };

    if (total <= (amount + tip)) {
        return vec![send_tx];
    }

    let change_tx = TxOut {
        value: total - (amount + tip),
        script_pubkey: schema.map_ext_keys(&change_addr).script_pubkey(),
    };

    return vec![send_tx, change_tx];
}
