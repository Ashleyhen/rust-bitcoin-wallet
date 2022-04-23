
use std::{str::FromStr, borrow::BorrowMut, hash::Hash, ops::{Add, Deref}};


use bdk::{KeychainKind};
use bitcoin::{secp256k1::{rand::{rngs::OsRng, RngCore},  constants, Secp256k1, SecretKey }, Network, util::{bip32::{ExtendedPrivKey, ChildNumber, ExtendedPubKey, self, KeySource, DerivationPath}, taproot::{TapLeafHash, LeafVersion, TaprootBuilder, TaprootMerkleBranch, TapBranchHash, TapBranchTag, TaprootSpendInfo}}, Script, Address, schnorr::{TapTweak, TweakedKeyPair, UntweakedKeyPair}, psbt::TapTree, KeyPair, Txid, PublicKey, hashes::hex::FromHex};
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
	// wpkh([d34db33f/44'/0'/0']tpubDEnoLuPdBep9bzw5LoGYpsxUQYheRQ9gcgrJhJEcdKFB9cWQRyYmkCyRoTqeD4tJYiVVgt6A3rN6rWn9RYhR9sBsGxji29LYWHuKKbdb1ev/0/*)
	fn get_network(&self)-> Network{return Network::from_magic(self.network).unwrap();}
	
	fn get_zprv(&self)->ExtendedPrivKey{return ExtendedPrivKey::new_master(self.get_network(), &self.seed).unwrap()}

	fn get_p2wpkh_path(&self)->DerivationPath{
		// bip84 For the purpose-path level it uses 84'. The rest of the levels are used as defined in BIP44 or BIP49.
		// m / purpose' / coin_type' / account' / change / address_index

		let keychain=KeychainKind::External;
		return bip32::DerivationPath::from(vec![
			bip32::ChildNumber::from_hardened_idx(84).unwrap(),// purpose 
			bip32::ChildNumber::from_hardened_idx(0).unwrap(),// first recieve 
			bip32::ChildNumber::from_hardened_idx(0).unwrap(),// second recieve 
			bip32::ChildNumber::from_normal_idx(keychain as u32).unwrap(),// change address
		]);

	}
	
	pub fn derive_key_pair(&self)->KeySource {
		let secp =Secp256k1::new();
		let parent_finger_print=self.get_zprv().fingerprint(&secp);
		let child_list= self.get_p2wpkh_path();
		return (parent_finger_print,child_list);
	}



		pub fn get_balance(&self){
		let secp =Secp256k1::new();

		let prvx= ExtendedPrivKey::new_master(self.get_network(), &self.seed).unwrap().derive_priv(&secp, &self.get_p2wpkh_path()).unwrap();
		let pubx=ExtendedPubKey::from_priv(&secp , &prvx);
		let child_pubx=pubx.ckd_pub(&secp, ChildNumber::from_normal_idx(2).unwrap()).unwrap();//index

		let p_k=bitcoin::PublicKey::new(child_pubx.public_key);
		
		let address=Address::p2wpkh(&p_k, Network::Testnet).unwrap();
		
		let client =Client::new("ssl://electrum.blockstream.info:60002").expect("client connection failed !!!");
	
		let script =bdk::bitcoin::blockdata::script::Script::from(address.script_pubkey().to_bytes());
		let balance =client.script_get_balance(&script).unwrap();
		println!("address {}",address.to_qr_uri().to_lowercase());
		
		println!("The current balance is {}",balance.confirmed);
	}

// https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki

	pub fn adapt_bitcoin_extended_priv_keys_to_bdk_version(&self)->bdk::bitcoin::util::bip32::ExtendedPrivKey{
		let network=self.get_network();
		let ext_prv_key_str=ExtendedPrivKey::new_master(network, &self.seed).unwrap().to_string();

		return bdk::bitcoin::util::bip32::ExtendedPrivKey::from_str(&ext_prv_key_str).unwrap();
	}
}
