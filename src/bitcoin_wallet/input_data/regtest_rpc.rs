use std::{str::FromStr};

use bitcoin::{Address, OutPoint, Script, Transaction, TxIn, Witness, Txid};
use bitcoincore_rpc::{ Client, RpcApi};

use super::{ RpcCall};
pub struct RegtestRpc {
    amount: u64,
    tx_in: Vec<TxIn>,
    previous_tx: Vec<Transaction>,
}

impl RpcCall for RegtestRpc {
    fn contract_source(&self) -> Vec<Transaction> {
        return self.previous_tx.clone();
    }

    fn prev_input(&self) -> Vec<TxIn> {
        return self.tx_in.clone();
    }

    fn script_get_balance(&self) -> u64 {
        return self.amount.clone();
    }
}

impl<'a> RegtestRpc {
  
    pub fn get_client()-> Client{
        return Client::new(
                    "http://127.0.0.1:18444",
                    bitcoincore_rpc::Auth::UserPass("polaruser".to_string(), "polarpass".to_owned()),
                ).unwrap();
    } 

    pub fn transaction_broadcast(&self, tx:&Transaction)->Txid{
        return RegtestRpc::get_client().send_raw_transaction(tx).unwrap();
    }

    pub fn new(script_list: &'a Vec<String>) -> Box<dyn Fn() -> Self + 'a> {
        let client = RegtestRpc::get_client();
        return Box::new(move || {
            let address_list = script_list
                .iter()
                .map(|addr| Address::from_str(addr).unwrap())
                .collect::<Vec<Address>>();

            let tx_in = client
                .list_unspent(
                    None,
                    None,
                    Some(&address_list.iter().collect::<Vec<&Address>>()),
                    None,
                    None,
                )
                .unwrap()
                .iter()
                .map(|entry| {
                    return TxIn {
                        previous_output: OutPoint::new(entry.txid, entry.vout),
                        script_sig: Script::new(),
                        sequence: 0xFFFFFFFF,
                        witness: Witness::default(),
                    };
                })
                .collect::<Vec<TxIn>>();

            let previous_tx = tx_in
                .iter()
                .map(|tx_id| {
                    client
                        .get_transaction(&tx_id.previous_output.txid, Some(true))
                        .unwrap()
                        .transaction()
                        .unwrap()
                })
                .collect::<Vec<Transaction>>();

            let amt = previous_tx
                .iter()
                .map(|tx| {
                    tx.output
                        .iter()
                        .filter(|p| {
                            script_list
                                .iter()
                                .map(|addr| Address::from_str(addr).unwrap().script_pubkey())
                                .collect::<Vec<Script>>()
                                .contains(&p.script_pubkey)
                        })
                        .map(|output_tx| output_tx.value)
                        .sum::<u64>()
                })
                .sum::<u64>();

            return RegtestRpc {
                amount: amt,
                tx_in,
                previous_tx,
            };
        });
    }
}
