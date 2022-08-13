use bitcoin::{
    psbt::PartiallySignedTransaction,
    util::bip32::{ExtendedPubKey, KeySource},
};
use miniscript::psbt::PsbtExt;

// use crate::btc_wallet::utils::UnlockAndSend;

use crate::btc_wallet::address_formats::AddressSchema;

use self::{
    address_formats::p2tr_addr_fmt::P2TR,
    input_data::{ApiCall, RpcCall},
    spending_path::Vault,
    wallet_methods::{BroadcastOp, ClientWithSchema},
};
// pub mod input_data;
pub mod input_data;

// pub(crate) mod lock;
pub mod address_formats;
pub mod constants;
pub mod spending_path;

pub mod script_services;
// pub mod unlock;
pub mod wallet_methods;

impl<'a, A> ClientWithSchema<'a, P2TR, A>
where
    A: RpcCall,
{
    pub fn submit_psbt<'v, V>(
        &self,
        vault: &'v V,
        broad_cast_op: BroadcastOp,
    ) -> PartiallySignedTransaction
    where
        V: Vault,
    {
        let psbt = self.submit_tx(vault);

        return match broad_cast_op {
            BroadcastOp::Finalize => {
                dbg!(psbt.clone());
                let complete = psbt.finalize(&self.schema.to_wallet().secp).unwrap();
                dbg!(complete.clone().extract_tx());
                complete
            }
            BroadcastOp::Broadcast(transaction_broadcast) => {
                let complete = psbt.finalize(&self.schema.to_wallet().secp).unwrap();
                transaction_broadcast(complete.clone().extract_tx());
                dbg!(complete.clone().extract_tx());
                complete
            }
            BroadcastOp::None => {
                dbg!(psbt.clone());
                psbt
            }
        };
    }
}
