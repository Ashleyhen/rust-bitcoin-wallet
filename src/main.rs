pub mod client_wallet;
pub mod bitcoin_keys;

fn main() {
    let from ="69d69668a50aec2837fe3b5bc1d4c070a5e8f496fd0ad53af416316c7a62a16b";
let wallet=client_wallet::WalletContext::new(Some(from.to_string()));
wallet.get_balance();

let to = "tb1ql7w62elx9ucw4pj5lgw4l028hmuw80sndtntxt";
wallet.send_coins(to, 100);
}
// tb1qsdkpm9hnm387qg8w5e0ehqr3ememjlrdlyf8nr