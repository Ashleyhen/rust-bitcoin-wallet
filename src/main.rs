pub mod bitcoin_keys;
pub mod client_wallet;
pub mod initialize_client;
fn main() {

	_test_transactionn();

}

fn _test_transactionn(){
	
	let seed= "1d454c6ab705f999d97e6465300a79a9595fb5ae1186ae20e33e12bea606c094";

	let to="mv4rnyY3Su5gjcDNzbMLKBQkBicCtHUtFB";

	let keys=bitcoin_keys::BitcoinKeys::new(Some(seed.to_owned()));
	
	let wallet_context =client_wallet::BtcWallet::new(keys);

	wallet_context.get_balance();
	

	// wallet_context.send_coins(to,10000);


}