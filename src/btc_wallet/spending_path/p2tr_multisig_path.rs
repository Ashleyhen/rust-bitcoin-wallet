use std::str::FromStr;

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::{Input, Output, PartiallySignedTransaction, TapTree},
    util::{bip32::ExtendedPubKey, taproot::TaprootBuilder},
    Address, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};

use crate::btc_wallet::{address_formats::AddressSchema, constants::TIP};

use super::Vault;

struct P2trMultisig<'a, T: Vault + ?Sized> {
    to_addr: Vec<String>,
    psbt: Option<PartiallySignedTransaction>,
    diff_lock: Option<&'a T>,
    diff_unlock: Option<&'a T>,
}
fn dynamic_builder(mut iter: impl Iterator<Item = XOnlyPublicKey>) -> Builder {
    return match iter.next() {
        Some(data) => dynamic_builder(iter)
            .push_x_only_key(&data)
            .push_opcode(opcodes::all::OP_CHECKSIGADD),
        None => Builder::new(),
    };
}
// <'a,T:Vault> P2trMultisig<'a,T>
impl<'a, T: Vault> Vault for P2trMultisig<'a, T> {
    fn create_tx(&self, output_list: &Vec<Output>, tx_in: Vec<TxIn>, total: u64) -> Transaction {
        let tx_input = tx_in.clone();
        let check_psbt = || self.psbt.clone().map(|f| f.extract_tx());

        let create_new_tx = Box::new(move || {
            let tx_out = TxOut {
                value: total - TIP,
                script_pubkey: output_list.get(0).unwrap().clone().witness_script.unwrap(),
            };
            return Transaction {
                version: 2,
                lock_time: 0,
                input: tx_input,
                output: vec![tx_out],
            };
        });

        return self
            .clone()
            .diff_unlock
            .map(|f| f.create_tx(output_list, tx_in, total))
            .or_else(check_psbt)
            .unwrap_or_else(create_new_tx);
    }

    fn lock_key<'b, S>(&self, schema: &'b S) -> Vec<Output>
    where
        S: AddressSchema,
    {
        return self
            .psbt
            .as_ref()
            .map(|f| f.outputs.clone())
            .unwrap_or(self.create_lock(schema));
    }

    fn unlock_key(&self, previous: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        todo!()
    }
}

impl<'a, T: Vault> P2trMultisig<'a, T> {
    pub fn new(
        to_addr: Vec<String>,
        psbt: Option<PartiallySignedTransaction>,
        diff_lock: Option<&'a T>,
        diff_unlock: Option<&'a T>,
    ) -> Self {
        return P2trMultisig {
            to_addr,
            psbt,
            diff_lock,
            diff_unlock,
        };
    }

    fn create_lock<'b, S>(&self, schema: &'b S) -> Vec<Output>
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
        let internal = XOnlyPublicKey::from_slice(
            &Address::from_str(&self.to_addr[0]).unwrap().script_pubkey()[2..],
        )
        .unwrap();
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
