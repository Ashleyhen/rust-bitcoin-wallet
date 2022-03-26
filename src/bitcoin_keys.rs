
use std::{str::FromStr};


use bitcoin::{secp256k1::{rand::{rngs::OsRng}, SecretKey}, Network, util::bip32::ExtendedPrivKey};

pub struct BitcoinKeys{
	pub master_key: String,
	pub network: u32
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

		let master_node=ExtendedPrivKey::new_master(network, &seed.secret_bytes()).unwrap_or_else(invalid_master_key);

		return BitcoinKeys {
			master_key: master_node.to_string(),
			network:network.magic()
		 }
	}
}
