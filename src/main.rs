use std::env;

use configuration::{
    p2wsh_demo::pay_to_witness_pub_script_hash, tap_key_demo::key_sign,
    tap_script_demo::script_demo,
};

pub mod bitcoin_wallet;
pub mod configuration;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    key_sign();
    script_demo();
    pay_to_witness_pub_script_hash()
}
