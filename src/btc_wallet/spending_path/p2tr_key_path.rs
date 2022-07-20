use std::{str::FromStr, sync::Arc};

use crate::btc_wallet::{
    address_formats::AddressSchema,
    wallet_methods::{ClientWallet, NETWORK},
};
use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::{Input, Output},
    schnorr::TapTweak,
    secp256k1::{All, Message, Secp256k1},
    util::{
        bip32::{ExtendedPrivKey, ExtendedPubKey},
        sighash::{Prevouts, SighashCache},
    },
    Address, EcdsaSig, EcdsaSighashType, PublicKey, SchnorrSig, SchnorrSighashType, Script,
    Transaction, TxIn, TxOut, XOnlyPublicKey,
};

use super::{ standard_lock, Vault, standard_create_tx};

#[derive(Clone)]
pub struct P2TR {
    client_wallet: ClientWallet,
    amount: u64,
    to_addr: String,
}

impl Vault for P2TR {
    fn unlock_key(&self, previous_tx: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        let cw = self.client_wallet.clone();
        let secp = cw.secp.clone();
        let wallet_key = cw.create_wallet(self.wallet_purpose(), cw.recieve, cw.change);

        let (signer_pub_k, (signer_finger_p, signer_dp)) = wallet_key.clone();

        let ext_prv = ExtendedPrivKey::new_master(NETWORK, &cw.seed)
            .unwrap()
            .derive_priv(&secp, &signer_dp)
            .unwrap();

        let input_list: Vec<Input> = previous_tx
            .clone()
            .iter()
            .enumerate()
            .flat_map(|(index, prev_tx)| {
                let tx_out_list: Vec<TxOut> = prev_tx
                    .output
                    .iter()
                    .filter(|tx_out| {
                        tx_out
                            .script_pubkey
                            .eq(&self.map_ext_keys(&signer_pub_k).script_pubkey())
                    })
                    .map(|f| f.clone())
                    .collect();

                let inputs: Vec<Input> = tx_out_list
                    .iter()
                    .map(|utxo| {
                        let mut new_input =
                            self.pub_key_unlock(index, &current_tx, tx_out_list.to_vec(), ext_prv);
                        new_input.witness_utxo = Some(utxo.clone());
                        return new_input;
                    })
                    .collect();
                return inputs;
            })
            .collect();
        return input_list;
    }

    fn lock_key<'a, S>(&self, schema: &'a S) -> Vec<Output>
    where
        S: AddressSchema,
    {
        let cw = schema.to_wallet();
        let change_address = cw
            .create_wallet(schema.wallet_purpose(), cw.recieve, cw.change + 1)
            .0;

        return standard_lock(schema,  change_address, &self.to_addr);
    }

    fn create_tx(&self, output_list: &Vec<Output>, tx_in: Vec<TxIn>, total: u64) -> Transaction {
        return standard_create_tx(self.amount, output_list, tx_in, total);
    }
}

impl P2TR {
    pub fn get_client_wallet(&self) -> ClientWallet {
        return self.client_wallet.clone();
    }
    pub fn new(client_wallet: ClientWallet, amount: u64, to_addr: &String) -> Self {
        return P2TR {
            client_wallet,
            amount,
            to_addr: to_addr.to_string(),
        };
    }

    pub fn pub_key_unlock(
        &self,
        index: usize,
        current_tx: &Transaction,
        prev_txout: Vec<TxOut>,
        extended_priv_k: ExtendedPrivKey,
    ) -> Input {
        let cw = self.client_wallet.clone();
        let tweaked_key_pair = extended_priv_k
            .to_keypair(&cw.secp)
            .tap_tweak(&cw.secp, None)
            .into_inner();
        let sig_hash = SighashCache::new(&mut current_tx.clone())
            .taproot_key_spend_signature_hash(
                index,
                &Prevouts::All(&prev_txout),
                SchnorrSighashType::AllPlusAnyoneCanPay,
            )
            .unwrap();
        let msg = Message::from_slice(&sig_hash).unwrap();
        let signed_shnorr = cw.secp.sign_schnorr(&msg, &tweaked_key_pair);
        let schnorr_sig = SchnorrSig {
            sig: signed_shnorr,
            hash_ty: SchnorrSighashType::AllPlusAnyoneCanPay,
        };
        let mut input = Input::default();
        input.tap_key_sig = Some(schnorr_sig);
        return input;
    }

    pub fn extract_tx(&self, output_list: Vec<(Output, u64)>, tx_in: Vec<TxIn>) {
        output_list.iter().map(|(output, value)| {
            return TxOut {
                value: *value,
                script_pubkey: Script::new_v1_p2tr(
                    &self.client_wallet.secp,
                    output.tap_internal_key.unwrap(),
                    None,
                ),
            };
        });
    }
}
