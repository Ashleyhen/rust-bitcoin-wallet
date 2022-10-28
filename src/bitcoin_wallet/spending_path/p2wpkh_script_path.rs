use bitcoin::{
    blockdata::{opcodes, script::Builder},
    hashes::hex::ToHex,
    secp256k1::{All, Secp256k1},
    util::bip32::ExtendedPrivKey,
    Script, Transaction,
};

use crate::bitcoin_wallet::script_services::{
    input_service::sign_segwit_v0, psbt_factory::UnlockFn,
};

pub struct P2wpkh {
    secp: Secp256k1<All>,
}

impl P2wpkh {
    pub fn input_factory<'a>(
        &'a self,
        ext_prv: ExtendedPrivKey,
    ) -> Box<dyn Fn(Vec<Transaction>, Transaction) -> Vec<UnlockFn<'a>> + 'a> {
        return Box::new(
            move |previous_list: Vec<Transaction>, current: Transaction| {
                let pubkey = bitcoin::PublicKey::from_private_key(&self.secp, &ext_prv.to_priv());

                let script = Script::new_v0_p2wpkh(&pubkey.wpubkey_hash().unwrap());
                let mut unlock_vec: Vec<UnlockFn> = vec![];
                for (input_index, prev) in previous_list.iter().enumerate() {
                    let tx_out = prev
                        .output
                        .iter()
                        .find(|t| t.script_pubkey.eq(&script))
                        .expect("missing expected witness");
                    unlock_vec.push(sign_segwit_v0(
                        &self.secp,
                        current.clone(),
                        tx_out.clone(),
                        input_index,
                        p2wpkh_script_code(&script).clone(),
                        ext_prv,
                    ));
                }
                return unlock_vec;
            },
        );
    }
}

fn p2wpkh_script_code(script: &Script) -> Script {
    Builder::new()
        .push_opcode(opcodes::all::OP_DUP)
        .push_opcode(opcodes::all::OP_HASH160)
        .push_slice(&script[2..])
        .push_opcode(opcodes::all::OP_EQUALVERIFY)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .into_script()
}
