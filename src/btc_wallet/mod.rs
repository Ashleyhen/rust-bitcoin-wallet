use bitcoin::{util::bip32::{ExtendedPubKey, KeySource}, Txid, psbt::{PartiallySignedTransaction, Input}, TxOut};
use miniscript::psbt::PsbtExt;

// use crate::btc_wallet::utils::UnlockAndSend;

use self::{
    input_data::electrum_rpc::ElectrumRpc,
    lock::pub_key_lock,
    unlock::SignTx,
    wallet_methods::{Broadcast_op, ClientWithSchema},
    wallet_traits::{AddressSchema, ApiCall},
};
// pub mod input_data;
pub mod input_data;

pub(crate) mod lock;
pub mod wallet_traits;

pub type WalletKeys = (ExtendedPubKey, KeySource);
pub mod p2tr;
pub mod p2wpkh;
pub mod unlock;

pub mod wallet_methods;
impl<'a,'b, S: AddressSchema, A: ApiCall> ClientWithSchema<'a, S, A> {
    pub fn submit_psbt(&self, lock: Vec<TxOut>,unlock:&dyn Fn(SignTx) -> Input, broad_cast_op: Broadcast_op) ->PartiallySignedTransaction {
        let psbt = self.submit_tx(
            unlock,
            lock
            ,
        );

        return match broad_cast_op {
            
            Broadcast_op::Finalize => {
                let complete=psbt.finalize(&self.schema.to_wallet().secp).unwrap();
                dbg!(complete.clone().extract_tx());
                complete
            },
            Broadcast_op::Broadcast => {
                let complete=psbt.finalize(&self.schema.to_wallet().secp).unwrap();
                self.api_call.transaction_broadcast(&complete.clone().extract_tx()).unwrap();
                dbg!(complete.clone().extract_tx());
                complete
            },
            Broadcast_op::None => {
                dbg!(psbt.clone());
                psbt
            },
        }; 
    }

    pub fn get_pub_key_lock(&self, to_addr:String)->Vec<TxOut>{
        return pub_key_lock(
                self.schema,
                1000,
                self.get_balance().confirmed,
                self.change_addr().0,
                to_addr.to_string(),
            )
    }
}