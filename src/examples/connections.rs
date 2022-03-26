use std::{str::FromStr, fmt::Debug};

use bdk::{bitcoin::{ Network, util::bip32::{ExtendedPubKey, DerivationPath, Fingerprint, ExtendedPrivKey}, Address, secp256k1::Secp256k1}, electrum_client::Client, Wallet, database::MemoryDatabase, miniscript::{ descriptor::{DescriptorXKey, Wpkh, Wildcard}, DescriptorPublicKey}, descriptor::{ExtendedDescriptor, KeyMap, IntoWalletDescriptor}, blockchain::{ElectrumBlockchain, Blockchain}, SyncOptions, wallet::AddressIndex, FeeRate, SignOptions, keys::{DescriptorSinglePriv, DescriptorSecretKey}, KeychainKind, template::Bip84};
use crate::init_wallet::Bitcoin_keys;

pub fn blockchain_connection(bitcoin_keys: Bitcoin_keys){
	let extended_pub_key=ExtendedPubKey::from_str(&bitcoin_keys.ext_public_key).unwrap();

		
	let extended_private_key=ExtendedPrivKey::from_str(&bitcoin_keys.ext_private_key).unwrap();

	

	let origin=Option::Some((Fingerprint::from_str(&bitcoin_keys.fingure_print)
	.unwrap_or_else(|err| panic!("failed to derive fingerprint, {}",err)),DerivationPath::default()));
	
	let private_descriptor_xkey=DescriptorXKey::<ExtendedPrivKey>{
		xkey: extended_private_key,
		origin: origin.clone(),
		derivation_path: DerivationPath::default(),
		wildcard: Wildcard::Unhardened,
	};
	let descriptor_xkey =DescriptorXKey::<ExtendedPubKey>{
		xkey: extended_pub_key,
		origin,
		derivation_path: DerivationPath::default(),
		wildcard: Wildcard::Unhardened,
	};

	let descriptor_keys=DescriptorPublicKey::XPub(descriptor_xkey);

	let private_descriptor_keys=DescriptorSecretKey::XPrv(private_descriptor_xkey);

	let key_map=KeyMap::from([(descriptor_keys.clone(),private_descriptor_keys);0]);
	// private_descriptor_keys

	let pk =Wpkh::new(descriptor_keys).unwrap();
	let public_descripto_key=ExtendedDescriptor::Wpkh(pk);

	let descriptor3=(public_descripto_key, key_map);
// let descriptor=IntoWalletDescriptor::into_wallet_descriptor(&secp256,  Network::Testnet).unwrap();
let key = bdk::bitcoin::util::bip32::ExtendedPrivKey::from_str("tprv8ZgxMBicQKsPcx5nBGsR63Pe8KnRUqmbJNENAfGftF3yuXoMMoVJJcYeUw5eVkm9WBPjWYt6HMWYJNesB5HaNVBaFc1M6dRjWSYnmewUMYy").unwrap();
let descriptor2=Bip84(key, KeychainKind::External);
    let descriptor= Bip84(extended_private_key, KeychainKind::External);
	// let wallet = Wallet::new(
	// 	Bip84(key, KeychainKind::External),
	// 	None,
	// 	Network::Testnet,
	// 	MemoryDatabase::new()
	// ).unwrap();
	
	let wallet = Wallet::new(
		descriptor,
		None,
		Network::Testnet,
		MemoryDatabase::default(),
	)
	.unwrap();
	let blockchain = ElectrumBlockchain::from(Client::new("ssl://electrum.blockstream.info:60002").unwrap());
	let get_balance=wallet.sync(&blockchain, SyncOptions::default());

	get_balance.and_then(|_|{
		Ok({
			println!("p2wpkh {}", wallet.get_address(AddressIndex::LastUnused).unwrap().address);
			println!("extended key {}",extended_pub_key.to_string());
			println!("Balance {}", wallet.get_balance().unwrap())
		})
	}).unwrap();
	
	let send_address=Address::from_str("mv4rnyY3Su5gjcDNzbMLKBQkBicCtHUtFB").unwrap();
	let (mut psbt, details)={
		let mut builder=wallet.build_tx();
		builder.drain_wallet().fee_rate(FeeRate::from_sat_per_vb(2.0)).drain_to(send_address.script_pubkey());
		builder.finish().unwrap()
	};
	let is_transaction_valid = wallet.sign(&mut psbt, Default::default() ).unwrap();

		println!("valid transaction: {}",is_transaction_valid);

		let signed_transaction=psbt.clone().extract_tx();

		println!("transaction id: {}",signed_transaction.txid().to_string());
	/*
	if(is_transaction_valid){
		let signed_transaction=psbt.clone().extract_tx();
		let broadcast=blockchain.broadcast(&signed_transaction);
		
		match broadcast {
			Ok(_)=>{
				println!("transaction sent successfully, {}", signed_transaction.txid())
			},
			Err(err)=>{
				panic!("transaction failed, {}", err)
			} }
		}else{
			println!("transaction failed");
		}
*/
		// let descriptor="wpkh(tprv8ZgxMBicQKsPdpkqS7Eair4YxjcuuvDPNYmKX3sCniCf16tHEVrjjiSXEkFRnUH77yXc6ZcwHHcLNfjdi5qUvw3VDfgYiH5mNsj5izuiu2N/1/2/*)#tqz0nc62";
	// let a=PublicKey::from_str(descriptor).unwrap();
	// DescriptorPublicKey::XPub()
	



   }