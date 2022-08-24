use std::str::FromStr;

use bitcoin::{secp256k1::{Secp256k1, SecretKey}, KeyPair, Address};

use crate::bitcoin_wallet::{spending_path::{tap_script_spending_ex::TapScriptSendEx, p2tr_key_path::P2tr}, address_formats::{map_seeds_to_scripts, map_tr_address}, script_services::psbt_factory::{get_output, create_partially_signed_tx}, input_data::{electrum_rpc::ElectrumRpc, RpcCall}};

pub fn script_demo() {
    let seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";
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
        .map(|scrt| KeyPair::from_secret_key(&secp, SecretKey::from_str(&scrt).unwrap()))
        .collect::<Vec<KeyPair>>();

    let tap_script = TapScriptSendEx::new(&secp);
    let tap_key = P2tr::new(&secp);

    let addr_generator =
        map_seeds_to_scripts(Some(seed.to_string()), &secp, 341, map_tr_address(None));
    let addr_list = (0..5)
        .map(|i| addr_generator(0, i))
        .collect::<Vec<Address>>();

    let my_add = Address::from_str(
        &"tb1ppjj995khlhftanw7ak4zyzu3650rlmpfr9p4tafegw3u38h7vx4q7lnavj".to_string(),
    )
    .unwrap()
    .script_pubkey();
    
    let output_factory = ||vec![vec![tap_key.single_output(my_add.clone())]];
    let output_func = || {
        vec![tap_script.output_factory(
            keys[internal_secret].public_key(),
            keys[alice_secret].public_key(),
            keys[bob_secret].public_key(),
        )]
    };

    TapScriptSendEx::get_script_addresses(get_output(output_func()))
        .iter()
        .for_each(|f| println!("target address {}", f.to_string()));

    let electrum = ElectrumRpc::new(&my_add);
    dbg!(electrum.script_get_balance());
    let lock_func = TapScriptSendEx::create_tx();
    let unlock_func =
        tap_script.input_factory(&keys[bob_secret], keys[internal_secret].public_key());
    let psbt = create_partially_signed_tx(output_factory(), lock_func, unlock_func)(&electrum);
    // dbg!(psbt);
    let tx = TapScriptSendEx::finialize_script(psbt, &keys[bob_secret].public_key());
    // let tx_id = electrum.transaction_broadcast(tx);
    // dbg!(tx_id);
    //
}
// tb1paq75m2jlhjeywx75g3t08d8yplt5w9a0ecar3mdp5ay3laxva7vqng2jak
