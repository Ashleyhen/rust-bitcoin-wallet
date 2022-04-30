
use std::{str::FromStr, borrow::{BorrowMut, Borrow}, hash::Hash, ops::{Add, Deref}, collections::BTreeMap, io::Read};


use bdk::{KeychainKind, Wallet, keys::DescriptorSecretKey};
use bitcoin::{secp256k1::{rand::{rngs::OsRng, RngCore},  constants, Secp256k1, SecretKey, ecdsa::{Signature, SerializedSignature}, Message  }, Network, util::{bip32::{ExtendedPrivKey, ChildNumber, ExtendedPubKey, self, KeySource, DerivationPath, Fingerprint}, taproot::{TapLeafHash, LeafVersion, TaprootBuilder, TaprootMerkleBranch, TapBranchHash, TapBranchTag, TaprootSpendInfo}, bip143::{SigHashCache, self}, sighash::{self, SighashCache}, merkleblock::PartialMerkleTree, misc::signed_msg_hash}, Script, Address, schnorr::{TapTweak, TweakedKeyPair, UntweakedKeyPair}, psbt::{TapTree, Input, Output, PartiallySignedTransaction, serialize::Serialize}, KeyPair, Txid, PublicKey, hashes::{hex::FromHex, serde_macros::serde_details::SerdeHash }, OutPoint, TxIn, blockdata::{witness, opcodes, script::Builder}, Witness, TxOut, Transaction, WitnessMerkleNode, WitnessCommitment, EcdsaSighashType, SigHashType, EcdsaSig, PubkeyHash};
use electrum_client::{Client, ElectrumApi};
use miniscript::{psbt::{PsbtInputSatisfier, PsbtExt}, descriptor::{Tr, DescriptorXKey, Wpkh}, Descriptor, DescriptorPublicKey, Segwitv0, Tap, ToPublicKey};

pub struct BitcoinKeys{
	pub network: Network,
	pub seed: [u8;constants::SECRET_KEY_SIZE],
	pub prv_seed: [u8;constants::SECRET_KEY_SIZE],
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
		
		// SecretKey::new();

		let prvz= ExtendedPrivKey::new_master(network, &seed.secret_bytes()).unwrap().derive_priv(&secp, &get_p2wpkh_path(2)).unwrap();
		let pubz=ExtendedPubKey::from_priv(&secp , &prvz);
		;

		// let pubz=pubx.ckd_pub(&secp, ChildNumber::from_normal_idx(2).unwrap()).unwrap();//index

		return BitcoinKeys {
			network,
			seed:seed.secret_bytes(),
			prv_seed:prvz.private_key.secret_bytes(),
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
		let child_list= get_p2wpkh_path(2);
		return (parent_finger_print,child_list);
	}

	pub fn get_balance(&self){
		
		let address=self.get_address();
		let balance =self.client.script_get_balance(&address.script_pubkey()).unwrap();
		println!("address {}",address.to_qr_uri().to_lowercase());
		
		println!("The current balance is {}",balance.confirmed);
	}

	pub fn get_address(&self)->Address{
		return Address::p2wpkh(&bitcoin::PublicKey::new(self.pubz.public_key), self.network).unwrap();
	}

