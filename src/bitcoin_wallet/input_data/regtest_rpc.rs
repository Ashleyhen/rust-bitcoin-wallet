use std::str::FromStr;

use bitcoin::{Address, BlockHash, OutPoint, Script, Transaction, TxIn, Txid, Witness};
use bitcoincore_rpc::{bitcoincore_rpc_json::LoadWalletResult, Client, RpcApi};

use super::RpcCall;
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
// String to be appended to bitcoin.conf:
// rpcauth=polaruser:bc2c2886919756bb91cd3849ca2ef7a0$1164f3265c8876f9c1081f3fda6195e828f1e85c03f4112ab927f0661aad86e7
// Your password:
// -qYFk0IOoOVPRQCaYZpmm95ffPbZq9XPbHwNkYXNwx4=

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
    // bitcoin-cli -rpcuser=foo -rpcpassword=qDDZdeQ5vw9XXFeVnXT4PZ--tGN2xNjjR4nrtyszZx0=  -rpcwallet=mywallet generatetodescriptor 100 "addr(bcrt1pe6lgv0eucta4l23yk69wmjza4m89w5a8p4g7dhjl4w9jvhj30jjq0cjwxw)#ysp3m4rs"

    // bitcoin-cli -named createwallet wallet_name=mywallet descriptors=true disable_private_keys=true
    // bitcoin-cli loadwallet mywallet
    // bitcoin-cli getdescriptorinfo "addr(bcrt1pe6lgv0eucta4l23yk69wmjza4m89w5a8p4g7dhjl4w9jvhj30jjq0cjwxw)"
    // bitcoin-cli -rpcwallet=mywallet importdescriptors '[{"desc":"addr(bcrt1pe6lgv0eucta4l23yk69wmjza4m89w5a8p4g7dhjl4w9jvhj30jjq0cjwxw)#ysp3m4rs","timestamp":"now"}]'
    // bitcoin-cli -rpcwallet=mywallet generatetodescriptor 10 "addr(bcrt1pe6lgv0eucta4l23yk69wmjza4m89w5a8p4g7dhjl4w9jvhj30jjq0cjwxw)#ysp3m4rs"
    // bitcoin-cli -rpcwallet=mywallet listunspent

    pub fn create_wallet(
        client: &Client,
        wallet_name: &str,
        load_wallet: bool,
    ) -> LoadWalletResult {
        let result = client
            .create_wallet(wallet_name, Some(true), Some(true), None, None)
            .unwrap();
        if (load_wallet) {
            return client.load_wallet(wallet_name).unwrap();
        }
        return result;
    }

    pub fn importdescriptors(client: &Client, address: &Address) {
        // bitcoin-cli -rpcwallet=mywallet importdescriptors '[{"desc":"addr(bcrt1pp375ce9lvxs8l9rlsl78u4szhqa7za748dfhtjj5ht05lufu4dwsshpxl6)#ngm593tu","timestamp":"now"}]'
        // let mut arg = [

        // ];
        //         client.call("importdescriptors", []);
        //         client
        //             .import_address(&address, None,None)
        //             .unwrap();
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

    pub fn from_string(script_list: &'a Vec<String>) -> Box<dyn Fn() -> Self + 'a> {
        let address_list = script_list
            .iter()
            .map(|addr| Address::from_str(addr).unwrap())
            .collect::<Vec<Address>>();
        let regtest = RegtestRpc::from_address(address_list);
        return regtest;
    }

    pub fn from_address(address_list: Vec<Address>) -> Box<dyn Fn() -> Self + 'a> {
        let client = RegtestRpc::get_client();
        return Box::new(move || {

            let temp = client
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
                        sequence: bitcoin::Sequence(0xFFFFFFFF),
                        witness: Witness::default(),
                    };
                })
                .collect::<Vec<TxIn>>();

                let tx_in=temp[..1].to_vec();

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
                            address_list
                                .iter()
                                .map(|addr| addr.script_pubkey())
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
