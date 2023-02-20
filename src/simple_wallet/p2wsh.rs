use std::{ops::Add, result, str::FromStr};

use bitcoin::{
    blockdata::{opcodes::all, script::Builder},
    psbt::{Input, Output, PartiallySignedTransaction},
    secp256k1::{All, Message, Scalar, Secp256k1, SecretKey},
    util::{bip32::KeySource, sighash::SighashCache},
    Address, EcdsaSig, EcdsaSighashType, PackedLockTime, PrivateKey, PublicKey, Script,
    Transaction, TxOut,
};

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
        address: &Address,
        maybe_psbt: Option<PartiallySignedTransaction>,
    ) -> PartiallySignedTransaction {
        let private_key = PrivateKey::new(self.secret_key, NETWORK);

        let tx_in_list = self.client.prev_input();

        let transaction_list = self.client.contract_source();

        let prevouts = transaction_list
            .iter()
            .flat_map(|tx| tx.output.clone())
            .filter(|p| address.script_pubkey().eq(&p.script_pubkey))
            .collect::<Vec<TxOut>>();

        let total: u64 = prevouts.iter().map(|tx_out| tx_out.value).sum();

        let out_put = create_output(total, self.client);

        let unsigned_tx = Transaction {
            version: 2,
            lock_time: PackedLockTime(0),
            input: tx_in_list,
            output: out_put.clone(),
        };
        let mut psbt = maybe_psbt.unwrap_or_else(|| {
            PartiallySignedTransaction::from_unsigned_tx(unsigned_tx.clone()).unwrap()
        });

        // let mut input = psbt
        //     .inputs
        //     .iter()
        //     .cloned()
        //     .find(|input| {
        //         if input.witness_script.is_some() {
        //             return input
        //                 .witness_script
        //                 .as_ref()
        //                 .unwrap()
        //                 .script_hash()
        //                 .eq(&out_put[0].script_pubkey.script_hash());
        //         } else {
        //             return false;
        //         };
        //     })
        //     .unwrap_or_else(|| Input::default())
        //     .clone();

        psbt.inputs = sign_all_unsigned_tx(
            &self.secp,
            &prevouts,
            &unsigned_tx,
            &private_key,
            psbt.inputs,
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
        let secp = Secp256k1::new();
        return from_seed(&secret_string).public_key(&secp);
    }

    fn from_seed(secret_string: &Option<&str>) -> PrivateKey {
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

    pub fn sum_multi_sig(pub_keys: &Vec<PublicKey>) -> Address {
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
        let script = partial_p2wsh_multi_sig(pub_keys.iter(), len.try_into().unwrap())
            .push_int(len.try_into().unwrap())
            .push_opcode(all::OP_CHECKMULTISIG)
            .into_script();
        return Address::p2wsh(&script, NETWORK);
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
    input: Vec<Input>,
) -> Vec<Input> {
    let mut input_iter=input.iter();
    return prevouts
        .iter()
        .enumerate()
        .map(|(index, tx_out)| {
            let maybe_input =input_iter.next().cloned()
            .filter(|i|i.witness_utxo.as_ref().map(|utxo|utxo.eq(tx_out)).unwrap_or(false));   
            sign_tx(secp, index, unsigned_tx, private_key, tx_out, maybe_input).clone()
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
) -> Input {
    
    let hash_ty = EcdsaSighashType::All;
    let sighash = SighashCache::new(&mut unsigned_tx.clone())
        .segwit_signature_hash(index, &tx_out.script_pubkey, tx_out.value, hash_ty)
        .unwrap();

    let message = Message::from_slice(&sighash).unwrap();

    let sig = secp.sign_ecdsa(&message, &private_key.inner);

    let ecdsa_sig = EcdsaSig { sig, hash_ty };

    let mut input =maybe_input.unwrap_or(Input::default());
    
    input.witness_script = Some(tx_out.script_pubkey.clone());

    input
        .partial_sigs
        .insert(PublicKey::from_private_key(&secp, private_key), ecdsa_sig);

    input.witness_utxo = Some(tx_out.clone());

    return input.clone();
}
