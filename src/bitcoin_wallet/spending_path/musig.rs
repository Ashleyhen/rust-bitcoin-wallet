use crate::bitcoin_wallet::script_services::{output_service::merge_x_only, psbt_factory::LockFn};
use bitcoin::{
    secp256k1::{All, Secp256k1},
    XOnlyPublicKey,
};
use bitcoincore_rpc::bitcoincore_rpc_json::AddMultiSigAddressResult;

pub struct MuSig_Script<'a> {
    secp: &'a Secp256k1<All>,
}

impl<'a> MuSig_Script<'a> {
    pub fn new(secp: &'a Secp256k1<All>) -> Self {
        return MuSig_Script { secp };
    }

    pub fn merge(&'a self, x_only: XOnlyPublicKey, x_only_2: XOnlyPublicKey) -> Vec<LockFn<'a>> {
        return vec![merge_x_only(self.secp, x_only, x_only_2)];
    }

    // call endpoint and subscribe to a endpoint
    pub fn broadcast(i_xonly: &XOnlyPublicKey, x_xonly: &XOnlyPublicKey) -> Vec<XOnlyPublicKey> {
        return vec![i_xonly.clone(), x_xonly.clone()];
        // let musig = MuSig::<Sha256, Schnorr<Sha256, Deterministic<Sha256>>>::new(schnorr);
    }

    pub fn keyAggCoef(l: Vec<XOnlyPublicKey>, x: XOnlyPublicKey) {
        // H_agg(l,x)
    }

    pub fn compute_challenge(x_only: &Vec<XOnlyPublicKey>) {
        dbg!(x_only.len());
    }
}
