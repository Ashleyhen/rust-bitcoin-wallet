use crate::bitcoin_wallet::input_data::RpcCall;

pub mod p2tr_key;
pub mod p2wpkh;
pub mod p2wsh;

pub trait Wallet<'a, R> {
    fn new(secret_string: Option<&str>, client: &'a R) -> Self
    where
        R: RpcCall;
}
