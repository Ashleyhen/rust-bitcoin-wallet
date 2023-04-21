use bitcoin::{TxOut, psbt::{Input, Output}, secp256k1::{Message, SecretKey}, schnorr::TapTweak, SchnorrSig};

use crate::bitcoin_wallet::constants::secp;

use super::FreeLancerWallet;

pub struct BisqKey{
	secret_key:SecretKey
 }
 
 impl FreeLancerWallet for BisqKey{
    fn sign_tx(secret_key:&SecretKey,tx_out:&TxOut, input:&Input, message:&Message,output:&Output)->Input {
		let tap_info = output
            .clone()
            .tap_tree
            .unwrap()
            .clone()
            .into_builder()
            .finalize(&secp(), output.tap_internal_key.unwrap())
            .unwrap();

		let tweaked_key_pair = secret_key.keypair(&secp()).tap_tweak(&secp(), tap_info.merkle_root());
		tweaked_key_pair.to_inner().tap_tweak(&secp(), None);
	
		let sig = secp().sign_schnorr(&message, &tweaked_key_pair.to_inner());
	
		let schnorr_sig = SchnorrSig {
		    sig,
		    hash_ty: bitcoin::SchnorrSighashType::AllPlusAnyoneCanPay,
		};
	
		let mut input = input.clone();
	
		input.witness_script = Some(tx_out.script_pubkey.clone());
	
		input.tap_key_sig = Some(schnorr_sig);
	
		input.witness_utxo = Some(tx_out.clone());
	
		return input;
    }
}