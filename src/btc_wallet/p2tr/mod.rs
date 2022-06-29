use std::{borrow::Borrow, collections::BTreeMap, str::FromStr};

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::Input,
    schnorr::{TapTweak, TweakedPublicKey, UntweakedPublicKey},
    secp256k1::{schnorr::Signature, schnorrsig::PublicKey, All, Message, Parity, Secp256k1},
    util::{
        address::Payload,
        bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey, KeySource},
        sighash::{Prevouts, SighashCache},
        taproot::{LeafVersion::TapScript, TapLeafHash},
    },
    Address, KeyPair, SchnorrSig, SchnorrSighashType, Script, Transaction, TxIn, TxOut,
    XOnlyPublicKey,
};
use miniscript::{interpreter::KeySigPair, ToPublicKey};

use super::{
    utils::UnlockAndSend, AddressSchema, Broadcast_op, ClientWallet, SignTx, WalletKeys, NETWORK,
};

#[derive(Clone)]
pub struct P2TR(pub ClientWallet);

impl AddressSchema for P2TR {
    fn map_ext_keys(&self, recieve: &bitcoin::util::bip32::ExtendedPubKey) -> bitcoin::Address {
        return Address::p2tr(&self.0.secp, recieve.to_x_only_pub(), None, NETWORK);
    }

    fn new(seed: Option<String>, recieve: u32, change: u32) -> Self {
        return P2TR(ClientWallet::new(seed, recieve, change));
    }

    fn to_wallet(&self) -> ClientWallet {
        return self.0.clone();
    }

    fn wallet_purpose(&self) -> u32 {
        return 341;
    }

    fn prv_tx_input(
        &self,
        previous_tx: Vec<Transaction>,
        current_tx: Transaction,
        signing_fn: &dyn Fn(SignTx) -> Input,
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
            .flat_map(|(i, previous_tx)| {
                let input_list: Vec<Input> = previous_tx
                    .output
                    .clone()
                    .iter()
                    .filter(|tx_out| {
                        tx_out
                            .script_pubkey
                            .eq(&self.map_ext_keys(&signer_pub_k).script_pubkey())
                    })
                    .map(|utxo| {
                        let sign_tx = SignTx::new(
                            ext_prv,
                            i,
                            current_tx.clone(),
                            previous_tx.output.clone(),
                            secp.clone(),
                        );
                        let mut new_input = signing_fn(sign_tx);

                        new_input.witness_utxo = Some(utxo.clone());
                        let tap_leaf_hash_list =
                            TapLeafHash::from_script(&utxo.script_pubkey, TapScript);
                        // new_input.tap_key_origins=tap_key_origin;

                        new_input.non_witness_utxo = Some(previous_tx.clone());
                        new_input.tap_internal_key = Some(signer_pub_k.to_x_only_pub());
                        return new_input;

                        // tx_out;
                    })
                    .collect();

                // let mut tap_key_origin=BTreeMap::new();

                // tap_key_origin.insert
                // (signer_pub_k.to_x_only_pub(),
                // (tap_leaf_hash_list,(signer_finger_p.clone(),signer_dp.clone())));

                // let tweaked_key_pair=ext_prv.to_keypair(&secp).tap_tweak(&secp,None).into_inner();
                // let sig_hash=SighashCache::new(&mut current_tx.clone())
                //           .taproot_key_spend_signature_hash( i, &Prevouts::All(&previous_tx.output), SchnorrSighashType::AllPlusAnyoneCanPay).unwrap();
                // let msg=Message::from_slice(&sig_hash).unwrap();

                // let signed_shnorr=secp.sign_schnorr(&msg, &tweaked_key_pair);

                // let schnorr_sig=SchnorrSig{sig:signed_shnorr, hash_ty:SchnorrSighashType::AllPlusAnyoneCanPay};

                // new_input.tap_key_sig=Some(schnorr_sig.clone());
                // if(broadcast_op.eq(&Broadcast_op::Finalize)){
                //   secp.verify_schnorr(&signed_shnorr, &msg, &XOnlyPublicKey::from_slice(&uxto.script_pubkey[2..]).unwrap()).is_ok();
                // }
                return input_list;
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
}
