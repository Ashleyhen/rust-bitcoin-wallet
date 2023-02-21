use std::env;

use bitcoin_wallet::{constants::SEED, input_data::regtest_call::RegtestCall, configuration::{p2wpkh_demo, p2wsh_demo}};

use crate::simple_wallet::{p2tr_key::P2TR, p2wpkh::P2WPKH, p2wsh::P2WSH, Wallet};

pub mod bitcoin_wallet;
pub mod simple_wallet;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");

}

#[test]
fn test_tap_root_key_sig() {
    println!("Testing layer 1 pay to tap root with key signature");
    let client = RegtestCall::init(
        &vec!["bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj"],
        "my_wallet",
        110,
    );

    P2TR::new(Some(SEED), &client).send();
}

#[test]
fn test_pay_2_witness_public_key_hash() {
    println!("Testing layer 1 pay to witness public key signature");
    let client = RegtestCall::init(
        &vec!["bcrt1qzvsdwjay5x69088n27h0qgu0tm4u6gwqgxna9d"],
        "my_wallet",
        110,
    );
    P2WPKH::new(Some(SEED), &client).send();
}

#[test]
fn test_pay_2_witness_script_hash() {
    println!("Testing layer 1 pay to witness public key signature");

    let alice_seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";

    let bob_seed = "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9";

    type P2wsh<'a> = P2WSH<'a, RegtestCall>;

    let alice_pub_key = P2wsh::seed_to_pubkey(&Some(alice_seed));

    let bob_pub_key = P2wsh::seed_to_pubkey(&Some(bob_seed));
    
    let pub_keys=vec![bob_pub_key,alice_pub_key];
    let target_address = P2wsh::multi_sig_address(&pub_keys);

    println!("target address {}", target_address.to_string());

    let client = RegtestCall::init(&vec![&target_address.to_string()], "my_wallet", 110);

    let alice_psbt = P2WSH::new(&Some(alice_seed), &client).parital_sig(&pub_keys, None);

    let bob =
        P2WSH::new(&Some(bob_seed), &client);

    let bob_psbt= bob.parital_sig(&pub_keys, Some(alice_psbt));

    bob.broadcasted(bob_psbt);
    
}
