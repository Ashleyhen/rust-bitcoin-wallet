use std::sync::Arc;

use bitcoin::{OutPoint, Script, Transaction, TxIn, Txid, Witness};
use electrum_client::{Error, GetBalanceRes};

pub mod electrum_rpc;
pub mod test_rpc_call;
use electrum_client::{ListUnspentRes};

pub trait ApiCall {
    fn transaction_broadcast(&self, tx: &Transaction) -> Result<Txid, Error>;
    fn script_list_unspent(&self, script: &Script) -> Result<Vec<ListUnspentRes>, Error>;
    fn transaction_get(&self, txid: &Txid) -> Result<Transaction, Error>;
    fn script_get_balance(&self, script: &Script) -> Result<GetBalanceRes, Error>;
}
pub trait RpcCall {
    fn contract_source(&self) -> (Vec<TxIn>, Vec<Transaction>);
    fn script_get_balance(&self) -> Result<GetBalanceRes, Error>;
    
}
