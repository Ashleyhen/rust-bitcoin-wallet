use std::collections::BTreeMap;

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::{Input, Output},
    secp256k1::{All, Message, Secp256k1},
    util::{bip32::ExtendedPrivKey, sighash::SighashCache},
    EcdsaSig, EcdsaSighashType, PublicKey, Script, Transaction, TxIn, TxOut,
};
use miniscript::ToPublicKey;



use crate::bitcoin_wallet::{address_formats::{p2wpkh_addr_fmt::P2WPKH, AddressSchema}, constants::NETWORK};

use super::{standard_create_tx, standard_lock, Vault};

#[derive(Clone)]
pub struct P2WPKHVault<'a> {
    p2wpkh: &'a P2WPKH,
    amount: u64,
    to_addr: String,
}

impl<'a> Vault for P2WPKHVault<'a> {
    fn create_tx(&self, output_list: &Vec<Output>, tx_in: Vec<TxIn>, total: u64) -> Transaction {
        return standard_create_tx(self.amount, output_list, tx_in, total);
    }

    fn lock_key(&self) -> Vec<Output> {
        let cw = self.p2wpkh.to_wallet();
        let extend_pub_k = self.p2wpkh.get_ext_pub_key();
        return standard_lock(self.p2wpkh, extend_pub_k, &self.to_addr);
    }

    fn unlock_key(&self, previous_tx: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        let cw = self.p2wpkh.get_client_wallet();
        let signer_pub_k = self.p2wpkh.get_client_wallet();
        let signer_dp = self.p2wpkh.get_derivation_p();

        let secp = self.p2wpkh.to_wallet().secp;
        let ext_prv = ExtendedPrivKey::new_master(NETWORK, &cw.seed)
            .unwrap()
            .derive_priv(&secp, &signer_dp)
            .unwrap();

        // confirm
        let input_list: Vec<Input> = previous_tx
            .iter()
            .enumerate()
            .map(|(i, previous_tx)| {
                let mut input_tx = self.p2wpkh_script_sign(
                    self.p2wpkh.wallet_purpose(),
                    i,
                    current_tx.clone(),
                    previous_tx.output.clone(),
                    secp.clone(),
                    ext_prv,
                );
                input_tx.non_witness_utxo = Some(previous_tx.clone());
                return input_tx;
            })
            .collect();
        return input_list;
    }
}
impl<'a> P2WPKHVault<'a> {
    pub fn new(p2wpkh: &'a P2WPKH, amount: u64, to_addr: String) -> Self {
        return P2WPKHVault {
            p2wpkh,
            amount,
            to_addr,
        };
    }

    fn p2wpkh_script_sign(
        &self,
        purpose: u32,
        index: usize,
        current_tx: Transaction,
        previous_tx: Vec<TxOut>,
        secp: Secp256k1<All>,
        extended_priv_k: ExtendedPrivKey,
    ) -> Input {
        let cw = self.p2wpkh.clone();
        let extend_pub_k = self.p2wpkh.get_ext_pub_key();

        let mut input = Input::default();
        let b_tree: BTreeMap<PublicKey, EcdsaSig> = previous_tx
            .iter()
            .map(|witness| {
                input.witness_utxo = Some(witness.clone());
                let sig_hash = SighashCache::new(&mut current_tx.clone())
                    .segwit_signature_hash(
                        index,
                        &p2wpkh_script_code(&witness.script_pubkey),
                        witness.value,
                        EcdsaSighashType::All,
                    )
                    .unwrap();
                let msg = Message::from_slice(&sig_hash).unwrap();
                let sig = EcdsaSig::sighash_all(
                    cw.to_wallet()
                        .secp
                        .sign_ecdsa(&msg, &extended_priv_k.private_key),
                );
                let pub_key = extended_priv_k
                    .to_keypair(&cw.to_wallet().secp)
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
