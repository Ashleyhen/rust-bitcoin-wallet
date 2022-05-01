use std::{str::FromStr, borrow::{BorrowMut, Borrow}, time::SystemTime, sync::Arc, rc};

use bdk::{template::Bip84, KeychainKind, bitcoin::{util::bip32::ExtendedPrivKey, Network, Address, Transaction, secp256k1::{Secp256k1, All, constants}, BlockHash, Script}, Wallet, database::MemoryDatabase, blockchain::{ElectrumBlockchain, Blockchain}, electrum_client::Client, SyncOptions, wallet::AddressIndex, FeeRate};
use bitcoin::hashes::hex::{FromHex, HexIterator};
use lightning::{chain::{chaininterface::{BroadcasterInterface, FeeEstimator, ConfirmationTarget}, chainmonitor::{Persist, MonitorUpdateId}, keysinterface::{Sign, self, KeysManager, InMemorySigner}, channelmonitor::{ChannelMonitorUpdate, ChannelMonitor}, transaction::OutPoint, ChannelMonitorUpdateErr, BestBlock}, util::logger::{Logger, Record}, ln::channelmanager::ChainParameters};
use lightning_persister::FilesystemPersister;

use crate::bitcoin_keys::{self, BitcoinKeys};

pub struct WalletContext{
	pub wallet_state: Wallet<MemoryDatabase>,
	pub blockchain:ElectrumBlockchain,
	network:u32,
	seed: [u8;constants::SECRET_KEY_SIZE] 
}

impl WalletContext {
	pub fn new (seed:Option<String>)-> WalletContext{

		let keys = bitcoin_keys::BitcoinKeys::new(seed.to_owned());
		// keys.get_balance();

		let network=Network::from_magic(keys.network.magic()).unwrap();
		
		let extended_priv_key=keys.adapt_bitcoin_extended_priv_keys_to_bdk_version() ;

		let descriptor =Bip84(extended_priv_key,KeychainKind::External);

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
		return WalletContext{ wallet_state,blockchain,network:keys.network.magic(),seed: keys.seed };
	}

	


 pub fn  get_balance(&self){

		self.wallet_state.sync(&self.blockchain, SyncOptions::default()).and_then(|_|Ok({
			println!("p2wpkh {}", self.wallet_state.get_address(AddressIndex::LastUnused).unwrap_or_else(|err|panic!("failed derive the next address !! {}",err)).address);
			println!("Balance {}",self.wallet_state.get_balance().unwrap_or_else(|err| panic!("failed to retrieve the balance from the current wallet !! {}", err)))
		})).unwrap_or_else(|_|println!("failed to sync wallet !!"));
	
	}

	pub fn send_coins(&self, send_address: &str,stats: u64){
		let address=Address::from_str(send_address).unwrap_or_else(|err|panic!("invalid address bitcoin : {}",err));
			let mut builder=self.wallet_state.build_tx();
			builder.fee_rate(FeeRate::from_sat_per_vb(2.0)).add_recipient(address.script_pubkey(), stats);
			let (mut psbt, details)=builder.finish().unwrap_or_else(|err|panic!("error invalid transaction! {}",err));
		
			// dbg!(psbt.clone());
			let is_transaction_valid = self.wallet_state.sign(&mut psbt, Default::default() )
		.unwrap_or_else(|err|panic!("wallet signature failed!!! {}",err));

		dbg!(psbt.clone());
		let signed_transaction=psbt.clone().extract_tx();
/* 
		for inp in psbt.clone().extract_tx().input{
	// dbg!(Witness::from_vec(inp.witness));
	for w in inp.witness{
let scr=Script::bytes_to_asm(&w);
		dbg!(scr);
		// Script::from_byte_iter(w.iter()).unwrap();

	}

}

		println!("valid transaction: {}",is_transaction_valid);
		let signed_transaction=psbt.clone().extract_tx();
		*/
		// println!("transaction id: {}",signed_transaction.txid().to_string());
		// self.broadcast_transaction( &signed_transaction);
		// println!("broadcasted transaction successfully");
	}
}

impl BroadcasterInterface for WalletContext{
    fn broadcast_transaction(&self, tx: &Transaction) {
		return self.blockchain.broadcast(tx).unwrap_or_else(|err|panic!("transaction failed to broadcast! {}",err));
    }
}

impl FeeEstimator for WalletContext{
    fn get_est_sat_per_1000_weight(&self, confirmation_target: ConfirmationTarget) -> u32 {
	match confirmation_target {
			ConfirmationTarget::Background=> 10000,
			ConfirmationTarget::Normal=> 100000,
			ConfirmationTarget::HighPriority=> 10000000,
		}
    }
}

	// Step 2: Initialize the Logger
impl Logger for WalletContext{
        // <insert code to print this log and/or write this log to disk>
		fn log(&self, record: &Record) {
		let raw_log = record.args.to_string();
		let log = format!( "{:<5} [{}:{}] {}\n", record.level.to_string(), record.module_path, record.line, raw_log);
		dbg!(log);
	}
}

impl WalletContext{
	pub fn generate_channel_keys(&self)->Arc<KeysManager>{
		let current_time =SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap(); 
		return Arc::new(KeysManager::new( &self.seed ,current_time.as_secs(),current_time.subsec_nanos()));
	}

	pub fn get_network(&self)-> Network{return Network::from_magic(self.network).unwrap();}

	pub fn get_chain_parameters(&self) -> ChainParameters{ 
		let network =self.get_network();
		return ChainParameters{ network, best_block:BestBlock::from_genesis(network) }

	 }

	pub fn persister()->FilesystemPersister{
		let path_to_channel_data="network/channel";
		return FilesystemPersister::new(path_to_channel_data.to_string());
	}

	pub fn channel_monitor_and_block_hash(&self)->Vec<(BlockHash, ChannelMonitor<InMemorySigner>)>{
		return  WalletContext::persister().read_channelmonitors(self.generate_channel_keys().clone()).unwrap();
	}

 pub fn map_channel_monitor<'s>(arg:&'s Vec<(BlockHash, ChannelMonitor<InMemorySigner>)>)->Vec::<&'s lightning::chain::channelmonitor::ChannelMonitor<InMemorySigner>>{
 return arg.clone().iter().map(|t| t.1.borrow())
	.collect::<Vec::<&lightning::chain::channelmonitor::ChannelMonitor<InMemorySigner>>>();

		// let mut vec=Vec::new();
		// arg.clone().iter().for_each(| t  |{
		// 	  vec.push(t.1.borrow());  
		// 	});
		// 	return vec ;
	}


}

/*fn test1(key: &'static Arc<KeysManager> ) ->Vec::<&lightning::chain::channelmonitor::ChannelMonitor<InMemorySigner>>{
//  let arg=WalletContext::persister().read_channelmonitors(key.clone()).unwrap();
	// arg.clone();
	//  WalletContext::persister().read_channelmonitors(key.clone()).unwrap();
	let ss=WalletContext::persister().read_channelmonitors(key.clone()).unwrap();
	 return ss   
	 .iter().borrow().clone().map(|t:&'static (BlockHash,ChannelMonitor<InMemorySigner>)  |t.1.borrow().clone())
	.collect::<Vec::<&lightning::chain::channelmonitor::ChannelMonitor<InMemorySigner>>>();;

	// return s;

}*/