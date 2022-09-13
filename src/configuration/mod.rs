use std::str::FromStr;

use bitcoin::{
    secp256k1::{Secp256k1, SecretKey},
    Address, KeyPair,
};

use crate::bitcoin_wallet::{
    address_formats::{map_seeds_to_scripts, map_tr_address},
    input_data::{electrum_rpc::ElectrumRpc, RpcCall},
    script_services::psbt_factory::{create_partially_signed_tx, get_output},
    spending_path::{p2tr_key_path::P2tr, tap_script_spending_ex::TapScriptSendEx},
};

pub mod script_demo;


// tb1paq75m2jlhjeywx75g3t08d8yplt5w9a0ecar3mdp5ay3laxva7vqng2jak
