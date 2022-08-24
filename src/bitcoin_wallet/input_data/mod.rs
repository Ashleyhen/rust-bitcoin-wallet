use std::sync::Arc;

use bitcoin::{Transaction, TxIn};
use electrum_client::{Error, GetBalanceRes};

pub mod electrum_rpc;
pub mod reuse_rpc_call;
pub mod tapscript_ex_input;
// pub mod json_input;

pub trait RpcCall {
    fn contract_source(&self) ->  Vec<Transaction>;
    fn prev_input(&self)->Vec<TxIn>;
    fn script_get_balance(&self) -> Arc<GetBalanceRes>;
}
