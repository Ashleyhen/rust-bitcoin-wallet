use std::{str::FromStr};

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    psbt::{Input, Output, PartiallySignedTransaction, Prevouts, TapTree},
    secp256k1::{All, Message, Scalar, Secp256k1, SecretKey},
    util::{
        bip32::{DerivationPath, Fingerprint},
        sighash::{ScriptPath, SighashCache},
        taproot::{LeafVersion, TapLeafHash, TaprootBuilder},
    },
    Address, KeyPair, PackedLockTime, SchnorrSig, SchnorrSighashType, Script, Transaction, TxIn,
    TxOut, Witness, XOnlyPublicKey,
};
use bitcoin_hashes::{
    hex::{FromHex},
    Hash,
};

use crate::bitcoin_wallet::{
    constants::NETWORK,
    input_data::{ RpcCall},
};

pub struct P2TRS<'a, R: RpcCall> {
    secret_key: SecretKey,
    secp: Secp256k1<All>,
    image: String,
    client: &'a R,
}

impl<'a, R> P2TRS<'a, R>
where
    R: RpcCall,
{
    pub fn new(secret_string: &str, image: &str, client: &'a R) -> P2TRS<'a, R> {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_str(&secret_string).unwrap();

        return Self {
            secret_key,
            secp,
            image: image.to_string(),
            client,
        };
    }
}

impl<'a, R> P2TRS<'a, R>
where
    R: RpcCall,
{
    pub fn sign(&self, output: &Output, send_to: Box<dyn Fn(u64) -> Vec<TxOut>>) {
        let tx_in_list = self.client.prev_input();

        let transaction_list = self.client.contract_source();

        let key_pair = KeyPair::from_secret_key(&self.secp, &self.secret_key);

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
            self.sign_all_unsigned_tx(&self.secp, &prevouts, &unsigned_tx, &key_pair, &output);

        psbt.outputs = vec![output.clone()];

        let tx = self.finialize_script(psbt);

        self.client.broadcasts_transacton(&tx);
    }

    fn sign_all_unsigned_tx(
        &self,
        secp: &Secp256k1<All>,
        prevouts: &Vec<TxOut>,
        unsigned_tx: &Transaction,
        key_pair: &KeyPair,
        output: &Output,
    ) -> Vec<Input> {
        return prevouts
            .iter()
            .enumerate()
            .map(|(index, tx_out)| {
                self.sign_tx(
                    secp,
                    index,
                    unsigned_tx,
                    &prevouts,
                    key_pair,
                    tx_out,
                    output,
                )
                .clone()
            })
            .collect();
    }

    fn sign_tx(
        &self,
        secp: &Secp256k1<All>,
        index: usize,
        unsigned_tx: &Transaction,
        prevouts: &Vec<TxOut>,
        key_pair: &KeyPair,
        tx_out: &TxOut,
        output: &Output,
    ) -> Input {
        let tap_info = output
            .clone()
            .tap_tree
            .unwrap()
            .clone()
            .into_builder()
            .finalize(&secp, output.tap_internal_key.unwrap())
            .unwrap();
        let x_only = &self.secret_key.x_only_public_key(&secp).0;

        let bob_script = bob_scripts(x_only, &preimage(&self.image));
        let control = tap_info.control_block(&(bob_script.clone(), LeafVersion::TapScript));

        let verify = control.as_ref().unwrap().verify_taproot_commitment(
            &secp,
            tap_info.output_key().to_inner(),
            &bob_script,
        );

        println!("is this control block valid {}", verify);

        let sighash = SighashCache::new(unsigned_tx)
            .taproot_script_spend_signature_hash(
                index,
                &Prevouts::All(&prevouts),
                ScriptPath::with_defaults(&bob_script),
                SchnorrSighashType::AllPlusAnyoneCanPay,
            )
            .unwrap();

        let message = Message::from_slice(&sighash).unwrap();

        let sig = secp.sign_schnorr_no_aux_rand(&message, &key_pair);

        let schnorr_sig = SchnorrSig {
            sig,
            hash_ty: bitcoin::SchnorrSighashType::AllPlusAnyoneCanPay,
        };

        let tap_leaf_hash = TapLeafHash::from_script(&bob_script, LeafVersion::TapScript);

        let mut input = Input::default();

        input.witness_script = Some(tx_out.script_pubkey.clone());

        input.witness_utxo = Some(tx_out.clone());

        input.tap_merkle_root = tap_info.merkle_root();
        input.tap_scripts.insert(
            control.unwrap(),
            (bob_script.clone(), LeafVersion::TapScript),
        );
        input
            .tap_script_sigs
            .insert((x_only.clone(), tap_leaf_hash), schnorr_sig);

        return input;
    }

    pub fn finialize_script(&self, psbt: PartiallySignedTransaction) -> Transaction {
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

                witness.push(Vec::from_hex(&self.image).unwrap());

                let bob_script = bob_scripts(
                    &self.secret_key.x_only_public_key(&self.secp).0,
                    &preimage(&self.image),
                );
                witness.push(bob_script.as_bytes());

                input.tap_scripts.iter().for_each(|control| {
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

        return Transaction {
            version: tx.version,
            lock_time: tx.lock_time,
            input: tx_in,
            output: tx.output,
        };
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

pub fn preimage(image: &str) -> Vec<u8> {
    return bitcoin_hashes::sha256::Hash::hash(&Vec::from_hex(image).unwrap()).to_vec();
}
pub fn bob_scripts(x_only: &XOnlyPublicKey, preimage: &[u8]) -> Script {
    let bob_script = Builder::new()
        .push_opcode(all::OP_SHA256)
        .push_slice(&preimage)
        .push_opcode(all::OP_EQUALVERIFY)
        .push_x_only_key(&x_only)
        .push_opcode(all::OP_CHECKSIG)
        .into_script();
    return bob_script;
}

//  Script(OP_PUSHBYTES_2 9000 OP_CSV OP_DROP OP_PUSHBYTES_32 9997a497d964fc1a62885b05a51166a65a90df00492c8d7cf61d6accf54803be OP_CHECKSIG)
pub fn alice_script(x_only: &XOnlyPublicKey) -> Script {
    return Builder::new()
        .push_int(0x0090)
        .push_opcode(all::OP_CSV)
        .push_opcode(all::OP_DROP)
        .push_x_only_key(&x_only)
        .push_opcode(all::OP_CHECKSIG)
        .into_script();
}

pub fn create_address(
    alice_x_only: XOnlyPublicKey,
    bob_x_only: XOnlyPublicKey,
    preimage: Vec<u8>,
) -> Output {
    let mut output = Output::default();
    let secp = Secp256k1::new();
    let alice = alice_script(&alice_x_only);
    let bob = bob_scripts(&bob_x_only, &preimage);
    let combined_script = vec![(1, bob.clone()), (1, alice.clone())];
    let builder = TaprootBuilder::with_huffman_tree(combined_script).unwrap();
    let tap_tree = Some(TapTree::try_from(builder).unwrap());
    let internal = KeyPair::from_seckey_slice(&secp, &preimage)
        .unwrap()
        .x_only_public_key()
        .0;
    let tap_info = tap_tree
        .clone()
        .unwrap()
        .into_builder()
        .finalize(&secp, internal)
        .unwrap();
    let merkle_root = tap_info.merkle_root();
    let address = Address::p2tr(&secp, internal, merkle_root, NETWORK);

    println!("address {}", address.to_string());
    output.tap_tree = tap_tree;
    output.tap_internal_key = Some(internal);
    output.witness_script = Some(address.script_pubkey());

    let alice_leaf_hash = vec![TapLeafHash::from_script(&alice, LeafVersion::TapScript)];
    output.tap_key_origins.insert(
        alice_x_only,
        (
            alice_leaf_hash,
            (Fingerprint::default(), DerivationPath::default()),
        ),
    );

    let bob_leaf_hash = vec![TapLeafHash::from_script(&bob, LeafVersion::TapScript)];
    output.tap_key_origins.insert(
        bob_x_only,
        (
            bob_leaf_hash,
            (Fingerprint::default(), DerivationPath::default()),
        ),
    );

    return output;
}

//  "Script(OP_SHA256 OP_PUSHBYTES_32 6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5333 OP_EQUALVERIFY OP_PUSHBYTES_32 4edfcf9dfe6c0b5c83d1ab3f78d1b39a46ebac6798e08e19761f5ed89ec83c10 OP_CHECKSIG)"
// "Script(OP_SHA256 OP_PUSHBYTES_32 6c60f404f8167a38fc70eaf8aa17ac351023bef86bcb9d1086a19afe95bd5333 OP_EQUALVERIFY OP_PUSHBYTES_32 4edfcf9dfe6c0b5c83d1ab3f78d1b39a46ebac6798e08e19761f5ed89ec83c10 OP_CHECKSIG)"