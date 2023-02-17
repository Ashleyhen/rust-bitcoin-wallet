use std::{env, str::from_utf8};

use bitcoin_wallet::{
    constants::SEED, input_data::regtest_call::RegtestCall,
};
use simple_wallet::{p2tr::p2tr, p2wpkh};

pub mod bitcoin_wallet;
pub mod simple_wallet;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");

    let client = RegtestCall::init(
        &vec!["bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj"],
        "my_wallet",
        150,
    );

    p2tr(Some(SEED), client);
}

#[test]
fn test_tap_root_key_sig(){
    let client = RegtestCall::init(
        &vec!["bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj"],
        "my_wallet",
        150,
    );
    p2tr(Some(SEED), client);
}

#[test]
fn test_pay_2_witness_public_key_hash(){
    let client = RegtestCall::init(
        &vec!["bcrt1qzvsdwjay5x69088n27h0qgu0tm4u6gwqgxna9d"],
        "my_wallet",
        150,
    );
    p2wpkh::p2wpkh(Some(SEED), client);
}

