use std::vec;

use bitcoin::{secp256k1::Secp256k1, util::bip32::KeySource, KeyPair};
use miniscript::psbt::PsbtExt;

use crate::bitcoin_wallet::{
    constants::LOG,
    input_data::regtest_rpc::RegtestRpc,
    script_services::psbt_factory::{create_partially_signed_tx, default_output, get_output},
    spending_path::{get_script_addresses, p2wsh_path::P2wsh, single_create_tx},
};

pub fn pay_to_witness_pub_script_hash() {
    let alice_seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094"; //alice
    let bob_seed = "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9"; //bob
    let secp = Secp256k1::new();

    let alice_key_pair = KeyPair::from_seckey_str(&secp, &alice_seed.to_string()).unwrap();
    let bob_key_pair = KeyPair::from_seckey_str(&secp, &bob_seed.to_string()).unwrap();

    let public_k_list = vec![
        (alice_key_pair.public_key(), KeySource::default()),
        (bob_key_pair.public_key(), KeySource::default()),
    ];
    let api = RegtestRpc::from_string(
        &vec!["bcrt1q8sjkz7a37sy08u27r58c584gwdjmtp7g8erd3f4f9frmnnvfwfqsss86dg"],
        Some(Box::new(|tx_handler| tx_handler[..2].to_vec())),
    );
    let p2wsh = P2wsh::new(&secp);

    let output_vec_vec_func = || vec![p2wsh.output_factory(&public_k_list)];

    let unlock_func = p2wsh.input_factory(bob_key_pair.secret_key(), alice_key_pair.secret_key());

    let psbt =
        create_partially_signed_tx(output_vec_vec_func(), single_create_tx(), unlock_func)(&api);

    if LOG {
        let output_list = get_output(output_vec_vec_func(), &mut default_output());
        get_script_addresses(output_list).iter().for_each(|addr| {
            dbg!(addr.script_pubkey());
        });
    }

    let tx_id = psbt
        .finalize(&secp)
        .map(|finalized| api.transaction_broadcast(&finalized.extract(&secp).unwrap()))
        .unwrap();

    println!("p2wsh tx broadcasted successfully tx hash: {}", tx_id)
}
