use std::{borrow::Borrow, collections::BTreeMap, str::FromStr};

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::Input,
    schnorr::{TapTweak, TweakedPublicKey, UntweakedPublicKey},
    secp256k1::{schnorr::Signature, schnorrsig::PublicKey, All, Message, Parity, Secp256k1},
    util::{
        address::Payload,
        bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey, KeySource},
        sighash::{Prevouts, SighashCache},
        taproot::{LeafVersion::TapScript, TapLeafHash},
    },
    Address, KeyPair, SchnorrSig, SchnorrSighashType, Script, Transaction, TxIn, TxOut,
    XOnlyPublicKey,
};
use miniscript::{interpreter::KeySigPair, ToPublicKey};

use super::{
    address_formats,
    wallet_methods::{ClientWallet, NETWORK},
};

pub mod p2tr_addr_fmt;
pub mod p2wpkh_addr_fmt;

pub trait AddressSchema {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> Address;
    fn wallet_purpose(&self) -> u32;
    // fn new(seed: Option<String>, recieve: u32, change: u32) -> Self;
    fn to_wallet(&self) -> ClientWallet;
}
