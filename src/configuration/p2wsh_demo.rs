use std::ops::Add;

use bitcoin::{secp256k1::Secp256k1, util::bip32::KeySource, Address, KeyPair, PublicKey, Script};
use miniscript::ToPublicKey;

use crate::bitcoin_wallet::{
    constants::NETWORK,
    script_services::{output_service::segwit_v0, psbt_factory::get_output},
    scripts::p2wsh_multi_sig,
    spending_path::{
        get_script_addresses, p2wpkh_script_path::P2wpkh, p2wsh_path::P2wsh, single_create_tx,
    },
};

pub fn pay_to_witness_pub_key_hash() {
    let alice_seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094"; //alice
    let bob_seed = "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9"; //bob
    let secp = Secp256k1::new();

    let alice_key_pair = KeyPair::from_seckey_str(&secp, &alice_seed.to_string()).unwrap();
    let bob_key_pair = KeyPair::from_seckey_str(&secp, &bob_seed.to_string()).unwrap();

    let public_k_list = vec![
        (alice_key_pair.public_key(), KeySource::default()),
        (bob_key_pair.public_key(), KeySource::default()),
    ];

    // segwit_v0(public_k_list);
    //
    let p2wsh = P2wsh::new(&secp);

    // single_create_tx();

    // get_output(output_vec_vec_func, output_vec);
    // get_script_addresses(output_list);
}
