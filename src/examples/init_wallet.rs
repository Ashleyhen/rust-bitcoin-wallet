use std::{str::FromStr };
use bitcoin::{secp256k1::{ Secp256k1, KeyPair,rand,PublicKey, SecretKey },util::{address, bip32::{ExtendedPrivKey, ExtendedPubKey}}, Network,  XOnlyPublicKey, hashes::hex::FromHex};

pub struct Bitcoin_keys {
	pub ext_private_key: String,
	pub ext_public_key: String,
	pub public_key: String,
	pub network: u32,
	pub fingure_print:String
}

impl Bitcoin_keys{

	pub fn new(secret_key:Option<String> ) ->  Bitcoin_keys {
		let network =Network::Testnet;

		// G(x,y)=(y^2=x^3+b mod s)
		let secp = Secp256k1::new(); // Bitcoins elliptic public  
		
		let secret_key_mapping= |prv:&String| {SecretKey::from_str(&prv).unwrap_or_else(
			|err| panic!("invalid secrete key, {}", err))};

		let (secret_key, public_key) = match secret_key {
			Some(prv)=>(secret_key_mapping(&prv),PublicKey::from_secret_key(&secp, &secret_key_mapping(&prv))),
			None => secp.generate_keypair(&mut rand::thread_rng())
		};
		let p2wpkh_address =address::Address::p2wpkh( &bitcoin::PublicKey::new(public_key) , network).unwrap().to_qr_uri().to_ascii_lowercase(); // derives a pay two witness public key hash

		let seed=Vec::from_hex(&secret_key.display_secret().to_string()).unwrap();
		let master_node=ExtendedPrivKey::new_master(network, &seed).unwrap();
		let public_key=ExtendedPubKey::from_priv(&secp, &master_node);
		let fingure_print=master_node.fingerprint(&secp).to_string();

		return Bitcoin_keys { 
			ext_private_key:master_node.to_string(), 
			ext_public_key:public_key.to_string(),
			public_key: p2wpkh_address,
			network:network.magic(), 
			fingure_print,
		}
	}

	pub fn to_string(&self){
		println!("Bitcoin_keys:\next_private_key: {}\next_public_key: {}\nnetwork: {}",self.ext_private_key,self.ext_public_key,self.network);

	}
}

// master node 
// purpose node = p2pk/p2wpkh/p2sh/p2tr 
// cointype recieve = testnet/mainnet/regnet
// recieve addresses

pub fn generate_wallet(secret_key:Option<String> ){

let network =Network::Testnet;

// G(x,y)=(y^2=x^3+b mod s)
let secp = Secp256k1::new(); // Bitcoins elliptic public  
let secret_key_mapping= |prv:&String| {SecretKey::from_str(&prv).unwrap_or_else(
	|err| panic!("invalid secrete key, {}", err))};

let (secret_key, public_key) =match secret_key {
	Some(prv)=>(secret_key_mapping(&prv),PublicKey::from_secret_key(&secp, &secret_key_mapping(&prv))),
    None => secp.generate_keypair(&mut rand::thread_rng())
};



let key_pair = KeyPair::from_secret_key(&secp, secret_key); // creates a new public key from our schnorr pair 

let x_only_public_key =XOnlyPublicKey::from_keypair(&key_pair); // verifies our schnorr signature and serializes our public key 

let tap_address =address::Address::p2tr(&secp, x_only_public_key, None, network); // derives a taproot address

let p2wpkh_address =address::Address::p2wpkh( &bitcoin::PublicKey::new(public_key) , network); // derives a pay two witness public key hash

let p2wpkh_address =address::Address::p2wpkh( &bitcoin::PublicKey::new(public_key) , network); // derives a pay two witness public key hash





println!("following pay two public key hash address {}", p2wpkh_address.ok().unwrap().to_qr_uri().to_ascii_lowercase());

println!("following taproot address {}", tap_address.to_qr_uri().to_ascii_lowercase());

println!("your secrete key is {}", secret_key.display_secret());

}

