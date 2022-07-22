use std::str::FromStr;

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::{Input, Output, PartiallySignedTransaction, TapTree},
    util::{
        bip32::ExtendedPubKey,
        taproot::{TaprootBuilder},
    },
    Address, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};

use crate::btc_wallet::{
    address_formats::{AddressSchema, p2tr_addr_fmt::P2TR}, constants::TIP, wallet_methods::ClientWallet,
};

use super::Vault;

pub struct P2trMultisig<'a,'b> {
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

impl <'a,'b> Vault for P2trMultisig<'a,'b> {
    fn unlock_key(&self, previous: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        let psbt = self.psbt.clone().unwrap();
        psbt.outputs.iter().for_each(|f| {
            // f.tap_tree.unwrap().script_leaves().for_each(|f|f.script().instructions())
            f.tap_internal_key;
        });
        return psbt.clone().inputs;
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

impl <'a,'b>P2trMultisig<'a, 'b> {

    fn get_ext_pub_key(&self) -> ExtendedPubKey {
        let tr = self.p2tr;
        let cw=tr.to_wallet();
        return cw.create_wallet(tr.wallet_purpose(), cw.recieve, cw.change).0;
    }

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
        let script = dynamic_builder(self.to_addr.iter().map(|addr| {
            XOnlyPublicKey::from_slice(&Address::from_str(&addr).unwrap().script_pubkey()[2..])
                .unwrap()
        }))
        .push_int(1)
        .push_opcode(opcodes::all::OP_EQUAL)
        .into_script();

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
        output.tap_tree = Some(TapTree::from_builder(tap_builder).unwrap());
        output.tap_internal_key = Some(internal);
        output.witness_script = Some(script_pub_k);
        return vec![output];
    }
}
