use std::env;

use configuration::{
    p2wpkh_demo::pay_to_witness_pub_key_hash, p2wsh_demo::pay_to_witness_pub_script_hash,
    tap_key_demo::key_sign, tap_script_demo::script_demo,
};
use lighting_wallet::{rpc_call::tls_certificate::TLSCertificate, testrpc};

pub mod bitcoin_wallet;
pub mod configuration;
pub mod lighting_wallet;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    // testrpc();

    TLSCertificate::from_path("/home/ash/.polar/networks/1/volumes/lnd/alice/tls.cert").unwrap();
}

fn layer1() {
    key_sign();
    script_demo();
    pay_to_witness_pub_script_hash();
    pay_to_witness_pub_key_hash();
}
