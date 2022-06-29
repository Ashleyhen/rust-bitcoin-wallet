use std::{collections::BTreeMap, ops::Add, str::FromStr, sync::Arc};

use bdk::KeychainKind;
use bitcoin::{
    psbt::{Input, Output, PartiallySignedTransaction},
    secp256k1::{constants, rand::rngs::OsRng, All, Message, PublicKey, Secp256k1, SecretKey},
    util::{
        bip143::SigHashCache,
        bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey, KeySource},
        sighash::SighashCache,
        taproot::{LeafVersion, TapLeafHash},
    },
    Address, EcdsaSig, EcdsaSighashType, Network, OutPoint, Script, Sighash, Transaction, TxIn,
    TxOut, Witness,
};
use electrum_client::{Client, ElectrumApi};
use miniscript::{psbt::PsbtExt, ToPublicKey};

use crate::btc_wallet::utils::UnlockAndSend;

use self::{
    sign_tx::SignTx,
    wallet_traits::{AddressSchema, ApiCall},
};
// pub mod input_data;
pub mod input_data;

mod submit;
mod utils;
pub mod wallet_traits;

pub type WalletKeys = (ExtendedPubKey, KeySource);
pub mod p2tr;
pub mod p2wpkh;
pub mod sign_tx;

#[derive(Clone)]
pub struct ClientWallet {
    secp: Secp256k1<All>,
    seed: Seed,
    recieve: u32,
    change: u32,
}

#[derive(Clone)]
pub struct ClientWithSchema<'a, S: AddressSchema, A: ApiCall> {
    schema: &'a S,
    electrum_rpc_call: Arc<A>,
}
type Seed = [u8; constants::SECRET_KEY_SIZE];

pub const NETWORK: bitcoin::Network = Network::Testnet;

impl<'a, S: AddressSchema, A: ApiCall> ClientWithSchema<'a, S, A> {
    pub fn new(schema: &S, api_call: A) -> ClientWithSchema<S, A> {
        return ClientWithSchema {
            schema,
            electrum_rpc_call: Arc::new(api_call),
        };
    }

    pub fn print_balance(&self) {
        let wallet = self.schema.to_wallet();
        let (ext_pub, _) = self.schema.to_wallet().create_wallet(
            self.schema.wallet_purpose(),
            wallet.recieve,
            wallet.change,
        );
        let address = self.schema.map_ext_keys(&ext_pub);
        let get_balance = self
            .electrum_rpc_call
            .script_get_balance(&address.script_pubkey())
            .unwrap();
        println!("address: {}", address);
        println!("confirmed: {}", get_balance.confirmed);
        println!("unconfirmed: {}", get_balance.unconfirmed)
    }

    pub fn submit_tx(
        &self,
        to_addr: String,
        amount: u64,
        signing_fn: &dyn Fn(SignTx) -> Input,
    ) -> PartiallySignedTransaction {
        let wallet = self.schema.to_wallet();
        let signer =
            wallet.create_wallet(self.schema.wallet_purpose(), wallet.recieve, wallet.change);
        let (signer_pub_k, (signer_finger_p, signer_dp)) = signer.clone();
        let (change_pub_k, (change_finger_p, change_dp)) = wallet.create_wallet(
            self.schema.wallet_purpose(),
            wallet.recieve,
            wallet.change + 1,
        );
        let signer_addr = self.schema.map_ext_keys(&signer_pub_k);

        let history = Arc::new(
            self.electrum_rpc_call
                .script_list_unspent(&signer_addr.script_pubkey())
                .expect("address history call failed"),
        );

        let tx_in = history
            .clone()
            .iter()
            .map(|tx| {
                return TxIn {
                    previous_output: OutPoint::new(tx.tx_hash, tx.tx_pos.try_into().unwrap()),
                    script_sig: Script::new(), // The scriptSig must be exactly empty or the validation fails (native witness program)
                    sequence: 0xFFFFFFFF,
                    witness: Witness::default(),
                };
            })
            .collect::<Vec<TxIn>>();

        let previous_tx = tx_in
            .iter()
            .map(|tx_id| {
                self.electrum_rpc_call
                    .transaction_get(&tx_id.previous_output.txid)
                    .unwrap()
            })
            .collect::<Vec<Transaction>>();

        let unlock_and_send = UnlockAndSend::new(self.schema, signer.clone());
        let total=history.iter().map(|f| f.value).sum::<u64>();
        let tx_out = unlock_and_send.initialize_output(amount, total, change_pub_k, to_addr);

        let current_tx = Transaction {
            version: 0,
            lock_time: 0,
            input: tx_in,
            output: tx_out,
        };

        let input_vec =
            self.schema
                .prv_tx_input(previous_tx.to_vec().clone(), current_tx.clone(), signing_fn);

        let mut xpub = BTreeMap::new();

        // the current public key and derivation path
        xpub.insert(signer_pub_k, (signer_finger_p, signer_dp.clone()));

        let mut output = Output::default();
        output.bip32_derivation = BTreeMap::new();
        output
            .bip32_derivation
            .insert(change_pub_k.public_key, (change_finger_p, change_dp));

        // we have all the infomation for the partially signed transaction
        let psbt = PartiallySignedTransaction {
            unsigned_tx: current_tx.clone(),
            version: 1,
            xpub,
            proprietary: BTreeMap::new(),
            unknown: BTreeMap::new(),
            outputs: vec![output],
            inputs: input_vec,
        };
        return psbt;
    }
}
impl ClientWallet {
    pub fn new(secret_seed: Option<String>, recieve: u32, change: u32) -> ClientWallet {
        return ClientWallet {
            secp: Secp256k1::new(),
            seed: (match secret_seed {
                Some(secret) => SecretKey::from_str(&secret)
                    .expect("failed to initalize seed, check if the seed is valid"),
                _ => SecretKey::new(
                    &mut OsRng::new().expect("failed to create a random number generator !!! "),
                ),
            })
            .secret_bytes(),
            recieve,
            change,
        };
    }

    fn create_wallet(&self, purpose: u32, recieve: u32, index: u32) -> WalletKeys {
        // bip84 For the purpose-path level it uses 84'. The rest of the levels are used as defined in BIP44 or BIP49.
        // m / purpose' / coin_type' / account' / change / address_index
        let keychain = KeychainKind::External;
        let path = DerivationPath::from(vec![
            ChildNumber::from_hardened_idx(purpose).unwrap(), // purpose
            ChildNumber::from_hardened_idx(recieve).unwrap(), // first recieve
            ChildNumber::from_hardened_idx(0).unwrap(),       // second recieve
            ChildNumber::from_normal_idx(keychain as u32).unwrap(),
            ChildNumber::from_normal_idx(index).unwrap(),
        ]);
        let ext_prv = ExtendedPrivKey::new_master(NETWORK, &self.seed)
            .unwrap()
            .derive_priv(&self.secp, &path)
            .unwrap();
        let ext_pub = ExtendedPubKey::from_priv(&self.secp, &ext_prv);

        return (ext_pub, (ext_pub.fingerprint(), path));
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Broadcast_op {
    Finalize,
    None,
    Broadcast,
}

// fn path<F>(change:F)
//         where F: FnOnce(Script) ->dyn UnlockPreviousUTXO{

// }

// multi sig
