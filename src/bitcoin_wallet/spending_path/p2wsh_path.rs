use bitcoin::{
    secp256k1::{All, PublicKey, Secp256k1},
    util::bip32::KeySource,
    Transaction,
};

use crate::bitcoin_wallet::script_services::{
    output_service::{segwit_v0_add_key, segwit_v0_agg_witness},
    psbt_factory::{LockFn, UnlockFn},
};

pub struct P2wsh {
    pub secp: Secp256k1<All>,
}

impl P2wsh {
    pub fn new(secp: &Secp256k1<All>) -> Self {
        return P2wsh { secp: secp.clone() };
    }

    pub fn input_factory<'a>(
        &'a self,
    ) -> Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<UnlockFn<'a>> + 'a> {
        return Box::new(
            move |previous_list: Vec<Transaction>, current: Transaction| {
                let mut unlock_vec: Vec<UnlockFn> = vec![];

                // let script = Script::new_v0_p2wpkh(&pubkey.wpubkey_hash().unwrap());
                for (input_index, prev) in previous_list.iter().enumerate() {
                    // prev.output.iter().find(|t|t.script_pubkey.eq(other))
                }
                return unlock_vec;
            },
        );
    }

    pub fn output_factory<'a>(&'a self, keys: &'a Vec<(PublicKey, KeySource)>) -> Vec<LockFn<'a>> {
        let mut lock_fn:Vec<LockFn> = vec![];
        keys.iter()
            .for_each(|(pub_k, key_source)| {
                lock_fn.push(segwit_v0_add_key(pub_k, key_source));
            }
        );
        lock_fn.push(segwit_v0_agg_witness());
        return lock_fn;
    }
}

//
