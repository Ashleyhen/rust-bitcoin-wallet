use bitcoin::{secp256k1::{Secp256k1, All}, XOnlyPublicKey};

use crate::bitcoin_wallet::script_services::{psbt_factory::LockFn, output_service::{new_tap_internal_key, insert_tap_tree, insert_tap_key_origin, insert_tree_witness}};

use super::scripts::TapScripts;

pub struct AdaptorScript<'a> {
    secp: &'a Secp256k1<All>,
}

impl <'a> AdaptorScript<'a>{

	pub fn new(secp:&'a Secp256k1<All>)->Self{
		return AdaptorScript{secp};
	}

	pub fn adaptor_sig(
		&'a self,
		xinternal:XOnlyPublicKey,
		primary_xonly:XOnlyPublicKey,
		secondary_xonly: XOnlyPublicKey
	)->Vec<LockFn<'a>>{
		let delay=TapScripts::delay(&primary_xonly);
		let multi_sig=TapScripts::multi_2_of_2_script(&primary_xonly, &secondary_xonly);
		let combined_script = vec![(1, delay.get_script()), (1, multi_sig.get_script())];
		return vec![
			new_tap_internal_key(xinternal),
			insert_tap_tree(combined_script.clone()),
			insert_tap_key_origin(combined_script, primary_xonly),
			insert_tree_witness(&self.secp)
			];
	}

}
