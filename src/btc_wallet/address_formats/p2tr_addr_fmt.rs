use std::str::FromStr;

use bitcoin::{
    psbt::Input, util::bip32::ExtendedPrivKey, Address, Transaction, TxOut, XOnlyPublicKey,
};
use miniscript::ToPublicKey;

use crate::btc_wallet::{
    spending_path::p2tr_key_path::P2TR,
    wallet_methods::{ClientWallet, NETWORK},
};

use super::AddressSchema;

impl AddressSchema for P2TR {
    fn map_ext_keys(&self, recieve: &bitcoin::util::bip32::ExtendedPubKey) -> bitcoin::Address {
        return Address::p2tr(
            &self.get_client_wallet().secp,
            recieve.to_x_only_pub(),
            None,
            NETWORK,
        );
    }

    // fn new(seed: Option<String>, recieve: u32, change: u32) -> Self {
    //     return P2TR(ClientWallet::new(seed, recieve, change));
    // }

    fn to_wallet(&self) -> ClientWallet {
        return self.get_client_wallet().clone();
    }

    fn wallet_purpose(&self) -> u32 {
        return 341;
    }
    /*
        fn prv_tx_input(
            &self,
            previous_tx: Vec<Transaction>,
            current_tx: Transaction,
            unlocking_fn: &dyn Fn(SignTx) -> Input,
        ) -> Vec<Input> {
            let secp = self.clone().0.secp;
            let wallet_key = self
                .0
                .create_wallet(self.wallet_purpose(), self.0.recieve, self.0.change);

            let (signer_pub_k, (signer_finger_p, signer_dp)) = wallet_key.clone();

            let ext_prv = ExtendedPrivKey::new_master(NETWORK, &self.0.seed)
                .unwrap()
                .derive_priv(&secp, &signer_dp)
                .unwrap();

            let input_list: Vec<Input> = previous_tx
                .clone()
                .iter()
                .enumerate()
                .flat_map(|(i, prev_tx)| {
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
                            let sign_tx = SignTx::new(
                                ext_prv,
                                i,
                                current_tx.clone(),
                                tx_out_list.to_vec(),
                                secp.clone(),
                            );
                            let mut new_input = unlocking_fn(sign_tx);

                            new_input.witness_utxo = Some(utxo.clone());
                            return new_input;
                        })
                        .collect();
                    return inputs;
                })
                .collect();
            return input_list;
        }
    }

    impl P2TR {
        pub fn aggregate(&self, address_list: Vec<String>) -> String {
            let wallet_key = self
                .0
                .create_wallet(self.wallet_purpose(), self.0.recieve, self.0.change);
            let (signer_pub_k, (signer_finger_p, signer_dp)) = wallet_key.clone();
            let secp = self.0.secp.clone();

            return address_list
                .iter()
                .map(|address| {
                    let addr = Address::from_str(address).unwrap();
                    let x_only_pub_k = signer_pub_k
                        .public_key
                        .to_public_key()
                        .inner
                        .combine(
                            &XOnlyPublicKey::from_slice(&addr.script_pubkey()[2..])
                                .unwrap()
                                .to_public_key()
                                .inner,
                        )
                        .unwrap()
                        .to_x_only_pubkey();
                    let address = Address::p2tr(&secp, x_only_pub_k, None, NETWORK);
                    return address.to_string();
                })
                .last()
                .unwrap();
        }
     */
}
