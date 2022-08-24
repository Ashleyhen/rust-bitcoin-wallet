use std::sync::Arc;

use bitcoin::{psbt::PartiallySignedTransaction, Script, Transaction, TxIn};
use electrum_client::{Error, GetBalanceRes};

use super::RpcCall;
pub struct ReUseCall {
    pub psbt: PartiallySignedTransaction,
    witness: Script,
}

impl RpcCall for ReUseCall {
    fn contract_source(&self) -> Vec<Transaction> {
        return  vec![self.psbt.clone().extract_tx().clone()];
    }

    fn script_get_balance(&self) -> Arc<GetBalanceRes> {
        let value = self
            .psbt
            .clone()
            .extract_tx()
            .output
            .iter()
            .filter(|t| t.script_pubkey.eq(&self.witness))
            .map(|f| f.value)
            .sum::<u64>();
        return Arc::new(GetBalanceRes {
            confirmed: value,
            unconfirmed: 0,
        });
    }

    fn prev_input(&self)->Vec<TxIn> {
        return self.psbt.clone().extract_tx().clone().input;
    }
}
