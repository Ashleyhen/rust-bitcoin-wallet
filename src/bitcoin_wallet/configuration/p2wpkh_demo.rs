use std::str::FromStr;

use bitcoin::{
    secp256k1::{Secp256k1, SecretKey},
    Address, PrivateKey, PublicKey, Script,
};

use crate::bitcoin_wallet::{
    constants::{LOG, NETWORK, SEED},
    input_data::{electrum_rpc::ElectrumRpc, regtest_rpc::RegtestRpc},
    script_services::psbt_factory::{create_partially_signed_tx, default_output, get_output},
    spending_path::{
        get_script_addresses, p2wpkh_script_path::P2wpkh, single_create_tx, single_output,
    },
};

pub fn get_private_key_from_seed() -> PrivateKey {
    let secret_key = SecretKey::from_str(SEED).unwrap();
    return PrivateKey::new(secret_key, NETWORK);
}
pub fn get_private_key() -> PrivateKey {
    let seed = "KxmekbLSnJzpwo4si6bkm7XwfCCie6LVnbkPcC4hzp82mvMPppi7";
    return PrivateKey::from_str(seed).unwrap();
}
pub fn pay_to_witness_pub_key_hash() {
    let secp = Secp256k1::new();

    let private_key = get_private_key();
    let public_key = private_key.public_key(&secp);
    let addr = Address::p2wpkh(&private_key.public_key(&secp), NETWORK);
    // dbg!(addr.unwrap().to_string());

    let pubkey_hash = PublicKey::from_private_key(&secp, &private_key);

    let script = Script::new_v0_p2wpkh(&pubkey_hash.wpubkey_hash().unwrap());
    let address = Address::from_script(&script, NETWORK).unwrap();

    let p2wpkh = P2wpkh::new(&secp);

    // let api = RegtestRpc::from_address(
    //     vec![address],
    //     Some(Box::new(|tx_handler| tx_handler[..2].to_vec())),
    // );

    let api = ElectrumRpc::new(&addr.unwrap().script_pubkey());

    let single_tx = single_create_tx();
    let output_vec_vec_func = || vec![single_output(&script)];
    let unlock_func = p2wpkh.input_factory(private_key.inner);
    let psbt = create_partially_signed_tx(output_vec_vec_func(), single_tx, unlock_func)(&api);

    // if LOG {
    //     let output_list = get_output(output_vec_vec_func(), &mut default_output());
    //     get_script_addresses(output_list).iter().for_each(|addr| {
    //         dbg!(addr.script_pubkey());
    //     });
    // }

    // let tx_id = psbt
    //     .finalize(&secp)
    //     .map(|finalized| api.transaction_broadcast(&finalized.extract(&secp).unwrap()))
    //     .unwrap();
}
