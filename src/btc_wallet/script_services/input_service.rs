use bitcoin::{
    psbt::{Input, Output},
    schnorr::TapTweak,
    secp256k1::{All, Message, Secp256k1},
    util::{
        bip32::ExtendedPrivKey,
        sighash::{Prevouts, SighashCache},
        taproot::{LeafVersion, TaprootSpendInfo},
    },
    KeyPair, SchnorrSig, SchnorrSighashType, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};

use crate::btc_wallet::address_formats::{p2tr_addr_fmt::P2TR, AddressSchema};

pub fn insert_givens<'a>() -> Box<impl FnOnce(&Output, &mut Input) + 'a> {
    return Box::new(move |output: &Output, input: &mut Input| {
        let out = output.clone();
        input.witness_script = out.witness_script;
        input.tap_internal_key = out.tap_internal_key;
        input.tap_key_origins = out.tap_key_origins;
    });
}

pub fn insert_control_block<'a>(
    secp: &'a Secp256k1<All>,
    x_only: XOnlyPublicKey,
    script: Script,
) -> Box<impl FnOnce(&Output, &mut Input) + 'a> {
    let mut err_msg = "missing expected scripts for x_only ".to_string();
    err_msg.push_str(&x_only.to_string());

    return Box::new(move |output: &Output, input: &mut Input| {
        let internal_key = input.tap_internal_key.expect("msg");
        let spending_info = output
            .tap_tree
            .as_ref()
            .unwrap()
            .to_builder()
            .finalize(&secp, internal_key)
            .unwrap();

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
) -> Box<impl FnOnce(&Output, &mut Input) + 'a> {
    let x_only = key_pair.public_key();
    return Box::new(move |output: &Output, input: &mut Input| {
        let witness_script = output
            .witness_script
            .as_ref()
            .expect("missing witness script");
        let prev = filter_for_wit(previous_tx, witness_script);
        let tap_leaf_hash = output.tap_key_origins.get(&x_only).unwrap().0.clone();
        let tap_sig_hash = SighashCache::new(&current_tx)
            .taproot_script_spend_signature_hash(
                input_index,
                &Prevouts::All(&prev),
                tap_leaf_hash[0],
                SchnorrSighashType::Default,
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
            .insert((x_only, tap_leaf_hash[0]), schnorrsig);
    });
}
