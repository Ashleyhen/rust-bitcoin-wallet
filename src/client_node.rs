
use std::{sync::Arc, time::SystemTime, borrow::{Borrow, BorrowMut}, ops::Deref};

use bdk::{FeeRate, bitcoin::{Transaction, Script, Txid, Network, BlockHash}, blockchain::{Blockchain, GetHeight}, Wallet};
use lightning::{chain::{chaininterface::{FeeEstimator, ConfirmationTarget, BroadcasterInterface}, chainmonitor::{Persist, self}, keysinterface::{Sign, InMemorySigner, KeysManager, KeysInterface}, Filter, WatchedOutput, BestBlock, channelmonitor::ChannelMonitor}, util::{logger::{Logger, Record}, config::UserConfig, events::PaymentPurpose}, ln::channelmanager::{ChannelManagerReadArgs, self, ChainParameters, ChannelManager}};
use lightning_persister::FilesystemPersister;
use bitcoin::secp256k1::rand::{rngs::OsRng, RngCore};

use crate::client_wallet::WalletContext;

type ChainMonitor = chainmonitor::ChainMonitor<
	InMemorySigner,
	Arc<dyn Filter + Send + Sync>,
	Arc<WalletContext>,
	Arc<WalletContext>,
	Arc<WalletContext>,
	Arc<FilesystemPersister>,
>;



		// step 1

impl Filter for WalletContext {
	fn register_tx(&self, txid: &Txid, script_pubkey: &Script) {
        // <insert code for you to watch for this transaction on-chain>
	}

fn register_output(&self, output: WatchedOutput) -> Option<(usize, Transaction)> {
        todo!()
    }

	
}




pub fn new(seed:Option<String>){
	let wallet=WalletContext::new(seed);
	let arc_wallet=Arc::new(wallet);

	let chain_monitor :Arc<ChainMonitor> = Arc::new(chainmonitor::ChainMonitor::new(
		None, 
		arc_wallet.clone(),
		arc_wallet.clone(),
		arc_wallet.clone(),
		Arc::new(WalletContext::persister()))
	);

	let mut user_config= UserConfig::default();
	user_config.peer_channel_config_limits.force_announced_channel_preference = false;
		

	let mut mutable_channel_monitor=arc_wallet.channel_monitor_and_block_hash();

	let channel_monitors=mutable_channel_monitor.iter_mut()
	.map(|t| t.1.borrow_mut())
	.collect::<Vec::<&mut lightning::chain::channelmonitor::ChannelMonitor<InMemorySigner>>>();
	
	// initialize the channelManager
	let best_block=BestBlock::from_genesis(Network::Testnet);
	if (channel_monitors.len()>0) {

		let channel_manager_reader=ChannelManagerReadArgs::new(
		arc_wallet.generate_channel_keys(),
	arc_wallet.clone(),
		chain_monitor.clone(),
		arc_wallet.clone(),
		arc_wallet.clone(),
		user_config,
		channel_monitors);

	} else { 

		let chain_params=ChainParameters{
			network: bdk::bitcoin::Network::Testnet,
			best_block
		};

	let channel_manager =ChannelManager::new(
			arc_wallet.clone(), 
			chain_monitor.clone(), 
			arc_wallet.clone(),
			arc_wallet.clone(),
			arc_wallet.generate_channel_keys(),
			user_config,
			chain_params);
	};

// Step 5: Initialize the ChainMonitor
impl WalletContext  {
	pub fn startLdk(self){

	let wallet_arc =Arc::new(self);
		let path_to_channel_data="network/channel";
		let logger_data="network/logger";

	let persister=Arc::new(FilesystemPersister::new(path_to_channel_data.to_string()));

	let chain_monitor :Arc<ChainMonitor> = Arc::new(
		chainmonitor::ChainMonitor::new( None, wallet_arc.clone(), wallet_arc.clone(), wallet_arc.clone(), persister.clone()));
		
		// step 7: read channelmonitor state from disk 
	let key_manager=wallet_arc.clone().generate_channel_keys();

	// 8. Marshal ChannelMonitors from disk
	let mut channel_monitors=persister.read_channelmonitors(key_manager.clone()).unwrap();

	// initialize the channelManager
	let mut user_config= UserConfig::default();
	user_config.peer_channel_config_limits.force_announced_channel_preference=false;

let chain_params=ChainParameters{
	network: bdk::bitcoin::Network::Testnet,
	best_block:BestBlock::from_genesis(Network::Testnet),
	};
	
	// 9. Initialize the ChannelManager
	let fresh_channel_manager=ChannelManager::new(
		wallet_arc.clone(), 
		chain_monitor.clone(), 
		wallet_arc.clone(),
		 wallet_arc.clone(),
		 wallet_arc.generate_channel_keys(),
		  user_config,chain_params);
	}
}
	

} 





// trait A {
//     fn execute(&self) -> ();
// }

// impl<T> A for T where T: Fn() -> () {
//     fn execute(&self) {
//         self()
//     }
// }


	 
	//  let a=(t)->BroadcasterInterface::broadcast_transaction;
