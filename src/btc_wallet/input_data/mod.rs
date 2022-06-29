use bitcoin::{Script, Transaction, Txid};
use electrum_client::ListUnspentRes;
use electrum_client::{Error, GetBalanceRes};
pub mod electrum_rpc;
pub mod test_rpc_call;
