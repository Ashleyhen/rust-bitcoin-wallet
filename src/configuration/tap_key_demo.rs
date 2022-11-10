use std::str::FromStr;

use bitcoin::{secp256k1::Secp256k1, Address, KeyPair};
use miniscript::psbt::PsbtExt;

use crate::bitcoin_wallet::{
    constants::NETWORK,
    input_data::regtest_rpc::RegtestRpc,
    script_services::psbt_factory::create_partially_signed_tx,
    spending_path::{p2tr_key_path::P2tr, single_create_tx, single_output},
};

pub fn key_sign() {
    let seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";
    let secp = Secp256k1::new();
    let from_key_pair = KeyPair::from_seckey_str(&secp, &seed.to_string()).unwrap();
    let from_address = Address::p2tr(&secp, from_key_pair.x_only_public_key().0, None, NETWORK);

    let to_address =
        Address::from_str("bcrt1ppjj995khlhftanw7ak4zyzu3650rlmpfr9p4tafegw3u38h7vx4qnxemeg")
            .unwrap();
    let tap_fn = P2tr::new(&secp);

    let output_factory = || vec![single_output(to_address.clone().script_pubkey())];
    let lock_func = single_create_tx();

    let unlock_func = tap_fn.input_factory(&from_key_pair, from_address.script_pubkey());
    let address_list = vec![from_address.clone()].to_vec();
    let api = RegtestRpc::from_address(
        address_list,
        Some(Box::new(|tx_handler| tx_handler[..3].to_vec())),
    );
    let psbt = create_partially_signed_tx(output_factory(), lock_func, unlock_func)(&api);
    let tx_id = psbt
        .finalize(&secp)
        .map(|finalized| api.transaction_broadcast(&finalized.extract(&secp).unwrap()))
        .unwrap();

    println!("tx broadcasted successfully tx hash: {}", tx_id)
}
