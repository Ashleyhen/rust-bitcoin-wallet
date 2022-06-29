use std::{str::FromStr, borrow::BorrowMut, sync::Arc};
use bitcoin::{Transaction, Script, TxOut, TxIn, Address, Witness, psbt::{Input, Output}, util::bip32::ExtendedPubKey};
use electrum_client::ListUnspentRes;
use crate::btc_wallet::{ WalletKeys};

use super::wallet_traits::AddressSchema;

#[derive( Clone)]
pub struct UnlockAndSend< 'a, T:AddressSchema>{
    schema: &'a T,
    wallet_keys:WalletKeys 
}



 impl <'a, T: AddressSchema> UnlockAndSend<'a, T>{
    
 pub fn  new(schema:&'a T,wallet_keys:WalletKeys)->Self{
     return UnlockAndSend{
         schema,wallet_keys
     };

}   
    pub fn initialize_output(
        &self,amount:u64, 
        previous_tx_list:Arc<Vec<ListUnspentRes>>,
        change_addr:ExtendedPubKey,
        to_addr:String)->Vec<TxOut> {
        let tip:u64=300;
        let total=previous_tx_list.iter().map(|f|f.value).sum::<u64>();
        self.schema.map_ext_keys(&self.wallet_keys.0).script_pubkey();
        
        let send_tx=TxOut{
            value: amount,
            script_pubkey: Address::from_str(&to_addr).unwrap().script_pubkey(),
        };

        if(total<=(amount+tip)){
            return vec![send_tx]
        }

        let change_tx=TxOut{
            value: total-(amount+tip),
            script_pubkey: self.schema.map_ext_keys(&change_addr).script_pubkey(),
        };

        return vec![send_tx,change_tx];
    }
}