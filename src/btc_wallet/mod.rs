use bitcoin::util::bip32::{ExtendedPubKey, KeySource};

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
pub mod submit;
pub mod wallet_traits;

pub type WalletKeys = (ExtendedPubKey, KeySource);
pub mod p2tr;
pub mod p2wpkh;
pub mod unlock;

pub mod wallet_methods;
impl<'a, S: AddressSchema> ClientWithSchema<'a, S, ElectrumRpc> {
    pub fn submit_psbt(&self, to_addr: String, broad_cast_op: Broadcast_op) -> () {
        let electrum_rpc = ElectrumRpc::new();
        let psbt = self.submit_tx(
            &|s| s.tr_key_sign(),
            pub_key_lock(
                self.schema,
                100,
                self.get_balance().confirmed,
                self.change_addr().0,
                to_addr.to_string(),
            ),
        );
        self.schema
            .to_wallet()
            .finalize(psbt, &|tx| match broad_cast_op {
                Broadcast_op::Broadcast => {
                    dbg!(electrum_rpc.transaction_broadcast(tx).unwrap());
                }
                _ => {
                    dbg!(tx);
                }
            });
    }
}
