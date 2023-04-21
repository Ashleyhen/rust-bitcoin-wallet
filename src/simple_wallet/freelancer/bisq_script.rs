use bitcoin::{
    psbt::{Input, Output, Prevouts},
    secp256k1::{Message, SecretKey},
    util::{
        sighash::{ScriptPath, SighashCache},
        taproot::{LeafVersion, TapLeafHash},
    },
    SchnorrSig, SchnorrSighashType, Script, Transaction, TxOut,
};

use crate::bitcoin_wallet::constants::secp;

use super::{ISigner, TrType};

pub struct BisqScript {
    pub output: Output,
    pub input: Vec<Input>,
}

impl ISigner for BisqScript {

    fn sign_all_unsigned_tx(
        &self,
        secret_key: &SecretKey,
        prevouts: &Vec<TxOut>,     //
        unsigned_tx: &Transaction, //
    ) -> Vec<Input> {
        return prevouts
            .iter()
            .enumerate()
            .map(|(index, tx_out)| {
                let binding = self.output.clone().tap_tree.unwrap();
                let target_script = binding.script_leaves().next().unwrap().script();
                let message = create_script_message(index, unsigned_tx, prevouts, target_script);
                sign_tx(
                    &secret_key,
                    tx_out,
                    &self.input
                        .get(index)
                        .clone()
                        .unwrap_or(&Input::default())
                        .clone(),
                    &message,
                    &self.output,
                )
                .clone()
            })
            .collect();
    }
}

pub fn create_script_message(
    index: usize,
    unsigned_tx: &Transaction,
    prevouts: &Vec<TxOut>,
    target_script: &Script,
) -> Message {
    let sighash = SighashCache::new(unsigned_tx)
        .taproot_script_spend_signature_hash(
            index,
            &Prevouts::All(&prevouts),
            ScriptPath::with_defaults(&target_script),
            SchnorrSighashType::AllPlusAnyoneCanPay,
        )
        .unwrap();

    return Message::from_slice(&sighash).unwrap();
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

    let binding = output.clone().tap_tree.unwrap();

    let target_script = binding.script_leaves().next().unwrap().script();

    let control = tap_info.control_block(&(target_script.clone(), LeafVersion::TapScript));

    let verify = control.as_ref().unwrap().verify_taproot_commitment(
        &secp(),
        tap_info.output_key().to_inner(),
        &target_script,
    );

    if !verify {
        panic!("invalid block {:#?}", control.unwrap());
    }

    let sig = secp().sign_schnorr(&message, &secret_key.keypair(&secp()));

    let tap_leaf_hash = TapLeafHash::from_script(&target_script, LeafVersion::TapScript);

    let mut input = input.clone();

    input.witness_script = Some(tx_out.script_pubkey.clone());

    input.witness_utxo = Some(tx_out.clone());

    input.tap_merkle_root = tap_info.merkle_root();

    input.tap_scripts.insert(
        control.unwrap(),
        (target_script.clone(), LeafVersion::TapScript),
    );

    let x_only = &secret_key.x_only_public_key(&secp()).0;

    let schnorr_sig = SchnorrSig {
        sig,
        hash_ty: bitcoin::SchnorrSighashType::AllPlusAnyoneCanPay,
    };

    input
        .tap_script_sigs
        .insert((x_only.clone(), tap_leaf_hash), schnorr_sig);

    return input;
}