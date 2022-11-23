use std::{env, str::from_utf8};

use configuration::{
    p2wpkh_demo::pay_to_witness_pub_key_hash, p2wsh_demo::pay_to_witness_pub_script_hash,
    tap_key_demo::key_sign, tap_script_demo::script_demo,
};
use lighting_wallet::{ testrpc, rpc_call::tls_certificate::LnClient};

pub mod bitcoin_wallet;
pub mod configuration;
pub mod lighting_wallet;

 fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(LnClient::new().get(1,"fees",|response|{
        dbg!(from_utf8(&response).unwrap());
    }));
    
}

fn layer1() {
    key_sign();
    script_demo();
    pay_to_witness_pub_script_hash();
    pay_to_witness_pub_key_hash();
}
