use std::{env, rc::Rc, str::FromStr, sync::Arc};

use bitcoin::{Address, Script, XOnlyPublicKey};
use bitcoin_hashes::hex::{FromHex, ToHex};
use bitcoin_wallet::configuration::tap_script_demo::script_demo;

use crate::{
    bitcoin_wallet::{
        constants::{NETWORK, SEED},
        input_data::regtest_call::RegtestCall,
    },
    simple_wallet::{
        p2tr_key::P2TR,
        p2tr_script::{self, bob_scripts, create_address, preimage, P2TRS},
        p2wpkh::P2WPKH,
        p2wsh::P2WSH,
        single_output, SendToImpl, Wallet,
    },
};

pub mod bitcoin_wallet;
pub mod simple_wallet;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    script_demo();
}

#[test]
fn test_tap_root_key_sig() {
    println!("Testing layer 1 pay to tap root with key signature");
    let client = RegtestCall::init(
        &vec!["bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj"],
        "my_wallet",
        110,
    );

    P2TR::new(Some(SEED), &client).send(single_output());
}

#[test]
fn test_pay_2_witness_public_key_hash() {
    println!("Testing layer 1 pay to witness public key signature");
    let client = RegtestCall::init(
        &vec!["bcrt1qzvsdwjay5x69088n27h0qgu0tm4u6gwqgxna9d"],
        "my_wallet",
        110,
    );
    P2WPKH::new(Some(SEED), &client).send(single_output());
}

#[test]
fn test_pay_2_witness_script_hash() {
    println!("Testing layer 1 pay to witness script signature");

    let alice_seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";

    let bob_seed = "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9";

    type P2wsh<'a> = P2WSH<'a, RegtestCall>;

    let alice_pub_key = P2wsh::seed_to_pubkey(&Some(alice_seed));

    let bob_pub_key = P2wsh::seed_to_pubkey(&Some(bob_seed));

    let pub_keys = vec![bob_pub_key, alice_pub_key];
    let target_address = P2wsh::multi_sig_address(&pub_keys);

    println!("target address {}", target_address.to_string());

    let client = RegtestCall::init(&vec![&target_address.to_string()], "my_wallet", 110);
    let output = single_output();
    let alice_psbt = P2WSH::new(&Some(alice_seed), &client).parital_sig(&pub_keys, None, &output);

    let bob = P2WSH::new(&Some(bob_seed), &client);

    let bob_psbt = bob.parital_sig(&pub_keys, Some(alice_psbt), &output);

    bob.broadcasted(bob_psbt);
}

#[test]
fn test_pay_2_taproot_script() {
    println!("Testing layer 1 pay to witness public key signature");

    let alice_seed = "2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90";

    let bob_seed = "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9";

    let bob_image = "107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f";

    type P2trs<'a> = P2TRS<'a, RegtestCall>;

    let alice_xonly = p2tr_script::seed_to_xonly(&Some(alice_seed));

    let bob_xonly = p2tr_script::seed_to_xonly(&Some(bob_seed));

    let preimage = preimage(bob_image);

    let output = create_address(alice_xonly, bob_xonly, preimage);

    let address = Address::from_script(&output.clone().witness_script.unwrap(), NETWORK).unwrap();

    let client = RegtestCall::init(&vec![&address.to_string()], "my_wallet", 110);

    let bob_wallet = P2trs::new(bob_seed, bob_image, &client);

    bob_wallet.sign(&output, single_output());
}
