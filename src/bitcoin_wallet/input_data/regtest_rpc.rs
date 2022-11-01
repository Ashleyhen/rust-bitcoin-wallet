use std::str::FromStr;

use bitcoin::{Address, BlockHash, OutPoint, Script, Transaction, TxIn, Txid, Witness};
use bitcoincore_rpc::{bitcoincore_rpc_json::LoadWalletResult, Client, RpcApi};

use super::RpcCall;
pub struct RegtestRpc {
    amount: u64,
    tx_in: Vec<TxIn>,
    previous_tx: Vec<Transaction>,
    address_list: Vec<Address>,
    client: Client,
}

type OptionFilter = Option<Box<dyn Fn(Vec<TxHandlar>) -> Vec<TxHandlar>>>;
#[derive(Clone)]
pub struct TxHandlar {
    pub tx_vec: Transaction,
    pub tx_in: TxIn,
}

impl TxHandlar {
    pub fn new(tx_in: TxIn, tx_vec: Transaction) -> Self {
        return TxHandlar { tx_vec, tx_in };
    }
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
    pub fn get_client() -> Client {
        return Client::new(
            "http://127.0.0.1:18443",
            bitcoincore_rpc::Auth::UserPass(
                "foo".to_string(),
                "qDDZdeQ5vw9XXFeVnXT4PZ--tGN2xNjjR4nrtyszZx0=".to_owned(),
            ),
        )
        .unwrap();
    }

    pub fn create_wallet(
        client: &Client,
        wallet_name: &str,
        load_wallet: bool,
    ) -> LoadWalletResult {
        let result = client
            .create_wallet(wallet_name, Some(true), Some(true), None, None)
            .unwrap();
        if load_wallet {
            return client.load_wallet(wallet_name).unwrap();
        }
        return result;
    }

    pub fn generatetodescriptor(
        client: &Client,
        block_num: u64,
        address: &Address,
    ) -> Vec<BlockHash> {
        return client.generate_to_address(block_num, address).unwrap();
    }

    pub fn transaction_broadcast(&self, tx: &Transaction) -> Txid {
        return RegtestRpc::get_client().send_raw_transaction(tx).unwrap();
    }

    pub fn from_string(script_list: &'a Vec<&str>, optional_filter: OptionFilter) -> Self {
        let address_list = script_list
            .iter()
            .map(|addr| Address::from_str(addr).unwrap())
            .collect::<Vec<Address>>();
        let regtest = RegtestRpc::from_address(address_list, optional_filter);
        return regtest;
    }

    pub fn update(&self) -> Self {
        let tx_in = RegtestRpc::get_txin(&self.client, &self.address_list).to_vec();
        let previous_tx = RegtestRpc::get_previous_tx(&self.client, &tx_in);

        let amt = RegtestRpc::get_amount(&previous_tx, &self.address_list);
        return RegtestRpc {
            amount: amt,
            tx_in,
            previous_tx,
            address_list: self.address_list.clone(),
            client: RegtestRpc::get_client(),
        };
    }

    fn get_txin(client: &Client, address_list: &Vec<Address>) -> Vec<TxIn> {
        return client
            .list_unspent(
                None,
                None,
                Some(&address_list.clone().iter().collect::<Vec<&Address>>()),
                None,
                None,
            )
            .unwrap()
            .iter()
            .map(|entry| {
                return TxIn {
                    previous_output: OutPoint::new(entry.txid, entry.vout),
                    script_sig: Script::new(),
                    sequence: bitcoin::Sequence(0xFFFFFFFF),
                    witness: Witness::default(),
                };
            })
            .collect::<Vec<TxIn>>();
    }

    fn get_previous_tx(client: &Client, tx_in: &Vec<TxIn>) -> Vec<Transaction> {
        return tx_in
            .iter()
            .map(|tx_id| {
                let result = client
                    .get_transaction(&tx_id.previous_output.txid, Some(true))
                    .unwrap()
                    .transaction()
                    .unwrap();
                return result;
            })
            .collect::<Vec<Transaction>>();
    }

    fn get_amount(previous_tx: &Vec<Transaction>, address_list: &Vec<Address>) -> u64 {
        return previous_tx
            .iter()
            .map(|tx| {
                tx.output
                    .iter()
                    .filter(|p| {
                        address_list
                            .clone()
                            .iter()
                            .map(|addr| addr.script_pubkey())
                            .collect::<Vec<Script>>()
                            .contains(&p.script_pubkey)
                    })
                    .map(|output_tx| output_tx.value)
                    .sum::<u64>()
            })
            .sum::<u64>();
    }

    pub fn from_address(address_list: Vec<Address>, option_filter: OptionFilter) -> Self {
        let client = RegtestRpc::get_client();
        let tx_input = RegtestRpc::get_txin(&client, &address_list).to_vec();
        let prev_tx = RegtestRpc::get_previous_tx(&client, &tx_input);
        let handler_vec: Vec<TxHandlar> = tx_input
            .iter()
            .zip(prev_tx.clone())
            .map(|(tx_in, tx)| TxHandlar::new(tx_in.to_owned(), tx))
            .collect();
        let filter_tx_handler = option_filter
            .map(|mapper| mapper(handler_vec.clone()))
            .unwrap_or(handler_vec.clone());
        let (previous_tx, tx_in): (Vec<Transaction>, Vec<TxIn>) = filter_tx_handler
            .iter()
            .map(|tx_handler| (tx_handler.tx_vec.clone(), tx_handler.tx_in.clone()))
            .unzip();

        let amt = RegtestRpc::get_amount(&previous_tx, &address_list);

        return RegtestRpc {
            amount: amt,
            tx_in,
            previous_tx,
            address_list,
            client,
        };
    }
}
