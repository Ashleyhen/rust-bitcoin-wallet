use bitcoin::{Transaction, Txid, Script};
use electrum_client::{Error, ListUnspentRes, GetBalanceRes};

use super::ApiCall;

struct TestRpc();
impl ApiCall for TestRpc{
    
    fn transaction_broadcast(&self, tx: &Transaction) -> Result<Txid,Error> {
        todo!()
    }

    fn script_list_unspent(&self, script: &Script) -> Result<Vec<ListUnspentRes>, Error> {
        todo!()
    }

    fn transaction_get(&self, txid: &Txid) -> Result<Transaction, Error> {
        todo!()
    }

    fn script_get_balance(&self, script: &Script) -> Result<GetBalanceRes,Error> {
        todo!()
    }

    fn new()->Self {
        return TestRpc();
    }
}