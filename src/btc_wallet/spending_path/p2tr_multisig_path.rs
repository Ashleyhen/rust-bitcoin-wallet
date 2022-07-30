use std::{collections::BTreeMap, str::FromStr};

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::{Input, Output, PartiallySignedTransaction, TapTree},
    schnorr::{TapTweak, UntweakedPublicKey},
    secp256k1::{Message, Parity},
    util::{
        bip32::ExtendedPubKey,
        sighash::{Prevouts, SighashCache},
        taproot::{
            LeafVersion, TapLeafHash, TapLeafTag, TapSighashHash, TaprootBuilder,
            TaprootMerkleBranch, ControlBlock,
        },
    },
    Address, SchnorrSig, SchnorrSighashType, Script, Transaction, TxIn, TxMerkleNode, TxOut,
    XOnlyPublicKey,
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

                                
                                let merkle_root= output
                                    .tap_tree
                                    .clone()
                                    .unwrap()
                                    .into_builder()
                                    .finalize(&cw.secp, self.p2tr.get_ext_pub_key().to_x_only_pub())
                                    .map(|op| op.merkle_root())
                                    .ok()
                                    .flatten();
                                let x_only_knows_pub_k =
                                    self.p2tr.get_ext_pub_key().to_x_only_pub();

                                let single_script=Script::new_v1_p2tr( &cw.secp, self.p2tr.get_ext_pub_key().to_x_only_pub(), None,);
                                let tap_leaf_hash = TapLeafHash::from_script(
                                    &single_script,
                                    LeafVersion::TapScript,
                                );
                                let mut b_tree_map = BTreeMap::new();
                                b_tree_map.insert((x_only_knows_pub_k, tap_leaf_hash), schnorr_sig);

                                ;
                                let control_block=ControlBlock{
                                    leaf_version: LeafVersion::TapScript,
                                    output_key_parity: Parity::Odd,
                                    internal_key: UntweakedPublicKey::from_keypair(&self.p2tr.get_ext_prv_k().to_keypair(&cw.secp)),
                                    // internal_key: x_only_knows_pub_k,
                                    merkle_branch:TaprootMerkleBranch::from_slice(&merkle_root.unwrap()).unwrap(),
                                } ;
                                

                                
                                let mut b_tree_control=BTreeMap::new();
                                b_tree_control.insert(control_block, (single_script,LeafVersion::TapScript));
                                
                                input.tap_scripts=b_tree_control;
                                input.tap_merkle_root =merkle_root;
                                input.tap_script_sigs = b_tree_map;
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
        let musig_script = self.get_script();
        let cw = schema.to_wallet();
        let internal = schema.get_ext_pub_key().to_x_only_pub();
        let single_script = Script::new_v1_p2tr(&cw.secp, internal, None);
        let tap_builder = TaprootBuilder::new()
            // .add_leaf(0, musig_script.clone()).unwrap();
            .add_leaf(0, single_script.clone())
            .unwrap();

        let script_pub_k = Script::new_v1_p2tr(
            &schema.to_wallet().secp,
            tap_builder
                .clone()
                .finalize(&schema.to_wallet().secp, internal) // tweak
                .unwrap()
                .internal_key(),
            None,
        ); // TaprootMerkleBranch

        let mut output = Output::default();
        let mut taproot_origin = BTreeMap::new();
        taproot_origin.insert(
            internal,
            (
                vec![
                    // TapLeafHash::from_script(&musig_script,LeafVersion::TapScript),
                    TapLeafHash::from_script(&single_script, LeafVersion::TapScript),
                ]
                .to_vec(),
                (
                    self.p2tr.get_ext_pub_key().fingerprint(),
                    self.p2tr.get_derivation_p(),
                ),
            ),
        );
        // tap_builder.has_hidden_nodes()
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
