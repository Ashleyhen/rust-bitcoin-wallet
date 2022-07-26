use core::fmt;
use std::sync::Arc;

use bitcoin::{
    psbt::{Input, Output, PartiallySignedTransaction},
    Script, Transaction, TxIn, Txid,
};
use electrum_client::{Error, GetBalanceRes, ListUnspentRes};

use crate::btc_wallet::address_formats::AddressSchema;

use super::{ApiCall, RpcCall};

pub struct TestRpc<'a>(&'a PartiallySignedTransaction);
impl<'a> ApiCall for TestRpc<'a> {
    fn transaction_broadcast(&self, tx: &Transaction) -> Result<Txid, Error> {
        return Ok(tx.txid());
    }

    fn script_list_unspent(&self, script: &Script) -> Result<Vec<ListUnspentRes>, Error> {
        let tx = self.0.clone().extract_tx();

        let unspent_res: Vec<ListUnspentRes> = tx
            .output
            .iter()
            .enumerate()
            .filter(|(_, tx_out)| tx_out.script_pubkey.eq(script))
            .map(|(pos, tx_out)| ListUnspentRes {
                tx_hash: tx.txid(),
                tx_pos: pos,
                value: tx_out.value,
                height: 744613,
            })
            .collect();
        return match unspent_res.is_empty() {
            true => Err(Error::Message(format!(
                "{} public key not found in the current transaction ",
                script.to_string()
            ))),
            false => Ok(unspent_res),
        };
    }

    fn transaction_get(&self, _tx_id: &Txid) -> Result<Transaction, Error> {
        return Ok(self.0.clone().extract_tx());
    }

    fn script_get_balance(&self, script: &Script) -> Result<GetBalanceRes, Error> {
        let value = self
            .0
            .clone()
            .extract_tx()
            .output
            .iter()
            .filter(|t| t.script_pubkey.eq(script))
            .map(|f| f.value)
            .sum::<u64>();
        return Ok(GetBalanceRes {
            confirmed: value,
            unconfirmed: 0,
        });
    }
}

impl<'a> TestRpc<'a> {
    pub fn new(psbt: &'a PartiallySignedTransaction) -> Self {
        return TestRpc(psbt);
    }
}

pub struct TestCall<'p, 'a, A> {
    psbt: &'p PartiallySignedTransaction,
    address: &'a A,
}
impl<'p, 'a, A: AddressSchema> RpcCall for TestCall<'p, 'a, A> {
    fn contract_source(&self) -> (Vec<TxIn>, Vec<Transaction>) {
        let tx = self.psbt.clone().extract_tx().clone();
        return (tx.clone().input, vec![tx]);
    }

    fn script_get_balance(&self) -> Result<GetBalanceRes, Error> {
        let value = self
            .psbt
            .clone()
            .extract_tx()
            .output
            .iter()
            .filter(|t| {
                t.script_pubkey.eq(&self
                    .address
                    .map_ext_keys(&self.address.get_ext_pub_key())
                    .script_pubkey())
            })
            .map(|f| f.value)
            .sum::<u64>();
        return Ok(GetBalanceRes {
            confirmed: value,
            unconfirmed: 0,
        });
    }
}

impl<'p, 'a, A> TestCall<'p, 'a, A> {
    pub fn new(address: &'a A, psbt: &'p PartiallySignedTransaction) -> Self
    where
        A: AddressSchema,
    {
        return TestCall { address, psbt };
    }
}
