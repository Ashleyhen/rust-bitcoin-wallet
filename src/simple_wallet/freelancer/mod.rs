use bitcoin::{psbt::{Input, Output}, TxOut, secp256k1::{Message, SecretKey}};

pub mod bisq;
pub mod bisq_script;
pub mod bisq_key;
trait FreeLancerWallet{
	 fn sign_tx(secret_key:&SecretKey,tx_out:&TxOut, input:&Input, message:&Message,output:&Output)->Input;
}
