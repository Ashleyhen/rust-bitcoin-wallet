use bitcoin::{
    psbt::{Input, PartiallySignedTransaction},
    util::bip32::{ExtendedPubKey, KeySource},
    TxOut, Txid,
};
use miniscript::psbt::PsbtExt;

// use crate::btc_wallet::utils::UnlockAndSend;

use crate::btc_wallet::address_formats::AddressSchema;

use self::{
    input_data::ApiCall,
    wallet_methods::{Broadcast_op, ClientWithSchema}, address_formats::p2tr_addr_fmt::P2TR, spending_path::Vault,
};
// pub mod input_data;
pub mod input_data;

// pub(crate) mod lock;
pub mod spending_path;
pub type WalletKeys = (ExtendedPubKey, KeySource);
pub mod address_formats;
pub mod constants;
// pub mod unlock;
pub mod wallet_methods;

impl<'a, A> ClientWithSchema<'a, P2TR, A>
where
    A: ApiCall,
{
    pub fn submit_psbt<'v, V>(
        &self,
        vault: &'v V,
        broad_cast_op: Broadcast_op,
    ) -> PartiallySignedTransaction
    where
        V: Vault,
    {
        let psbt = self.submit_tx(vault);

        return match broad_cast_op {
            Broadcast_op::Finalize => {
                let complete = psbt.finalize(&self.schema.to_wallet().secp).unwrap();
                dbg!(complete.clone().extract_tx());
                complete
            }
            Broadcast_op::Broadcast => {
                let complete = psbt.finalize(&self.schema.to_wallet().secp).unwrap();
                self.api_call
                    .transaction_broadcast(&complete.clone().extract_tx())
                    .unwrap();
                dbg!(complete.clone().extract_tx());
                complete
            }
            Broadcast_op::None => {
                dbg!(psbt.clone());
                psbt
            }
        };
    }
}
