use std::{sync::Arc, str::FromStr};

use bitcoin::{Transaction, TxIn, TxOut, Script, OutPoint};

use crate::wallet_test::wallet_test_vector_traits::WalletTestVectors;

use super::RpcCall;

impl RpcCall for JsonInput{
    fn contract_source(&self) -> (Vec<bitcoin::TxIn>, Vec<bitcoin::Transaction>) {
        // Transaction::
        self.walletTestVectors.key_spending_path.iter().map(|f|
            {
                
                let tx_out:Vec<TxOut>=f.given.utxos_spent.iter().map(|tx_out| -> TxOut 
                {TxOut{ value: tx_out.amount_sats, script_pubkey: Script::from_str(&tx_out.script_pub_key).unwrap() }}
                ).collect();


// OutPoint::from_str(f.intermediary.h)
//     let tx_out:Vec<TxIn>=f.intermediary.iter().map(|tx_out| -> TxIn 
//                 {TxIn{ previous_output: OutPoint{
//                     txid: f.intermediary,
//                     vout: todo!(),
//                 }, script_sig: todo!(), sequence: f.intermediary.hash_sequences.unwrap(), witness: todo!() }}
//                 ).collect();


                
// Transaction{ version: todo!(), lock_time: todo!(), input: todo!(), output: todo!() };
            
            // todo!();
            }
            // f.given.raw_unsigned_tx.unwrap()
            // todo!();
        );


        todo!()
    }

    fn script_get_balance(&self) -> Result<electrum_client::GetBalanceRes, electrum_client::Error> {
        todo!()
    }
}
struct JsonInput{
    walletTestVectors:Arc<WalletTestVectors>
}
impl JsonInput{
    pub fn new(walletTestVectors:Arc<WalletTestVectors>)->Self{
        return JsonInput{walletTestVectors};


    }

}