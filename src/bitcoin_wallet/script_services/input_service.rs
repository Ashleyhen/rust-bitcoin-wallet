use bitcoin::{
    psbt::Input,
    schnorr::TapTweak,
    secp256k1::{All, Message, Secp256k1, SecretKey},
    util::{
        sighash::{Error, Prevouts, ScriptPath, SighashCache},
        taproot::{LeafVersion, TapLeafHash, TaprootSpendInfo},
    },
    Address, EcdsaSig, KeyPair, PrivateKey, SchnorrSig, SchnorrSighashType, Script, Transaction,
    TxIn, TxOut,
};

use crate::bitcoin_wallet::constants::NETWORK;

pub fn insert_control_block<'a>(
    secp: &'a Secp256k1<All>,
    spending_script: Script,
    spending_info: TaprootSpendInfo,
) -> Box<impl FnOnce(&mut Input) + 'a> {
    return Box::new(move |input: &mut Input| {
        let control =
            spending_info.control_block(&(spending_script.clone(), LeafVersion::TapScript));
        let verify = control.as_ref().unwrap().verify_taproot_commitment(
            &secp,
            spending_info.output_key().to_inner(),
            &spending_script,
        );
        println!("is this control block valid {}", verify);
        input.tap_scripts.insert(
            control.unwrap(),
            (spending_script.clone(), LeafVersion::TapScript),
        );
        let witness = Address::p2tr_tweaked(spending_info.output_key(), NETWORK);

        input.witness_script = Some(witness.script_pubkey());
        input.tap_merkle_root = spending_info.merkle_root();
    });
}

pub fn insert_witness_tx_out<'a>(tx_out: TxOut) -> Box<impl FnOnce(&mut Input) + 'a> {
    return Box::new(move |input: &mut Input| {
        input.witness_utxo = Some(tx_out);
    });
}

pub fn insert_witness<'a>(script: Script) -> Box<impl FnOnce(&mut Input) + 'a> {
    return Box::new(move |input: &mut Input| {
        input.witness_script = Some(script.clone());
    });
}
pub fn sign_2_of_2<'a>(
    secp: &'a Secp256k1<All>,
    current_tx: Transaction,
    previous_tx: Vec<TxOut>,
    input_index: usize,
    key_pair: &'a KeyPair,
    witness_script: Script,
    contract: Script,
    auxiliary: &'a [u8; 32],
) -> Box<impl FnOnce(&mut Input) + 'a> {
    return Box::new(move |input: &mut Input| {
        let prev = filter_for_wit(&previous_tx, &witness_script);
        let tap_leaf_hash = TapLeafHash::from_script(&contract, LeafVersion::TapScript);
        let tap_sighash_cache = SighashCache::new(&mut current_tx.clone())
            .taproot_script_spend_signature_hash(
                input_index,
                &Prevouts::All(&prev),
                ScriptPath::with_defaults(&contract),
                SchnorrSighashType::AllPlusAnyoneCanPay,
            )
            .map_err(|err| print_tx_out_addr(&prev, &current_tx.input, &witness_script, err))
            .unwrap();

        let sig = secp.sign_schnorr_with_aux_rand(
            &Message::from_slice(&tap_sighash_cache).unwrap(),
            &key_pair,
            auxiliary,
        );

        let schnorrsig = SchnorrSig {
            sig,
            hash_ty: SchnorrSighashType::AllPlusAnyoneCanPay,
        };
        input.tap_script_sigs.insert(
            (key_pair.public_key().x_only_public_key().0, tap_leaf_hash),
            schnorrsig,
        );
    });
}

