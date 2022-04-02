use std::str::FromStr;

use bitcoin::{Network, secp256k1::{SecretKey, rand::rngs::OsRng}, util::bip32::ExtendedPrivKey};

pub struct BitcoinKeys{
    pub master_key: String,
    pub network: u32
}


impl BitcoinKeys{
    pub fn new (secret_seed:Option<String>)-> BitcoinKeys {
        let network= Network::Testnet;
        
        let seed =match secret_seed{
            Some(secret)=>SecretKey::from_str(&secret).unwrap(),
            _=>SecretKey::new(&mut OsRng::new().unwrap())
        };

        let master_key=ExtendedPrivKey::new_master(network, &seed.secret_bytes()).unwrap();

        println!("your seed is {}",seed.display_secret());
        return BitcoinKeys { master_key:master_key.to_string(), network:network.magic() };

    }
}