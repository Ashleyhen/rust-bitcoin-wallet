
use std::str::FromStr;
use bitcoin::{Transaction, Script, TxOut, TxIn, Address};
use crate::btc_wallet::{AddressSchema, WalletKeys};

// type Thunk = Box<dyn Fn() + Send + 'static>;
 

type Thunk =   Box<dyn (FnMut(&TxOut)->TxOut)>;
#[derive( Clone)]
pub struct UnlockAndSend< T:AddressSchema>{
    schema: T,
    wallet_keys:WalletKeys 
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
        previous_tx_list:Vec<Transaction>,
        tx_out_mapping:F)->Transaction
    where 
    Self:Sized,
    F:Copy,
    F:FnMut(&bitcoin::TxOut)->TxOut
    {
        let tip:u64=200;
        let (output,input):(Vec<TxOut>,Vec<TxIn>)=previous_tx_list.iter().enumerate().map(|(i, previous_tx)|{
            // getting previous lock
            let tx_out:Vec<TxOut>=self.find_relevent_utxo(previous_tx,tx_out_mapping);

            let value=tx_out[i].value;
        // create transaction
        let change_amt=value-(amount+tip);
        let send_out=vec![
            if change_amt>=tip { 
                Some(TxOut{ value: change_amt, script_pubkey:change_addr.clone() })
            } else {None},
            Some(TxOut{ value: amount, script_pubkey:Address::from_str(&to_addr).unwrap().script_pubkey()})
        ].iter().filter(|f|f.is_some()).map(|f|f.clone().unwrap()).collect();

          return (send_out,previous_tx.input.clone());
        }).reduce(|(a_out,a_input),(b_out,b_input)|([a_out,b_out].concat(),[a_input,b_input].concat())).unwrap();
        return Transaction{ version: 0, lock_time: 0, input, output };
    }
    
    pub fn find_relevent_utxo<F>(&self, previous_tx:&Transaction, tx_out_mapping:F)->Vec<TxOut>
            where Self:Sized, F:Copy, F:FnMut(&bitcoin::TxOut)->TxOut {
            return previous_tx.output.iter()
            .filter(|tx_out|tx_out.script_pubkey.eq(&self.schema.map_ext_keys(&self.wallet_keys.0).script_pubkey()))
            .map( tx_out_mapping).collect();
    }
    
    
}