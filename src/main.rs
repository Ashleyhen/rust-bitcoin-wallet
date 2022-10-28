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
use configuration::{
    musig_demo::musig_demo,
    script_demo::{self, single_sign},
};

use crate::bitcoin_wallet::{
    input_data::{
        regtest_rpc::RegtestRpc, reuse_rpc_call::ReUseCall, tapscript_ex_input::get_signed_tx,
    },
    spending_path::p2tr_key_path::P2tr,
};

pub mod bitcoin_wallet;
pub mod configuration;

fn main() {
    env::set_var("RUST_BACKTRACE", "full");

    // let client = RegtestRpc::get_client();
    // let address =
    //     Address::from_str("bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj")
    //         .unwrap();
    // bcrt1prnpxwf9tpjm4jll4ts72s2xscq66qxep6w9hf6sqnvwe9t4gvqasklfhyj
    // RegtestRpc::importdescriptors(&client, &address);
    // RegtestRpc::create_wallet(&client, "mywallet",false);
    // dbg!(RegtestRpc::generatetodescriptor(&client, 100, &address));
    // ;
    single_sign();
    // musig_demo();
    // key_tx();
}

pub fn get_base_balance() {
    let client = Client::new(
        "http://127.0.0.1:18443",
        bitcoincore_rpc::Auth::UserPass("polaruser".to_string(), "polarpass".to_owned()),
    )
    .unwrap();

    client.get_block_count();
    let address_list =
        vec!["bcrt1p5kaqsuted66fldx256lh3en4h9z4uttxuagkwepqlqup6hw639gsm28t6c".to_owned()];
    let reg_test = RegtestRpc::from_string(&address_list);

    dbg!(reg_test().script_get_balance());
}

// tb1paq75m2jlhjeywx75g3t08d8yplt5w9a0ecar3mdp5ay3laxva7vqng2jak
