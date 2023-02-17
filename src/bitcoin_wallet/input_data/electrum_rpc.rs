use std::{ops::Mul, sync::Arc};

use super::RpcCall;
use bitcoin::{OutPoint, Script, Transaction, TxIn, Txid, Witness};
use electrum_client::{Client, ElectrumApi};

pub struct ElectrumRpc {
    amount: u64,
    tx_in: Vec<TxIn>,
    previous_tx: Vec<Transaction>,
}

impl ElectrumRpc {
    pub fn new(script_pub_k: &Script) -> Self {
        return ElectrumRpc::update(script_pub_k)();
    }

    pub fn transaction_broadcast(&self, tx: Transaction) -> Txid {
        return get_client().transaction_broadcast(&tx).unwrap();
    }
}

impl RpcCall for ElectrumRpc {
    fn contract_source(&self) -> Vec<Transaction> {
        return self.previous_tx.clone();
    }

    fn script_get_balance(&self) -> u64 {
        return self.amount.clone();
    }

    fn prev_input(&self) -> Vec<TxIn> {
        return self.tx_in.clone();
    }

    fn fee(&self) -> u64 {
        return 3551;
    }

    fn broadcasts_transacton(&self, tx: &Transaction) {
        let tx_id = get_client().transaction_broadcast(&tx).unwrap();
        println!("transaction send transaction id is: {}", tx_id)
    }
}

pub fn get_client() -> Client {
    return Client::new("ssl://electrum.blockstream.info:50002").unwrap();
}
impl<'a> ElectrumRpc {
    fn update(script_pub_k: &'a Script) -> Box<dyn Fn() -> Self + 'a> {
        let client = get_client();

        return Box::new(move || {
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
                        sequence: bitcoin::Sequence(0xFFFFFFFF),
                        witness: Witness::default(),
                    };
                })
                .collect::<Vec<TxIn>>();

            let previous_tx = tx_in
                .iter()
                .map(|tx_id| client.transaction_get(&tx_id.previous_output.txid).unwrap())
                .collect::<Vec<Transaction>>();
            return ElectrumRpc {
                amount: Arc::new(client.script_get_balance(&script_pub_k.clone()).unwrap())
                    .confirmed,
                tx_in,
                previous_tx: previous_tx,
            };
        });
    }
}
