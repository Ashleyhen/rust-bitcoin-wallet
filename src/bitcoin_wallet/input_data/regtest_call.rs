use std::{collections::BTreeMap, str::FromStr};

use bitcoin::{Address, BlockHash, OutPoint, Script, Transaction, TxIn, Txid, Witness};
use bitcoincore_rpc::{
    bitcoincore_rpc_json::{ImportMultiResult, LoadWalletResult},
    jsonrpc::serde_json::{json, Map, Value},
    Client, RpcApi,
};

use super::RpcCall;

pub struct RegtestCall {
    amount: u64,
    tx_in: Vec<TxIn>,
    previous_tx: Vec<Transaction>,
    pub address_list: Vec<Address>,
    client: Client,
}

impl RpcCall for RegtestCall {
    fn contract_source(&self) -> Vec<Transaction> {
        return self.previous_tx.clone();
    }

    fn prev_input(&self) -> Vec<TxIn> {
        return self.tx_in.clone();
    }

    fn script_get_balance(&self) -> u64 {
        return self.amount.clone();
    }

    fn fee(&self) -> u64 {
        return 100000;
    }
    fn broadcasts_transacton(&self, tx: &Transaction) {
        let tx_id = RegtestCall::get_client().send_raw_transaction(tx).unwrap();
        println!("transaction send transaction id is: {}", tx_id)
    }
}

impl<'a> RegtestCall {
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

    pub fn init(address_list: &Vec<&str>, wallet_name: &str, mine: u8) -> Self {
        let client = RegtestCall::get_client();

        address_list.iter().for_each(|address| {
            let mut addr = "addr(".to_owned();
            addr.push_str(&address);
            addr.push_str(")");

            client
                .get_descriptor_info(&addr)
                .map(|desc| {
                    let descriptor = desc.descriptor;
                    println!("assigned a descriptor {} ", descriptor);
                    create_wallet(&client, wallet_name, mine, &descriptor)
                })
                .unwrap();
        });
        return RegtestCall::from_string(address_list);
    }

    pub fn generatetodescriptor(
        client: &Client,
        block_num: u64,
        address: &Address,
    ) -> Vec<BlockHash> {
        return client.generate_to_address(block_num, address).unwrap();
    }

    pub fn transaction_broadcast(&self, tx: &Transaction) -> Txid {
        let tx_id = RegtestCall::get_client().send_raw_transaction(tx).unwrap();
        println!("transaction id: {}", &tx_id);
        return tx_id;
    }

    pub fn from_string(address_list: &'a Vec<&str>) -> RegtestCall {
        return RegtestCall::from_address(
            address_list
                .iter()
                .map(|addr| Address::from_str(addr).unwrap())
                .collect::<Vec<Address>>(),
        );
    }

    pub fn update(&self) -> Self {
        let tx_in = RegtestCall::get_txin(&self.client, &self.address_list).to_vec();
        let previous_tx = RegtestCall::get_previous_tx(&self.client, &tx_in);

        let amt = RegtestCall::get_amount(&previous_tx, &self.address_list);
        return RegtestCall {
            amount: amt,
            tx_in,
            previous_tx,
            address_list: self.address_list.clone(),
            client: RegtestCall::get_client(),
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

    pub fn from_address(address_list: Vec<Address>) -> Self {
        let client = RegtestCall::get_client();
        let tx_in = RegtestCall::get_txin(&client, &address_list).to_vec();
        let previous_tx = RegtestCall::get_previous_tx(&client, &tx_in);
        let amt = RegtestCall::get_amount(&previous_tx, &address_list);
        return RegtestCall {
            amount: amt,
            tx_in,
            previous_tx,
            address_list,
            client,
        };
    }
}

fn create_wallet(client: &Client, wallet_name: &str, mine: u8, desc: &String) {
    if client
        .list_wallets()
        .unwrap()
        .contains(&wallet_name.to_owned())
    {
        importdescriptors(client, desc, mine);
        return;
    }
    client
        .create_wallet(wallet_name, Some(true), Some(true), None, Some(false))
        .map(|load_wallet| {
            println!("wallet {} created successfully ", load_wallet.name);

            if let Some(msg) = load_wallet.warning {
                println!("Warning! {}", msg)
            }

            println!("wallet {} created successfully ", load_wallet.name);

            importdescriptors(client, desc, mine);
        })
        .unwrap();
}

fn importdescriptors(client: &Client, desc: &String, mine: u8) {
    let mut params = Map::new();
    params.insert("desc".to_owned(), Value::String(desc.to_string()));
    params.insert("timestamp".to_owned(), Value::String("now".to_owned()));

    client
        .call::<Vec<ImportMultiResult>>(
            "importdescriptors",
            &[Value::Array([Value::Object(params)].to_vec())],
        )
        .map(|import| {
            import.iter().for_each(|result| {
                if result.success {
                    println!("descriptor successfully imported");
                }
                result.error.iter().for_each(|err| {
                    panic!("error importing wallet {:#?}", err);
                })
            });

            mine_to_descriptors(client, mine, desc);
        })
        .unwrap()
}

fn mine_to_descriptors(client: &Client, mine: u8, desc: &String) {
    client
        .call::<Vec<BlockHash>>("generatetodescriptor", &[json!(mine), json!(desc)])
        .unwrap();
    println!("successfully mined blocks");
}
