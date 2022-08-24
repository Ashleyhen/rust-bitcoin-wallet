use std::sync::Arc;

use bitcoin::{OutPoint, Script, Transaction, TxIn, Txid, Witness};
use electrum_client::{Client, ElectrumApi, Error, GetBalanceRes};

use super::RpcCall;

pub struct ElectrumRpc{
    amount: Arc<GetBalanceRes>,
    tx_in:Vec<TxIn>,
    previous_tx:Vec<Transaction>
}


impl ElectrumRpc {
    pub fn  new(script_pub_k: & Script) -> Self {

        let client =get_client();
        
        let history = Arc::new(
            client
                .script_list_unspent(&script_pub_k)
                .expect("address history call failed"),
        );

        let tx_in = history
            .clone()
            .iter()
            .map(|tx| {
                return TxIn {
                    previous_output: OutPoint::new(tx.tx_hash, tx.tx_pos.try_into().unwrap()),
                    script_sig: Script::new(), // The scriptSig must be exactly empty or the validation fails (native witness program)
                    sequence: 0xFFFFFFFF,
                    witness: Witness::default(),
                };
            })
            .collect::<Vec<TxIn>>();

        let previous_tx = tx_in
            .iter()
            .map(|tx_id| {
            client
                    .transaction_get(&tx_id.previous_output.txid)
                    .unwrap()
            })
            .collect::<Vec<Transaction>>();
let amount =client.script_get_balance(&script_pub_k.clone()).unwrap();
 return ElectrumRpc {
            amount: Arc::new(amount),
            tx_in,
            previous_tx,

        };
    }

    pub fn transaction_broadcast(&self, tx: Transaction) -> Txid {
        return get_client().transaction_broadcast(&tx).unwrap();
    }

}

impl RpcCall for ElectrumRpc {
    fn contract_source(&self) ->  Vec<Transaction> {
        return self.previous_tx.clone();
    }

    fn script_get_balance(&self) -> Arc<GetBalanceRes> {
        return self.amount.clone();
    }

    fn prev_input(&self)->Vec<TxIn> {
        return self.tx_in.clone();
    }
}

pub fn get_client() -> Client {
    return Client::new("ssl://electrum.blockstream.info:60002").unwrap();
}
