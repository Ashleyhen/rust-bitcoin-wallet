
use std::{str::FromStr, borrow::{BorrowMut, Borrow}, hash::Hash, ops::{Add, Deref}, collections::BTreeMap, io::Read, sync::Arc};


use bdk::{KeychainKind, Wallet, keys::DescriptorSecretKey, template::P2Pkh, bitcoin::SigHash};
use bitcoin::{secp256k1::{rand::{rngs::OsRng, RngCore},  constants, Secp256k1, SecretKey, ecdsa::{Signature, SerializedSignature}, Message, self, PublicKey  }, Network, util::{bip32::{ExtendedPrivKey, ChildNumber, ExtendedPubKey, self, KeySource, DerivationPath, Fingerprint}, taproot::{TapLeafHash, LeafVersion, TaprootBuilder, TaprootMerkleBranch, TapBranchHash, TapBranchTag, TaprootSpendInfo}, bip143::{SigHashCache, self}, sighash::{self, SighashCache}, merkleblock::PartialMerkleTree, misc::signed_msg_hash}, Script, Address, schnorr::{TapTweak, TweakedKeyPair, UntweakedKeyPair}, psbt::{TapTree, Input, Output, PartiallySignedTransaction, serialize::Serialize}, KeyPair, Txid,  hashes::{hex::FromHex, serde_macros::serde_details::SerdeHash }, OutPoint, TxIn, blockdata::{witness, opcodes, script::Builder}, Witness, TxOut, Transaction, WitnessMerkleNode, WitnessCommitment, EcdsaSighashType, SigHashType, EcdsaSig, PubkeyHash, Sighash};
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

		let prvz= ExtendedPrivKey::new_master(network, &seed.secret_bytes()).unwrap().derive_priv(&secp, &get_derivation_path(2)).unwrap();
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
	pub fn generate_ext_pub_k(&self, i: u32)->ExtendedPubKey{
		let secp=Secp256k1::new();
		let prvz= ExtendedPrivKey::new_master(self.network, &self.seed).unwrap().derive_priv(&secp, &get_derivation_path(i)).unwrap();
		return ExtendedPubKey::from_priv(&secp , &prvz);
		

	}

	// wpkh([d34db33f/44'/0'/0']tpubDEnoLuPdBep9bzw5LoGYpsxUQYheRQ9gcgrJhJEcdKFB9cWQRyYmkCyRoTqeD4tJYiVVgt6A3rN6rWn9RYhR9sBsGxji29LYWHuKKbdb1ev/0/*)
	fn get_network(&self)-> Network{return Network::from_magic(self.network.magic()).unwrap();}
	
	fn get_zprv(&self)->ExtendedPrivKey{return ExtendedPrivKey::new_master(self.get_network(), &self.seed).unwrap()}

	pub fn derive_key_pair(&self)->KeySource {
		let secp =Secp256k1::new();
		let parent_finger_print=self.get_zprv().fingerprint(&secp);
		let child_list= get_derivation_path(2);
		return (parent_finger_print,child_list);
	}

	pub fn get_balance(&self){
		
		let address=self.get_address();
		let scr=self.generate_address(4).script_pubkey();
		let balance =self.client.script_get_balance(&scr).unwrap();
		println!("address {}",address.to_qr_uri().to_lowercase());
		
		println!("The current balance is {}",balance.confirmed);
	}

	pub fn get_address(&self)->Address{
		return Address::p2wpkh(&bitcoin::PublicKey::new(self.pubz.public_key), self.network).unwrap();
	}

	pub fn generate_address(&self,i: u32)->Address{
		return Address::p2wpkh(&bitcoin::PublicKey::new(self.generate_ext_pub_k(i).public_key), self.network).unwrap();
	}


	
	pub fn transaction(&self){
let index=4;

	let secp =Secp256k1::new();
		let to_addr="tb1qgdzfpafdhdgkfum7mnemk4e2vm2rx63ltd8z7t";
		let change_addr=self.generate_address(index+1);
		let history= Arc::new(self.client.script_get_history(&self.generate_address(index).script_pubkey()).unwrap());
		
		let tx_in=history.iter().enumerate().map(|tx_id|
			TxIn{
				previous_output:OutPoint::new(tx_id.1.tx_hash, tx_id.0 as u32+1),
				script_sig: Script::new(),// The scriptSig must be exactly empty or the validation fails (native witness program)
				sequence: 0xFFFFFFFF,
				witness: Witness::default() 
			}
		).collect::<Vec<TxIn>>();
		
		// dbg!(tx_in.clone());
		let previous_tx=Arc::new(tx_in.iter().map(|tx_id|self.client.transaction_get(&tx_id.previous_output.txid).unwrap()).collect::<Vec<Transaction>>());
		dbg!(previous_tx.clone());

	let tx_out=vec![
		TxOut{ value: 1000, script_pubkey:Address::from_str(to_addr).unwrap().script_pubkey()},
		TxOut{ value: 1815011, script_pubkey:change_addr.clone().script_pubkey() }
		];

		let mut transaction =Transaction{
			version: 1,
			lock_time: 0,
			input: tx_in,
			output: tx_out,
		};
// :(Vec<TxOut>,Vec<TxIn>)

let ext_pub_k=self.generate_ext_pub_k(index);
let input_vec=previous_tx.iter().map(|prev|{
	let mut input_tx=Input::default();
	input_tx.non_witness_utxo=Some(prev.clone());
	input_tx.witness_utxo=Some(prev.output.iter().filter(|w|Script::is_v0_p2wpkh(&w.script_pubkey)&&w.script_pubkey.eq(&Script::new_v0_p2wpkh(&ext_pub_k.public_key.to_public_key().wpubkey_hash().unwrap()))).next().unwrap().clone());
	return input_tx;
}).collect::<Vec<Input>>();

let mut xpub =BTreeMap::new();
xpub.insert(ext_pub_k,(ext_pub_k.clone().fingerprint() ,get_derivation_path(index)));

let mut output=Output::default();
output.bip32_derivation =BTreeMap::new();
output.bip32_derivation.insert(self.generate_ext_pub_k(index+1).public_key,(self.generate_ext_pub_k(index+1).fingerprint() ,get_derivation_path(index+1)));

let mut psbt=PartiallySignedTransaction{
    unsigned_tx: transaction,
    version: 0,
    xpub,
    proprietary: BTreeMap::new(),
    unknown: BTreeMap::new(),
    outputs: vec![output],
    inputs: input_vec,
};


let hash=psbt.inputs.iter().filter(|w|w.witness_utxo.is_some()).enumerate().map(|w| {
// dbg!(w.clone().1.witness_utxo.as_ref().unwrap().script_pubkey);
let addr=Address::from_script(&w.clone().1.witness_utxo.as_ref().unwrap().script_pubkey, Network::Testnet).unwrap().to_qr_uri().to_lowercase();
println!("address {}",addr);
	return sighash::SighashCache::new(&mut psbt.unsigned_tx)
	.segwit_signature_hash(
		w.0,
		 &BitcoinKeys::p2wpkh_script_code(&w.1.witness_utxo.as_ref().unwrap().script_pubkey), 
		 w.1.witness_utxo.as_ref().unwrap().value, 
		 EcdsaSighashType::All).unwrap()
}).next().unwrap();
// dbg!(Address::p2wpkh(&ext_pub_k.public_key.to_public_key(),Network::Testnet).unwrap().to_qr_uri().to_ascii_lowercase());

let prvz= ExtendedPrivKey::new_master(self.network, &self.seed).unwrap().derive_priv(&secp, &get_derivation_path(index)).unwrap();
let ecdsa=EcdsaSig::sighash_all(secp.sign_ecdsa(&Message::from_slice(&hash).unwrap(),&prvz.private_key));
psbt.inputs[0].partial_sigs.insert(ext_pub_k.public_key.to_public_key(), ecdsa);
let stadisfied_tx=psbt.clone().finalize(&secp).unwrap();
dbg!(psbt.clone());
		
// broadcast transaction
// self.client.transaction_broadcast(&stadisfied_tx.extract_tx()).unwrap();
	
	}












pub fn p2wpkh_script_code(script: &Script) -> Script {
    Builder::new()
        .push_opcode(opcodes::all::OP_DUP)
        .push_opcode(opcodes::all::OP_HASH160)
        .push_slice(&script[2..])
        .push_opcode(opcodes::all::OP_EQUALVERIFY)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .into_script()
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
pub fn get_derivation_path(i: u32)->DerivationPath{
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