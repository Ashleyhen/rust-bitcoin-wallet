use std::{collections::BTreeMap, io::Read, str::FromStr, sync::Arc};

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::{Input, Output},
    secp256k1::{Message, SecretKey},
    util::{
        bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey, KeySource},
        sighash::SighashCache,
    },
    Address, EcdsaSig, EcdsaSighashType, Script, Sighash, Transaction, TxIn, TxOut,
};
use miniscript::ToPublicKey;

use crate::btc_wallet::{wallet_methods::{ClientWallet, NETWORK}};

use super::{
    address_schema, SignTx, WalletKeys, AddressSchema, 
};

#[derive(Clone)]
pub struct P2PWKh(pub ClientWallet);

impl AddressSchema for P2PWKh {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> bitcoin::Address {
        return Address::p2wpkh(&recieve.public_key.to_public_key(), NETWORK).unwrap();
    }

    fn new(seed: Option<String>, recieve: u32, change: u32) -> Self {
        return P2PWKh(ClientWallet::new(seed, recieve, change));
    }

    fn to_wallet(&self) -> ClientWallet {
        return self.0.clone();
    }

    fn wallet_purpose(&self) -> u32 {
        return 84;
    }

    fn prv_tx_input(
        &self,
        previous_tx: Vec<Transaction>,
        current_tx: Transaction,
        unlocking_fn: &dyn Fn(SignTx) -> Input,
    ) -> Vec<Input> {
        let wallet_keys =
            self.0
                .create_wallet(self.wallet_purpose(), self.0.recieve, self.0.change);
        let (signer_pub_k, (_, signer_dp)) = wallet_keys.clone();
        let secp = &self.0.secp;
        let ext_prv = ExtendedPrivKey::new_master(NETWORK, &self.0.seed)
            .unwrap()
            .derive_priv(&secp, &signer_dp)
            .unwrap();

        // confirm
        let input_list: Vec<Input> = previous_tx
            .iter()
            .enumerate()
            .map(|(i, previous_tx)| {
                let sign_tx = SignTx::new(
                    ext_prv,
                    i,
                    current_tx.clone(),
                    previous_tx.output.clone(),
                    secp.clone(),
                );
                let mut input_tx = unlocking_fn(sign_tx);
                input_tx.non_witness_utxo = Some(previous_tx.clone());
                return input_tx;
            })
            .collect();
        return input_list;
    }
}
