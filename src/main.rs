pub mod bitcoin_client_extended;
pub mod bitcoin_keys;
pub mod connections;
fn main() {

	// bitcoin_client_extended::bitcoin_client_extended();

	let key= "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";
	// Some(key.to_owned())
	// bitcoin_client::generate_wallet(Some(key.to_owned()) );

	 let bitcoin_keys=bitcoin_keys::Bitcoin_keys::new(Some(key.to_owned()));
	 connections::blockchain_connection(bitcoin_keys);

	// let address =bitcoin_client::generate_wallet(None );
	

// connections::blockchain_connection(address);

}

// tb1qzvsdwjay5x69088n27h0qgu0tm4u6gwq202sjy