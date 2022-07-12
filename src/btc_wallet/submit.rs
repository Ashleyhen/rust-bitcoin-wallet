use bitcoin::{psbt::PartiallySignedTransaction, Transaction, Txid};
use miniscript::psbt::PsbtExt;

use super::wallet_methods::ClientWallet;

impl ClientWallet {
    pub fn finalize(
        &self,
        psbt: PartiallySignedTransaction,
        broad_cast_fn: &dyn Fn(&Transaction) -> Txid,
    ) -> PartiallySignedTransaction {
        return psbt
            .clone()
            .finalize(&self.secp)
            .map(|final_psbt| {
                broad_cast_fn(&final_psbt.clone().extract_tx());
                return final_psbt.clone();
            })
            .unwrap();
    }
}
