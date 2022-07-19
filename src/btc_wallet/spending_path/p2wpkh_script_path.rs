use std::collections::BTreeMap;

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::{Input, Output},
    secp256k1::{All, Message, Secp256k1},
    util::{bip32::ExtendedPrivKey, sighash::SighashCache},
    EcdsaSig, EcdsaSighashType, PublicKey, Script, Transaction, TxIn, TxOut,
};
use miniscript::ToPublicKey;

use crate::btc_wallet::{
    address_formats::AddressSchema,
    wallet_methods::{ClientWallet, NETWORK},
};

use super::{Vault, standard_lock, standard_extraction};


#[derive(Clone)]
pub struct P2PWKh {
    pub client_wallet: ClientWallet,
    amount: u64,
    to_addr: String,
}

impl Vault for P2PWKh {
    fn unlock_key(&self, previous_tx: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        let wallet_keys = self.client_wallet.create_wallet(
            self.wallet_purpose(),
            self.client_wallet.recieve,
            self.client_wallet.change,
        );
        let (signer_pub_k, (_, signer_dp)) = wallet_keys.clone();
        let secp = &self.client_wallet.secp;
        let ext_prv = ExtendedPrivKey::new_master(NETWORK, &self.client_wallet.seed)
            .unwrap()
            .derive_priv(&secp, &signer_dp)
            .unwrap();

        // confirm
        let input_list: Vec<Input> = previous_tx
            .iter()
            .enumerate()
            .map(|(i, previous_tx)| {
                let mut input_tx = self.p2wpkh_script_sign(
                    self.wallet_purpose(),
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

    fn lock_key<'a, S>(&self, schema: &'a S, total: u64) -> Vec<(Output,u64)>
    where
        S: AddressSchema,
    {
        let cw = schema.to_wallet();
        let extend_pub_k = cw
            .create_wallet(schema.wallet_purpose(), cw.recieve, cw.change + 1)
            .0;
        return standard_lock(schema, self.amount, total, extend_pub_k, &self.to_addr);
        
    }

  
}
impl P2PWKh {
    pub fn new(client_wallet: ClientWallet, amount: u64, to_addr: String) -> Self {
        return P2PWKh {
            client_wallet,
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
        let cw = self.client_wallet.clone();
        let extend_pub_k = cw.create_wallet(purpose, cw.recieve, cw.change).0;

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
                let sig =
                    EcdsaSig::sighash_all(cw.secp.sign_ecdsa(&msg, &extended_priv_k.private_key));
                let pub_key = extended_priv_k
                    .to_keypair(&cw.secp)
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
