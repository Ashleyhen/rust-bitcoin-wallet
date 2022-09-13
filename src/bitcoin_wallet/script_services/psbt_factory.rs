use std::{collections::BTreeMap, sync::Arc};

use bitcoin::{
    psbt::{Input, Output, PartiallySignedTransaction},
    Transaction, TxIn,
};

use crate::bitcoin_wallet::input_data::RpcCall;

pub type UnlockFn<'a> = Box<dyn FnOnce(&mut Input) + 'a>;

pub type LockFn<'a> = Box<dyn FnMut(&mut Output) + 'a>;

pub type CreateTxFn<'a> = Box<dyn Fn(Vec<Output>, Vec<TxIn>, u64) -> Transaction + 'a>;

pub type SpendFn<'a>=Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<UnlockFn<'a>> + 'a>;

pub fn create_partially_signed_tx<'a, R>(
    output_vec_vec_func: Vec<Vec<LockFn>>,
    lock_func: CreateTxFn<'a>,
    unlock_func: SpendFn<'a>,
) -> Box<dyn Fn(&R) -> PartiallySignedTransaction + 'a>
where
    R: RpcCall,
{
    let output_vec = get_output(output_vec_vec_func);

    return Box::new(move |api_call| {
        let confirmed = api_call.script_get_balance();
        let previous_tx = api_call.contract_source();
        let tx_in = api_call.prev_input();
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
pub fn get_output<'a>(output_vec_vec_func: Vec<Vec<LockFn>>) -> Vec<Output> {
    let mut output_vec: Vec<Output> = Vec::<Output>::new();
    for func_list in output_vec_vec_func {
        let mut output = Output::default();
        for mut func in func_list {
            func(&mut output);
        }
        output_vec.push(output);
    }
    return output_vec;
}
