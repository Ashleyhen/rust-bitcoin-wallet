
use std::{str::FromStr, borrow::BorrowMut};
use bitcoin::{Transaction, Script, TxOut, TxIn, Address, Witness, psbt::Input};
use crate::btc_wallet::{AddressSchema, WalletKeys};

use super::{ClientWithSchema, ClientWallet};

// type Thunk = Box<dyn Fn() + Send + 'static>;
 

type Thunk =   Box<dyn (FnMut(&TxOut)->TxOut)>;
#[derive( Clone)]
pub struct UnlockAndSend< T:AddressSchema>{
    schema: T,
    wallet_keys:WalletKeys 
}

pub trait Temp {

    

    fn hello();
    fn temp<F>(tx_out_mapping:F)->Transaction
where 
    F:Sized,
    F:Copy,
    F:FnMut(&bitcoin::TxOut)->TxOut;
}

 struct Hello{
pub tr:Box<dyn FnMut(&bitcoin::TxOut)->TxOut>
 }
 
    fn temp<F>(tx_out_mapping:F)->Transaction
where 
    F:Sized,
    F:Copy,
    F:FnMut(&bitcoin::TxOut)->TxOut {
        todo!()
    }

 impl <T: AddressSchema> UnlockAndSend<T>{
    
 pub(crate) fn  new(schema:T,wallet_keys:WalletKeys)->Self{
     return UnlockAndSend{
         schema: schema,wallet_keys
     };

}   
    pub fn initialize<F>(
        &self,amount:u64, 
        to_addr:String,
        change_addr:Script,
        input:Vec<TxIn>,
        previous_tx_list:Vec<Transaction>,
        hello: &mut Hello 
    )->Transaction
 where 
    F:Sized,
    F:Copy,
    F:FnMut(&bitcoin::TxOut)->TxOut
    
    {
 hello.tr=Box::new(|a:&TxOut|{*a});
        
        
        let tip:u64=200;
        let output:(Vec<TxOut>)=previous_tx_list.iter().flat_map(|previous_tx|{
        let value=self.find_relevent_utxo(previous_tx,tx_out_mapping).iter().map(|v|v.value).count() as u64;
        let change_amt=value-(amount+tip);
        let tx_out=TxOut::default();
 hello.tr(tx_out);
        return vec![
            if change_amt>=tip { 
                Some(TxOut{ value: change_amt, script_pubkey:change_addr.clone() })
            } else {None},
            Some(TxOut{ value: amount, script_pubkey:Address::from_str(&to_addr).unwrap().script_pubkey()})
        ].iter().filter(|f|f.is_some()).map(|f|f.unwrap());
    
    }).collect(); 
    // send_out; 
    
        return Transaction{ version: 0, lock_time: 0, input, output };
 
        }
    
    }
    //     let tip:u64=200;
    //     let output:(Vec<TxOut>)=previous_tx_list.iter().flat_map(|previous_tx|{
    //     let value=self.find_relevent_utxo(previous_tx,tx_out_mapping).iter().map(|v|v.value).count() as u64;
    //     let change_amt=value-(amount+tip);
    //     return vec![
    //         if change_amt>=tip { 
    //             Some(TxOut{ value: change_amt, script_pubkey:change_addr.clone() })
    //         } else {None},
    //         Some(TxOut{ value: amount, script_pubkey:Address::from_str(&to_addr).unwrap().script_pubkey()})
    //     ].iter().filter(|f|f.is_some()).map(|f|f.unwrap());
    
    // }).collect(); 
    // // send_out; 
    
    //     return Transaction{ version: 0, lock_time: 0, input, output };
    // }
    
    pub fn find_relevent_utxo<F>(&self, previous_tx:&Transaction, tx_out_mapping:F)->Vec<TxOut>
            where Self:Sized, F:Copy, F:FnMut(&bitcoin::TxOut)->TxOut {
            return previous_tx.output.iter()
            .filter(|tx_out|tx_out.script_pubkey.eq(&self.schema.map_ext_keys(&self.wallet_keys.0).script_pubkey()))
            .map( tx_out_mapping).collect();
    }
    
    
}