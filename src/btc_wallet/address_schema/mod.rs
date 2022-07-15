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
    wallet_methods::{ClientWallet, NETWORK},
    address_schema, SignTx, WalletKeys,
};

pub mod p2tr;
pub mod p2wpkh;


pub trait AddressSchema {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> Address;
    fn wallet_purpose(&self) -> u32;
    fn new(seed: Option<String>, recieve: u32, change: u32) -> Self;
    fn to_wallet(&self) -> ClientWallet;
    fn prv_tx_input(
        &self,
        previous_tx: Vec<Transaction>,
        current_input: Transaction,
        unlocking_fn: &dyn Fn(SignTx) -> Input,
    ) -> Vec<Input>;
}