	pub fn transaction(&self){

// let test_script=Script::from_str("OP_0 OP_PUSHBYTES_20 bbded7f44f6b54434cbc4579a39b0114ded1f512").unwrap();

		let to_addr="tb1qhzwe6wr46rvfndz9kzrdtzddepah82vqtyqde5";
		let test_change_addr=Address::from_str("tb1qh00d0az0dd2yxn9ug4u68xcpzn0dragjfpcynh").unwrap();
		let history=self.client.script_get_history(&self.get_address().script_pubkey()).unwrap();
		let tx_in=history.iter().enumerate().map(|tx_id|
			TxIn{
				previous_output:OutPoint::new(tx_id.1.tx_hash, tx_id.0.try_into().unwrap()),
				script_sig: Script::new(),// The scriptSig must be exactly empty or the validation fails (native witness program)
				sequence: 0xFFFFFFFF,
				witness: Witness::default() 
			}
		).collect::<Vec<TxIn>>();
		
		let previous_tx=tx_in.iter().map(|tx_id|self.client.transaction_get(&tx_id.previous_output.txid).unwrap()).collect::<Vec<Transaction>>();
// dbg!(tx_in.clone());
// dbg!(previous_tx.clone());

	let tx_out=vec![

		TxOut{ value: 1000, script_pubkey:Address::from_str(to_addr).unwrap().script_pubkey()},
		TxOut{ value: 1816211, script_pubkey:test_change_addr.clone().script_pubkey() }
		];
		// TxOut{ value: 1816111, script_pubkey:test_script.clone() }];

		let mut transaction =Transaction{
			version: 1,
			lock_time: 0,
			input: tx_in,
			output: tx_out,
		};
		dbg!(transaction);
		let secp = Secp256k1::new();
	;
	

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
pub fn get_p2wpkh_path(i: u32)->DerivationPath{
		// bip84 For the purpose-path level it uses 84'. The rest of the levels are used as defined in BIP44 or BIP49.
		// m / purpose' / coin_type' / account' / change / address_index
		let keychain=KeychainKind::External;
		return DerivationPath::from(vec![
			ChildNumber::from_hardened_idx(84).unwrap(),// purpose 
			ChildNumber::from_hardened_idx(0).unwrap(),// first recieve 
			ChildNumber::from_hardened_idx(0).unwrap(),// second recieve 
			ChildNumber::from_normal_idx(keychain as u32).unwrap(),// change address
			ChildNumber::from_normal_idx(i).unwrap()
		]);
	}
pub fn to_txid(id:bdk::bitcoin::hash_types::Txid)->Txid{
	return Txid::from_hash(id.as_hash());
	// id.as_hash();
	// return bdk::bitcoin::hash_types::Txid::from_hash(id.as_hash());
}









/* 
	// sig_hash.into_inner()
		// let sig=secp.sign_ecdsa(&Message::from_slice(&sig_hash).unwrap(),&SecretKey::from_slice(&self.seed).unwrap());
		

		// signed_msg_hash(msg);
		// Wallet::sign(&self, psbt, sign_options)

		// (self.derive_key_pair());
		
let descriptor=DescriptorXKey{ 
	origin: Some(self.derive_key_pair()), 
	xkey: self.pubz, 
	derivation_path: self.derive_key_pair().1, 
	wildcard: miniscript::descriptor::Wildcard::Unhardened };

// let des=Descriptor::<DescriptorPublicKey>::Wpkh(Wpkh::new(DescriptorPublicKey::XPub(descriptor)).unwrap());
let mut tree=BTreeMap::new();
		tree.insert(self.pubz,self.derive_key_pair());

// let p =PartiallySignedTransaction{ 
// 	unsigned_tx: transaction.clone(), 
// 	version: 1, 
// 	xpub: tree, 
// 	proprietary: BTreeMap::new(), 
// 	unknown: BTreeMap::new(), 
// 	inputs: [tx_in].to_vec(), 
// 	outputs: tx_out };

	
	
		// psdt.update_desc(0, &des, None).unwrap();
		// let s=psdt.finalize(&secp ).unwrap();

		let input_count=transaction.input.len();

		let mut sig_hash_cache=sighash::SighashCache::new(&mut transaction);
		for inp in 0..input_count{
			/* 
			previous_tx.get(inp).unwrap().output.iter().enumerate().for_each(|iterator|{
				// new_p2pkh

let wpkh=Script::new_v0_p2wpkh(&bitcoin::PublicKey::new(self.pubz.public_key).wpubkey_hash().unwrap());
				// Script::new_v0_p2wpkh(self.pubz.public_key);
dbg!(&iterator.1.script_pubkey);
			let sig_hash=sig_hash_cache.segwit_signature_hash(inp,&iterator.1.script_pubkey, iterator.1.value, EcdsaSighashType::All).unwrap();

			let ecdsa=EcdsaSig::sighash_all(secp.sign_ecdsa(&Message::from_slice(&sig_hash).unwrap(),&SecretKey::from_slice(&self.prv_seed).unwrap()));

			sig_hash_cache.witness_mut(inp).unwrap().push(ecdsa.to_vec());
			sig_hash_cache.witness_mut(inp).unwrap().push(wpkh);

			// sig_hash_cache.witness_mut(inp).unwrap().push(test_script.as_bytes());
			// sig_hash_cache.witness_mut(inp).unwrap().push(&wit);
			

			// return ecdsa;
			});
*/
let tx_out =previous_tx.get(inp).unwrap().output.get(0).unwrap();








let wpkh=Script::new_v0_p2wpkh(&bitcoin::PublicKey::new(self.pubz.public_key).wpubkey_hash().unwrap());
let pkh=Script::new_p2pkh(&bitcoin::PublicKey::new(self.pubz.public_key).pubkey_hash());
				// Script::new_v0_p2wpkh(self.pubz.public_key);
;
				 let pubkey_hash = PubkeyHash::from_slice_delegated(&pkh[2..22])
                    .expect("PubkeyHash hash length failure");
                let script_code = Script::new_p2pkh(&pubkey_hash);
				
				
dbg!(&tx_out.script_pubkey);
let a=tx_out.script_pubkey.wscript_hash();

			let sig_hash=sig_hash_cache.segwit_signature_hash(inp,&script_code, 1000, EcdsaSighashType::All).unwrap();

			let ecdsa=EcdsaSig::sighash_all(secp.sign_ecdsa(&Message::from_slice(&sig_hash).unwrap(),&SecretKey::from_slice(&self.prv_seed).unwrap()));
// from_txdata();
// Inner::from_txdata();
// bip143::SighashComponents::sighash_all(&self, txin, script_code, value)
			sig_hash_cache.witness_mut(inp).unwrap().push(ecdsa.to_vec());
			sig_hash_cache.witness_mut(inp).unwrap().push(&tx_out.script_pubkey);

	
			secp.verify_ecdsa(&Message::from_slice(&sig_hash).unwrap(), &ecdsa.sig, &self.pubz.public_key).unwrap();



/*
			let sig_hash=sig_hash_cache.segwit_signature_hash(inp,&BitcoinKeys::p2wpkh_script_code(&self.get_address().script_pubkey()), 1817493, EcdsaSighashType::All).unwrap();
			
			
			let msg=Message::from_slice(&sig_hash).unwrap();
			let sig=secp.sign_ecdsa(&msg,&SecretKey::from_slice(&self.prv_seed).unwrap());
			let ecdsa=EcdsaSig::sighash_all(sig);


			// sig_hash_cache.witness_mut(inp).unwrap().push(arr);

		
			let vector=ecdsa.to_vec();
			// vector.push(ecdsa.hash_ty.to_u32() as u8);
			sig_hash_cache.witness_mut(inp).unwrap().push(vector);
			// sig_hash_cache.witness_mut(inp).unwrap().push(test_script.as_bytes());
			sig_hash_cache.witness_mut(inp).unwrap().push(&wit);
			
			
 */

			// dbg!(self.get_address().script_pubkey().wscript_hash());

			// sig_hash_cache.witness_mut(inp).unwrap().push(sig_hash);
			
			
			// wit.push(Vec::new())msg
		} 
		// Secp256k1::new().sign_ecdsa(msg, sk)
for tx_in in transaction.clone().input {
	// Witness::from_vec(tx_in.witness.to_vec());
		// dbg!(tx_in.witness.to_vec());
};
			// self.client.transaction_broadcast(&transaction).unwrap();
			
			// dbg!(transaction);

// SerializedSignature::from_signature(sig: &Signature)
		// dbg!(s.extract_tx());
		// std::collections::BTreeMap<bitcoin::util::bip32::ExtendedPubKey, (bitcoin::util::bip32::Fingerprint, bitcoin::util::bip32::DerivationPath)>
		// let secp=Secp256k1::new();
		// let parent_finger_print=self.get_zprv().fingerprint(&secp);
		// let child_list= get_p2wpkh_path();

		// PartiallySignedTransaction{ unsigned_tx: transaction, version: 0, xpub: BTreeMap::from([(self.pubz,self.derive_key_pair())]), proprietary: BTreeMap::default(), unknown: BTreeMap::default(), inputs: txIn, outputs: txOut };
// let psbt=PartiallySignedTransaction::from_unsigned_tx(transaction).unwrap();
// let init =PsbtInputSatisfier::new(&psbt, 0);

// dbg!(psbt.extract_tx());

*/