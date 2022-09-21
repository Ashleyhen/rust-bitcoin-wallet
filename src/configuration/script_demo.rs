use std::str::FromStr;

use bitcoin::{secp256k1::{Secp256k1, SecretKey, ffi::{secp256k1_ec_seckey_negate, secp256k1_ec_seckey_tweak_add}}, KeyPair, Address, util::bip32::{ExtendedPrivKey, ExtendedPubKey}, Script, Network, psbt::{Input, Output}, SchnorrSig, hashes::hex::FromHex};

use crate::{bitcoin_wallet::{spending_path::{tap_script_spending_ex::TapScriptSendEx, p2tr_key_path::P2tr, adaptor_script::AdaptorScript}, address_formats::{map_seeds_to_scripts, map_tr_address, derive_derivation_path, generate_key_pair}, script_services::psbt_factory::{get_output, create_partially_signed_tx, modify_partially_signed_tx, default_output}, input_data::{electrum_rpc::ElectrumRpc, RpcCall, regtest_rpc::RegtestRpc}, constants::{NETWORK, SEED}}};


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
        .map(|scrt| KeyPair::from_secret_key(&secp, &SecretKey::from_str(&scrt).unwrap()))
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

    let output_factory = || vec![tap_key.single_output(my_add.clone())];
    let output_func = || {
        vec![tap_script.output_factory(
            keys[internal_secret].public_key().x_only_public_key().0,
            keys[alice_secret].public_key().x_only_public_key().0,
            keys[bob_secret].public_key().x_only_public_key().0,
        )]
    };

    TapScriptSendEx::get_script_addresses(get_output(output_func(),&mut default_output()))
        .iter()
        .for_each(|f| println!("target address {}", f.to_string()));

    let electrum = ElectrumRpc::new(&my_add);
    dbg!(electrum.script_get_balance());
    let lock_func = TapScriptSendEx::create_tx();
    let unlock_func =||
        tap_script.input_factory(&keys[bob_secret], keys[internal_secret].public_key().x_only_public_key().0);
    let psbt = create_partially_signed_tx(output_factory(), lock_func, unlock_func())(&electrum);
    // dbg!(psbt);
    let tx = TapScriptSendEx::finialize_script(psbt, &keys[bob_secret].public_key().x_only_public_key().0);
    // send bitcoin to this address on testnet bcrt1pp375ce9lvxs8l9rlsl78u4szhqa7za748dfhtjj5ht05lufu4dwsshpxl6
    // let tx_id = electrum.transaction_broadcast(tx);
    // dbg!(tx_id);
    //
}
// tb1paq75m2jlhjeywx75g3t08d8yplt5w9a0ecar3mdp5ay3laxva7vqng2jak

pub fn adaptor_demo(){
    let secp = Secp256k1::new();
    let get_xonly_fn=|i|derive_derivation_path(generate_key_pair(Some(SEED.to_string())),32)
    (0,i).to_keypair(&secp);
    let internal =get_xonly_fn(0).public_key().x_only_public_key().0;
    let x_only_1=get_xonly_fn(1).public_key().x_only_public_key().0;//bcrt1pcfc3whp89g2va7zwndpzjgdlzr4x03754nln0n0lvnj6rupm6pjq8xkj0r
    let x_only_2=get_xonly_fn(2).public_key().x_only_public_key().0;//bcrt1p9pfqwxwe5v25mw5452nswzvwp6qnpk3q8hjduvc2p6sllr6plfjsjls0z5
    let adaptor_script=AdaptorScript::new(&secp);
    let output_vec_vec_func=||vec![adaptor_script.adaptor_script(internal, x_only_1 , x_only_2)];

    let tr = P2tr::new(&secp);
    TapScriptSendEx::get_script_addresses(get_output(output_vec_vec_func(),&mut default_output()))
        .iter()
        .for_each(|f| println!("target address {}", f.to_string()));

    
    let auxiliary=AdaptorScript::generate_auxiliary(None);

    let btc_address=vec!["bcrt1pp375ce9lvxs8l9rlsl78u4szhqa7za748dfhtjj5ht05lufu4dwsshpxl6".to_owned()]; //bitcoin address
    let eth_address=vec!["bcrt1phkshr74fsn3n04v0xm9pq0ru80n03t4hjum978zr6sp3a9zalh9sjag4pc".to_owned()]; //bitcoin address

    let btc_api_call=RegtestRpc::new(&btc_address)();
    let eth_api_call=RegtestRpc::new(&eth_address)();
    let btc_output_fn=vec![tr.single_output(Address::p2tr(&secp,x_only_2, None, NETWORK).script_pubkey())];
    let eth_output_fn=vec![tr.single_output(Address::p2tr(&secp,x_only_1, None, NETWORK).script_pubkey())];
    // dbg!(api_call.contract_source());
    
    let key_pair_btc=get_xonly_fn(1);
    let key_pair_eth=get_xonly_fn(2);
    let eth_unlock_fn=adaptor_script.adaptor_sig(&internal, &key_pair_eth, &x_only_1, &auxiliary);
    let btc_unlock_fn=adaptor_script.adaptor_sig(&internal, &key_pair_btc, &x_only_2, &auxiliary);

    let single_unlock=||TapScriptSendEx::create_tx();
    let  btc_psbt = create_partially_signed_tx(
        btc_output_fn, 
        single_unlock(), 
        btc_unlock_fn)(&btc_api_call);

    let eth_psbt=create_partially_signed_tx(eth_output_fn, single_unlock(), eth_unlock_fn)(&eth_api_call);

    let psbt=AdaptorScript::merge_psbt(btc_psbt, eth_psbt);
    let tx =AdaptorScript::finialize_script(psbt);

    let tx_id=eth_api_call.transaction_broadcast(&tx);
    dbg!(tx_id);

    let mut alice_sig=tx.input[0].clone().witness.to_vec()[0].clone();
    let mut bob_sig=tx.input[0].clone().witness.to_vec()[1].clone();
    
// let c=Secp256k1::new().ctx();

    dbg!(alice_sig.clone());
    
    unsafe{
        // secp256k1_ec_seckey_negate(*secp.ctx(),alice_sig.as_mut_ptr());
        secp256k1_ec_seckey_tweak_add(*secp.ctx(),alice_sig.as_mut_ptr(),bob_sig.as_mut_ptr());
    // secp256k1_ec_seckey_tweak_add
    };


    dbg!(alice_sig);


}

