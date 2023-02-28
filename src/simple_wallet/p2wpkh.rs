use std::str::FromStr;

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    psbt::{Input, PartiallySignedTransaction},
    secp256k1::{All, Message, Scalar, Secp256k1, SecretKey},
    util::sighash::SighashCache,
    Address, EcdsaSig, EcdsaSighashType, PackedLockTime, PrivateKey, PublicKey, Transaction, TxOut,
};

use miniscript::psbt::PsbtExt;

use crate::bitcoin_wallet::{constants::NETWORK, input_data::RpcCall};

use super::Wallet;

pub struct P2WPKH<'a, R: RpcCall> {
    secret_key: SecretKey,
    secp: Secp256k1<All>,
    client: &'a R,
}
impl<'a, R> Wallet<'a, R> for P2WPKH<'a, R>
where
    R: RpcCall,
{
    fn new(secret_string: Option<&str>, client: &'a R) -> Self
    where
        R: RpcCall,
    {
        let secp = Secp256k1::new();

        let private_key = from_seed(&secret_string);
        let address = Address::p2wpkh(&private_key.public_key(&secp), NETWORK).unwrap();

        let secret_key = match secret_string {
            Some(sec_str) => SecretKey::from_str(&sec_str).unwrap(),
            None => {
                let secret = SecretKey::from_slice(&Scalar::random().to_be_bytes()).unwrap();
                println!("secret_key: {}", secret.display_secret());
                secret
            }
        };
        println!("address {}", address.to_string());
        P2WPKH {
            secret_key,
            secp,
            client,
        }
    }
}

impl<'a, R> P2WPKH<'a, R>
where
    R: RpcCall,
{
    pub fn send(&self, send_to: Box<dyn Fn(u64) -> Vec<TxOut>>) {
        let private_key = PrivateKey::new(self.secret_key, NETWORK);
        let address = Address::p2wpkh(&private_key.public_key(&self.secp), NETWORK).unwrap();

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

        psbt.inputs = sign_all_unsigned_tx(&self.secp, &prevouts, &unsigned_tx, &private_key);

        let transaction = psbt.finalize(&self.secp).unwrap().extract_tx();
        self.client.broadcasts_transacton(&transaction);
    }
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
) -> Vec<Input> {
    return prevouts
        .iter()
        .enumerate()
        .map(|(index, tx_out)| sign_tx(secp, index, unsigned_tx, private_key, tx_out).clone())
        .collect();
}

fn sign_tx(
    secp: &Secp256k1<All>,
    index: usize,
    unsigned_tx: &Transaction,
    private_key: &PrivateKey,
    tx_out: &TxOut,
) -> Input {
    let script_pubkey = Builder::new()
        .push_opcode(all::OP_DUP)
        .push_opcode(all::OP_HASH160)
        .push_slice(&tx_out.script_pubkey[2..])
        .push_opcode(all::OP_EQUALVERIFY)
        .push_opcode(all::OP_CHECKSIG)
        .into_script();
    let hash_ty = EcdsaSighashType::All;
    let sighash = SighashCache::new(&mut unsigned_tx.clone())
        .segwit_signature_hash(index, &script_pubkey, tx_out.value, hash_ty)
        .unwrap();

    let message = Message::from_slice(&sighash).unwrap();

    let sig = secp.sign_ecdsa(&message, &private_key.inner);

    let ecdsa_sig = EcdsaSig { sig, hash_ty };

    let mut input = Input::default();

    input.witness_script = Some(tx_out.script_pubkey.clone());

    input
        .partial_sigs
        .insert(PublicKey::from_private_key(&secp, private_key), ecdsa_sig);

    input.witness_utxo = Some(tx_out.clone());

    return input;
}

pub fn from_seed(secret_string: &Option<&str>) -> PrivateKey {
    let scalar = Scalar::random();
    let secret = match secret_string {
        Some(sec_str) => SecretKey::from_str(&sec_str).unwrap(),
        None => {
            let secret_key = SecretKey::from_slice(&scalar.to_be_bytes()).unwrap();
            println!("secret_key: {}", secret_key.display_secret());
            secret_key
        }
    };
    return PrivateKey::new(secret, NETWORK);
}

pub fn from_prv(secret_string: &Option<&str>) -> PrivateKey {
    let scalar = Scalar::random();
    return match secret_string {
        Some(sec_str) => PrivateKey::from_str(&sec_str).unwrap(),
        None => {
            let secret_key = SecretKey::from_slice(&scalar.to_be_bytes()).unwrap();
            println!("secret_key: {}", secret_key.display_secret());
            PrivateKey::new(secret_key, NETWORK)
        }
    };
}
