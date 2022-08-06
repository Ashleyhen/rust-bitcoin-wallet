use std::collections::BTreeMap;

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    hashes::{sha256, Hash},
    psbt::{Input, Output, PartiallySignedTransaction, TapTree},
    schnorr::TapTweak,
    secp256k1::{ecdh::SharedSecret, All, Secp256k1, SecretKey, Parity},
    util::{
        bip32::{DerivationPath, Fingerprint},
        taproot::{
            LeafVersion, TapBranchHash, TapLeafHash, TapLeafTag, TaprootBuilder, TaprootSpendInfo, ControlBlock, TaprootMerkleBranch,
        },
    },
    KeyPair, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};
use miniscript::ToPublicKey;

use crate::btc_wallet::{
    address_formats::{p2tr_addr_fmt::P2TR, AddressSchema},
    constants::TIP,
};

use super::Vault;

pub struct MultiSigPath<'a, 'b, 's> {
    pub p2tr: &'a P2TR,
    optional_psbt: Option<&'b PartiallySignedTransaction>,
    script: &'s Script,
}

impl<'a, 'b, 's> Vault for MultiSigPath<'a, 'b, 's> {
    fn create_tx(
        &self,
        output_list: &Vec<Output>,
        tx_in: Vec<TxIn>,
        total: u64,
    ) -> bitcoin::Transaction {
		let outputs=self.optional_psbt.map(|o|o.outputs.clone()).unwrap_or(output_list.to_vec());
        // TODO fix filter 
        let value=self.optional_psbt.map(|f|
            f.clone().extract_tx().output.iter().map(|a|a.value).sum::<u64>()
        ).unwrap_or_else(||total - TIP);

            let tweak_pub_k=outputs
            .iter()
            .map(|out_put| {
				if (out_put.tap_key_origins.len()==1){
					return self.script.clone();
				}
                let secp = self.p2tr.to_wallet().secp;
                let tap_leaf_hash: Vec<TapLeafHash> = out_put
                    .tap_key_origins
                    .values()
                    .into_iter()
                    .flat_map(|f| f.0.clone())
                    .collect();
				
                let branch = TapBranchHash::from_node_hashes(
                    sha256::Hash::from_inner(tap_leaf_hash[0].into_inner()),
                    sha256::Hash::from_inner(tap_leaf_hash[1].into_inner()),
                );
                return Script::new_v1_p2tr(&secp, out_put.tap_internal_key.unwrap(), Some(branch));
            })
            .collect::<Vec<Script>>();
        return Transaction {
            version: 2,
            lock_time: 0,
            input: tx_in,
            output: vec![TxOut {
                value,
                script_pubkey: tweak_pub_k[0].clone(),
            }],
        };
    }

    fn lock_key(&self) -> Vec<Output> {
        let secp = self.p2tr.to_wallet().secp;
        let output_list = self
            .optional_psbt
            .map(|f| f.outputs.clone())
            .unwrap_or_else(|| vec![Output::default()]);

        return output_list
            .iter()
            .map(|output| {
                let mut tap_key_origins = output.tap_key_origins.clone();
                
                let tap_leaf_hash=vec![(TapLeafHash::from_script(
                    self.script,
                    LeafVersion::TapScript,
                ))];

                tap_key_origins.insert(
                    self.p2tr.get_ext_pub_key().to_x_only_pub(),
                    (
                        tap_leaf_hash,
                        (Fingerprint::default(), DerivationPath::default()),
                    ),
                );
                let mut result = output.clone();
         

                result.tap_internal_key = Some(
                    KeyPair::from_secret_key(
                        &secp,
                        self.sharedSecret(result.tap_key_origins.into_keys()),
                    )
                    .public_key(),
                );
                result.tap_key_origins = tap_key_origins;
                return result;
            })
            .collect();

        // todo!()
    }

    fn unlock_key(&self, previous: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        dbg!(self.optional_psbt.unwrap());
        dbg!(self.p2tr.get_ext_pub_key().public_key.to_x_only_pubkey());
        let output=self.optional_psbt.unwrap().outputs[0].clone();
        let leaf_list:Vec<TapLeafHash>=output.tap_key_origins.iter().filter(
            |p|p.0.ne(&self.p2tr.get_ext_pub_key().public_key.to_x_only_pubkey()))
            .flat_map(|(x_only,(tap_leaf,_))|tap_leaf.clone()).
            collect();

            // dbg!(leaf_list);
        // output.tap_key_origins.iter().map(|f|)
        if(leaf_list.is_empty()){
            return self.optional_psbt.unwrap().inputs.clone();
        }

        let actual_control = ControlBlock {
        leaf_version: LeafVersion::TapScript,
        output_key_parity: Parity::Odd,
        internal_key:output.tap_internal_key.unwrap().to_x_only_pubkey(),
        merkle_branch: TaprootMerkleBranch::from_slice(&leaf_list[0]).unwrap(),
    };
let mut bTreeMap = BTreeMap::<ControlBlock, (Script, LeafVersion)>::default();
    bTreeMap.insert(actual_control.clone(), (self.script.clone(), LeafVersion::TapScript));

    let mut input=Input::default();
    input.tap_scripts=bTreeMap;
    input.witness_utxo=Some(previous[0].clone().output[0].clone());


        dbg!(TapLeafHash::from_script(self.script,LeafVersion::TapScript));
        let res = actual_control.verify_taproot_commitment(&self.p2tr.to_wallet().secp, output.tap_internal_key.unwrap().to_x_only_pubkey(), &self.script);
        dbg!(res);
        return self.optional_psbt.unwrap().inputs.clone();
    }

}
impl<'a, 'b, 's> MultiSigPath<'a, 'b, 's> {
    pub fn new(
        p2tr: &'a P2TR,
        optional_psbt: Option<&'b PartiallySignedTransaction>,
        script: &'s Script,
    ) -> Self {
        return MultiSigPath {
            p2tr,
            optional_psbt,
            script,
        };
    }

    pub fn sharedSecret(&self, mut iter: impl Iterator<Item = XOnlyPublicKey>) -> SecretKey {
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

    pub fn bob_script(public_key: XOnlyPublicKey, preimage_hash: &[u8]) -> Script {
        return Builder::new()
            .push_opcode(all::OP_SHA256)
            .push_slice(&preimage_hash)
            .push_opcode(all::OP_EQUALVERIFY)
            .push_x_only_key(&public_key)
            .push_opcode(all::OP_CHECKSIG)
            .into_script();
    }
}
