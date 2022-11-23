use bitcoin::{
    secp256k1::{All, Secp256k1, SecretKey},
    PrivateKey, Script, Transaction,
};

use crate::bitcoin_wallet::{
    constants::NETWORK,
    script_services::{
        input_service::{insert_witness, insert_witness_tx_out, sign_segwit_v0},
        psbt_factory::UnlockFn,
    },
    scripts::p2wpkh_script_code,
};

pub struct P2wpkh {
    secp: Secp256k1<All>,
}

impl P2wpkh {
    pub fn new(secp: &Secp256k1<All>) -> Self {
        return P2wpkh { secp: secp.clone() };
    }
    pub fn input_factory<'a>(
        &'a self,
        secret: SecretKey,
    ) -> Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<Vec<UnlockFn<'a>>> + 'a> {
        return Box::new(
            move |previous_list: Vec<Transaction>, current: Transaction| {
                let pubkey = bitcoin::PublicKey::from_private_key(
                    &self.secp,
                    &PrivateKey::new(secret, NETWORK),
                );
                let script = Script::new_v0_p2wpkh(&pubkey.wpubkey_hash().unwrap());
                let mut vec_vec_unlock: Vec<Vec<UnlockFn>> = vec![];
                for (input_index, prev) in previous_list.iter().enumerate() {
                    let mut unlock_vec: Vec<UnlockFn> = vec![];
                    let tx_out = prev
                        .output
                        .iter()
                        .find(|t| t.script_pubkey.eq(&script))
                        .expect("missing expected witness");

                    unlock_vec.push(insert_witness_tx_out(tx_out.clone()));
                    unlock_vec.push(insert_witness(tx_out.script_pubkey.to_owned()));
                    unlock_vec.push(sign_segwit_v0(
                        &self.secp,
                        current.clone(),
                        tx_out.value,
                        input_index,
                        p2wpkh_script_code(&script).clone(),
                        secret,
                    ));
                    vec_vec_unlock.push(unlock_vec);
                }
                return vec_vec_unlock;
            },
        );
    }
}
