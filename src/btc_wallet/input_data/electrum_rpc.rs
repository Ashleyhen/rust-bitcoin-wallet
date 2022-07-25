use std::sync::Arc;

use bitcoin::{OutPoint, Script, Transaction, TxIn, Txid, Witness};
use electrum_client::{Client, ElectrumApi, Error, GetBalanceRes};

use crate::btc_wallet::{address_formats::AddressSchema, wallet_methods::BroadcastOp};

use super::{ApiCall, RpcCall};

pub struct ElectrumRpc(Client);

impl ApiCall for ElectrumRpc {
    fn transaction_broadcast(&self, tx: &Transaction) -> Result<bitcoin::Txid, Error> {
        return self.0.transaction_broadcast(tx);
    }

    fn script_list_unspent(
        &self,
        script: &Script,
    ) -> Result<Vec<electrum_client::ListUnspentRes>, Error> {
        return self.0.script_list_unspent(script);
    }

    fn transaction_get(&self, txid: &Txid) -> Result<Transaction, Error> {
        return self.0.transaction_get(txid);
    }

    fn script_get_balance(&self, script: &Script) -> Result<GetBalanceRes, Error> {
        return self.0.script_get_balance(script);
    }
}
impl ElectrumRpc {
    pub fn new() -> Self {
        return ElectrumRpc(
            Client::new("ssl://electrum.blockstream.info:60002")
                .expect("client connection failed !!!"),
        );
    }
}

pub struct ElectrumCall<'a, A: AddressSchema> {
    client: Client,
    address: &'a A,
}

impl<'a, A: AddressSchema> RpcCall for ElectrumCall<'a, A> {
    fn contract_source(&self) -> (Vec<TxIn>, Vec<Transaction>) {
        let cw=self.address.to_wallet();

        let signer_pub_k = self.address.get_ext_pub_key();
        let signer_addr = self.address.map_ext_keys(&signer_pub_k);

        let history = Arc::new(
            self.client
                .script_list_unspent(&signer_addr.script_pubkey())
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

    fn script_get_balance(&self) -> Result<GetBalanceRes, Error>  {
        return self.client.script_get_balance(
            &self
                .address
                .map_ext_keys(&self.address.get_ext_pub_key())
                .script_pubkey(),
        );
    }
}

impl<'a, A: AddressSchema> ElectrumCall<'a, A> {
    pub fn new(address: &'a A) -> Self {
        return ElectrumCall {
            address,
            client: Client::new("ssl://electrum.blockstream.info:60002")
                .expect("client connection failed !!!"),
        };
    }

    pub fn transaction_broadcast(&self)->BroadcastOp{
         return BroadcastOp::Broadcast(Box::new( |tx: Transaction|{
            self.client.transaction_broadcast(&tx).unwrap().clone()
         }));
    }

}
