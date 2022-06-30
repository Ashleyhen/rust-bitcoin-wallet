use bitcoin::{psbt::Input, util::bip32::ExtendedPubKey, Address, Script, Transaction, Txid};
use electrum_client::{Error, GetBalanceRes, ListUnspentRes};

use super::{unlock::SignTx, wallet_methods::ClientWallet};

pub trait AddressSchema {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> Address;
    fn wallet_purpose(&self) -> u32;
    fn new(seed: Option<String>, recieve: u32, change: u32) -> Self;
    fn to_wallet(&self) -> ClientWallet;
    fn prv_tx_input(
        &self,
        previous_tx: Vec<Transaction>,
        current_input: Transaction,
        unlocking_fn: &dyn Fn(SignTx) -> (Input),
    ) -> Vec<Input>;
}

pub trait ApiCall {
    fn transaction_broadcast(&self, tx: &Transaction) -> Result<Txid, Error>;
    fn script_list_unspent(&self, script: &Script) -> Result<Vec<ListUnspentRes>, Error>;
    fn transaction_get(&self, txid: &Txid) -> Result<Transaction, Error>;
    fn script_get_balance(&self, script: &Script) -> Result<GetBalanceRes, Error>;
    fn new() -> Self;
}
