use std::str::FromStr;

use bdk::{bitcoin::{ Network, util::bip32::{ExtendedPubKey, DerivationPath, Fingerprint}}, electrum_client::Client, Wallet, database::MemoryDatabase, miniscript::{ descriptor::{DescriptorXKey, Wpkh, Wildcard}, DescriptorPublicKey}, descriptor::ExtendedDescriptor, blockchain::ElectrumBlockchain, SyncOptions, wallet::AddressIndex};
use crate::bitcoin_keys::Bitcoin_keys;
pub fn blockchain_connection(bitcoin_keys: Bitcoin_keys){
		let extended_pub_key=ExtendedPubKey::from_str(&bitcoin_keys.ext_public_key).unwrap();

		
		extended_pub_key.depth;

	let origin=Option::Some((Fingerprint::from_str(&bitcoin_keys.fingure_print)
	.unwrap_or_else(|err| panic!("failed to derive fingerprint, {}",err)),DerivationPath::default()));
	
	
	let descriptor_xkey =DescriptorXKey::<ExtendedPubKey>{
    xkey: extended_pub_key,
    origin: origin,
    derivation_path: DerivationPath::default(),
    wildcard: Wildcard::Unhardened,
	};

	let descriptor_keys=DescriptorPublicKey::XPub(descriptor_xkey);

	let pk =Wpkh::new(descriptor_keys).unwrap();
	let descriptor=ExtendedDescriptor::Wpkh(pk);



	let wallet = Wallet::new(
		descriptor,
		None,
		Network::Testnet,
		MemoryDatabase::new()
	).unwrap();
	;

	let blockchain = ElectrumBlockchain::from(Client::new("ssl://electrum.blockstream.info:60002").unwrap());
	let getBalance=wallet.sync(&blockchain, SyncOptions::default());

	getBalance.and_then(|_|{
		Ok({
			println!("p2wpkh {}", wallet.get_address(AddressIndex::LastUnused).unwrap().address);
			println!("extended key {}",extended_pub_key.to_string());
			println!("Balance {}", wallet.get_balance().unwrap())
		})
	}).unwrap();

		// let descriptor="wpkh(tprv8ZgxMBicQKsPdpkqS7Eair4YxjcuuvDPNYmKX3sCniCf16tHEVrjjiSXEkFRnUH77yXc6ZcwHHcLNfjdi5qUvw3VDfgYiH5mNsj5izuiu2N/1/2/*)#tqz0nc62";
	// let a=PublicKey::from_str(descriptor).unwrap();
	// DescriptorPublicKey::XPub()
	



   }