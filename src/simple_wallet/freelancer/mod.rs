use bitcoin::{
    psbt::{Input, Output, PartiallySignedTransaction},
    secp256k1::{Message, SecretKey},
    Transaction, TxOut,
};

use crate::bitcoin_wallet::input_data::RpcCall;

pub mod bisq;
pub mod bisq_key;
pub mod bisq_script;
pub trait ISigner {
    //  fn sign_tx(secret_key:&SecretKey,tx_out:&TxOut, input:&Input, message:&Message,output:&Output)->Input;
    fn sign_all_unsigned_tx(
        &self,
        secret_key: &SecretKey,
        prevouts: &Vec<TxOut>,
        unsigned_tx: &Transaction,
    ) -> Vec<Input>;

    fn finalize_tx<R: RpcCall>(rpc_call: &R, psbt: PartiallySignedTransaction) -> Transaction;
}
pub enum TrType {
    Script,
    Key,
}
