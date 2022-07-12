use core::fmt;

use bitcoin::{Script, Transaction, Txid, psbt::{PartiallySignedTransaction, Input, Output}};
use electrum_client::{Error, GetBalanceRes, ListUnspentRes};

use crate::btc_wallet::wallet_traits::ApiCall;

pub struct TestRpc(pub PartiallySignedTransaction);
 impl ApiCall for TestRpc{
    fn transaction_broadcast(&self, tx: &Transaction) -> Result<Txid, Error> {
        return Ok(tx.txid());
    }

    fn script_list_unspent(&self, script: &Script) -> Result<Vec<ListUnspentRes>, Error> {
        let tx=self.0.clone().extract_tx();

        let unspent_res:Vec<ListUnspentRes>=tx.output.iter().enumerate().filter(|(_,tx_out)|tx_out.script_pubkey.eq(script))
        .map(|(pos,tx_out)|ListUnspentRes{tx_hash: tx.txid(),  tx_pos: pos, value: tx_out.value, height: 744613 }).collect();
        return match  unspent_res.is_empty(){
            true => Err(Error::Message(format!("{} public key not found in the current transaction ",script.to_string()))),
            false => Ok(unspent_res),
        };
    }

    fn transaction_get(&self, tx_id: &Txid) -> Result<Transaction, Error> {
        return Ok(self.0.clone().extract_tx());
    }

    fn script_get_balance(&self, script: &Script) -> Result<GetBalanceRes, Error> {
        let value=self.0.clone().extract_tx().output.iter().filter(|t|t.script_pubkey.eq(script)).map(|f|f.value).sum::<u64>();
        return Ok(GetBalanceRes{ confirmed:value, unconfirmed: 0 });
    }
}

impl TestRpc{
    pub fn new(psbt:PartiallySignedTransaction) -> Self {
        return TestRpc(psbt);
    }
}
