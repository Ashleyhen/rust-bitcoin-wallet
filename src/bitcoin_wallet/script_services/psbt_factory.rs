use std::collections::BTreeMap;

use bitcoin::{
    psbt::{Input, Output, PartiallySignedTransaction},
    Transaction, TxIn,
};

use crate::bitcoin_wallet::input_data::RpcCall;

pub type UnlockFn<'a> = Box<dyn FnOnce(&mut Input) + 'a>;

pub type LockFn<'a> = Box<dyn FnMut(&mut Output) + 'a>;

pub fn create_partially_signed_tx<'a, R>(
    output_vec_vec_func: Vec<Vec<LockFn>>,
    lock_func: Box<dyn Fn(Vec<Output>, Vec<TxIn>, u64) -> Transaction>,
    unlock_func: Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<UnlockFn<'a>> + 'a>,
) -> Box<dyn Fn(&R) -> PartiallySignedTransaction + 'a>
where
    R: RpcCall,
{
    let mut output_vec: Vec<Output> = Vec::<Output>::new();
    for func_list in output_vec_vec_func {
        let mut output = Output::default();
        for mut func in func_list {
            func(&mut output);
        }
        output_vec.push(output);
    }

    return Box::new(move |api_call| {
        let confirmed = api_call.script_get_balance().unwrap().confirmed;
        let (tx_in, previous_tx) = api_call.contract_source();
        let unsigned_tx = lock_func(output_vec.clone(), tx_in, confirmed);
        let mut input_vec: Vec<Input> = Vec::<Input>::new();
        let mut input = Input::default();
        for func in unlock_func(previous_tx.clone(), unsigned_tx.clone()) {
            func(&mut input);
        }
        input_vec.push(input);

        return PartiallySignedTransaction {
            unsigned_tx,
            version: 2,
            xpub: BTreeMap::new(),
            proprietary: BTreeMap::new(),
            unknown: BTreeMap::new(),
            inputs: input_vec,
            outputs: output_vec.clone(),
        };
    });
}
