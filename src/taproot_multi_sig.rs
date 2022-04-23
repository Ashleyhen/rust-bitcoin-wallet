use std::{str::FromStr, io::Bytes, borrow::{Borrow, BorrowMut}};

// use bdk::bitcoin::{schnorr::PublicKey, util::contracthash::compute_tweak};
use  bitcoin::{Network, secp256k1::{Secp256k1, rand::{rngs::OsRng, RngCore}, PublicKey, constants::{UNCOMPRESSED_PUBLIC_KEY_SIZE, PUBLIC_KEY_SIZE, SCHNORR_PUBLIC_KEY_SIZE}, ecdh::SharedSecret, SecretKey, schnorr::Signature, Message, ffi::SchnorrNonceFn,}, util::{bip32::{ExtendedPrivKey, ChildNumber, ExtendedPubKey}, taproot::{TaprootSpendInfo, TapBranchHash, TaprootBuilder}, address::Payload},  psbt::{TapTree, PartiallySignedTransaction, self}, Script, XOnlyPublicKey, network::message_bloom::FilterAdd, PubkeyHash, KeyPair, Address, Witness, Transaction, schnorr::{TweakedPublicKey, TapTweak}, TxIn, TxOut};
use miniscript::serde::__private::de;

use crate::bitcoin_keys::{ BitcoinKeys};
pub struct WalletInfo{
	pub master_key: ExtendedPrivKey
}
impl WalletInfo{

	pub fn get_taproot_address(secret_seed: Option<String>)->WalletInfo{
		let btc_keys=BitcoinKeys::new(secret_seed);

		let secp=&Secp256k1::new();

		let master_key=ExtendedPrivKey::new_master(Network::from_magic(btc_keys.network).unwrap(), &btc_keys.seed).expect("invalid seed or network");

		let addr=Address::p2tr(secp, master_key.to_keypair(secp).public_key(), None, Network::Testnet);
		print!("tap addr {}",addr.to_qr_uri().to_ascii_lowercase().to_string());

		return WalletInfo{master_key};
	}

	pub fn add_signer(&self,their_public_key:String,secret_seed:Option<String>){
		let secp=&Secp256k1::new();
		let master_node=self.master_key;

		let mut p2tr=Address::from_str(their_public_key.as_str()).unwrap();

		let p2tr_k=p2tr.payload.script_pubkey().as_bytes()[2..].to_vec();

		let key_pair=master_node.to_keypair(secp);

		let mut public_key=PublicKey::from_keypair(&key_pair);


		println!("\nkey before ====");
		let tap_1=Address::p2tr(secp, key_pair.public_key() , None, Network::Testnet);
		println!("current address {}",tap_1.to_qr_uri().to_ascii_lowercase());
		println!("+");
		println!("other address bitcoin:{}",their_public_key);

		
		public_key.add_exp_assign(secp, &p2tr_k).unwrap();

		println!("===============================");
		
		let  result=Address::p2tr(secp, XOnlyPublicKey::from(public_key) , None, Network::Testnet);
		println!("changed taproot address {}",result.to_qr_uri().to_ascii_lowercase());

		println!("-------------------------");

		let p_k=bitcoin::PublicKey::new(public_key);
		let before =Address::p2pkh(&p_k,Network::Testnet);
		dbg!(before.to_qr_uri().to_ascii_lowercase());

	}

	pub fn single_sign(&self){
		// PartiallySignedTransaction::extract_tx(self)
		let secp=&Secp256k1::new();
		let key_pair =self.master_key.to_keypair(secp);
		let mut os_rand=OsRng::new().unwrap();
		let mut message_bytes = [0u8; 32];
		os_rand.fill_bytes( &mut message_bytes);
		let message=Message::from_slice(&message_bytes).unwrap();
		let signature=secp.sign_schnorr(&message,&key_pair);


		// psbt::Output
		// psbt::Input

// 		let s=TxIn{
//     previous_output: todo!(),
//     script_sig: todo!(),
//     sequence: todo!(),
//     witness: todo!(),
// };
// 	let s=TxOut{
//     value: todo!(),
//     script_pubkey: todo!(),
// };
// 		let transaction=Transaction{
// 			version: 1,
// 			lock_time: 0,
// 			input: todo!(),
// 			output: todo!(),
// 		};

		let is_valid=secp.verify_schnorr(&signature, &message, &key_pair.public_key()).is_ok();
		println!("is valid {}",is_valid);
	}

	
}
/*
	pub fn test_root(&self){
			let secp=&Secp256k1::new();

			let master_node=ExtendedPrivKey::new_master(Network::from_magic(self.network)
			.unwrap(), &self.seed).expect("invalid seed or network");

			let child_prv_k=master_node
			.ckd_priv(&secp, ChildNumber::from_hardened_idx(OsRng::new().unwrap().next_u32()).unwrap()).unwrap();

			let mut keypair =child_prv_k.to_keypair(&secp);
			// let tab_branch =TapBranchHash::default();

			let tab_branch =TaprootSpendInfo::new_key_spend(secp, keypair.public_key(), Some(TapBranchHash::default())).merkle_root();
			let unspendable_pub_k_before =Address::p2tr(&secp, keypair.public_key(), tab_branch, Network::Testnet);

			dbg!("before pub key ",unspendable_pub_k_before.to_qr_uri().to_ascii_lowercase());
			dbg!("before prv key ",keypair.display_secret());


			keypair.tweak_add_assign(secp, &self.seed).unwrap();
	// TaprootBuilder::new().add_leaf(0, Script::new().to_v1_p2tr(&secp,keypair.public_key())).unwrap()
			let taproot_builder=TaprootBuilder::new().add_leaf(0, Script::new().to_v1_p2tr(&secp,keypair.public_key())).unwrap();
			let tree=TapTree::from_inner(taproot_builder).unwrap();
			
			// TaprootSpendInfo::merkle_root(&self)
		


			let unspendable_pub_k =Address::p2tr(&secp, keypair.public_key(), tab_branch, Network::Testnet);

			dbg!("after pub key ",unspendable_pub_k.to_qr_uri().to_ascii_lowercase());
			dbg!("after prv key ",keypair.display_secret());

			// TapTweak::tap_tweak(self, secp, merkle_root)
			// Address::from_str(pub_k_1.as_str());
			// Address::from_str(&pub_k_2.as_str());
			
			// TapLeafHash::from_script(Script::new_p2pkh(pubkey_hash),LeafVersion::TapScript );
		}*/