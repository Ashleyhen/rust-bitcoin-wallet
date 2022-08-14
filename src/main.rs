use std::{env, str::FromStr};

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    hashes::hex::FromHex,
    psbt::{Input, Output},
    secp256k1::{Secp256k1, SecretKey},
    util::taproot::ControlBlock,
    Address, KeyPair, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};
use bitcoin_hashes::Hash;
use bitcoin_wallet::{spending_path::{ create_tx, tap_script_spending_ex::TapScriptSendEx}, input_data::{tapscript_ex_input::TapscriptExInput}, script_services::psbt_factory::create_partially_signed_tx};

use either::Either;
use miniscript::ToPublicKey;
use wallet_test::{tapscript_example_with_tap::Test, wallet_test_vector_traits::WalletTestVectors};

use crate::bitcoin_wallet::{address_formats::{p2tr_addr_fmt::P2TR, AddressSchema}, input_data::{tapscript_ex_input::get_signed_tx, reuse_rpc_call::ReUseCall}, spending_path::{ p2tr_key_path::P2TRVault}, wallet_methods::{ClientWithSchema, BroadcastOp}};

pub mod wallet_test;

pub mod bitcoin_wallet;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");

    script_tx();

    Test();
}

pub fn key_tx(){
 let seed = "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";
    let tr = vec![
        "tb1puma0fas8dgukcvhm8ewsganj08edgnm6ejyde3ev5lvxv4h7wqvqpjslxz".to_string(),
        "tb1phtgnyv6qj4n6kqkmm2uzg630vz2tmgv4kchdp44j7my6qre4qdys6hchvx".to_string(),
        "tb1p95xjusgkgh2zqhyr5q9hzwv607yc5dncnsastm9xygmmuu4xrcqs53468m".to_string(),
        "tb1pz6egnzpq0h92zjkv23vdt4gwy8thd4t0t66megj20cr32m64ds4qv2kcal".to_string(),
        "tb1p69eefuuvaalsdljjyqntnrrtc4yzpc038ujm3ppze8g6ljepskks2zzffj".to_string(),
    ];
}

pub fn script_tx() {
    let alice_secret = 0;
    let bob_secret = 1;
    let internal_secret=2;
    let secp = Secp256k1::new();
    let seeds = vec![
        "2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90", //alice
        "81b637d8fcd2c6da6359e6963113a1170de795e4b725b84d1e0b4cfd9ec58ce9", //bob
        "1229101a0fcf2104e8808dab35661134aa5903867d44deb73ce1c7e4eb925be8", //internal
    ];


    let keys = seeds
        .iter()
        .map(|scrt| KeyPair::from_secret_key(&secp, SecretKey::from_str(&scrt).unwrap()))
        .collect::<Vec<KeyPair>>();

        let tap_script=TapScriptSendEx{secp};
    let output_func = tap_script.output_factory(
        keys[internal_secret].public_key(),
        keys[alice_secret].public_key(),
        keys[bob_secret].public_key(),
    );

    let lock_func = create_tx();
    
    let unlock_func = tap_script.input_factory( &keys[1]);
    create_partially_signed_tx(vec![output_func], lock_func, unlock_func)(TapscriptExInput::new());

}

