use std::{collections::BTreeMap, str::FromStr};

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::{Input, Output, PartiallySignedTransaction, TapTree},
    schnorr::TapTweak,
    secp256k1::Message,
    util::{
        bip32::ExtendedPubKey,
        sighash::{Prevouts, SighashCache},
        taproot::{LeafVersion, TapLeafHash, TapLeafTag, TapSighashHash, TaprootBuilder},
    },
    Address, SchnorrSig, SchnorrSighashType, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};
use electrum_client::GetMerkleRes;

use crate::btc_wallet::{
    address_formats::{p2tr_addr_fmt::P2TR, AddressSchema},
    constants::TIP,
};

use super::Vault;

pub struct P2trMultisig<'a, 'b> {
    pub p2tr: &'a P2TR,
    to_addr: Vec<String>,
    psbt: Option<&'b PartiallySignedTransaction>,
}
fn dynamic_builder(mut iter: impl Iterator<Item = XOnlyPublicKey>) -> Builder {
    return match iter.next() {
        Some(data) => dynamic_builder(iter)
            .push_x_only_key(&data)
            .push_opcode(opcodes::all::OP_CHECKSIGADD),
        None => Builder::new(),
    };
}

impl<'a, 'b> Vault for P2trMultisig<'a, 'b> {
    fn unlock_key(&self, previous: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        let cw = self.p2tr.get_client_wallet();
        let psbt = self.psbt.clone().unwrap();
        return psbt
            .outputs
            .iter()
            .flat_map(|output_ref| {
                let mut output = output_ref.clone();
                return previous
                    .iter()
                    .enumerate()
                    .flat_map(|(index, tx)| {
                        return tx
                            .output
                            .iter()
                            .filter(|s| {
                                s.script_pubkey
                                    .eq(output.witness_script.get_or_insert(Script::new()))
                            })
                            .map(|tx_out| {
                                let leaf_hash = TapLeafHash::from_script(
                                    &tx_out.script_pubkey,
                                    LeafVersion::TapScript,
                                );
                                let tap_builder = TaprootBuilder::new();
                                let root = tap_builder
                                    .finalize(&cw.secp, output.tap_internal_key.unwrap())
                                    .unwrap()
                                    .merkle_root();

                                let tweaked_key_pair = self
                                    .p2tr
                                    .get_ext_prv_k()
                                    .to_keypair(&cw.secp)
                                    .tap_tweak(&cw.secp, root)
                                    .into_inner();

                                let sig_hash = SighashCache::new(&mut current_tx.clone())
                                    .taproot_script_spend_signature_hash(
                                        index,
                                        &Prevouts::All(&current_tx.output),
                                        leaf_hash,
                                        SchnorrSighashType::AllPlusAnyoneCanPay,
                                    )
                                    .unwrap();

                                let msg = Message::from_slice(&sig_hash).unwrap();

                                let signed_shnorr = cw.secp.sign_schnorr(&msg, &tweaked_key_pair);
                                let schnorr_sig = SchnorrSig {
                                    sig: signed_shnorr,
                                    hash_ty: SchnorrSighashType::AllPlusAnyoneCanPay,
                                };
                                // let mut input =psbt.inputs.iter();
                                let mut input = Input::default();
                                // input.tap_merkle_root=self.get_script()
                                // input.tap_script_sigs = Some(schnorr_sig);
                                input.witness_utxo = Some(tx_out.clone());
                                return input.clone();
                            })
                            .collect::<Vec<Input>>();
                    })
                    .collect::<Vec<Input>>();
            })
            .collect::<Vec<Input>>();
        // return psbt.clone().inputs;
    }

    fn lock_key<'s, S>(&self, schema: &'s S) -> Vec<Output>
    where
        S: AddressSchema,
    {
        return self
            .psbt
            .as_ref()
            .map(|f| f.outputs.clone())
            .unwrap_or(self.create_lock(schema));
    }

    fn create_tx(&self, output_list: &Vec<Output>, tx_in: Vec<TxIn>, total: u64) -> Transaction {
        return self
            .psbt
            .clone()
            .map(|f| f.clone().extract_tx())
            .unwrap_or_else(|| {
                let tx_out = TxOut {
                    value: total - TIP,
                    script_pubkey: output_list.get(0).unwrap().clone().witness_script.unwrap(),
                };
                return Transaction {
                    version: 2,
                    lock_time: 0,
                    input: tx_in,
                    output: vec![tx_out],
                };
            });
    }
}

impl<'a, 'b> P2trMultisig<'a, 'b> {
    pub fn new(
        p2tr: &'a P2TR,
        to_addr: Vec<String>,
        psbt: Option<&'b PartiallySignedTransaction>,
    ) -> Self {
        return P2trMultisig {
            p2tr,
            to_addr,
            psbt,
        };
    }

    fn create_lock<'s, S>(&self, schema: &'s S) -> Vec<Output>
    where
        S: AddressSchema,
    {
        let script = self.get_script();

        let tap_leaf = TapLeafHash::from_script(&script, LeafVersion::TapScript);
        let tap_builder = TaprootBuilder::new().add_leaf(0, script).unwrap();
        let internal = schema.get_ext_pub_key().to_x_only_pub();
        let script_pub_k = Script::new_v1_p2tr(
            &schema.to_wallet().secp,
            tap_builder
                .clone()
                .finalize(&schema.to_wallet().secp, internal)
                .unwrap()
                .internal_key(),
            None,
        ); // TaprootMerkleBranch

        let mut output = Output::default();
        let mut taproot_origin = BTreeMap::new();
        taproot_origin.insert(
            internal,
            (
                vec![tap_leaf].to_vec(),
                (
                    self.p2tr.get_ext_pub_key().fingerprint(),
                    self.p2tr.get_derivation_p(),
                ),
            ),
        );
        output.tap_tree = Some(TapTree::from_builder(tap_builder).unwrap());
        output.tap_internal_key = Some(internal);
        output.witness_script = Some(script_pub_k);
        output.tap_key_origins = taproot_origin;
        return vec![output];
    }

    fn get_script(&self) -> Script {
        return dynamic_builder(self.to_addr.iter().map(|addr| {
            XOnlyPublicKey::from_slice(&Address::from_str(&addr).unwrap().script_pubkey()[2..])
                .unwrap()
        }))
        .push_int(0)
        .push_opcode(opcodes::all::OP_EQUAL)
        .into_script();
    }
}
