use std::env;

use btc_wallet::{ClientWithSchema, p2wpkh::P2PWKh, AddressSchema, p2tr::P2TR,input_data::{ApiCall, electrum_rpc::ElectrumRpc}, Broadcast_op};
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
	let tr_3="tb1pz6egnzpq0h92zjkv23vdt4gwy8thd4t0t66megj20cr32m64ds4qv2kcal";

	let tr_5="tb1p69eefuuvaalsdljjyqntnrrtc4yzpc038ujm3ppze8g6ljepskks2zzffj";
	let schema=P2TR::new(Some(seed.to_string()),0,2);
	let address_list=vec![to_addr.to_string(),tr_3.to_string()];
	let aggregate=schema.aggregate(address_list);

	

	let client_with_schema=
	ClientWithSchema::new(schema,ElectrumRpc::new());
	client_with_schema.print_balance();
	client_with_schema.submit_tx(aggregate.to_string(), 1300, &Broadcast_op::Finalize);
		
}

// seed, vec<derivation path>
// p2wpkh 8
// tr 1