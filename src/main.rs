use std::env;

use configuration::{
    p2wsh_demo::pay_to_witness_pub_key_hash,
};

pub mod bitcoin_wallet;
pub mod configuration;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    // key_sign();
    // script_demo();
    pay_to_witness_pub_key_hash()
}

// tb1paq75m2jlhjeywx75g3t08d8yplt5w9a0ecar3mdp5ay3laxva7vqng2jak
