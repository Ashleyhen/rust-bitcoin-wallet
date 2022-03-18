use bitcoin::{secp256k1::{ Secp256k1, KeyPair,rand },util::address, Network,  XOnlyPublicKey};



pub fn generate_wallet(){
	
let network =Network::Bitcoin;

let secp = Secp256k1::new();

let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());

let key_pair = KeyPair::from_secret_key(&secp, secret_key);

let x_only_public_key =XOnlyPublicKey::from_keypair(&key_pair);

let tap_address =address::Address::p2tr(&secp, x_only_public_key, None, network);

let p2wpkh_address =address::Address::p2wpkh( &bitcoin::PublicKey::new(public_key) , network);

println!("following pay two public key hash address {}", p2wpkh_address.ok().unwrap().to_qr_uri().to_ascii_lowercase());

println!("following taproot address {}", tap_address.to_qr_uri().to_ascii_lowercase());

}

