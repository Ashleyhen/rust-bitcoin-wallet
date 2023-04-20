use bitcoin::{blockdata::{script::Builder, opcodes::all::{OP_CHECKSIGADD, OP_CHECKSIG, OP_CHECKSIGVERIFY, OP_EQUAL, OP_NUMEQUAL}},  Script, util::taproot::TaprootBuilder, psbt::TapTree, secp256k1::PublicKey, XOnlyPublicKey, Address };
use std::{str::FromStr, ops::Add, borrow::BorrowMut};

use bitcoin::{
    blockdata::{opcodes::all, },
    psbt::{Input, Output, PartiallySignedTransaction, Prevouts},
    secp256k1::{All, Message, Scalar, Secp256k1, SecretKey},
    util::{
        bip32::{DerivationPath, Fingerprint},
        sighash::{ScriptPath, SighashCache},
        taproot::{LeafVersion, TapLeafHash },
    },
     KeyPair, PackedLockTime, SchnorrSig, SchnorrSighashType,  Transaction, TxIn,
    TxOut, Witness, 
};
use bitcoin_hashes::{hex::{FromHex, ToHex}, Hash};

use crate::bitcoin_wallet::{ input_data::RpcCall};
use crate::bitcoin_wallet::constants::{secp, NETWORK};
// https://github.com/ElementsProject/elements-miniscript/blob/dc1f5ee748191086095a2c31284161a917174494/src/miniscript/astelem.rs
pub fn unlock_bond(host:&XOnlyPublicKey,client:&XOnlyPublicKey)->Script{
    Builder::new()
        .push_x_only_key(&host)
        .push_opcode(OP_CHECKSIG)
        .push_x_only_key(&client)
        .push_opcode(OP_CHECKSIGADD)
        .push_int(2)
        .push_opcode(OP_NUMEQUAL)
        .into_script()
}

fn unlock_support(support_key:&XOnlyPublicKey)->Script{
    Builder::new()
        .push_x_only_key(support_key)
        .push_opcode(all::OP_CHECKSIG).into_script()
}

pub fn create_address(host:&XOnlyPublicKey,client:&XOnlyPublicKey,support_key:&XOnlyPublicKey)->Output{

    let bond_script=unlock_bond(host,client);
    let support_script=unlock_support(support_key);

    let combined_scripts=vec![
        (1,bond_script.clone()),
        (1,support_script.clone())
        ];

    let tap_tree=
        TapTree::try_from(TaprootBuilder::with_huffman_tree(combined_scripts).unwrap()).unwrap();

    let tap_root_spend_info =tap_tree.clone().into_builder()
    .finalize(&secp(), *support_key).unwrap();

    let script=Script::new_v1_p2tr_tweaked(tap_root_spend_info.output_key());
    // let script=Script::new_v1_p2tr(&secp(),*support_key,tap_root_spend_info.merkle_root());
    
    let mut output=Output::default();

    output.tap_tree = Some(tap_tree);
    output.tap_internal_key = Some(*support_key);
    output.witness_script = Some(script);
    output.redeem_script=Some(bond_script.clone());
    return output;
}



pub struct Bisq<'a, R: RpcCall> {
    secret_key: SecretKey,
    client: &'a R,
}

