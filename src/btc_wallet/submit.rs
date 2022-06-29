use bitcoin::{psbt::PartiallySignedTransaction, Transaction};
use miniscript::psbt::PsbtExt;

use super::{ ClientWallet};

 
 impl ClientWallet{

	pub fn finalize(&self, psbt:PartiallySignedTransaction,broad_cast_fn:&dyn Fn(&Transaction)->())->PartiallySignedTransaction{
		return psbt.clone().finalize(&self.secp).map(|final_psbt| { 
				broad_cast_fn(&final_psbt.clone().extract_tx());
				return final_psbt.clone();
			}).unwrap();
	}

 }