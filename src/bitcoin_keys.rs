
use std::{str::FromStr, borrow::BorrowMut, hash::Hash, ops::{Add, Deref}};


use bdk::KeychainKind;
use bitcoin::{secp256k1::{rand::{rngs::OsRng, RngCore},  constants, Secp256k1, SecretKey }, Network, util::{bip32::{ExtendedPrivKey, ChildNumber, ExtendedPubKey, self}, taproot::{TapLeafHash, LeafVersion, TaprootBuilder, TaprootMerkleBranch, TapBranchHash, TapBranchTag, TaprootSpendInfo}}, Script, Address, schnorr::{TapTweak, TweakedKeyPair, UntweakedKeyPair}, psbt::TapTree, KeyPair, Txid, PublicKey, hashes::hex::FromHex};
use electrum_client::{Client, ElectrumApi};

pub struct BitcoinKeys{
	pub network: u32,
	pub seed: [u8;constants::SECRET_KEY_SIZE],
	
}
impl BitcoinKeys{
	
	pub fn new(secret_seed:Option<String>) -> BitcoinKeys{
		let network =Network::Testnet;

		let invalid_seed=|err|panic!("failed to initalize seed, check if the seed is valid {}", err);

		let invalid_generator =|err|panic!("failed to create a random number generator !!! {}",err);

		let seed=match secret_seed{
			Some(secret)=> SecretKey::from_str(&secret).unwrap_or_else(invalid_seed),
			_ => SecretKey::new(&mut OsRng::new().unwrap_or_else(invalid_generator) ) };
		println!("secrete seed is {}",seed.display_secret());


		return BitcoinKeys {
			network:network.magic(),
			seed:seed.secret_bytes()
		 }
	}
	// m / purpose' / coin_type' / account' / change / address_index
	fn get_network(&self)-> Network{return Network::from_magic(self.network).unwrap();}

	pub fn derive_key_pair(&self)->KeyPair{
		let secp =Secp256k1::new();
		let ext_prv_key_str=ExtendedPrivKey::new_master(self.get_network(), &self.seed).unwrap();

		let keychain=KeychainKind::External;

		


		// bip84
		// For the purpose-path level it uses 84'. The rest of the levels are used as defined in BIP44 or BIP49.
		return ext_prv_key_str.derive_priv(&secp, &vec![
			bip32::ChildNumber::from_hardened_idx(84).unwrap(),// purpose 
			bip32::ChildNumber::from_hardened_idx(0).unwrap(),// first recieve 
			bip32::ChildNumber::from_hardened_idx(0).unwrap(),// second recieve 
			bip32::ChildNumber::from_normal_idx(keychain as u32).unwrap(),// change address
		]).unwrap().to_keypair(&secp);
	}
		pub fn get_balance(&self){

		let client =Client::new("ssl://electrum.blockstream.info:60002").expect("client connection failed !!!");
		let key_pair=self.derive_key_pair();
		let p_k=bitcoin::PublicKey::new(bitcoin::secp256k1::PublicKey::from_keypair(&key_pair));
		let address=Address::p2wpkh(&p_k, Network::Testnet).unwrap();
		let script =bdk::bitcoin::blockdata::script::Script::from(address.script_pubkey().to_bytes());
		let balance =client.script_get_balance(&script).unwrap();
		println!("address {}",address.to_qr_uri().to_lowercase());
		
		println!("The current balance is {}",balance.confirmed);



	}


		// let mut private_extended_key=master_node.derive_priv(&secp, &vec![ChildNumber::Hardened { index: (0) }]).expect("failed to derive private key");
		// private_extended_key.child_number= ChildNumber::Hardened { index: (2) };

		
		// dbg!(ext_prv_key_str.to_priv().to_wif());
// PublicKey::new(bitcoin::secp256k1::PublicKey::from_keypair(&key_pair));
/*
let pub_k=bitcoin::secp256k1::PublicKey::from_keypair(&key_pair);
let ext_pub_key =ExtendedPubKey::from_priv(&secp, &ext_prv_key_str);
let derivedExtKey=ext_prv_key_str.derive_priv(&secp, &vec![
	ChildNumber::Hardened { index: (84) },
	ChildNumber::Hardened { index: (0) },
	ChildNumber::Hardened { index: (0) }, 
	ChildNumber::Normal  { index: (1) } 
	]
).unwrap();

dbg!( derivedExtKey.to_string());
let public_key=PublicKey::from_private_key(&secp, &derivedExtKey.to_priv());
		let address=Address::p2wpkh(&public_key, Network::Testnet).unwrap();
		println!("uri address{} ",address.to_qr_uri().to_lowercase().as_str());
		
		return public_key; 
	} 
*/
	








// https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki









	pub fn adapt_bitcoin_extended_priv_keys_to_bdk_version(&self)->bdk::bitcoin::util::bip32::ExtendedPrivKey{
		let network=self.get_network();
		let ext_prv_key_str=ExtendedPrivKey::new_master(network, &self.seed).unwrap().to_string();

		return bdk::bitcoin::util::bip32::ExtendedPrivKey::from_str(&ext_prv_key_str).unwrap();
	}

	pub fn proofOfConcept(&self){
			let secp=Secp256k1::new();

		let master_node=ExtendedPrivKey::new_master(self.get_network(), &self.seed).unwrap();
		// bdk::blockchain::rpc::RpcBlockchain::from_config(b)	
		// Client
let strAddr="tb1qct0ql88m4e820ujd6hhrqraxzq0w5m87v67s29";
		let key_pair=master_node.to_keypair(&secp);

		let network =bitcoin::Network::from_magic(self.get_network().magic()).unwrap();
		let master_node=bitcoin::util::bip32::ExtendedPrivKey::new_master(network, &self.seed).unwrap();
			let client =Client::new("ssl://electrum.blockstream.info:60002")
			.unwrap_or_else(|err|panic!("client connection failed !!!{}",err));
			
		// 	bdk::bitcoin::Script::default();

		let internal_key=master_node.to_keypair(&Secp256k1::new()).public_key();
		let script =Script::new_v1_p2tr(&Secp256k1::new(),internal_key, None);
		let p_k=bitcoin::PublicKey::new(bitcoin::secp256k1::PublicKey::from_keypair(&key_pair));

		// let address=Address::from_str(p_k.to_bytes()).unwrap_or_else(|err|panic!("invalid address bitcoin : {}",err));
		
		let script2 =Script::new_v0_p2wpkh(&p_k.wpubkey_hash().unwrap());

		let addr=Address::from_str("tb1qa7sxhwf83k7pqyu6qkgc6496md8nk8g7xvp6xy").unwrap();
		
		let scr=electrum_client::bitcoin::Script::from(addr.script_pubkey().to_bytes().to_vec());
		Address::from_str(strAddr).unwrap().script_pubkey().as_bytes();
		
		dbg!(script2.clone());
		let bal=client.script_get_balance(&scr).unwrap();
		println!("balance {}",bal.confirmed)
;
		let history= client.script_get_history(&electrum_client::bitcoin::Script::from(scr.to_bytes().to_vec())).expect("failed");
		
let s=electrum_client::bitcoin::Txid::from_str("6247913b3c1dfd312133d78f1a099f68848658a4ceeb69c38325d98c7a6fad8b").unwrap();

		dbg!(client.batch_transaction_get(vec!(&s)).unwrap());
		let a=client.relay_fee().unwrap();
		println!("{}",a);
		dbg!(history);
		// 	// let r=ElectrumApi::block_headers_pop_raw(&client);
		// 	// client.script_get_history(script)
			
	}
	
	
}
