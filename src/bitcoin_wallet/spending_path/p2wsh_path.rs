use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    secp256k1::{All, PublicKey, Secp256k1, SecretKey},
    util::bip32::KeySource,
    Address, PrivateKey, Script, Transaction,
};
use bitcoin_hashes::{sha256, Hash, HashEngine};
use miniscript::ToPublicKey;

use crate::bitcoin_wallet::{
    constants::NETWORK,
    script_services::{
        input_service::{sign_segwit_v0, insert_witness_tx},
        output_service::{segwit_v0_add_key, segwit_v0_agg_witness},
        psbt_factory::{LockFn, UnlockFn},
    },
    scripts::p2wsh_multi_sig,
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
        secret_key_1: SecretKey,
        secret_key_2: SecretKey,
    ) -> Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<Vec<UnlockFn<'a>>> + 'a> {
        return Box::new(
            move |previous_list: Vec<Transaction>, current: Transaction| {
                let pub_key = PublicKey::from_secret_key(&self.secp, &secret_key_1);
                let pub_key_2 = PublicKey::from_secret_key(&self.secp, &secret_key_2);
                let pub_keys = vec![pub_key, pub_key_2];
                let script =
                P2wsh::witness_program_fmt(p2wsh_multi_sig(&pub_keys));
                let mut unlock_vec_vec: Vec<Vec<UnlockFn>> = vec![];
                for (input_index, prev) in previous_list.iter().enumerate() {
                    let mut unlock_vec: Vec<UnlockFn> = vec![];
                    let tx_out = prev
                        .output
                        .iter()
                        .find(|t| 
                            t.script_pubkey.eq(&script)
                        )
                        .expect("missing expected witness");
                    unlock_vec.push(insert_witness_tx(tx_out.clone()));
                    unlock_vec.push(sign_segwit_v0(
                        &self.secp,
                        current.clone(),
                        tx_out.value,
                        input_index,
                        P2wsh::witness_program_fmt(p2wsh_multi_sig(&pub_keys)),
                        secret_key_2,
                    ));

                    unlock_vec.push(sign_segwit_v0(
                        &self.secp,
                        current.clone(),
                        tx_out.value,
                        input_index,
                        P2wsh::witness_program_fmt(p2wsh_multi_sig(&pub_keys)),
                        secret_key_1,
                    ));

                    unlock_vec_vec.push(unlock_vec);
                }
                return unlock_vec_vec;
            },
        );
    }

    pub fn output_factory<'a>(&'a self, keys: &'a Vec<(PublicKey, KeySource)>) -> Vec<LockFn<'a>> {
        let mut lock_fn: Vec<LockFn> = vec![];
        keys.iter().for_each(|(pub_k, key_source)| {
            lock_fn.push(segwit_v0_add_key(pub_k, key_source));
        });
        lock_fn.push(segwit_v0_agg_witness());
        return lock_fn;
    }

    // return  ;
    pub fn witness_program_fmt(script: Script) -> Script {
        let mut engine = sha256::HashEngine::default();
        engine.input(script.as_bytes());
        let hash = sha256::Hash::from_engine(engine).into_inner();
        return Builder::new()
            .push_opcode(all::OP_PUSHBYTES_0)
            .push_slice(&hash)
            .into_script();
    }
}
