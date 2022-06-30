use crate::btc_wallet::WalletKeys;
use bitcoin::{
    psbt::{Input, Output},
    util::bip32::ExtendedPubKey,
    Address, Script, Transaction, TxIn, TxOut, Witness,
};
use electrum_client::ListUnspentRes;
use std::{borrow::BorrowMut, str::FromStr, sync::Arc};

use super::wallet_traits::AddressSchema;

pub fn pub_key_lock<'a, S>(
    schema: &'a S,
    amount: u64,
    total: u64,
    change_addr: ExtendedPubKey,
    to_addr: String,
) -> Vec<TxOut>
where
    S: AddressSchema,
{
    let tip: u64 = 300;
    let wallet_keys = schema.to_wallet().create_wallet(
        schema.wallet_purpose(),
        schema.to_wallet().recieve,
        schema.to_wallet().change,
    );
    schema.map_ext_keys(&wallet_keys.0).script_pubkey();

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