pub fn sign_tapleaf<'a>(
    secp: &'a Secp256k1<All>,
    key_pair: &'a KeyPair,
    current_tx: Transaction,
    prev_out: Vec<TxOut>,
    input_index: usize,
    bob_script: Script,
) -> Box<impl FnOnce(&mut Input) + 'a> {
    let x_only = key_pair.public_key().x_only_public_key().0;
    return Box::new(move |input: &mut Input| {
        let witness_script = input.witness_script.as_ref().unwrap();
        let prev = filter_for_wit(&prev_out, &witness_script);
        let tap_leaf_hash = TapLeafHash::from_script(&bob_script, LeafVersion::TapScript);

        let tap_sig_hash = SighashCache::new(&current_tx)
            .taproot_script_spend_signature_hash(
                input_index,
                &Prevouts::All(&prev),
                ScriptPath::with_defaults(&bob_script),
                SchnorrSighashType::AllPlusAnyoneCanPay,
            )
            .map_err(|err| print_tx_out_addr(&prev, &current_tx.input, &witness_script, err))
            .unwrap();

        let sig = secp.sign_schnorr(&Message::from_slice(&tap_sig_hash).unwrap(), &key_pair);
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
        let witness_script = input.clone().witness_script.unwrap();
        let prev = filter_for_wit(&previous_tx, &witness_script);
        let tap_sig = SighashCache::new(&current_tx)
            .taproot_key_spend_signature_hash(
                input_index,
                &Prevouts::All(&prev),
                SchnorrSighashType::AllPlusAnyoneCanPay,
            )
            .map_err(|err| {
                print_tx_out_addr(
                    &prev,
                    &current_tx.input,
                    &input.clone().witness_script.unwrap(),
                    err,
                )
            })
            .unwrap();
        let tweaked_pair = key_pair.tap_tweak(&secp, input.tap_merkle_root);
        let msg = Message::from_slice(&tap_sig).unwrap();
        let sig = secp.sign_schnorr(&msg, &tweaked_pair.to_inner());
        let schnorrsig = SchnorrSig {
            sig,
            hash_ty: SchnorrSighashType::AllPlusAnyoneCanPay,
        };
        secp.verify_schnorr(&sig, &msg, &tweaked_pair.to_inner().x_only_public_key().0)
            .unwrap();
        input.tap_key_sig = Some(schnorrsig);
    });
}

pub fn sign_segwit_v0<'a>(
    secp: &'a Secp256k1<All>,
    current_tx: Transaction,
    sats: u64,
    input_index: usize,
    script_code: Script,
    priv_k: SecretKey,
) -> Box<impl FnOnce(&mut Input) + 'a> {
    Box::new(move |input: &mut Input| {
        let public_key =
            bitcoin::PublicKey::from_private_key(secp, &PrivateKey::new(priv_k, NETWORK));
        let sig_hash = SighashCache::new(&mut current_tx.clone())
            .segwit_signature_hash(
                input_index,
                &script_code,
                sats,
                bitcoin::EcdsaSighashType::All,
            )
            .unwrap();

        let msg = Message::from_slice(&sig_hash).unwrap();
        let sig = EcdsaSig::sighash_all(secp.sign_ecdsa(&msg, &priv_k));

        input.partial_sigs.insert(public_key, sig);
    })
}

fn filter_for_wit(previous_tx: &Vec<TxOut>, witness: &Script) -> Vec<TxOut> {
    return previous_tx
        .iter()
        .filter(|t| t.script_pubkey.eq(&witness))
        .map(|a| a.clone())
        .collect::<Vec<TxOut>>();
}

fn print_tx_out_addr(prev: &Vec<TxOut>, input: &Vec<TxIn>, witness: &Script, err: Error) -> String {
    eprintln!("ERROR!!! {} ", err.to_string());
    let dbg_err = String::from("\n");

    eprintln!(
        "witness: {}",
        Address::from_script(witness, NETWORK).unwrap().to_string()
    );
    eprintln!("previous output: ");
    prev.iter().for_each(|tx_out| {
        let prev_out_addr = Address::from_script(&tx_out.script_pubkey, NETWORK)
            .map(|a| a.to_string())
            .unwrap_or(tx_out.script_pubkey.to_string());

        eprintln!("script {}, amount, {}", prev_out_addr, tx_out.value)
    });

    eprintln!("current transaction inputs: ");
    input
        .iter()
        .for_each(|tx_in| eprintln!("previous output points {} ", tx_in.previous_output));

    eprintln!("{}", dbg_err.to_string());
    return dbg_err.to_string();
}
