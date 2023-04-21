use bitcoin::{
    blockdata::{
        opcodes::all::{OP_CHECKSIG, OP_CHECKSIGADD, OP_CHECKSIGVERIFY, OP_EQUAL, OP_NUMEQUAL},
        script::Builder,
    },
    psbt::TapTree,
    secp256k1::PublicKey,
    util::taproot::TaprootBuilder,
    Address, Script, XOnlyPublicKey,
};
use miniscript::ScriptContext;
use std::{borrow::BorrowMut, ops::Add, str::FromStr};

use bitcoin::{
    blockdata::opcodes::all,
    psbt::{Input, Output, PartiallySignedTransaction, Prevouts},
    secp256k1::{All, Message, Scalar, Secp256k1, SecretKey},
    util::{
        bip32::{DerivationPath, Fingerprint},
        sighash::{ScriptPath, SighashCache},
        taproot::{LeafVersion, TapLeafHash},
    },
    KeyPair, PackedLockTime, SchnorrSig, SchnorrSighashType, Transaction, TxIn, TxOut, Witness,
};
use bitcoin_hashes::{
    hex::{FromHex, ToHex},
    Hash,
};

use crate::bitcoin_wallet::constants::{secp, NETWORK};
use crate::bitcoin_wallet::input_data::RpcCall;

use super::{bisq_key, bisq_script, ISigner};
// https://github.com/ElementsProject/elements-miniscript/blob/dc1f5ee748191086095a2c31284161a917174494/src/miniscript/astelem.rs
pub fn unlock_bond(host: &XOnlyPublicKey, client: &XOnlyPublicKey) -> Script {
    Builder::new()
        .push_x_only_key(&host)
        .push_opcode(OP_CHECKSIG)
        .push_x_only_key(&client)
        .push_opcode(OP_CHECKSIGADD)
        .push_int(2)
        .push_opcode(OP_NUMEQUAL)
        .into_script()
}

fn unlock_support(support_key: &XOnlyPublicKey) -> Script {
    Builder::new()
        .push_x_only_key(support_key)
        .push_opcode(all::OP_CHECKSIG)
        .into_script()
}

pub fn create_address(
    host: &XOnlyPublicKey,
    client: &XOnlyPublicKey,
    support_key: &XOnlyPublicKey,
) -> Output {
    let bond_script = unlock_bond(host, client);
    let support_script = unlock_support(support_key);

    let combined_scripts = vec![(1, bond_script.clone()), (1, support_script.clone())];

    let tap_tree =
        TapTree::try_from(TaprootBuilder::with_huffman_tree(combined_scripts).unwrap()).unwrap();

    let tap_root_spend_info = tap_tree
        .clone()
        .into_builder()
        .finalize(&secp(), *support_key)
        .unwrap();

    let script = Script::new_v1_p2tr_tweaked(tap_root_spend_info.output_key());

    let mut output = Output::default();

    output.tap_tree = Some(tap_tree);
    output.tap_internal_key = Some(*support_key);
    output.witness_script = Some(script);
    return output;
}

pub struct Bisq<'a, R: RpcCall, I: ISigner> {
    secret_key: SecretKey,
    client: &'a R,
    signer: I,
}

impl<'a, R, I> Bisq<'a, R, I>
where
    R: RpcCall,
    I: ISigner,
{
    pub fn new(secret_string: &str, client: &'a R, signer: I) -> Bisq<'a, R, I> {
        let secret_key = SecretKey::from_str(&secret_string).unwrap();
        return Self {
            secret_key,
            client,
            signer,
        };
    }
}

impl<'a, R, I> Bisq<'a, R, I>
where
    R: RpcCall,
    I: ISigner,
{
    pub fn sign(
        &self,
        output: &Output,
        maybe_psbt: Option<PartiallySignedTransaction>,
        send_to: Box<dyn Fn(u64) -> Vec<TxOut>>,
    ) -> PartiallySignedTransaction {

        let tx_in_list = self.client.prev_input();

        let transaction_list = self.client.contract_source();

        let prevouts = transaction_list
            .iter()
            .flat_map(|tx| tx.output.clone())
            .filter(|p| output.clone().witness_script.unwrap().eq(&p.script_pubkey))
            .collect::<Vec<TxOut>>();

        let total: u64 = prevouts.iter().map(|tx_out| tx_out.value).sum();

        let tx_out = send_to(total - self.client.fee());

        let unsigned_tx = Transaction {
            version: 2,
            lock_time: PackedLockTime(0),
            input: tx_in_list,
            output: tx_out,
        };

        let mut psbt = maybe_psbt.unwrap_or_else(|| {
            PartiallySignedTransaction::from_unsigned_tx(unsigned_tx.clone()).unwrap()
        });

        psbt.inputs = self.signer.sign_all_unsigned_tx(
            &self.secret_key,
            &prevouts,
            &unsigned_tx,
        );

        return psbt;
    }

    pub fn finalize_script(
        &self,
        psbt: PartiallySignedTransaction,
        should_broad_cast: bool,
    ) -> Transaction {
        let tx = psbt.clone().extract_tx().clone();
        let tx_in = psbt
            .inputs
            .iter()
            .map(|input| {
                let mut witness = Witness::new();

                input.tap_script_sigs.iter().for_each(|sig| {
                    let shnor = sig.1;
                    witness.push(shnor.to_vec());
                });

                input.tap_scripts.iter().for_each(|control| {
                    witness.push(control.1 .0.as_bytes());
                    witness.push(control.0.serialize());
                });
                return witness;
            })
            .zip(tx.input)
            .map(|(witness, tx_input)| {
                return TxIn {
                    previous_output: tx_input.previous_output,
                    script_sig: tx_input.script_sig,
                    sequence: tx_input.sequence,
                    witness,
                };
            })
            .collect::<Vec<TxIn>>();

        let tx = Transaction {
            version: tx.version,
            lock_time: tx.lock_time,
            input: tx_in,
            output: tx.output,
        };
        if (should_broad_cast) {
            self.client.broadcasts_transacton(&tx);
        }

        return tx;
    }
}
pub fn seed_to_xonly(secret_string: &Option<&str>) -> bitcoin::XOnlyPublicKey {
    let scalar = Scalar::random();
    let secp = Secp256k1::new();
    let secret = match secret_string {
        Some(sec_str) => SecretKey::from_str(&sec_str).unwrap(),
        None => {
            let secret_key = SecretKey::from_slice(&scalar.to_be_bytes()).unwrap();
            println!("secret_key: {}", secret_key.display_secret());
            secret_key
        }
    };
    return secret.public_key(&secp).x_only_public_key().0;
}

//  "Script(OP_SHA256 OP_PUSHBYTES_32 6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5333 OP_EQUALVERIFY OP_PUSHBYTES_32 4edfcf9dfe6c0b5c83d1ab3f78d1b39a46ebac6798e08e19761f5ed89ec83c10 OP_CHECKSIG)"
// "Script(OP_SHA256 OP_PUSHBYTES_32 6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5333 OP_EQUALVERIFY OP_PUSHBYTES_32 4edfcf9dfe6c0b5c83d1ab3f78d1b39a46ebac6798e08e19761f5ed89ec83c10 OP_CHECKSIG)"
