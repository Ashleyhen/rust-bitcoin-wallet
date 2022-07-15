use bitcoin::{psbt::Input, util::bip32::ExtendedPubKey, Address, Script, Transaction, Txid, TxOut};
use electrum_client::{Error, GetBalanceRes, ListUnspentRes};

use super::{unlock::SignTx, wallet_methods::ClientWallet, address_schema::AddressSchema};


pub trait Vault{
    fn unlock_key(previous: Vec<Transaction>, current_tx:Transaction) -> Vec<TxOut>;
    fn lock_key(schema:impl AddressSchema) -> Vec<Input>;
}