use std::str::FromStr;

use bdk::{template::Bip84, KeychainKind, bitcoin::{util::bip32::ExtendedPrivKey, Network, Address}, Wallet, database::MemoryDatabase, blockchain::{ElectrumBlockchain, Blockchain}, electrum_client::Client, SyncOptions, wallet::AddressIndex, FeeRate};

use crate::bitcoin_keys::BitcoinKeys;

pub struct WalletContext{
	wallet_state: Wallet<MemoryDatabase>,
	blockchain:ElectrumBlockchain
}
impl WalletContext{
	pub fn new (keys :BitcoinKeys)-> WalletContext{
		let invalid_master_key=|err|panic!("invalid master key, using bdk {}",err);

		let master_key=ExtendedPrivKey::from_str(&keys.master_key).unwrap_or_else(invalid_master_key);

		let network=Network::from_magic(keys.network).unwrap();

		let descriptor =Bip84(master_key,KeychainKind::External);

		let wallet_state = Wallet::new(
			descriptor,
			None,
			network,
			MemoryDatabase::default(),
		)
		.unwrap_or_else(|err|panic!("invalid wallet, {} ", err));
		
		let blockchain = ElectrumBlockchain::from(
			Client::new("ssl://electrum.blockstream.info:60002")
			.unwrap_or_else(|err|panic!("client connection failed !!!{}",err)));

		let get_balance=wallet_state.sync(&blockchain, SyncOptions::default());

		get_balance.and_then(|_|Ok({
			println!("p2wpkh {}", wallet_state.get_address(AddressIndex::LastUnused).unwrap_or_else(|err|panic!("failed derive the next address !! {}",err)).address);
			println!("Balance {}",wallet_state.get_balance().unwrap_or_else(|err| panic!("failed to retrieve the balance from the current wallet !! {}", err)))
		})).unwrap_or_else(|_|println!("failed to sync wallet !!"));
		
		return WalletContext{ wallet_state,blockchain };

	}
// "mv4rnyY3Su5gjcDNzbMLKBQkBicCtHUtFB"
	pub fn send_coins(&self, send_address: &str,stats: u64){
		let address=Address::from_str(send_address).unwrap_or_else(|err|panic!("invalid address bitcoin : {}",err));

		let (mut psbt, details)={
			let mut builder=self.wallet_state.build_tx();
			builder.drain_wallet().fee_rate(FeeRate::from_sat_per_vb(2.0)).add_recipient(address.script_pubkey(), stats);
			builder.finish().unwrap_or_else(|err|panic!("error invalid transaction! {}",err))
		};

		let is_transaction_valid = self.wallet_state.sign(&mut psbt, Default::default() )
		.unwrap_or_else(|err|panic!("wallet signature failed!!! {}",err));

		println!("valid transaction: {}",is_transaction_valid);
		let signed_transaction=psbt.clone().extract_tx();
		println!("transaction id: {}",signed_transaction.txid().to_string());
		
		let broadcast=self.blockchain.broadcast(&signed_transaction);

		broadcast.unwrap_or_else(|err|panic!("transaction failed to broadcast! {}",err));
		println!("broadcasted transaction successfully");
	}
}
