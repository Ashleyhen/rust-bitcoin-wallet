use std::{borrow::Borrow, env};

use bdk::bitcoin::BlockHash;
use bitcoin::hashes::Hash;
use client_wallet::WalletContext;
use lightning::chain::{channelmonitor::ChannelMonitor, keysinterface::InMemorySigner};
// use taproot_multi_sig::WalletInfo;

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

	// let to="mv4rnyY3Su5gjcDNzbMLKBQkBicCtHUtFB";
	
	// let seed_2="6ad78e4a5a39f597618de01eb788b89dce4f23a4375ad70aa535ecd122145cd8";
	// let thereAddr= "tb1pph50luptud9wkq2lvn0h9kvm6s0dz058afvxhnu3fpltruyqzx4s9x39s7";

	let c=WalletContext::new(Some(seed.to_owned()));
	let key=bitcoin_keys::BitcoinKeys::new(Some(seed.to_owned()));

	key.get_balance();
	key.transaction();

		let to_addr="tb1qhzwe6wr46rvfndz9kzrdtzddepah82vqtyqde5";
		c.get_balance();
	c.send_coins(to_addr, 1000);
	
	// let wallet_info=WalletInfo::get_taproot_address(Some(seed_2.to_owned()));
	// wallet_info.single_sign();
	
}