use std::str::FromStr;

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::{Input, PartiallySignedTransaction, Output},
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

  

    

    fn lock_key<'a, S>(&self, schema: &'a S,  total: u64) -> Vec<(Output,u64)>
    where
        S: AddressSchema {
        todo!()
    }
}
impl P2TR_Multisig {
    pub fn new(to_addr: Vec<String>, psbt: Option<PartiallySignedTransaction>) -> Self {
        return P2TR_Multisig { to_addr, psbt };
    }

    fn create_tx<'a, S>(&self, schema: &'a S, tx_in: Vec<TxIn>, total: u64) -> Transaction
    where
        S: crate::btc_wallet::address_formats::AddressSchema,
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

        let send_tx = TxOut {
            value: total - tip,
            script_pubkey: script_pub_k,
        };

        return Transaction {
            version: 2,
            lock_time: 0,
            input: tx_in,
            output: vec![send_tx],
        };
    }
}
