use bitcoin::{Txid, Transaction,  Script};
use electrum_client::{GetBalanceRes, Error};


use electrum_client::{ListUnspentRes};

pub mod electrum_rpc;
pub mod test_rpc_call;
// use crate::btc_wallet::input_data::electrum_rpc as other_electrum_rpc;
pub trait ApiCall {
    fn transaction_broadcast(&self, tx: &Transaction) -> Result<Txid, Error>;
    fn script_list_unspent(&self, script: &Script) -> Result<Vec<ListUnspentRes>, Error>;
    fn transaction_get(&self, txid: &Txid) -> Result<Transaction, Error>;
    fn script_get_balance(&self, script: &Script) -> Result<GetBalanceRes, Error>;
    fn new()->Self;
}