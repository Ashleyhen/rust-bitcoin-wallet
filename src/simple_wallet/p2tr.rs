use std::{collections::BTreeMap, str::FromStr};

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    psbt::{Input, PartiallySignedTransaction, Prevouts},
    schnorr::TapTweak,
    secp256k1::{All, Message, Scalar, Secp256k1, SecretKey},
    util::sighash::SighashCache,
    Address, KeyPair, Network, OutPoint, PackedLockTime, SchnorrSig, Script, Transaction, TxIn,
    TxOut, Witness,
};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use miniscript::psbt::PsbtExt;

use crate::bitcoin_wallet::input_data::{regtest_rpc::RegtestRpc, RpcCall};

pub fn p2tr(secret_string: Option<&str>) {
    let secp = Secp256k1::new();
    let scalar = Scalar::random();
    let secret = match secret_string {
        Some(sec_str) => SecretKey::from_str(&sec_str).unwrap(),
        None => {
            let secret_key = SecretKey::from_slice(&scalar.to_be_bytes()).unwrap();
            println!("secret_key: {}", secret_key.display_secret());
            secret_key
        }
    };

    let key_pair = KeyPair::from_secret_key(&secp, &secret);

    let (x_only, _) = key_pair.x_only_public_key();

    let address = Address::p2tr(&secp, x_only, None, Network::Regtest);

    println!("address {}", address.to_string());

    if (secret_string.is_none()) {
        return;
    }

    let regtest = RegtestRpc::from_string(&vec!["bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj"], None);

    let tx_in_list = regtest.prev_input();

    let transaction_list = regtest.contract_source();

    let prevouts = transaction_list
        .iter()
        .flat_map(|tx| tx.output.clone())
        .filter(|p| address.script_pubkey().eq(&p.script_pubkey))
        .collect::<Vec<TxOut>>();

    let total: u64 = prevouts.iter().map(|tx_out| tx_out.value).sum();

    let out_put = vec![TxOut {
        value: total - 100000,
        script_pubkey: Address::from_str(
            "bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj",
        )
        .unwrap()
        .script_pubkey(),
    }];

    let unsigned_tx = Transaction {
        version: 2,
        lock_time: PackedLockTime(0),
        input: tx_in_list,
        output: out_put,
    };

    let mut psbt = PartiallySignedTransaction::from_unsigned_tx(unsigned_tx.clone()).unwrap();

    psbt.inputs = sign_all_unsigned_tx(&secp, &prevouts, &unsigned_tx, &key_pair);

    let tx = psbt.finalize(&secp).unwrap().extract_tx();

    regtest.transaction_broadcast(&tx);
}

fn sign_all_unsigned_tx(
    secp: &Secp256k1<All>,
    prevouts: &Vec<TxOut>,
    unsigned_tx: &Transaction,
    key_pair: &KeyPair,
) -> Vec<Input> {
    return prevouts
        .iter()
        .enumerate()
        .map(|(index, tx_out)| {
            sign_tx(secp, index, unsigned_tx, &prevouts, key_pair, tx_out).clone()
        })
        .collect();
}

fn sign_tx(
    secp: &Secp256k1<All>,
    index: usize,
    unsigned_tx: &Transaction,
    prevouts: &Vec<TxOut>,
    key_pair: &KeyPair,
    tx_out: &TxOut,
) -> Input {
    let sighash = SighashCache::new(&mut unsigned_tx.clone())
        .taproot_key_spend_signature_hash(
            index,
            &Prevouts::All(&prevouts),
            bitcoin::SchnorrSighashType::AllPlusAnyoneCanPay,
        )
        .unwrap();

    let message = Message::from_slice(&sighash).unwrap();

    let tweaked_key_pair = key_pair.tap_tweak(&secp, None);

    let sig = secp.sign_schnorr(&message, &tweaked_key_pair.to_inner());

    secp.verify_schnorr(
        &sig,
        &message,
        &tweaked_key_pair.to_inner().x_only_public_key().0,
    )
    .unwrap();

    let schnorr_sig = SchnorrSig {
        sig,
        hash_ty: bitcoin::SchnorrSighashType::AllPlusAnyoneCanPay,
    };

    let mut input = Input::default();

    input.witness_script = Some(tx_out.script_pubkey.clone());

    input.tap_key_sig = Some(schnorr_sig);

    input.witness_utxo = Some(tx_out.clone());

    return input;
}
