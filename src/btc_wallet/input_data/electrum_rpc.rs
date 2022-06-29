
use bitcoin::{Txid, Script, Transaction};
use electrum_client::{Client, GetBalanceRes, ElectrumApi, Error};

use crate::btc_wallet::wallet_traits::ApiCall;



pub struct ElectrumRpc(pub Client);

impl  ApiCall for ElectrumRpc {
    fn transaction_broadcast(&self, tx: &Transaction) -> Result<bitcoin::Txid, Error>{
        return self.0.transaction_broadcast(tx);
    }

    fn script_list_unspent(&self, script: &Script) -> Result<Vec<electrum_client::ListUnspentRes>, Error> {
        return self.0.script_list_unspent(script);
    }

    fn transaction_get(&self, txid: &Txid) -> Result<Transaction,Error> {
        return self.0.transaction_get(txid);
    }

    fn script_get_balance(&self, script: &Script) -> Result<GetBalanceRes,Error> {
        return self.0.script_get_balance(script);
    }

    fn new() -> Self {
        return ElectrumRpc(
            Client::new("ssl://electrum.blockstream.info:60002").expect("client connection failed !!!")
        );
    }
}