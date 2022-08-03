use bitcoin::{psbt::{PartiallySignedTransaction, Output, Input}, TxIn, Transaction};

use crate::btc_wallet::address_formats::p2tr_addr_fmt::P2TR;

use super::Vault;

pub struct MultiSigPath<'a, 'b> {
    pub p2tr: &'a P2TR,
    to_addr: Vec<String>,
    psbt: Option<&'b PartiallySignedTransaction>,
}

impl <'a, 'b> Vault for MultiSigPath<'a, 'b>{
    fn create_tx(&self, output_list: &Vec<Output>, tx_in: Vec<TxIn>, total: u64) -> bitcoin::Transaction {
        todo!()
    }

    fn lock_key(&self) -> Vec<Output> {
		

        todo!()
    }

    fn unlock_key(&self, previous: Vec<Transaction>, current_tx: &Transaction) -> Vec<Input> {
        todo!()
    }
}