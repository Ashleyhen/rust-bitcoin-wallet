use std::sync::Arc;

use bitcoin::{OutPoint, Script, Transaction, TxIn, Txid, Witness};
use electrum_client::{Error, GetBalanceRes};

pub mod electrum_rpc;
pub mod reuse_rpc_call;
pub mod tapscript_ex_input;
// pub mod json_input;

use electrum_client::ListUnspentRes;

pub trait RpcCall {
    fn contract_source(&self) -> (Vec<TxIn>, Vec<Transaction>);
    fn script_get_balance(&self) -> Result<GetBalanceRes, Error>;
}
