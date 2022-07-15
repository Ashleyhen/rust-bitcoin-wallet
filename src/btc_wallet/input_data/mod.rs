use bitcoin::{Script, Transaction, Txid};
use electrum_client::ListUnspentRes;
use electrum_client::{Error, GetBalanceRes};
pub mod electrum_rpc;
pub mod test_rpc_call;
pub trait ApiCall {
    fn transaction_broadcast(&self, tx: &Transaction) -> Result<Txid, Error>;
    fn script_list_unspent(&self, script: &Script) -> Result<Vec<ListUnspentRes>, Error>;
    fn transaction_get(&self, txid: &Txid) -> Result<Transaction, Error>;
    fn script_get_balance(&self, script: &Script) -> Result<GetBalanceRes, Error>;
}