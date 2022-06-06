use std::{str::FromStr, borrow::BorrowMut, sync::Arc};
use bitcoin::{Transaction, Script, TxOut, TxIn, Address, Witness, psbt::{Input, Output}, util::bip32::ExtendedPubKey};
use electrum_client::ListUnspentRes;
use crate::btc_wallet::{AddressSchema, WalletKeys};


// type Thunk = Box<dyn Fn() + Send + 'static>;
 

#[derive( Clone)]
pub struct UnlockAndSend< 'a, T:AddressSchema>{
    schema: &'a T,
    wallet_keys:WalletKeys 
}
// pub type TxOutMap=Box<dyn for<'r> Fn(&'r bitcoin::TxOut) -> bitcoin::TxOut>;

 impl <'a, T: AddressSchema> UnlockAndSend<'a, T>{
    
 pub fn  new(schema:&'a T,wallet_keys:WalletKeys)->Self{
     return UnlockAndSend{
         schema: schema,wallet_keys
     };

}   
    pub fn initialize_output(
        &self,amount:u64, 
        previous_tx_list:Arc<Vec<ListUnspentRes>>,
        )->Vec<TxOut> {
            
        let tip:u64=200;
        let total=previous_tx_list.iter().map(|f|f.value).sum::<u64>();
        let change_amt=total-(amount+tip);
        return vec![TxOut{
            value: change_amt,
            script_pubkey: self.schema.map_ext_keys(&self.wallet_keys.0).script_pubkey(),
        }];
    }
   
    
    pub fn find_relevent_utxo(&self, tx_out:&TxOut)-> bool {
             return tx_out.script_pubkey.eq(&self.schema.map_ext_keys(&self.wallet_keys.0).script_pubkey()) ;
    }
    
    
}