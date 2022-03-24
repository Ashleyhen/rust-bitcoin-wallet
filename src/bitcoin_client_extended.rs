use bdk::{keys::DescriptorKey, miniscript::DescriptorPublicKey};
// use bdk::{keys::DescriptorKey, miniscript::DescriptorPublicKey, bitcoin::{Network, util::{bip32::{ExtendedPrivKey, ExtendedPubKey}, key}, secp256k1::{self, Secp256k1}, hashes::hex::FromHex}};
// use bitcoin::{secp256k1::rand, hashes::hex::FromHex};
use bitcoin::{hashes::hex::FromHex, secp256k1::{rand, Secp256k1}, util::bip32::{self, ExtendedPrivKey, ExtendedPubKey}, Network};

pub fn bitcoin_client_extended(){
let secp = Secp256k1::new(); // Bitcoins elliptic public  
 
// bitcoin::util::bip32::ExtendedPrivKey
// key::PrivateKey::new(key, Network::Testnet);

// https://docs.rs/bdk/latest/bdk/index.html

// key::PrivateKey::new(, network)
// let a=key::PublicKey::from_private_key(secp, sk);

let (secrete,known_key)= secp.generate_keypair(&mut rand::thread_rng());

	let seed=Vec::from_hex(&secrete.display_secret().to_string()).unwrap();
let master_node=ExtendedPrivKey::new_master(Network::Bitcoin, &seed).unwrap();
let public_key=ExtendedPubKey::from_priv(&secp, &master_node);

// DescriptorPublicKey::XPub(public_key);



println!("{}",master_node.to_string());

}