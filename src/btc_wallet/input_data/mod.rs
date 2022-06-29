use bitcoin::{Txid, Transaction,  Script};
use electrum_client::{GetBalanceRes, Error};
use electrum_client::{ListUnspentRes};
pub mod electrum_rpc;
pub mod test_rpc_call;
