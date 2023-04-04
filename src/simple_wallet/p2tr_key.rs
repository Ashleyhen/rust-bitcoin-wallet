use std::{str::FromStr, ops::BitXor};

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


    let info=P2TRInfo::new();

    assert!(tweaked_key_pair
        .to_inner()
        .x_only_public_key()
        .0
        .eq(&info.whatis_tap_tweak( key_pair)
            .to_inner()
            .x_only_public_key()
            .0));

    let sig = secp.sign_schnorr(&message, &tweaked_key_pair.to_inner());
    info.whatis_shnorr(&message,&key_pair.secret_key());

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





struct P2TRInfo {
    secp:Secp256k1<All>
 }

impl P2TRInfo{
    pub fn new()->Self{ P2TRInfo { secp:Secp256k1::new() } }

pub fn how_is_shnorr_verified(secp: &Secp256k1<All>,signature:&Signature,message: &Message, x_only: &XOnlyPublicKey) -> () {
    secp.verify_schnorr(signature, message, x_only).unwrap();
    let random_aux=signature[..32].to_vec();
    let sig=signature[32..].to_vec();


    let e =Self::tagged_hash(&"BIP0340/challenge".to_owned(), vec![&random_aux,&x_only.serialize().to_vec(),&message[..].to_vec()]);


    // PH(R|P|m)+R
    let my_sig=x_only.public_key(bitcoin::secp256k1::Parity::Even).mul_tweak(&secp,&Scalar::from_be_bytes(e).unwrap()).unwrap()
    .combine(&XOnlyPublicKey::from_slice(&random_aux).unwrap().public_key(bitcoin::secp256k1::Parity::Even)).unwrap();

    
    // sG-PH(R|P|m)=R
    let their_sig=SecretKey::from_slice(&sig).unwrap().public_key(&secp);
    assert_eq!(their_sig.serialize(),my_sig.serialize());
    println!("{}: bitcoin rust sig \n{}: my sig value\n",their_sig.serialize().to_hex(),my_sig.serialize().to_hex());
}

pub fn tagged_hash(tag:&str,args:Vec<&Vec<u8>>)->[u8;32]{
    
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
 

// The secret key sk: a 32-byte array
// The message m: a 32-byte array
// Auxiliary random data a: a 32-byte array

// The algorithm Sign(sk, m) is defined as:
// Let d' = int(sk)
// Fail if d' = 0 or d' ≥ n
// Let P = d'⋅G
// Let d = d' if has_even_y(P), otherwise let d = n - d' .
// Let t be the byte-wise xor of bytes(d) and hashBIP0340/aux(a)[11].
// Let rand = hashBIP0340/nonce(t || bytes(P) || m)[12].
// Let k' = int(rand) mod n[13].
// Fail if k' = 0.
// Let R = k'⋅G.

// Let k = k' if has_even_y(R), otherwise let k = n - k' .
// Let e = int(hashBIP0340/challenge(bytes(R) || bytes(P) || m)) mod n.
// Let sig = bytes(R) || bytes((k + ed) mod n).
// If Verify(bytes(P), m, sig) (see below) returns failure, abort[14].
// Return the signature sig.

pub fn whatis_shnorr(&self,message: &Message, secret_key: &SecretKey) -> () {
    let key_pair =secret_key.keypair(&self.secp);
    let auxilary=Scalar::random();
    let sig=self.secp.sign_schnorr_with_aux_rand(&message, &key_pair, &auxilary.to_be_bytes());

    let x_only=secret_key.x_only_public_key(&self.secp).0;
    let d=secret_key.secret_bytes().to_vec();

    let t=d
    .iter()
    .zip(Self::tagged_hash(&"BIP0340/aux".to_owned(),vec![&auxilary.to_be_bytes().to_vec()] ).iter())
    .map(|(&x1, &x2)| x1 ^ x2)
    .collect();

   let rand = Self::tagged_hash("BIP0340/nonce", vec![&t,&x_only.serialize().to_vec(),&message[..].to_vec()]); 
   let k=SecretKey::from_slice(&rand).unwrap();
   let our_r=k.x_only_public_key(&self.secp).0;

   let e=Self::tagged_hash(&"BIP0340/challenge".to_owned(), vec![&our_r.serialize().to_vec(),&x_only.serialize().to_vec(),&message[..].to_vec()]);
   let our_sig=SecretKey::from_slice(&e).unwrap().mul_tweak(&Scalar::from_be_bytes(d.try_into().unwrap()).unwrap()).unwrap()
   .add_tweak(&Scalar::from_be_bytes(k.secret_bytes()).unwrap()).unwrap();

    let mut our_signature=our_r.serialize().to_vec();

    our_signature.extend(our_sig.secret_bytes());

//    println!("{}: bitcoin rust R \n{}: our random R\n",
//         sig.to_hex(),
//         our_signature.to_hex()
//     );

   println!("{}: bitcoin rust sig \n{}: our sig\n",
        SecretKey::from_slice(&sig[32..].to_vec()).unwrap().secret_bytes().to_vec().to_hex(),
        our_sig.secret_bytes().to_hex()
    );




}

pub fn whatis_tap_tweak(&self, key_pair: &KeyPair) -> TweakedKeyPair {
    // q=p+H(P|c)

    let mut engine = TapTweakHash::engine();
    let x_only = key_pair.x_only_public_key().0;
    engine.input(&x_only.serialize());

    // because our tapbranch is none we aren't hashing anything else
    // so our equation looks more like q=p+H(P)

    let tap_tweak_hash = TapTweakHash::from_engine(engine).to_scalar();
    
    // tap_tweak_hash
    let secret_key = key_pair.secret_key();
    let tweak_pair = secret_key
        .add_tweak(&tap_tweak_hash)
        .unwrap()
        .keypair(&self.secp)
        .dangerous_assume_tweaked();
    return tweak_pair;
}
}
