use bitcoin::{Transaction, Txid, Script};
use lightning::{chain::{chaininterface::{FeeEstimator, ConfirmationTarget, BroadcasterInterface}, Filter, WatchedOutput, chainmonitor::{ChainMonitor, Persist}, keysinterface::{Sign, InMemorySigner}, ChannelMonitorUpdateErr, channelmonitor::{ChannelMonitor, ChannelMonitorUpdate}, transaction::OutPoint}, util::logger::{Logger, Record}, ln::peer_handler::MessageHandler};

struct YourFeeEstimator();

pub mod base_protocal;

impl FeeEstimator for YourFeeEstimator {
	fn get_est_sat_per_1000_weight(
		&self, confirmation_target: ConfirmationTarget,
	) -> u32 {
		match confirmation_target {
			ConfirmationTarget::Background => 10,// fetch background feerate,
			ConfirmationTarget::Normal => 100,// fetch normal feerate (~6 blocks)
			ConfirmationTarget::HighPriority =>1000, // fetch high priority feerate
		}
	}
}

struct YourLogger();

impl Logger for YourLogger {
	fn log(&self, record: &Record) {
		let raw_log = record.args.to_string();
		let log = format!(
			"{:<5} [{}:{}] {}\n",
			// OffsetDateTime::now_utc().format("%F %T"),
			record.level.to_string(),
			record.module_path,
			record.line,
			raw_log
		);
        // <insert code to print this log and/or write this log to disk>
	}
}
struct YourTxBroadcaster();

impl BroadcasterInterface for YourTxBroadcaster {
	fn broadcast_transaction(&self, tx: &Transaction) {
		dbg!(tx);
        // <insert code to broadcast this transaction>
	}
}

struct YourTxFilter();

impl Filter for YourTxFilter {
	fn register_tx(&self, txid: &Txid, script_pubkey: &Script) {
        // <insert code for you to watch for this transaction on-chain>
	}

	fn register_output(&self, output: WatchedOutput) ->
        Option<(usize, Transaction)> {

			return None;
        // <insert code for you to watch for any transactions that spend this
        // output on-chain>
    }
}

use lightning_persister::FilesystemPersister; // import LDK sample persist module





pub fn testldk(){

	let filter: Option<Box<dyn Filter>> = Some(Box::new(YourTxFilter())); // leave this as None or insert the Filter trait
										// object, depending on what you did for Step 4
	let logger=YourLogger();

	let fee_estimator = YourFeeEstimator();
	let broadcaster=YourTxBroadcaster();
	
	let mut persister=FilesystemPersister::new("".to_owned());

	let chainMonitor:ChainMonitor<InMemorySigner, Box<dyn lightning::chain::Filter>, &YourTxBroadcaster, &YourFeeEstimator, &YourLogger, &mut FilesystemPersister>=ChainMonitor::new(filter, &broadcaster, &logger, &fee_estimator,  &mut persister);

 
}


// let fee_estimator = YourFeeEstimator();
 