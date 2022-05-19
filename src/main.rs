use std::env;

use btc_wallet::{ClientWithSchema, p2wpkh::P2PWKh, AddressSchema, p2tr::P2TR};
use lightning::chain::{channelmonitor::ChannelMonitor, keysinterface::InMemorySigner};
// use taproot_multi_sig::WalletInfo;
pub mod wallet_p2wpkh;
pub mod btc_wallet;
pub mod bitcoin_keys;
pub mod client_wallet;

// pub mod client_node;
// pub mod taproot_multi_sig;
fn main() {
	env::set_var("RUST_BACKTRACE", "full");
	_test_transactionn();

}

fn _test_transactionn(){
	// person 1 
	let seed= "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";
		let to_addr="tb1puma0fas8dgukcvhm8ewsganj08edgnm6ejyde3ev5lvxv4h7wqvqpjslxz";
	
	let client_with_schema=ClientWithSchema::new(P2PWKh::new(Some(seed.to_string())) );
	client_with_schema.print_balance(0,6);

		client_with_schema.submit_tx(to_addr.to_string(),1000, 0, 6);
		
}
// seed, vec<derivation path>