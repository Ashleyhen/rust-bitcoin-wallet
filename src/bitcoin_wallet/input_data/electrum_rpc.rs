use std::sync::Arc;

use bitcoin::{OutPoint, Script, Transaction, TxIn, Txid, Witness};
use electrum_client::{Client, ElectrumApi, Error, GetBalanceRes};

use super::RpcCall;

pub struct ElectrumRpc<'a> {
    pub client: Client,
    pub script_pub_k: &'a Script,
}

impl<'a> ElectrumRpc<'a> {
    pub fn new(script_pub_k: &'a Script) -> Self {
        return ElectrumRpc {
            client: get_client(),
            script_pub_k,
        };
    }

    pub fn transaction_broadcast(&self, tx: Transaction) -> Txid {
        return self.client.transaction_broadcast(&tx).unwrap();
    }
}

impl<'a> RpcCall for ElectrumRpc<'a> {
    fn contract_source(&self) -> (Vec<TxIn>, Vec<Transaction>) {
        let history = Arc::new(
            self.client
                .script_list_unspent(&self.script_pub_k)
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
                self.client
                    .transaction_get(&tx_id.previous_output.txid)
                    .unwrap()
            })
            .collect::<Vec<Transaction>>();
        return (tx_in, previous_tx);
    }

    fn script_get_balance(&self) -> Result<GetBalanceRes, Error> {
        return self.client.script_get_balance(&self.script_pub_k);
    }
}

pub fn get_client() -> Client {
    return Client::new("ssl://electrum.blockstream.info:60002").unwrap();
}
