use std::collections::BTreeMap;

use bitcoin::{
    psbt::{Input, Output, PartiallySignedTransaction},
    secp256k1::{ecdh::SharedSecret, SecretKey},
    util::{
        bip32::{DerivationPath, Fingerprint},
        taproot::{LeafVersion, TapLeafHash, TapLeafTag, TapBranchHash, TaprootSpendInfo},
    },
    KeyPair, Script, Transaction, TxIn, XOnlyPublicKey, hashes::{sha256, Hash}, schnorr::TapTweak, TxOut,
};
use miniscript::ToPublicKey;

use crate::btc_wallet::{address_formats::{p2tr_addr_fmt::P2TR, AddressSchema}, constants::TIP};

use super::Vault;

pub struct MultiSigPath<'a, 'b> {
    pub p2tr: &'a P2TR,
    to_addr: Vec<String>,
    optional_psbt: Option<&'b PartiallySignedTransaction>,
    script: Script,
}

impl<'a, 'b> Vault for MultiSigPath<'a, 'b> {
    fn create_tx(
        &self,
        output_list: &Vec<Output>,
        tx_in: Vec<TxIn>,
        total: u64,
    ) -> bitcoin::Transaction {

		let tweak_pub_k=self.optional_psbt.unwrap().outputs.iter().map(|out_put|{
		let secp=self.p2tr.to_wallet().secp;
		let tap_leaf_hash:Vec<TapLeafHash>=out_put.tap_key_origins.values().into_iter().flat_map(|f|f.0.clone()).collect();
		let internal=KeyPair::from_secret_key(&secp,self.sharedSecret(out_put.tap_key_origins.keys().into_iter()));
		let branch=TapBranchHash::from_node_hashes(
			sha256::Hash::from_inner(tap_leaf_hash[0].into_inner()),
			sha256::Hash::from_inner(tap_leaf_hash[1].into_inner()));
		return Script::new_v1_p2tr(&secp, internal.public_key(), Some(branch));
		}).collect::<Vec<Script>>();
		return Transaction{ version: 2, lock_time: 0, input:tx_in, output: vec![TxOut{ value: total-TIP, script_pubkey:tweak_pub_k[0].clone() }] };
    }

    fn lock_key(&self) -> Vec<Output> {
        let output_list = self
            .optional_psbt
            .map(|f| f.outputs.clone())
            .unwrap_or_else(|| vec![Output::default()]);

        return output_list
            .iter()
            .map(|output| {
                let mut result = output.clone();

                let mut tap_key_origins = output.tap_key_origins.clone();

                let mut tap_leaf_hash = output
                    .tap_key_origins
                    .values()
                    .into_iter()
                    .map(|value| value.clone().0)
                    .flatten()
                    .collect::<Vec<TapLeafHash>>();

                tap_leaf_hash.push(TapLeafHash::from_script(
                    &self.script,
                    LeafVersion::TapScript,
                ));
                tap_key_origins.insert(
                    self.p2tr.get_ext_pub_key().to_x_only_pub(),
                    (
                        tap_leaf_hash,
                        (Fingerprint::default(), DerivationPath::default()),
                    ),
                );
                result.tap_key_origins = tap_key_origins;
                return result;
            })
            .collect();

        // todo!()
    }

    fn unlock_key(&self, previous: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        todo!()
    }
}
impl<'a, 'b> MultiSigPath<'a, 'b> {
    pub fn new(
        p2tr: &'a P2TR,
        to_addr: Vec<String>,
        optional_psbt: Option<&'b PartiallySignedTransaction>,
        script: Script,
    ) -> Self {
        return MultiSigPath {
            p2tr,
            to_addr,
            optional_psbt,
            script,
        };
    }

    pub fn sharedSecret(&self, mut iter: impl Iterator<Item = &'a XOnlyPublicKey>) -> SecretKey {
        {
            match iter.next() {
                Some(x_only) => {
                    return SecretKey::from_slice(
                        &SharedSecret::new(&x_only.to_public_key().inner, &self.sharedSecret(iter))
                            .secret_bytes(),
                    )
                    .unwrap()
                }

                None => SecretKey::from_slice(&self.p2tr.get_client_wallet().seed).unwrap(),
            }
        }
    }

	
}
