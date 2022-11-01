use std::{collections::BTreeMap, sync::Arc};

use bitcoin::{
    psbt::{Input, Output, PartiallySignedTransaction},
    secp256k1::{ffi::secp256k1_ec_seckey_tweak_add, All, Secp256k1},
    SchnorrSig, Transaction, TxIn,
};

use crate::bitcoin_wallet::{constants::NETWORK, input_data::RpcCall};

pub type UnlockFn<'a> = Box<dyn FnOnce(&mut Input) + 'a>;

pub type LockFn<'a> = Box<dyn FnMut(&mut Output) + 'a>;

pub type CreateTxFn<'a> = Box<dyn Fn(Vec<Output>, Vec<TxIn>, u64) -> Transaction + 'a>;

pub type SpendFn<'a> = Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<Vec<UnlockFn<'a>>> + 'a>;

pub fn create_partially_signed_tx<'a, R>(
    output_vec_vec_func: Vec<Vec<LockFn>>,
    lock_func: CreateTxFn<'a>,
    unlock_func: SpendFn<'a>,
) -> Box<dyn Fn(&R) -> PartiallySignedTransaction + 'a>
where
    R: RpcCall,
{
    let mut output_list = Vec::<Output>::new();
    let output_vec = get_output(output_vec_vec_func, &mut output_list);
    return Box::new(move |api_call| {
        let confirmed = api_call.script_get_balance();
        let previous_tx = api_call.contract_source();
        let tx_in = api_call.prev_input();

        let unsigned_tx = lock_func(output_vec.clone(), tx_in, confirmed);

        let mut input_vec = Vec::<Input>::new();
        for func_list in unlock_func(previous_tx.clone(), unsigned_tx.clone()) {
            let mut input = Input::default();
            for func in func_list {
                func(&mut input);
            }
            input_vec.push(input);
        }

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

pub fn modify_partially_signed_tx<'a, 'b, R>(
    psbt: &'a mut PartiallySignedTransaction,
    output_vec_vec_func: Vec<Vec<LockFn>>,
    lock_func: &'a CreateTxFn<'b>,
    unlock_func: SpendFn<'b>,
) -> Box<dyn FnMut(&R) -> PartiallySignedTransaction + 'a>
where
    R: RpcCall,
{
    let output_vec = get_output(output_vec_vec_func, &mut psbt.outputs);
    return Box::new(move |api_call| {
        let confirmed = api_call.script_get_balance();
        let previous_tx = api_call.contract_source();
        let tx_in = api_call.prev_input();
        let unsigned_tx = lock_func(output_vec.clone(), tx_in, confirmed);
        let input_vec = get_input(&unlock_func, previous_tx, &unsigned_tx, &mut psbt.inputs);
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

fn get_input<'a, 'b>(
    unlock_func: &'a SpendFn<'b>,
    previous_tx: Vec<Transaction>,
    unsigned_tx: &Transaction,
    input_vec: &'a mut Vec<Input>,
) -> Vec<Input> {
    for func_list in unlock_func(previous_tx.clone(), unsigned_tx.clone()) {
        let mut input = Input::default();
        for mut func in func_list {
            func(&mut input);
        }
        input_vec.push(input);
    }
    return input_vec.to_vec();
}

pub fn get_output<'a>(
    output_vec_vec_func: Vec<Vec<LockFn>>,
    output_vec: &'a mut Vec<Output>,
) -> Vec<Output> {
    for func_list in output_vec_vec_func {
        let mut output = Output::default();
        for mut func in func_list {
            func(&mut output);
        }
        output_vec.push(output);
    }
    return output_vec.to_vec();
}
pub fn merge_psbt(
    secp: &Secp256k1<All>,
    psbt: &PartiallySignedTransaction,
    psbt_2: &PartiallySignedTransaction,
) -> PartiallySignedTransaction {
    let input_list = psbt
        .inputs
        .iter()
        .zip(psbt_2.inputs.iter())
        .map(|(input_1, input_2)| {
            let mut sig = input_1.tap_key_sig.unwrap().to_vec();
            unsafe {
                // let combined_sig=input_1.tap_key_sig.unwrap().sig.as_mut_ptr();

                println!("before: {:#?}", sig);
                // secp.verify_schnorr(sig, msg, pubkey)
                let is_successful = secp256k1_ec_seckey_tweak_add(
                    *secp.ctx(),
                    sig.as_mut_ptr(),
                    input_2.tap_key_sig.unwrap().to_vec().as_ptr(),
                );

                println!("is successful {}", is_successful);

                println!("after: {:#?}", sig);

                // input_1.clone().tap_key_sig=Some(SchnorrSig{ sig, hash_ty:SchnorrSighashType::AllPlusAnyoneCanPay });

                input_1.clone().tap_key_sig = Some(SchnorrSig::from_slice(&sig).unwrap());
                return input_1.clone();
            }
        })
        .collect::<Vec<Input>>();

    let mut combined_psbt = psbt.clone();
    combined_psbt.inputs = input_list;
    return combined_psbt;
}

pub fn default_input() -> Vec<Input> {
    Vec::new()
}
pub fn default_output() -> Vec<Output> {
    Vec::new()
}
