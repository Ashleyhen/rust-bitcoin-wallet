use std::collections::BTreeMap;

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::Input,
    schnorr::TapTweak,
    secp256k1::{All, Message, Secp256k1},
    util::{
        bip32::ExtendedPrivKey,
        sighash::{Prevouts, SighashCache},
    },
    EcdsaSig, EcdsaSighashType, PublicKey, SchnorrSig, SchnorrSighashType, Script, Transaction,
    TxOut,
};
use miniscript::ToPublicKey;

pub struct SignTx {
    extended_priv_k: ExtendedPrivKey,
    index: usize,
    current_tx: Transaction,
    previous_tx: Vec<TxOut>,
    secp: Secp256k1<All>,
}

impl SignTx {
    pub fn new(
        extended_priv_k: ExtendedPrivKey,
        index: usize,
        current_tx: Transaction,
        previous_tx: Vec<TxOut>,
        secp: Secp256k1<All>,
    ) -> Self {
        return SignTx {
            extended_priv_k,
            index,
            current_tx,
            previous_tx,
            secp,
        };
    }

    pub fn pub_key_unlock(&self) -> Input {
        
        let tweaked_key_pair = self
            .extended_priv_k
            .to_keypair(&self.secp)
            .tap_tweak(&self.secp, None)
            .into_inner();
        let sig_hash = SighashCache::new(&mut self.current_tx.clone())
            .taproot_key_spend_signature_hash(
                self.index,
                &Prevouts::All(&self.previous_tx),
                SchnorrSighashType::AllPlusAnyoneCanPay,
            )
            .unwrap();
        let msg = Message::from_slice(&sig_hash).unwrap();
        let signed_shnorr = self.secp.sign_schnorr(&msg, &tweaked_key_pair);
        let schnorr_sig = SchnorrSig {
            sig: signed_shnorr,
            hash_ty: SchnorrSighashType::AllPlusAnyoneCanPay,
        };
        let mut input = Input::default();
        input.tap_key_sig = Some(schnorr_sig);
        return input;
    }

    pub fn p2wpkh_script_sign(&self) -> Input {
        let mut input = Input::default();
        let b_tree: BTreeMap<PublicKey, EcdsaSig> = self
            .previous_tx
            .iter()
            .map(|witness| {
                input.witness_utxo = Some(witness.clone());
                let sig_hash = SighashCache::new(&mut self.current_tx.clone())
                    .segwit_signature_hash(
                        self.index,
                        &p2wpkh_script_code(&witness.script_pubkey),
                        witness.value,
                        EcdsaSighashType::All,
                    )
                    .unwrap();
                let msg = Message::from_slice(&sig_hash).unwrap();
                let sig = EcdsaSig::sighash_all(
                    self.secp
                        .sign_ecdsa(&msg, &self.extended_priv_k.private_key),
                );
                let pub_key = self
                    .extended_priv_k
                    .to_keypair(&self.secp)
                    .public_key()
                    .to_public_key();
                return (pub_key, sig);
            })
            .collect();
        input.partial_sigs = b_tree;
        return input;
    }
}

fn p2wpkh_script_code(script: &Script) -> Script {
    Builder::new()
        .push_opcode(opcodes::all::OP_DUP)
        .push_opcode(opcodes::all::OP_HASH160)
        .push_slice(&script[2..])
        .push_opcode(opcodes::all::OP_EQUALVERIFY)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .into_script()
}
