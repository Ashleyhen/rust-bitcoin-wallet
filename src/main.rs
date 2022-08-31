use std::{env, ops::Add, str::FromStr, sync::Arc};

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    hashes::hex::FromHex,
    psbt::{Input, Output},
    secp256k1::{Secp256k1, SecretKey},
    util::{
        bip32::{ExtendedPrivKey, ExtendedPubKey},
        taproot::ControlBlock,
    },
    Address, KeyPair, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};
use bitcoin_hashes::Hash;
use bitcoin_wallet::{
    address_formats::{derive_derivation_path, map_seeds_to_scripts, map_tr_address},
    constants::NETWORK,
    input_data::{electrum_rpc::ElectrumRpc, tapscript_ex_input::TapscriptExInput, RpcCall},
    script_services::psbt_factory::{create_partially_signed_tx, get_output},
    spending_path::tap_script_spending_ex::TapScriptSendEx,
};

use bitcoincore_rpc::{jsonrpc::Request, Client, RpcApi};
use configuration::script_demo;
use either::Either;
use miniscript::{psbt::PsbtExt, ToPublicKey};
use wallet_test::{tapscript_example_with_tap::Test, wallet_test_vector_traits::WalletTestVectors};

use crate::bitcoin_wallet::{
    input_data::{
        regtest_rpc::RegtestRpc, reuse_rpc_call::ReUseCall, tapscript_ex_input::get_signed_tx,
    },
    spending_path::p2tr_key_path::P2tr,
};

pub mod wallet_test;

pub mod lighting_wallet;
pub mod bitcoin_wallet;
pub mod configuration;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    // key_tx();
    // script_tx();
    // script_demo();
    let client = Client::new(
        "http://127.0.0.1:18443",
        bitcoincore_rpc::Auth::UserPass("polaruser".to_string(), "polarpass".to_owned()),
    )
    .unwrap();

    client.get_block_count();
    let reg_test = RegtestRpc::new(&vec![
        "bcrt1p5kaqsuted66fldx256lh3en4h9z4uttxuagkwepqlqup6hw639gsm28t6c".to_owned(),
    ]);

    dbg!(reg_test.script_get_balance());

    // let adresses =
    //     Address::from_str("bcrt1p5kaqsuted66fldx256lh3en4h9z4uttxuagkwepqlqup6hw639gsm28t6c")
    //         .unwrap();
    // let unspent = client
    //     .list_unspent(None, None, Some(&vec![&adresses]), None, None)
    //     .unwrap();

    // dbg!(unspent);

    // Test();
}

pub fn key_tx() {
    let secp = Secp256k1::new();
    let seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";
    let tr = vec![
        "tb1puma0fas8dgukcvhm8ewsganj08edgnm6ejyde3ev5lvxv4h7wqvqpjslxz".to_string(),
        "tb1phtgnyv6qj4n6kqkmm2uzg630vz2tmgv4kchdp44j7my6qre4qdys6hchvx".to_string(),
        "tb1p95xjusgkgh2zqhyr5q9hzwv607yc5dncnsastm9xygmmuu4xrcqs53468m".to_string(),
        "tb1pz6egnzpq0h92zjkv23vdt4gwy8thd4t0t66megj20cr32m64ds4qv2kcal".to_string(),
        "tb1p69eefuuvaalsdljjyqntnrrtc4yzpc038ujm3ppze8g6ljepskks2zzffj".to_string(),
    ];
    let derivation_fn = derive_derivation_path(
        ExtendedPrivKey::new_master(NETWORK, &SecretKey::from_str(seed).unwrap().secret_bytes())
            .unwrap(),
        341,
    );
    let private_ext_keys = (0..5)
        .map(|index| derivation_fn(0, index))
        .collect::<Vec<ExtendedPrivKey>>();
    let key_pair = private_ext_keys
        .iter()
        .map(|prv| KeyPair::from_secret_key(&secp, prv.private_key))
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
    let unlock_func = tr.input_factory(&key_pair[3]);
    let psbt =
        create_partially_signed_tx(output_func, P2tr::create_tx(10000), unlock_func)(&electrum);
    let finalize = psbt.finalize(&secp).unwrap().extract_tx();
    // dbg!(Address::from_script(&finalize.output[1].script_pubkey, NETWORK).unwrap().to_string());
    // dbg!(Address::from_script(&finalize.output[0].script_pubkey, NETWORK).unwrap().to_string());
    dbg!(finalize.clone());
    let tx = finalize.clone();
}

// tb1paq75m2jlhjeywx75g3t08d8yplt5w9a0ecar3mdp5ay3laxva7vqng2jak
