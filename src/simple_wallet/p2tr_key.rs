use std::str::FromStr;

use bitcoin::{
    psbt::{Input, PartiallySignedTransaction, Prevouts},
    schnorr::{TapTweak, TweakedKeyPair},
    secp256k1::{All, Message, Scalar, Secp256k1, SecretKey, schnorr::Signature},
    util::{
        bip32::{ExtendedPrivKey, ExtendedPubKey},
        sighash::SighashCache,
        taproot::{TapTweakHash, TapSighashTag, TapTweakTag},
    },
    Address, KeyPair, PackedLockTime, SchnorrSig, Transaction, TxOut, XOnlyPublicKey,
};
use bitcoin_hashes::{Hash, HashEngine, sha256t::{Tag, self}, hex::ToHex, sha256};
use miniscript::psbt::PsbtExt;
use serde::Serialize;

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
    key_pair.tap_tweak(&secp, None);

    assert!(tweaked_key_pair
        .to_inner()
        .x_only_public_key()
        .0
        .eq(&whatis_tap_tweak(&secp, key_pair)
            .to_inner()
            .x_only_public_key()
            .0));

    let sig = secp.sign_schnorr(&message, &tweaked_key_pair.to_inner());
    
    how_is_shnorr_verified(secp, &sig,&message,&tweaked_key_pair.to_inner().x_only_public_key().0);
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

pub fn how_is_shnorr_verified(secp: &Secp256k1<All>,signature:&Signature,message: &Message, x_only: &XOnlyPublicKey) -> () {
    secp.verify_schnorr(signature, message, x_only).unwrap();
    let random_aux=signature[..32].to_vec();
    let sig=signature[32..].to_vec();


    let e =tagged_hash(&"BIP0340/challenge".to_owned(), vec![&random_aux,&x_only.serialize().to_vec(),&message[..].to_vec()]);


    // PH(R|P|m)+R
    let my_sig=x_only.public_key(bitcoin::secp256k1::Parity::Even).mul_tweak(&secp,&Scalar::from_be_bytes(e).unwrap()).unwrap()
    .combine(&XOnlyPublicKey::from_slice(&random_aux).unwrap().public_key(bitcoin::secp256k1::Parity::Even)).unwrap();

    
    // sG-PH(R|P|m)=R
    let their_sig=SecretKey::from_slice(&sig).unwrap().public_key(&secp);

    println!("{}: bitcoin rust sig \n{}: my sig value\n",their_sig.serialize().to_hex(),my_sig.serialize().to_hex());


}

pub fn tagged_hash(tag:&String,args:Vec<&Vec<u8>>)->[u8;32]{
    
    // SHA256(SHA256(tag) || SHA256(tag) || x).
    ;
    let mut engine_tag =bitcoin::hashes::sha256::Hash::engine();
    engine_tag.input(tag.as_bytes());
    let sha_256_tag=bitcoin::hashes::sha256::Hash::from_engine(engine_tag).into_inner();

    let mut sha_256 =bitcoin::hashes::sha256::Hash::engine();
    sha_256.input(&sha_256_tag.to_vec());
    sha_256.input(&sha_256_tag.to_vec());
    for x in args{
        sha_256.input(&x);
    }
return bitcoin::hashes::sha256::Hash::from_engine(sha_256).into_inner();

}
 
pub fn whatis_shnorr(secp: &Secp256k1<All>,message: &Message, key_pair: &KeyPair) -> () {
    // q=p+H(P|c)

// P=sG
// t= s XOR hash(r)
// r=hash(t|P|m)
// R=rG

// let tag="aux";


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
}

pub fn whatis_tap_tweak(secp: &Secp256k1<All>, key_pair: &KeyPair) -> TweakedKeyPair {
    // q=p+H(P|c)

    let mut engine = TapTweakHash::engine();
    let x_only = key_pair.x_only_public_key().0;
    engine.input(&x_only.serialize());

    // because our tapbranch is none we aren't hashing anything else
    // so our equation looks more like q=p+H(P)


    let tap_tweak_hash = TapTweakHash::from_engine(engine).to_scalar();
    let my_tagged_hash=tagged_hash(&"TapTweak".to_string(), vec![&x_only.serialize().to_vec()].to_vec());

    // println!("{} my tagged hash\n{} there tagged hash \n",my_tagged_hash.to_vec().to_hex(), tap_tweak_hash.to_be_bytes().to_vec().to_hex());
    
    
// tap_tweak_hash
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
