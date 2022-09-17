use bitcoin::{secp256k1::{All, Secp256k1}, XOnlyPublicKey};

use crate::bitcoin_wallet::script_services::{psbt_factory::LockFn, output_service::merge_x_only};

pub struct MuSig<'a>{
	secp: &'a Secp256k1<All>,
}

impl <'a> MuSig<'a>{

	pub fn new(secp:&'a Secp256k1<All>)->Self{
		return MuSig { secp };
	}

	pub fn merge(&'a self, x_only:XOnlyPublicKey, x_only_2:XOnlyPublicKey)->Vec<LockFn<'a>>{
		return vec![merge_x_only(self.secp, x_only, x_only_2)];
	}

}