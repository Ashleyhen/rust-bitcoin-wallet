use bitcoin::{Transaction, TxIn, Txid, blockdata::transaction};

pub mod electrum_rpc;
pub mod regtest_rpc;
pub mod reuse_rpc_call;
pub mod tapscript_ex_input;

pub trait RpcCall {
    fn contract_source(&self) -> Vec<Transaction>;
    fn prev_input(&self) -> Vec<TxIn>;
    fn script_get_balance(&self) -> u64;
    fn fee(&self) -> u64;
    fn broadcasts_transacton(&self,transaction: &Transaction);
}
