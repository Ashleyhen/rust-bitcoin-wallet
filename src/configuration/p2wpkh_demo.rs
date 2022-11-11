use std::str::FromStr;

use bitcoin::{
    secp256k1::{Secp256k1, SecretKey},
    Address, PrivateKey, PublicKey, Script, WPubkeyHash,
};
use miniscript::psbt::PsbtExt;

use crate::bitcoin_wallet::{
    constants::{LOG, NETWORK},
    input_data::regtest_rpc::RegtestRpc,
    script_services::psbt_factory::{create_partially_signed_tx, default_output, get_output},
    spending_path::{
        get_script_addresses, p2wpkh_script_path::P2wpkh, single_create_tx, single_output,
    },
};

pub fn pay_to_witness_pub_key_hash() {
    let seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094"; //alice
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_str(seed).unwrap();
    let pubkey_hash = PublicKey::from_private_key(&secp, &PrivateKey::from_str(seed).unwrap());

    // WPubkeyHash::f
    let script = Script::new_v0_p2wpkh(&pubkey_hash.wpubkey_hash().unwrap());
    let address = Address::from_script(&script, NETWORK).unwrap();

    let p2wpkh = P2wpkh::new(&secp);

    let api = RegtestRpc::from_address(
        vec![address],
        Some(Box::new(|tx_handler| tx_handler[..1].to_vec())),
    );

    let single_tx = single_create_tx();
    let output_vec_vec_func = || vec![single_output(&script)];
    let unlock_func = p2wpkh.input_factory(secret_key);
    let psbt = create_partially_signed_tx(output_vec_vec_func(), single_tx, unlock_func)(&api);

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

    println!("p2wpk tx broadcasted successfully tx hash: {}", tx_id)
}
