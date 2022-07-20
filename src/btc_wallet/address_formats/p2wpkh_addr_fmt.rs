use std::{collections::BTreeMap, io::Read, str::FromStr, sync::Arc};

use bitcoin::{
    blockdata::{opcodes, script::Builder},
    psbt::{Input, Output},
    secp256k1::{Message, SecretKey},
    util::{
        bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey, KeySource},
        sighash::SighashCache,
    },
    Address, EcdsaSig, EcdsaSighashType, Script, Sighash, Transaction, TxIn, TxOut,
};
use miniscript::ToPublicKey;

use crate::btc_wallet::{
    spending_path::p2wpkh_script_path::P2PWKh,
    wallet_methods::{ClientWallet, NETWORK},
};

use super::{address_formats, AddressSchema};

impl AddressSchema for P2PWKh {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> bitcoin::Address {
        return Address::p2wpkh(&recieve.public_key.to_public_key(), NETWORK).unwrap();
    }
    
    fn to_wallet(&self) -> ClientWallet {
        return self.client_wallet.clone();
    }

    fn wallet_purpose(&self) -> u32 {
        return 84;
    }

}
