use std::str::FromStr;

use bitcoin::{
    secp256k1::{Secp256k1, SecretKey},
    Address, KeyPair,
};

use crate::bitcoin_wallet::{
    input_data::regtest_rpc::RegtestRpc,
    script_services::psbt_factory::{create_partially_signed_tx, default_output, get_output},
    spending_path::{
        get_script_addresses, single_create_tx, single_output,
        tap_script_spending_ex::TapScriptSendEx,
    },
};

pub fn script_demo() {
    let alice_secret = 0;
    let bob_secret = 1;
    let internal_secret = 2;
    let secp = Secp256k1::new();
    let seeds = vec![
        "2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90", //alice
        "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9", //bob
        "6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5333", //internal
    ];

    let keys = seeds
        .iter()
        .map(|scrt| KeyPair::from_secret_key(&secp, &SecretKey::from_str(&scrt).unwrap()))
        .collect::<Vec<KeyPair>>();

    let tap_script = TapScriptSendEx::new(&secp);

    let my_add = Address::from_str(
        &"tb1ppjj995khlhftanw7ak4zyzu3650rlmpfr9p4tafegw3u38h7vx4q7lnavj".to_string(),
    )
    .unwrap()
    .script_pubkey();

    let output_factory = || vec![single_output(&my_add)];
    let x_internal = keys[internal_secret].public_key().x_only_public_key().0;
    let x_alice = keys[alice_secret].public_key().x_only_public_key().0;
    let x_bob = &keys[bob_secret].public_key().x_only_public_key().0;
    let output_func = tap_script.output_factory(&x_internal, &x_alice, &x_bob);

    get_script_addresses(get_output(vec![output_func], &mut default_output()))
        .iter()
        .for_each(|f| println!("target address {}", f.to_string()));

    let api = RegtestRpc::from_string(
        &vec!["bcrt1ppjj995khlhftanw7ak4zyzu3650rlmpfr9p4tafegw3u38h7vx4qnxemeg"],
        Some(Box::new(|tx_handler| tx_handler[..3].to_vec())),
    );

    let lock_func = single_create_tx();

    let unlock_func = || {
        tap_script.input_factory(
            &keys[bob_secret],
            keys[internal_secret].public_key().x_only_public_key().0,
        )
    };
    let psbt = create_partially_signed_tx(output_factory(), lock_func, unlock_func())(&api);

    let tx = TapScriptSendEx::finialize_script(
        psbt,
        &keys[bob_secret].public_key().x_only_public_key().0,
    );

    let tx_id = api.transaction_broadcast(&tx);
    println!("tx broadcasted successfully tx hash: {}", tx_id)
}
