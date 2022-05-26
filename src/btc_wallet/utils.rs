use std::{str::FromStr, borrow::BorrowMut, sync::Arc};
use bitcoin::{Transaction, Script, TxOut, TxIn, Address, Witness, psbt::{Input, Output}, util::bip32::ExtendedPubKey};
use crate::btc_wallet::{AddressSchema, WalletKeys};


// type Thunk = Box<dyn Fn() + Send + 'static>;
 

#[derive( Clone)]
pub struct UnlockAndSend< 'a, T:AddressSchema>{
    schema: &'a T,
    wallet_keys:WalletKeys 
}
// pub type TxOutMap=Box<dyn for<'r> Fn(&'r bitcoin::TxOut) -> bitcoin::TxOut>;
 pub struct TxOutMap( pub Box<dyn Fn(&TxOut)->TxOut> );

 impl <'a, T: AddressSchema> UnlockAndSend<'a, T>{
    
 pub fn  new(schema:&'a T,wallet_keys:WalletKeys)->Self{
     return UnlockAndSend{
         schema: schema,wallet_keys
     };

}   
    pub fn initialize_output(
        &self,amount:u64, 
        change_addr:ExtendedPubKey,
        send_to:String,
        previous_tx_list:Vec<Transaction>,
        )->Vec<TxOut> {
            
            let tip:u64=200;
            return previous_tx_list.iter().flat_map(|previous_tx|{
            let total= previous_tx.output.iter().filter(|tx_out|self.find_relevent_utxo(tx_out)).map(|tx_out|tx_out.clone()).map(|v|v.value).sum::<u64>();

            let change_amt=total-(amount+tip);
         ;   
            let mut tx_out=vec![ TxOut{ value:amount, script_pubkey: Address::from_str(send_to.as_str()).unwrap().script_pubkey() } ];

            if change_amt>=tip{
                tx_out.push(TxOut{ value: change_amt, script_pubkey:self.schema.map_ext_keys(&change_addr).script_pubkey() });
            }
           return tx_out; 
        }).collect(); }
   
    
    pub fn find_relevent_utxo(&self, tx_out:&TxOut)-> bool {
             return tx_out.script_pubkey.eq(&self.schema.map_ext_keys(&self.wallet_keys.0).script_pubkey()) ;
    }
    
    
}