
use std::{str::FromStr, borrow::{BorrowMut, Borrow}, hash::Hash, ops::{Add, Deref}, collections::BTreeMap};


use bdk::{KeychainKind};
use bitcoin::{secp256k1::{rand::{rngs::OsRng, RngCore},  constants, Secp256k1, SecretKey }, Network, util::{bip32::{ExtendedPrivKey, ChildNumber, ExtendedPubKey, self, KeySource, DerivationPath, Fingerprint}, taproot::{TapLeafHash, LeafVersion, TaprootBuilder, TaprootMerkleBranch, TapBranchHash, TapBranchTag, TaprootSpendInfo}}, Script, Address, schnorr::{TapTweak, TweakedKeyPair, UntweakedKeyPair}, psbt::{TapTree, Input, Output, PartiallySignedTransaction}, KeyPair, Txid, PublicKey, hashes::hex::FromHex, OutPoint, TxIn, blockdata::witness, Witness, TxOut, Transaction};
use electrum_client::{Client, ElectrumApi};

pub struct BitcoinKeys{
	pub network: Network,
	pub seed: [u8;constants::SECRET_KEY_SIZE],
	pub client:Client,
	pub pubz:ExtendedPubKey,
	
}

impl BitcoinKeys{
	
	pub fn new(secret_seed:Option<String>) -> BitcoinKeys{

		let secp =Secp256k1::new();

		let network =Network::Testnet;

		let invalid_seed=|err|panic!("failed to initalize seed, check if the seed is valid {}", err);

		let invalid_generator =|err|panic!("failed to create a random number generator !!! {}",err);

		let seed=match secret_seed{
			Some(secret)=> SecretKey::from_str(&secret).unwrap_or_else(invalid_seed),
			_ => SecretKey::new(&mut OsRng::new().unwrap_or_else(invalid_generator) ) };
		println!("secrete seed is {}",seed.display_secret());

		let prvz= ExtendedPrivKey::new_master(network, &seed.secret_bytes()).unwrap().derive_priv(&secp, &get_p2wpkh_path()).unwrap();
		let pubx=ExtendedPubKey::from_priv(&secp , &prvz);
		let pubz=pubx.ckd_pub(&secp, ChildNumber::from_normal_idx(2).unwrap()).unwrap();//index

		return BitcoinKeys {
			network,
			seed:seed.secret_bytes(),
			client: Client::new("ssl://electrum.blockstream.info:60002").expect("client connection failed !!!"),
			pubz
		 }
	}

	// wpkh([d34db33f/44'/0'/0']tpubDEnoLuPdBep9bzw5LoGYpsxUQYheRQ9gcgrJhJEcdKFB9cWQRyYmkCyRoTqeD4tJYiVVgt6A3rN6rWn9RYhR9sBsGxji29LYWHuKKbdb1ev/0/*)
	fn get_network(&self)-> Network{return Network::from_magic(self.network.magic()).unwrap();}
	
	fn get_zprv(&self)->ExtendedPrivKey{return ExtendedPrivKey::new_master(self.get_network(), &self.seed).unwrap()}

	pub fn derive_key_pair(&self)->KeySource {
		let secp =Secp256k1::new();
		let parent_finger_print=self.get_zprv().fingerprint(&secp);
		let child_list= get_p2wpkh_path();
		return (parent_finger_print,child_list);
	}

	pub fn get_balance(&self){
		
		let address=self.get_address();
		let script =to_bdk_script(address.script_pubkey());
		let balance =self.client.script_get_balance(&script).unwrap();
		println!("address {}",address.to_qr_uri().to_lowercase());
		
		println!("The current balance is {}",balance.confirmed);
	}

	pub fn get_address(&self)->Address{
		return Address::p2wpkh(&bitcoin::PublicKey::new(self.pubz.public_key), self.network).unwrap();
	}

	pub fn transaction(&self){

		let to_addr="tb1qhzwe6wr46rvfndz9kzrdtzddepah82vqtyqde5";
		let history=self.client.script_get_history(&to_bdk_script(self.get_address().script_pubkey())).unwrap();
		let txid=to_txid(history.get(0).unwrap().tx_hash);
		let out_point=OutPoint::new(txid, 0);
		
		
		let tx_in=TxIn{
			previous_output: out_point,
			script_sig: Script::new(),// The scriptSig must be exactly empty or the validation fails (native witness program)
			sequence: 0xFFFFFFFF,
			witness: Witness::default() 
		};

		let txOut=TxOut{ value: 1000, script_pubkey:Address::from_str(to_addr).unwrap().script_pubkey()  };
		
		let transaction =Transaction{
			version: 1,
			lock_time: 0,
			input: [tx_in].to_vec(),
			output: [txOut].to_vec(),
		};
		
		// std::collections::BTreeMap<bitcoin::util::bip32::ExtendedPubKey, (bitcoin::util::bip32::Fingerprint, bitcoin::util::bip32::DerivationPath)>
		// let secp=Secp256k1::new();
		// let parent_finger_print=self.get_zprv().fingerprint(&secp);
		// let child_list= get_p2wpkh_path();

		// PartiallySignedTransaction{ unsigned_tx: transaction, version: 0, xpub: BTreeMap::from([(self.pubz,self.derive_key_pair())]), proprietary: BTreeMap::default(), unknown: BTreeMap::default(), inputs: txIn, outputs: txOut };

		dbg!(PartiallySignedTransaction::from_unsigned_tx(transaction).unwrap());
	}

// https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki

	pub fn adapt_bitcoin_extended_priv_keys_to_bdk_version(&self)->bdk::bitcoin::util::bip32::ExtendedPrivKey{
		let network=self.get_network();
		let ext_prv_key_str=ExtendedPrivKey::new_master(network, &self.seed).unwrap().to_string();

		return bdk::bitcoin::util::bip32::ExtendedPrivKey::from_str(&ext_prv_key_str).unwrap();
	}
}

pub fn to_bdk_script(script:Script)->bdk::bitcoin::blockdata::script::Script{
	return bdk::bitcoin::blockdata::script::Script::from(script.to_bytes());
}
pub fn get_p2wpkh_path()->DerivationPath{
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
pub fn to_txid(id:bdk::bitcoin::hash_types::Txid)->Txid{
	return Txid::from_hash(id.as_hash());
	// id.as_hash();
	// return bdk::bitcoin::hash_types::Txid::from_hash(id.as_hash());
}