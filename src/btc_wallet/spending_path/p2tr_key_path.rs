use std::{str::FromStr, sync::Arc};

use crate::btc_wallet::{
    address_formats::{AddressSchema, p2tr_addr_fmt::P2TR}, constants::NETWORK, wallet_methods::ClientWallet,
};
use bitcoin::{
    psbt::{Input, Output},
    schnorr::TapTweak,
    secp256k1::{All, Message, Secp256k1},
    util::{
        bip32::{ExtendedPrivKey, ExtendedPubKey},
        sighash::{Prevouts, SighashCache},
    }, SchnorrSig, SchnorrSighashType, Script,
    Transaction, TxIn, TxOut
};

use super::{standard_create_tx, standard_lock, Vault};

#[derive(Clone)]
pub struct P2TRVault<'a> {
    p2tr: &'a P2TR,
    amount: u64,
    to_addr: String,
}

impl<'a> Vault for P2TRVault<'a> {
    fn unlock_key(&self, previous_tx: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        let schema=self.p2tr;
        let cw= &schema.to_wallet();
        let secp = &cw.secp;
        let wallet_key = cw.create_wallet(schema.wallet_purpose(), cw.recieve, cw.change);

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
                            .eq(&schema.map_ext_keys(&signer_pub_k).script_pubkey())
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

    fn lock_key<'s, S>(&self, schema: &'s S) -> Vec<Output>
    where
        S: AddressSchema,
    {
        let cw = schema.to_wallet();
        let change_address = cw
            .create_wallet(schema.wallet_purpose(), cw.recieve, cw.change + 1)
            .0;

        return standard_lock(schema, change_address, &self.to_addr);
    }

    fn create_tx(&self, output_list: &Vec<Output>, tx_in: Vec<TxIn>, total: u64) -> Transaction {
        return standard_create_tx(self.amount, output_list, tx_in, total);
    }
}

impl<'a> P2TRVault<'a> {
    pub fn new(p2tr: &'a P2TR, amount: u64, to_addr: &String) -> Self {
        return P2TRVault {
            p2tr,
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
        let cw = self.p2tr.to_wallet();
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
}
