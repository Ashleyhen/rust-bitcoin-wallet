use bitcoin::{psbt::PartiallySignedTransaction, Script, Transaction, TxIn};

use super::RpcCall;
pub struct ReUseCall {
    pub psbt: PartiallySignedTransaction,
    witness: Script,
}

impl RpcCall for ReUseCall {
    fn contract_source(&self) -> Vec<Transaction> {
        return vec![self.psbt.clone().extract_tx().clone()];
    }

    fn script_get_balance(&self) -> u64 {
        return self
            .psbt
            .clone()
            .extract_tx()
            .output
            .iter()
            .filter(|t| t.script_pubkey.eq(&self.witness))
            .map(|f| f.value)
            .sum::<u64>();
    }

    fn prev_input(&self) -> Vec<TxIn> {
        return self.psbt.clone().extract_tx().clone().input;
    }

    fn fee(&self) -> u64 {
        return 2000;
    }
    fn broadcasts_transacton(&self,tx:&Transaction) {
        
    }
}
