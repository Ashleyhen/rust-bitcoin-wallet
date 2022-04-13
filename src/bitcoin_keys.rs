
use std::{str::FromStr};


use bitcoin::{secp256k1::{rand::{rngs::OsRng, RngCore},  constants, Secp256k1, SecretKey }, Network, util::{bip32::{ExtendedPrivKey, ChildNumber}, taproot::{TapLeafHash, LeafVersion, TaprootBuilder, TaprootMerkleBranch, TapBranchHash, TapBranchTag, TaprootSpendInfo}}, Script, Address, schnorr::{TapTweak, TweakedKeyPair, UntweakedKeyPair}, psbt::TapTree, KeyPair, hashes::sha256t::Hash};

pub struct BitcoinKeys{
	pub network: u32,
	pub seed: [u8;constants::SECRET_KEY_SIZE] 
	
}
impl BitcoinKeys{
	
	pub fn new(secret_seed:Option<String>) -> BitcoinKeys{
		let network =Network::Testnet;

		let invalid_seed=|err|panic!("failed to initalize seed, check if the seed is valid {}", err);

		let invalid_generator =|err|panic!("failed to create a random number generator !!! {}",err);

		let invalid_master_key=|err|panic!("failed to derive a master node {}",err);

		let seed=match secret_seed{
			Some(secret)=> SecretKey::from_str(&secret).unwrap_or_else(invalid_seed),
			_ => SecretKey::new(&mut OsRng::new().unwrap_or_else(invalid_generator) ) };
		println!("secrete seed is {}",seed.display_secret());

		let _master_node=ExtendedPrivKey::new_master(network, &seed.secret_bytes()).unwrap_or_else(invalid_master_key);

		return BitcoinKeys {
			network:network.magic(),
			seed:seed.secret_bytes()
		 }
	}
	pub fn adapt_bitcoin_extended_priv_keys_to_bdk_version(&self)->bdk::bitcoin::util::bip32::ExtendedPrivKey{
		let network=self.get_network();
		let ext_prv_key_str=ExtendedPrivKey::new_master(network, &self.seed).unwrap().to_string();
		return bdk::bitcoin::util::bip32::ExtendedPrivKey::from_str(&ext_prv_key_str).unwrap();
	}
	fn get_network(&self)-> Network{return Network::from_magic(self.network).unwrap();}

	
	
}
