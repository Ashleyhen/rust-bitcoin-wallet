use bitcoin::{util::bip32::ExtendedPubKey, Address, Script, XOnlyPublicKey, blockdata::{script::Builder, opcodes::all}, hashes::hex::FromHex};
use bitcoin_hashes::Hash;

use crate::btc_wallet::{
    constants::NETWORK, spending_path::p2tr_multisig_path::P2trMultisig,
    wallet_methods::ClientWallet,
};

use super::AddressSchema;

#[derive(Clone)]
pub struct P2TR(ClientWallet);

impl AddressSchema for P2TR {
    fn map_ext_keys(&self, recieve: &ExtendedPubKey) -> Address {
        return Address::p2tr(
            &self.get_client_wallet().secp,
            recieve.to_x_only_pub(),
            None,
            NETWORK,
        );
    }

    fn wallet_purpose(&self) -> u32 {
        return 341;
    }

    fn to_wallet(&self) -> ClientWallet {
        return self.0.clone();
    }
}

impl P2TR {
    pub fn get_client_wallet(&self) -> ClientWallet {
        return self.0.clone();
    }

    pub fn new(secret_seed: Option<String>, recieve: u32, change: u32) -> Self {
        return P2TR(ClientWallet::new(secret_seed, recieve, change));
    }

    pub fn alice_script(&self, internal_key: XOnlyPublicKey) -> Script {
        return Script::new_v1_p2tr(&self.get_client_wallet().secp, internal_key, None);
    }

    pub fn bob_script(xonly:&XOnlyPublicKey)->Script{
        let preimage =
        Vec::from_hex("107661134f21fc7c02223d50ab9eb3600bc3ffc3712423a1e47bb1f9a9dbf55f").unwrap();
        let preimage_hash = bitcoin_hashes::sha256::Hash::hash(&preimage);
        return Builder::new()
        .push_opcode(all::OP_SHA256)
        .push_slice(&preimage_hash)
        .push_opcode(all::OP_EQUALVERIFY)
        .push_x_only_key(xonly)
        .push_opcode(all::OP_CHECKSIG)
        .into_script();
    }

}
