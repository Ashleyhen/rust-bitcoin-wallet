use bitcoin::{psbt::PartiallySignedTransaction, Script, Transaction, TxIn};
use electrum_client::{Error, GetBalanceRes};

use super::RpcCall;
pub struct ReUseCall {
    pub psbt: PartiallySignedTransaction,
    witness: Script,
}

impl RpcCall for ReUseCall {
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
            .filter(|t| t.script_pubkey.eq(&self.witness))
            .map(|f| f.value)
            .sum::<u64>();
        return Ok(GetBalanceRes {
            confirmed: value,
            unconfirmed: 0,
        });
    }
}