pub fn key_tx() {
    let secp = Secp256k1::new();
    let tr: Vec<std::string::String> = vec![
        "tb1puma0fas8dgukcvhm8ewsganj08edgnm6ejyde3ev5lvxv4h7wqvqpjslxz".to_string(),
        "tb1phtgnyv6qj4n6kqkmm2uzg630vz2tmgv4kchdp44j7my6qre4qdys6hchvx".to_string(),
        "tb1p95xjusgkgh2zqhyr5q9hzwv607yc5dncnsastm9xygmmuu4xrcqs53468m".to_string(),
        "tb1pz6egnzpq0h92zjkv23vdt4gwy8thd4t0t66megj20cr32m64ds4qv2kcal".to_string(),
        "tb1p69eefuuvaalsdljjyqntnrrtc4yzpc038ujm3ppze8g6ljepskks2zzffj".to_string(),
    ];

    let derivation_fn = derive_derivation_path(
        ExtendedPrivKey::new_master(NETWORK, &SecretKey::from_str(SEED).unwrap().secret_bytes())
            .unwrap(),
        341,
    );

    let private_ext_keys = (0..5)
        .map(|index| derivation_fn(0, index))
        .collect::<Vec<ExtendedPrivKey>>();
    let key_pair = private_ext_keys
        .iter()
        .map(|prv| KeyPair::from_secret_key(&secp, &prv.private_key))
        .collect::<Vec<KeyPair>>();
    let address_generate = map_tr_address(None);
    let addresses = private_ext_keys
        .iter()
        .map(|f| address_generate(&secp, ExtendedPubKey::from_priv(&secp, f)))
        .collect::<Vec<Address>>();
    let my_add = addresses[3].script_pubkey();
    let electrum = ElectrumRpc::new(&my_add);
    let tr = P2tr::new(&secp);
    let send_addr = "tb1p5kaqsuted66fldx256lh3en4h9z4uttxuagkwepqlqup6hw639gskndd0z".to_string();
    let output_func = tr.output_factory(
        Address::from_str(&send_addr).unwrap().script_pubkey(),
        addresses[3].script_pubkey(),
    );
let signer=key_pair[3];

    let unlock_func = tr.input_factory(&key_pair[3],Script::new_v1_p2tr(&secp, signer.public_key().x_only_public_key().0, None));
    let psbt =
        create_partially_signed_tx(output_func, P2tr::create_tx(10000), unlock_func)(&electrum);
    let finalize = psbt.extract_tx();
    // dbg!(Address::from_script(&finalize.output[1].script_pubkey, NETWORK).unwrap().to_string());
    // dbg!(Address::from_script(&finalize.output[0].script_pubkey, NETWORK).unwrap().to_string());
    dbg!(finalize.clone());
    let tx = finalize.clone();
}

