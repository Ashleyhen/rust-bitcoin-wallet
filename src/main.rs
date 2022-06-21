use std::env;

use btc_wallet::{ClientWithSchema, p2wpkh::P2PWKh, AddressSchema, p2tr::P2TR,input_data::{ApiCall, electrum_rpc::ElectrumRpc}};
use lightning::chain::{channelmonitor::ChannelMonitor, keysinterface::InMemorySigner};
// use taproot_multi_sig::WalletInfo;
pub mod btc_wallet;


// pub mod client_node;
// pub mod taproot_multi_sig;
fn main() {
	env::set_var("RUST_BACKTRACE", "full");
	_test_transactionn();

}

fn _test_transactionn(){
	// person 1 
	let seed= "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";
	let to_addr="tb1p95xjusgkgh2zqhyr5q9hzwv607yc5dncnsastm9xygmmuu4xrcqs53468m";
	let first_addr="tb1pz6egnzpq0h92zjkv23vdt4gwy8thd4t0t66megj20cr32m64ds4qv2kcal";

	let schema=P2TR::new(Some(seed.to_string()),0,3);
	let address_list=vec![to_addr.to_string(),first_addr.to_string()];
	dbg!(schema.aggregate(address_list));

	let client_with_schema=
	ClientWithSchema::new(schema,ElectrumRpc::new());
	client_with_schema.print_balance();
		
}

// seed, vec<derivation path>
// p2wpkh 8
// tr 1