use std::sync::Arc;

use bitcoin::{OutPoint, Script, Transaction, TxIn, Txid, Witness};
use electrum_client::{Client, ElectrumApi, Error, GetBalanceRes};

use crate::bitcoin_wallet::{address_formats::AddressSchema, wallet_methods::BroadcastOp};

use super::RpcCall;

struct ElectrumRpc {
    pub client: Client,
    pub script_pub_k: Script,
}

impl ElectrumRpc {
    pub fn get_electrum(script_pub_k: Script) -> Self {
        return ElectrumRpc {
            client: Client::new("ssl://electrum.blockstream.info:60002").unwrap(),
            script_pub_k,
        };
    }

    pub fn transaction_broadcast(&self) -> BroadcastOp {
        return BroadcastOp::Broadcast(Box::new(|tx: Transaction| {
            self.client.transaction_broadcast(&tx).unwrap().clone()
        }));
    }
}

impl RpcCall for ElectrumRpc {
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
