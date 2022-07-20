use std::str::FromStr;

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::{Input, Output, PartiallySignedTransaction},
    util::{bip32::ExtendedPubKey, taproot::TaprootBuilder},
    Address, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};

use crate::btc_wallet::address_formats::AddressSchema;

use super::Vault;

pub struct P2TR_Multisig {
    to_addr: Vec<String>,
    psbt: Option<PartiallySignedTransaction>,
}
fn dynamic_builder(mut iter: impl Iterator<Item = XOnlyPublicKey>) -> Builder {
    return match iter.next() {
        Some(data) => dynamic_builder(iter)
            .push_x_only_key(&data)
            .push_opcode(opcodes::all::OP_CHECKSIGADD),
        None => Builder::new(),
    };
}

impl Vault for P2TR_Multisig {
    fn unlock_key(&self, previous: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        todo!()
    }

    fn lock_key<'a, S>(&self, schema: &'a S) -> Vec<Output>
    where
        S: AddressSchema,
    {
        return self
            .psbt.as_ref()
            .map(|f| f.outputs.clone())
            .unwrap_or(self.create_lock(schema));
    }

    fn create_tx(&self, output_list: &Vec<Output>, tx_in: Vec<TxIn>, total: u64) -> Transaction {
        todo!()
    }
}

impl P2TR_Multisig {
    pub fn new(to_addr: Vec<String>, psbt: Option<PartiallySignedTransaction>) -> Self {
        return P2TR_Multisig { to_addr, psbt };
    }

    fn create_lock<'a, S>(&self, schema: &'a S) -> Vec<Output>
    where
        S: AddressSchema,
    {
        let tip: u64 = 300;
        let script = dynamic_builder(self.to_addr.iter().map(|addr| {
            XOnlyPublicKey::from_slice(&Address::from_str(&addr).unwrap().script_pubkey()[2..])
                .unwrap()
        }))
        .push_int(1)
        .push_opcode(opcodes::all::OP_EQUAL)
        .into_script();

        let trap = TaprootBuilder::new().add_leaf(0, script).unwrap();
        let internal = XOnlyPublicKey::from_slice(
            &Address::from_str(&self.to_addr[0]).unwrap().script_pubkey()[2..],
        )
        .unwrap();

        let script_pub_k = Script::new_v1_p2tr(
            &schema.to_wallet().secp,
            trap.finalize(&schema.to_wallet().secp, internal)
                .unwrap()
                .internal_key(),
            None,
        ); // TaprootMerkleBranch

        let mut output = Output::default();

        output.witness_script = Some(script_pub_k);
        return vec![output];
    }
}
