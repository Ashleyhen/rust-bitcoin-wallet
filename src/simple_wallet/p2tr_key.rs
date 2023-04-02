use std::str::FromStr;

use bitcoin::{
    psbt::{Input, PartiallySignedTransaction, Prevouts},
    schnorr::{TapTweak, TweakedKeyPair},
    secp256k1::{All, Message, Scalar, Secp256k1, SecretKey},
    util::{
        bip32::{ExtendedPrivKey, ExtendedPubKey},
        sighash::SighashCache,
        taproot::TapTweakHash,
    },
    Address, KeyPair, PackedLockTime, SchnorrSig, Transaction, TxOut,
};
use bitcoin_hashes::{Hash, HashEngine};
use miniscript::psbt::PsbtExt;

use crate::bitcoin_wallet::{constants::NETWORK, input_data::RpcCall};

use super::Wallet;
pub struct P2TR<'a, R: RpcCall> {
    secret_key: SecretKey,
    secp: Secp256k1<All>,
    client: &'a R,
}

impl<'a, R> Wallet<'a, R> for P2TR<'a, R>
where
    R: RpcCall,
{
    fn new(secret_string: Option<&str>, client: &'a R) -> P2TR<'a, R> {
        let secp = Secp256k1::new();
        let scalar = Scalar::random();
        let secret_key = match secret_string {
            Some(sec_str) => SecretKey::from_str(&sec_str).unwrap(),
            None => {
                let secret = SecretKey::from_slice(&scalar.to_be_bytes()).unwrap();
                println!("secret_key: {}", secret.display_secret());
                secret
            }
        };

        let key_pair = KeyPair::from_secret_key(&secp, &secret_key);

        let (x_only, _) = key_pair.x_only_public_key();

        let address = Address::p2tr(&secp, x_only, None, NETWORK);

        println!("address {}", address.to_string());

        let ext_pub = ExtendedPubKey::from_priv(
            &secp,
            &ExtendedPrivKey::new_master(NETWORK, &secret_key.secret_bytes()).unwrap(),
        );

        println!("xpub {}", ext_pub.to_string());

        return Self {
            secret_key,
            secp,
            client,
        };
    }
}

impl<'a, R> P2TR<'a, R>
where
    R: RpcCall,
{
    pub fn send(&self, send_to: Box<dyn Fn(u64) -> Vec<TxOut>>) {
        let key_pair = KeyPair::from_secret_key(&self.secp, &self.secret_key);

        let (x_only, _) = key_pair.x_only_public_key();

        let address = Address::p2tr(&self.secp, x_only, None, NETWORK);

        let tx_in_list = self.client.prev_input();

        let transaction_list = self.client.contract_source();

        let prevouts = transaction_list
            .iter()
            .flat_map(|tx| tx.output.clone())
            .filter(|p| address.script_pubkey().eq(&p.script_pubkey))
            .collect::<Vec<TxOut>>();

        let total: u64 = prevouts.iter().map(|tx_out| tx_out.value).sum();

        let out_put = send_to(total - self.client.fee());

        let unsigned_tx = Transaction {
            version: 2,
            lock_time: PackedLockTime(0),
            input: tx_in_list,
            output: out_put,
        };

        let mut psbt = PartiallySignedTransaction::from_unsigned_tx(unsigned_tx.clone()).unwrap();

        psbt.inputs = sign_all_unsigned_tx(&self.secp, &prevouts, &unsigned_tx, &key_pair);

        let tx = psbt.finalize(&self.secp).unwrap().extract_tx();

        self.client.broadcasts_transacton(&tx);
    }
}

fn sign_all_unsigned_tx(
    secp: &Secp256k1<All>,
    prevouts: &Vec<TxOut>,
    unsigned_tx: &Transaction,
    key_pair: &KeyPair,
) -> Vec<Input> {
    return prevouts
        .iter()
        .enumerate()
        .map(|(index, tx_out)| {
            let message = create_message(index, unsigned_tx, &prevouts);
            sign_tx(secp, message, key_pair, tx_out).clone()
        })
        .collect();
}

fn sign_tx(secp: &Secp256k1<All>, message: Message, key_pair: &KeyPair, tx_out: &TxOut) -> Input {
    let tweaked_key_pair = key_pair.tap_tweak(&secp, None);

    assert!(tweaked_key_pair
        .to_inner()
        .x_only_public_key()
        .0
        .eq(&whatis_tap_tweak(&secp, key_pair)
            .to_inner()
            .x_only_public_key()
            .0));

    let sig = secp.sign_schnorr(&message, &tweaked_key_pair.to_inner());

    let schnorr_sig = SchnorrSig {
        sig,
        hash_ty: bitcoin::SchnorrSighashType::AllPlusAnyoneCanPay,
    };

    let mut input = Input::default();

    input.witness_script = Some(tx_out.script_pubkey.clone());

    input.tap_key_sig = Some(schnorr_sig);

    input.witness_utxo = Some(tx_out.clone());

    return input;
}

pub fn whatis_tap_tweak(secp: &Secp256k1<All>, key_pair: &KeyPair) -> TweakedKeyPair {
    // q=p+H(P|c)

    let mut engine = TapTweakHash::engine();
    let x_only = key_pair.x_only_public_key().0;
    engine.input(&x_only.serialize());

    // because our tapbranch is none we aren't hashing anything else
    // so our equation looks more like q=p+H(P)

    let tap_tweak_hash = TapTweakHash::from_engine(engine).to_scalar();
    let secret_key = key_pair.secret_key();
    let tweak_pair = secret_key
        .add_tweak(&tap_tweak_hash)
        .unwrap()
        .keypair(&secp)
        .dangerous_assume_tweaked();
    return tweak_pair;
}

pub fn create_message(index: usize, unsigned_tx: &Transaction, prevouts: &Vec<TxOut>) -> Message {
    let sighash = SighashCache::new(&mut unsigned_tx.clone())
        .taproot_key_spend_signature_hash(
            index,
            &Prevouts::All(&prevouts),
            bitcoin::SchnorrSighashType::AllPlusAnyoneCanPay,
        )
        .unwrap();
    let message = Message::from_slice(&sighash).unwrap();
    return message;
}
