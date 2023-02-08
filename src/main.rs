use std::{env, str::from_utf8};

use bitcoin_wallet::configuration::{
    p2wpkh_demo::pay_to_witness_pub_key_hash, p2wsh_demo::pay_to_witness_pub_script_hash,
    tap_key_demo::key_sign, tap_script_demo::script_demo,
};

pub mod bitcoin_wallet;

 fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    key_sign();
    script_demo();
    pay_to_witness_pub_script_hash();
    pay_to_witness_pub_key_hash();
    
}