impl<'a, R> Bisq<'a, R>
where
    R: RpcCall,
{
    pub fn new(secret_string: &str, client: &'a R) -> Bisq<'a, R> {
        let secret_key = SecretKey::from_str(&secret_string).unwrap();
        return Self {
            secret_key,
            client,
        };
    }
}

impl<'a, R> Bisq<'a, R>
where
    R: RpcCall,
{
    pub fn sign(&self, output: &Output,send_to: Box<dyn Fn(u64) -> Vec<TxOut>>)->PartiallySignedTransaction {
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

        let mut psbt = PartiallySignedTransaction::from_unsigned_tx(unsigned_tx.clone()).unwrap();

        psbt.inputs =
            self.sign_all_unsigned_tx( &prevouts, &unsigned_tx, &output);

        psbt.outputs = vec![output.clone()];

        return psbt;
    }

    fn sign_all_unsigned_tx(
        &self,
        prevouts: &Vec<TxOut>,
        unsigned_tx: &Transaction,
        output: &Output,
    ) -> Vec<Input> {
        return prevouts
            .iter()
            .enumerate()
            .map(|(index, tx_out)| {
                self.sign_tx(
                    index,
                    unsigned_tx,
                    &prevouts,
                    tx_out,
                    output,
                )
                .clone()
            })
            .collect();
    }

    fn sign_tx(
        &self,
        index: usize,
        unsigned_tx: &Transaction,
        prevouts: &Vec<TxOut>,
        tx_out: &TxOut,
        output: &Output,
    ) -> Input {

        let tap_info = output
            .clone()
            .tap_tree
            .unwrap()
            .clone()
            .into_builder()
            .finalize(&secp(), output.tap_internal_key.unwrap())
            .unwrap();

        
        
        let target_script=output.redeem_script.clone().unwrap();
        
        let control = tap_info.control_block(
            &(target_script.clone(),LeafVersion::TapScript)
            
        );

        let verify = control.as_ref().unwrap().verify_taproot_commitment(
            &secp(),
            tap_info.output_key().to_inner(),
            &target_script,
        );

        if (!verify) {
            panic!("invalid block {:#?}", control.unwrap());
        }

        let sighash = SighashCache::new(unsigned_tx)
            .taproot_script_spend_signature_hash(
                index,
                &Prevouts::All(&prevouts),
                ScriptPath::with_defaults(&output.redeem_script.clone().unwrap()),
                SchnorrSighashType::AllPlusAnyoneCanPay,
            )
            .unwrap();

        let message = Message::from_slice(&sighash).unwrap();

        let sig = secp().sign_schnorr(&message, &self.secret_key.keypair(&secp()));

        let schnorr_sig = SchnorrSig {
            sig,
            hash_ty: bitcoin::SchnorrSighashType::AllPlusAnyoneCanPay,
        };

        let tap_leaf_hash = TapLeafHash::from_script(&output.redeem_script.clone().unwrap(), LeafVersion::TapScript);

        let mut input = Input::default();

        input.witness_script = Some(tx_out.script_pubkey.clone());

        input.witness_utxo = Some(tx_out.clone());

        input.tap_merkle_root = tap_info.merkle_root();

        input.redeem_script=output.clone().redeem_script;

        input.tap_scripts.insert(
            control.unwrap(),
            (output.clone().redeem_script.unwrap(), LeafVersion::TapScript),
        );

        let x_only = &self.secret_key.x_only_public_key(&secp()).0;

        input
            .tap_script_sigs
            .insert((x_only.clone(), tap_leaf_hash), schnorr_sig);

        return input;

    }

    
    pub fn finalize_script(&self, psbt: PartiallySignedTransaction, should_broad_cast:bool) -> Transaction {
        let tx = psbt.clone().extract_tx().clone();
        let tx_in = psbt
            .inputs
            .iter()
            .map(|input| {
                let mut witness = Witness::new();

                input.tap_script_sigs.iter().for_each(|sig| {
                    let shnor = sig.1;
                    witness.push(shnor.to_vec());
                    dbg!(shnor.sig.to_hex());
                });

                input.tap_scripts.iter().for_each(|control| {
                    witness.push(control.1.0.as_bytes());
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

        let tx=Transaction {
            version: tx.version,
            lock_time: tx.lock_time,
            input: tx_in,
            output: tx.output,
        };
        if(should_broad_cast){
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


pub fn merge_psbt(our_psbt: &mut PartiallySignedTransaction,their_psbt: &PartiallySignedTransaction)-> PartiallySignedTransaction{
        
        our_psbt.inputs=their_psbt.inputs.iter().zip(our_psbt.inputs.iter()).map(|( our_input,their_input)|{
            let our_sig =&mut their_input.clone().tap_script_sigs;
            let their_sig=&mut our_input.clone().tap_script_sigs;
            our_sig.append(their_sig);
            
        let mut input= our_input.clone();
        input.tap_script_sigs=our_sig.clone();
        return input;
        }).collect::<Vec<Input>>();

        return our_psbt.clone();
    }

//  "Script(OP_SHA256 OP_PUSHBYTES_32 6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5333 OP_EQUALVERIFY OP_PUSHBYTES_32 4edfcf9dfe6c0b5c83d1ab3f78d1b39a46ebac6798e08e19761f5ed89ec83c10 OP_CHECKSIG)"
// "Script(OP_SHA256 OP_PUSHBYTES_32 6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5333 OP_EQUALVERIFY OP_PUSHBYTES_32 4edfcf9dfe6c0b5c83d1ab3f78d1b39a46ebac6798e08e19761f5ed89ec83c10 OP_CHECKSIG)"
