use bitcoin::{
    psbt::{Input, Output, PartiallySignedTransaction, Prevouts},
    schnorr::TapTweak,
    secp256k1::{Message, SecretKey},
    util::sighash::SighashCache,
    SchnorrSig, Transaction, TxOut, Script,
};
use miniscript::psbt::PsbtExt;

use crate::bitcoin_wallet::{constants::secp, input_data::RpcCall};

use super::{ISigner, TrType};

pub struct BisqKey {
    pub output: Output,
}

impl ISigner for BisqKey {
    fn sign_all_unsigned_tx(
        &self,
        secret_key: &SecretKey,
        prevouts: &Vec<TxOut>,
        unsigned_tx: &Transaction,
    ) -> Vec<Input> {
        return prevouts
            .iter()
            .enumerate()
            .map(|(index, tx_out)| {
                let message = create_message(index, unsigned_tx, &prevouts);
                return sign_tx(
                    secret_key,
                    tx_out,
                    &Input::default(),
                    &message,
                    &self.output,
                );
            })
            .collect();
    }

    fn finalize_tx<R: RpcCall>(rpc_call: &R, psbt: PartiallySignedTransaction) -> Transaction {
        let tx = psbt.finalize(&secp()).unwrap().extract_tx();
        rpc_call.broadcasts_transacton(&tx);
        return tx;
    }
}

pub fn create_message(index: usize, unsigned_tx: &Transaction, prevouts: &Vec<TxOut>) -> Message {
    let sighash = SighashCache::new(&mut unsigned_tx.clone())
        .taproot_key_spend_signature_hash(
            index,
            &Prevouts::All(&prevouts),
            bitcoin::SchnorrSighashType::AllPlusAnyoneCanPay,
        )
        .unwrap();
    let message = Message::from_slice(&sighash).unwrap();
    return message;
}

fn sign_tx(
    secret_key: &SecretKey,
    tx_out: &TxOut,
    input: &Input,
    message: &Message,
    output: &Output,
) -> Input {
    let tap_info = output
        .clone()
        .tap_tree
        .unwrap()
        .clone()
        .into_builder()
        .finalize(&secp(), output.tap_internal_key.unwrap())
        .unwrap();

    let tweaked_key_pair = secret_key
        .keypair(&secp())
        .tap_tweak(&secp(), tap_info.merkle_root());

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
