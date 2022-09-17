use std::{collections::BTreeMap, iter::Map, str::FromStr};

use bitcoin::{
    psbt::{Input, Output},
    util::bip32::ExtendedPubKey,
    Address, Script, Transaction, TxIn, TxOut, XOnlyPublicKey,
};

pub const RECEIVER: usize = 0;
pub const CHANGE: usize = 1;

pub mod p2tr_key_path;
pub mod p2wpkh_script_path;
pub mod tap_script_spending_ex;
pub mod scripts;
pub mod adaptor_script;
pub mod musig;