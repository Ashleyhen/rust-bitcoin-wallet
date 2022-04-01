
use std::sync::Arc;

use bdk::{bitcoin::Transaction, blockchain::Blockchain};
use lightning::chain::{chaininterface::BroadcasterInterface,  transaction::OutPoint, channelmonitor::{ChannelMonitor, ChannelMonitorUpdate}, ChannelMonitorUpdateErr, chainmonitor::Persist, keysinterface::Sign};
use lightning_persister::FilesystemPersister;
use crate::client_wallet::BtcWallet;

impl BroadcasterInterface for BtcWallet{
    fn broadcast_transaction(&self, tx: &Transaction) {
		self.blockchain.broadcast(tx);
    }
}
fn startLdk(){

	

// let persister=Arc::new(FilesystemPersister::new(ldk_data_dir.clone()));
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
