use bitcoin::{
    psbt::{Input, Output, TapTree},
    schnorr::TapTweak,
    secp256k1::{All, Message, Secp256k1},
    util::{
        bip32::ExtendedPrivKey,
        sighash::{Prevouts, SighashCache},
        taproot::{LeafVersion, TapLeafHash, TaprootSpendInfo},
    },
    Address, KeyPair, SchnorrSig, SchnorrSighashType, Script, Transaction, TxIn, TxOut,
    XOnlyPublicKey,
};

use crate::bitcoin_wallet::constants::NETWORK;

pub fn insert_control_block<'a>(
    secp: &'a Secp256k1<All>,
    script: Script,
    spending_info: TaprootSpendInfo,
) -> Box<impl FnOnce(&mut Input) + 'a> {
    return Box::new(move |input: &mut Input| {
        let control = spending_info.control_block(&(script.clone(), LeafVersion::TapScript));
        let verify = control.as_ref().unwrap().verify_taproot_commitment(
            &secp,
            spending_info.output_key().to_inner(),
            &script,
        );
        print!("{}", verify);
        input
            .tap_scripts
            .insert(control.unwrap(), (script.clone(), LeafVersion::TapScript));
    });
}

pub fn insert_witness_tx<'a>(tx_out: TxOut) -> Box<impl FnOnce(&mut Input) + 'a> {
    return Box::new(move |input: &mut Input| {
        input.witness_utxo = Some(tx_out);
    });
}

fn filter_for_wit(previous_tx: Vec<TxOut>, witness: &Script) -> Vec<TxOut> {
    return previous_tx
        .iter()
        .filter(|t| t.script_pubkey.eq(&witness))
        .map(|a| a.clone())
        .collect::<Vec<TxOut>>();
}

pub fn sign_tapleaf<'a>(
    secp: &'a Secp256k1<All>,
    key_pair: &'a KeyPair,
    current_tx: Transaction,
    previous_tx: Vec<TxOut>,
    input_index: usize,
    witness_script: Script,
    leaf_script: Script,
) -> Box<impl FnOnce(&mut Input) + 'a> {
    let x_only = key_pair.public_key();
    return Box::new(move |input: &mut Input| {
        let prev = filter_for_wit(previous_tx, &witness_script);
        let tap_leaf_hash = TapLeafHash::from_script(&leaf_script, LeafVersion::TapScript);
        let tap_sig_hash = SighashCache::new(&current_tx)
            .taproot_script_spend_signature_hash(
                input_index,
                &Prevouts::All(&prev),
                tap_leaf_hash,
                SchnorrSighashType::AllPlusAnyoneCanPay,
            )
            .unwrap();
        let tweaked_pair = key_pair.tap_tweak(&secp, input.tap_merkle_root);
        let sig = secp.sign_schnorr(
            &Message::from_slice(&tap_sig_hash).unwrap(),
            &tweaked_pair.into_inner(),
        );
        let schnorrsig = SchnorrSig {
            sig,
            hash_ty: SchnorrSighashType::AllPlusAnyoneCanPay,
        };
        input
            .tap_script_sigs
            .insert((x_only, tap_leaf_hash), schnorrsig);
    });
}
pub fn sign_key_sig<'a>(
    secp: &'a Secp256k1<All>,
    key_pair: &'a KeyPair,
    current_tx: Transaction,
    previous_tx: Vec<TxOut>,
    input_index: usize,
) -> Box<impl FnOnce(&mut Input) + 'a> {
    return Box::new(move |input: &mut Input| {
        let witness_script =
            Address::p2tr(secp, key_pair.public_key(), None, NETWORK).script_pubkey();
        let prev = filter_for_wit(previous_tx, &witness_script);
        let tap_sig = SighashCache::new(&current_tx)
            .taproot_key_spend_signature_hash(
                input_index,
                &Prevouts::All(&prev),
                SchnorrSighashType::AllPlusAnyoneCanPay,
            )
            .unwrap();
        let tweaked_pair = key_pair.tap_tweak(&secp, None);

        let sig = secp.sign_schnorr(
            &Message::from_slice(&tap_sig).unwrap(),
            &tweaked_pair.into_inner(),
        );
        let schnorrsig = SchnorrSig {
            sig,
            hash_ty: SchnorrSighashType::AllPlusAnyoneCanPay,
        };
        input.tap_key_sig = Some(schnorrsig);
    });
}
