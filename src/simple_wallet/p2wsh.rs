use std::{ops::Add, result, str::FromStr};

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    psbt::{Input, Output, PartiallySignedTransaction},
    secp256k1::{All, Message, Scalar, Secp256k1, SecretKey},
    util::{bip32::KeySource, sighash::SighashCache},
    Address, EcdsaSig, EcdsaSighashType, PackedLockTime, PrivateKey, PublicKey, Script,
    Transaction, TxOut,
};

use bitcoin_hashes::{hex::ToHex, sha256};
use miniscript::{psbt::PsbtExt, ToPublicKey};

use crate::bitcoin_wallet::{constants::NETWORK, input_data::RpcCall};

use super::{p2wpkh::from_seed, Wallet};

pub struct P2WSH<'a, R: RpcCall> {
    secret_key: SecretKey,
    secp: Secp256k1<All>,
    client: &'a R,
}

impl<'a, R> P2WSH<'a, R>
where
    R: RpcCall,
{
    pub fn new(secret_string: &Option<&str>, client: &'a R) -> Self
    where
        R: RpcCall,
    {
        let secp = Secp256k1::new();

        let secret_key = match secret_string {
            Some(sec_str) => SecretKey::from_str(&sec_str).unwrap(),
            None => {
                let secret = SecretKey::from_slice(&Scalar::random().to_be_bytes()).unwrap();
                println!("secret_key: {}", secret.display_secret());
                secret
            }
        };

        P2WSH {
            secret_key,
            secp,
            client,
        }
    }
}

impl<'a, R> P2WSH<'a, R>
where
    R: RpcCall,
{
    pub fn parital_sig(
        &self,
        pub_ks: &Vec<PublicKey>,
        maybe_psbt: Option<PartiallySignedTransaction>,
        send_to: &Box<dyn Fn(u64) -> Vec<TxOut>>,
    ) -> PartiallySignedTransaction {
        let private_key = PrivateKey::new(self.secret_key, NETWORK);

        let tx_in_list = self.client.prev_input();

        let transaction_list = self.client.contract_source();

        let prevouts = transaction_list
            .iter()
            .flat_map(|tx| tx.output.clone())
            .filter(|p| {
                Self::multi_sig_address(pub_ks)
                    .script_pubkey()
                    .eq(&p.script_pubkey)
            })
            .collect::<Vec<TxOut>>();

        let total: u64 = prevouts.iter().map(|tx_out| tx_out.value).sum();

        let out_put = send_to(total - self.client.fee());

        let unsigned_tx = Transaction {
            version: 2,
            lock_time: PackedLockTime(0),
            input: tx_in_list,
            output: out_put.clone(),
        };

        let mut psbt = maybe_psbt.unwrap_or_else(|| {
            PartiallySignedTransaction::from_unsigned_tx(unsigned_tx.clone()).unwrap()
        });

        psbt.inputs = sign_all_unsigned_tx(
            &self.secp,
            &prevouts,
            &unsigned_tx,
            &private_key,
            psbt.inputs,
            pub_ks,
        );

        return psbt;
    }

    pub fn broadcasted(&self, psbt: PartiallySignedTransaction) {
        let tx = psbt
            .finalize(&self.secp)
            .unwrap()
            .extract(&self.secp)
            .unwrap();
        self.client.broadcasts_transacton(&tx)
    }

    pub fn seed_to_pubkey(secret_string: &Option<&str>) -> PublicKey {
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
        return PrivateKey::new(secret, NETWORK).public_key(&secp);
    }

    pub fn multi_sig_address(pub_keys: &Vec<PublicKey>) -> Address {
        return Address::p2wsh(&multi_sig_script(pub_keys), NETWORK);
    }
}

pub fn multi_sig_script(pub_keys: &Vec<PublicKey>) -> Script {
    fn partial_p2wsh_multi_sig<'a>(
        mut iter: impl Iterator<Item = &'a PublicKey>,
        len: i64,
    ) -> Builder {
        match iter.next() {
            Some(pub_k) => partial_p2wsh_multi_sig(iter, len).push_key(&pub_k.to_public_key()),
            None => Builder::new().push_int(len),
        }
    }

    let len = pub_keys.len();
    return partial_p2wsh_multi_sig(pub_keys.iter(), len.try_into().unwrap())
        .push_int(len.try_into().unwrap())
        .push_opcode(all::OP_CHECKMULTISIG)
        .into_script();
}

fn create_output<'a>(total: u64, client: &'a impl RpcCall) -> Vec<TxOut> {
    let send_amt = (total - client.fee()) / 2;
    let out_put = vec![
        TxOut {
            value: send_amt,
            script_pubkey: Address::from_str(
                "bc1pgavdtwzfxf70nkh4r44wtjscf4h39mvwqhpdqkn0r58640y77hhsfn6u43",
            )
            .unwrap()
            .script_pubkey(),
        },
        TxOut {
            value: send_amt,
            script_pubkey: Address::from_str("bc1qlczlmzcq872wfpsyspafcluna6apzcrm348aty")
                .unwrap()
                .script_pubkey(),
        },
    ];
    out_put
}

fn sign_all_unsigned_tx(
    secp: &Secp256k1<All>,
    prevouts: &Vec<TxOut>,
    unsigned_tx: &Transaction,
    private_key: &PrivateKey,
    input: Vec<Input>,
    pub_ks: &Vec<PublicKey>,
) -> Vec<Input> {
    return prevouts
        .iter()
        .enumerate()
        .map(|(index, tx_out)| {
            sign_tx(
                secp,
                index,
                unsigned_tx,
                private_key,
                tx_out,
                input.get(index).cloned(),
                pub_ks,
            )
            .clone()
        })
        .collect();
}

fn sign_tx(
    secp: &Secp256k1<All>,
    index: usize,
    unsigned_tx: &Transaction,
    private_key: &PrivateKey,
    tx_out: &TxOut,
    maybe_input: Option<Input>,
    pub_ks: &Vec<PublicKey>,
) -> Input {
    let hash_ty = EcdsaSighashType::All;
    let witness_script = multi_sig_script(pub_ks).clone();
    let sighash = SighashCache::new(&mut unsigned_tx.clone())
        .segwit_signature_hash(index, &witness_script, tx_out.value, hash_ty)
        .unwrap();

    let message = Message::from_slice(&sighash).unwrap();

    let sig = secp.sign_ecdsa(&message, &private_key.inner);

    let ecdsa_sig = EcdsaSig::sighash_all(sig);

    let mut input = maybe_input.unwrap_or(Input::default());

    input.witness_script = Some(witness_script);

    let pub_key = bitcoin::PublicKey::from_private_key(&secp, private_key);
    input.partial_sigs.insert(pub_key, ecdsa_sig);

    input.witness_utxo = Some(tx_out.clone());

    return input.clone();
}
